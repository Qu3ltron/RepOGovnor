use crate::model::{EVENTS_PATH, EventRecord, Result};
use crate::reports::RuntimeResult;
use crate::runtime::{append_event, timestamp};
use crate::schema::{
    CliCommand, CostAmount, CostAttributionKind, CostAttributionTarget, CostEvidence,
    CostEvidenceStatus, CostPricingRates, CostPricingSnapshot, EventOutcome, TokenUsage,
    UsageContribution,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_CODEX_SESSIONS: &str = ".codex/sessions";

#[derive(Debug, Serialize)]
pub(crate) struct CostIngestReport {
    pub(crate) schema_version: u8,
    pub(crate) provider: String,
    pub(crate) evidence_source: String,
    pub(crate) transcript_path: String,
    pub(crate) pricing_snapshot_path: String,
    pub(crate) commit_sha: String,
    pub(crate) receipts_appended: usize,
    pub(crate) measured: Vec<MeasuredCostEvidence>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MeasuredCostEvidence {
    pub(crate) model_slug: String,
    pub(crate) usage: TokenUsage,
    pub(crate) pricing_rates: CostPricingRates,
    pub(crate) amount: CostAmount,
    pub(crate) contribution: UsageContribution,
}

#[derive(Debug, Deserialize)]
struct PricingSnapshot {
    provider: String,
    surface: String,
    currency: String,
    source_url: String,
    retrieved_at: String,
    version: String,
    models: Vec<PricingModel>,
}

#[derive(Debug, Deserialize)]
struct PricingModel {
    model_slug: String,
    input_micros_per_million: u64,
    cached_input_micros_per_million: u64,
    output_micros_per_million: u64,
}

#[derive(Debug, Default)]
struct UsageAggregate {
    input_tokens: u64,
    cached_input_tokens: u64,
    output_tokens: u64,
    reasoning_tokens: u64,
    start_line: usize,
    end_line: usize,
    event_count: usize,
    turn_ids: BTreeSet<String>,
}

pub(crate) fn run_command(root: &Path, args: &[String]) -> RuntimeResult<String> {
    let request = IngestRequest::parse(args)?;
    let report = ingest(root, &request)?;
    let output = if request.json {
        serde_json::to_string_pretty(&report)
            .map_err(|error| format!("serialize cost ingest report: {error}"))?
    } else {
        format!(
            "cost ingest: {} measured model(s), {} receipt(s) appended, commit {}",
            report.measured.len(),
            report.receipts_appended,
            report.commit_sha
        )
    };
    Ok(output)
}

pub(crate) fn ingest(root: &Path, request: &IngestRequest) -> Result<CostIngestReport> {
    let transcript_path = match (&request.transcript_path, request.latest) {
        (Some(path), false) => path.clone(),
        (None, true) => latest_transcript_path()?,
        _ => return Err("provide exactly one of --transcript-path <path> or --latest".to_string()),
    };
    let commit_sha = resolve_commit(root, request.commit.as_deref())?;
    let pricing_snapshot_path = request
        .pricing_snapshot
        .clone()
        .ok_or_else(|| "missing --pricing-snapshot <path>".to_string())?;
    let pricing = load_pricing_snapshot(root, &pricing_snapshot_path)?;
    let aggregates =
        parse_codex_transcript(&transcript_path, request.since_line, request.until_line)?;
    if aggregates.is_empty() {
        return Err(format!(
            "no Codex token_count usage events found in {}",
            transcript_path.display()
        ));
    }

    let mut measured = Vec::new();
    for (model_slug, aggregate) in aggregates {
        let rates = pricing
            .models
            .iter()
            .find(|model| model.model_slug.eq_ignore_ascii_case(&model_slug))
            .ok_or_else(|| format!("pricing snapshot missing model {model_slug}"))?;
        let pricing_rates = CostPricingRates {
            input_micros_per_million: rates.input_micros_per_million,
            cached_input_micros_per_million: rates.cached_input_micros_per_million,
            output_micros_per_million: rates.output_micros_per_million,
        };
        let uncached_input = aggregate
            .input_tokens
            .checked_sub(aggregate.cached_input_tokens)
            .ok_or_else(|| format!("cached input exceeds input tokens for {model_slug}"))?;
        let amount_micros = credit_micros(
            uncached_input,
            aggregate.cached_input_tokens,
            aggregate.output_tokens,
            &pricing_rates,
        )?;
        measured.push(MeasuredCostEvidence {
            model_slug: model_slug.clone(),
            usage: TokenUsage {
                input_tokens: aggregate.input_tokens,
                output_tokens: aggregate.output_tokens,
                cached_input_tokens: Some(aggregate.cached_input_tokens),
                reasoning_tokens: Some(aggregate.reasoning_tokens),
            },
            pricing_rates,
            amount: CostAmount {
                currency: pricing.currency.clone(),
                amount_micros,
            },
            contribution: UsageContribution {
                source_kind: "codex-transcript-token-count".to_string(),
                source_path: transcript_path.display().to_string(),
                start_line: aggregate.start_line,
                end_line: aggregate.end_line,
                event_count: aggregate.event_count,
                model_slug,
                turn_ids: aggregate.turn_ids.into_iter().collect(),
            },
        });
    }

    let mut receipts_appended = 0;
    if request.append_receipt {
        for evidence in &measured {
            if !receipt_exists(
                root,
                &pricing,
                &pricing_snapshot_path,
                &commit_sha,
                evidence,
            )? {
                append_event(
                    root,
                    cost_receipt(
                        &pricing,
                        &pricing_snapshot_path,
                        &commit_sha,
                        evidence.clone(),
                    ),
                )?;
                receipts_appended += 1;
            }
        }
    }

    Ok(CostIngestReport {
        schema_version: 1,
        provider: pricing.provider,
        evidence_source: "codex-transcript-token-count".to_string(),
        transcript_path: transcript_path.display().to_string(),
        pricing_snapshot_path,
        commit_sha,
        receipts_appended,
        measured,
    })
}

#[derive(Debug)]
pub(crate) struct IngestRequest {
    transcript_path: Option<PathBuf>,
    latest: bool,
    since_line: Option<usize>,
    until_line: Option<usize>,
    pricing_snapshot: Option<String>,
    commit: Option<String>,
    append_receipt: bool,
    json: bool,
}

impl IngestRequest {
    fn parse(args: &[String]) -> Result<Self> {
        if args.first().map(String::as_str) != Some("codex-transcript") {
            return Err(usage());
        }
        let mut request = Self {
            transcript_path: None,
            latest: false,
            since_line: None,
            until_line: None,
            pricing_snapshot: None,
            commit: None,
            append_receipt: false,
            json: false,
        };
        let mut iter = args.iter().skip(1);
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--transcript-path" => {
                    let value = iter.next().ok_or_else(usage)?;
                    request.transcript_path = Some(PathBuf::from(value));
                }
                "--latest" => request.latest = true,
                "--since-line" => {
                    let value = iter.next().ok_or_else(usage)?;
                    request.since_line = Some(
                        value
                            .parse::<usize>()
                            .map_err(|_| "since-line must be a positive integer".to_string())?,
                    );
                }
                "--until-line" => {
                    let value = iter.next().ok_or_else(usage)?;
                    request.until_line = Some(
                        value
                            .parse::<usize>()
                            .map_err(|_| "until-line must be a positive integer".to_string())?,
                    );
                }
                "--pricing-snapshot" => {
                    request.pricing_snapshot = Some(iter.next().ok_or_else(usage)?.to_string());
                }
                "--commit" => request.commit = Some(iter.next().ok_or_else(usage)?.to_string()),
                "--append-receipt" => request.append_receipt = true,
                "--format" => {
                    let value = iter.next().ok_or_else(usage)?;
                    if value != "json" {
                        return Err(usage());
                    }
                    request.json = true;
                }
                _ => return Err(usage()),
            }
        }
        if request.commit.is_none() {
            return Err("missing --commit <sha|HEAD>".to_string());
        }
        if request.pricing_snapshot.is_none() {
            return Err("missing --pricing-snapshot <path>".to_string());
        }
        if let (Some(since_line), Some(until_line)) = (request.since_line, request.until_line)
            && until_line < since_line
        {
            return Err("until-line must be greater than or equal to since-line".to_string());
        }
        Ok(request)
    }
}

fn usage() -> String {
    "usage: task-registry-flow cost-ingest codex-transcript (--latest|--transcript-path <path>) --pricing-snapshot <path> --commit <sha|HEAD> [--since-line <n>] [--until-line <n>] [--append-receipt] [--format json]".to_string()
}

fn load_pricing_snapshot(root: &Path, path: &str) -> Result<PricingSnapshot> {
    let full_path = root.join(path);
    let body = fs::read_to_string(&full_path)
        .map_err(|error| format!("read pricing snapshot {}: {error}", full_path.display()))?;
    let snapshot = toml::from_str::<PricingSnapshot>(&body)
        .map_err(|error| format!("parse pricing snapshot {}: {error}", full_path.display()))?;
    if snapshot.provider.trim().is_empty()
        || snapshot.surface != "codex"
        || snapshot.currency != "CREDITS"
        || snapshot.source_url.trim().is_empty()
        || snapshot.retrieved_at.trim().is_empty()
        || snapshot.version.trim().is_empty()
        || snapshot.models.is_empty()
    {
        return Err("pricing snapshot missing required provenance or rates".to_string());
    }
    Ok(snapshot)
}

fn parse_codex_transcript(
    transcript_path: &Path,
    since_line: Option<usize>,
    until_line: Option<usize>,
) -> Result<BTreeMap<String, UsageAggregate>> {
    let body = fs::read_to_string(transcript_path)
        .map_err(|error| format!("read transcript {}: {error}", transcript_path.display()))?;
    let mut model_slug = None::<String>;
    let mut turn_id = None::<String>;
    let mut aggregates = BTreeMap::<String, UsageAggregate>::new();
    for (index, line) in body.lines().enumerate() {
        let line_number = index + 1;
        if since_line.is_some_and(|since| line_number < since)
            || until_line.is_some_and(|until| line_number > until)
            || line.trim().is_empty()
        {
            continue;
        }
        let value = serde_json::from_str::<Value>(line)
            .map_err(|error| format!("parse transcript line {line_number}: {error}"))?;
        if value.get("type").and_then(Value::as_str) == Some("session_meta")
            && let Some(provider_model) = value
                .pointer("/payload/model")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
        {
            model_slug = Some(provider_model.to_string());
        }
        if value.get("type").and_then(Value::as_str) == Some("turn_context") {
            if let Some(turn_model) = value
                .pointer("/payload/model")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
            {
                model_slug = Some(turn_model.to_string());
            }
            turn_id = value
                .pointer("/payload/turn_id")
                .and_then(Value::as_str)
                .map(str::to_string);
        }
        if value.get("type").and_then(Value::as_str) != Some("event_msg")
            || value.pointer("/payload/type").and_then(Value::as_str) != Some("token_count")
        {
            continue;
        }
        let usage = value
            .pointer("/payload/info/last_token_usage")
            .ok_or_else(|| format!("token_count line {line_number} missing last_token_usage"))?;
        let model = model_slug
            .clone()
            .ok_or_else(|| format!("token_count line {line_number} missing preceding model"))?;
        let input_tokens = usage_field(usage, "input_tokens", line_number)?;
        let cached_input_tokens = usage_field(usage, "cached_input_tokens", line_number)?;
        let output_tokens = usage_field(usage, "output_tokens", line_number)?;
        let reasoning_tokens = usage_field(usage, "reasoning_output_tokens", line_number)?;
        if cached_input_tokens > input_tokens {
            return Err(format!(
                "token_count line {line_number} cached_input_tokens exceeds input_tokens"
            ));
        }
        let aggregate = aggregates.entry(model.clone()).or_default();
        if aggregate.event_count == 0 {
            aggregate.start_line = line_number;
        }
        aggregate.end_line = line_number;
        aggregate.event_count += 1;
        aggregate.input_tokens += input_tokens;
        aggregate.cached_input_tokens += cached_input_tokens;
        aggregate.output_tokens += output_tokens;
        aggregate.reasoning_tokens += reasoning_tokens;
        if let Some(turn_id) = &turn_id {
            aggregate.turn_ids.insert(turn_id.clone());
        }
    }
    Ok(aggregates)
}

fn usage_field(usage: &Value, field: &str, line_number: usize) -> Result<u64> {
    usage
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("token_count line {line_number} missing numeric {field}"))
}

fn credit_micros(
    uncached_input_tokens: u64,
    cached_input_tokens: u64,
    output_tokens: u64,
    rates: &CostPricingRates,
) -> Result<u64> {
    let input = (uncached_input_tokens as u128) * (rates.input_micros_per_million as u128);
    let cached = (cached_input_tokens as u128) * (rates.cached_input_micros_per_million as u128);
    let output = (output_tokens as u128) * (rates.output_micros_per_million as u128);
    let total = (input + cached + output) / 1_000_000;
    u64::try_from(total).map_err(|_| "calculated credit amount overflows u64".to_string())
}

fn latest_transcript_path() -> Result<PathBuf> {
    let home = env::var("CODEX_HOME")
        .map(PathBuf::from)
        .or_else(|_| env::var("HOME").map(|home| PathBuf::from(home).join(DEFAULT_CODEX_SESSIONS)))
        .map_err(|_| "CODEX_HOME or HOME must be set for --latest".to_string())?;
    let root = if home.ends_with("sessions") {
        home
    } else {
        home.join("sessions")
    };
    let mut newest = None::<(std::time::SystemTime, PathBuf)>;
    collect_latest_transcript(&root, &mut newest)?;
    newest.map(|(_, path)| path).ok_or_else(|| {
        format!(
            "no Codex transcript jsonl files found under {}",
            root.display()
        )
    })
}

fn collect_latest_transcript(
    dir: &Path,
    newest: &mut Option<(std::time::SystemTime, PathBuf)>,
) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in
        fs::read_dir(dir).map_err(|error| format!("read dir {}: {error}", dir.display()))?
    {
        let entry = entry.map_err(|error| format!("read dir entry {}: {error}", dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_latest_transcript(&path, newest)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("jsonl") {
            let modified = entry
                .metadata()
                .and_then(|metadata| metadata.modified())
                .map_err(|error| format!("read metadata {}: {error}", path.display()))?;
            if newest.as_ref().is_none_or(|(known, _)| modified > *known) {
                *newest = Some((modified, path));
            }
        }
    }
    Ok(())
}

fn resolve_commit(root: &Path, commit: Option<&str>) -> Result<String> {
    let commit = commit.ok_or_else(|| "missing --commit <sha|HEAD>".to_string())?;
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--verify")
        .arg(format!("{commit}^{{commit}}"))
        .current_dir(root)
        .output()
        .map_err(|error| format!("run git rev-parse: {error}"))?;
    if !output.status.success() {
        return Err(format!("invalid commit {commit}"));
    }
    let sha = String::from_utf8(output.stdout)
        .map_err(|error| format!("decode git rev-parse output: {error}"))?;
    Ok(sha.trim().to_string())
}

fn cost_receipt(
    pricing: &PricingSnapshot,
    pricing_snapshot_path: &str,
    commit_sha: &str,
    evidence: MeasuredCostEvidence,
) -> EventRecord {
    let model_slug = evidence.model_slug.clone();
    let mut event = EventRecord::new(
        timestamp(),
        CliCommand::CostIngest,
        EventOutcome::Ok,
        0,
        format!("measured Codex cost evidence for commit {commit_sha} using {model_slug}"),
    );
    event.cost_evidence = Some(CostEvidence {
        status: CostEvidenceStatus::Measured,
        evidence_source: "codex-transcript-token-count".to_string(),
        attribution_target: CostAttributionTarget {
            kind: CostAttributionKind::Commit,
            id: commit_sha.to_string(),
        },
        provider: Some(pricing.provider.clone()),
        model_slug: Some(model_slug),
        usage: Some(evidence.usage),
        pricing: Some(CostPricingSnapshot {
            source: pricing.source_url.clone(),
            version: format!("{} ({pricing_snapshot_path})", pricing.version),
            currency: pricing.currency.clone(),
        }),
        amount: Some(evidence.amount),
        pricing_rates: Some(evidence.pricing_rates),
        usage_contributions: vec![evidence.contribution],
        measurement_timestamp: Some(timestamp()),
        estimation_method: None,
        unmeasured_reason: None,
    });
    event
}

fn receipt_exists(
    root: &Path,
    pricing: &PricingSnapshot,
    pricing_snapshot_path: &str,
    commit_sha: &str,
    expected: &MeasuredCostEvidence,
) -> Result<bool> {
    let events_path = root.join(EVENTS_PATH);
    if !events_path.is_file() {
        return Ok(false);
    }
    let body = fs::read_to_string(&events_path)
        .map_err(|error| format!("read {}: {error}", events_path.display()))?;
    let expected_version = format!("{} ({pricing_snapshot_path})", pricing.version);
    for (index, line) in body.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let value = serde_json::from_str::<Value>(line)
            .map_err(|error| format!("parse receipt line {}: {error}", index + 1))?;
        if value.get("cost_evidence").is_none() {
            continue;
        }
        let event = serde_json::from_value::<EventRecord>(value)
            .map_err(|error| format!("parse cost receipt line {}: {error}", index + 1))?;
        let Some(evidence) = event.cost_evidence else {
            continue;
        };
        let Some(contribution) = evidence.usage_contributions.first() else {
            continue;
        };
        let same_receipt = evidence.status == CostEvidenceStatus::Measured
            && evidence.attribution_target.kind == CostAttributionKind::Commit
            && evidence.attribution_target.id == commit_sha
            && evidence.provider.as_deref() == Some(pricing.provider.as_str())
            && evidence.model_slug.as_deref() == Some(expected.model_slug.as_str())
            && evidence
                .pricing
                .as_ref()
                .map(|pricing| pricing.version.as_str())
                == Some(expected_version.as_str())
            && contribution.source_kind == expected.contribution.source_kind
            && contribution.source_path == expected.contribution.source_path
            && contribution.start_line == expected.contribution.start_line
            && contribution.end_line == expected.contribution.end_line
            && contribution.event_count == expected.contribution.event_count
            && contribution.model_slug == expected.contribution.model_slug;
        if same_receipt {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
pub(crate) fn ingest_for_test(root: &Path, request: &IngestRequest) -> Result<CostIngestReport> {
    ingest(root, request)
}

#[cfg(test)]
pub(crate) fn request_for_test(
    transcript_path: PathBuf,
    pricing_snapshot: String,
    commit: String,
) -> IngestRequest {
    IngestRequest {
        transcript_path: Some(transcript_path),
        latest: false,
        since_line: None,
        until_line: None,
        pricing_snapshot: Some(pricing_snapshot),
        commit: Some(commit),
        append_receipt: false,
        json: true,
    }
}

#[cfg(test)]
pub(crate) fn append_request_for_test(
    transcript_path: PathBuf,
    pricing_snapshot: String,
    commit: String,
) -> IngestRequest {
    IngestRequest {
        transcript_path: Some(transcript_path),
        latest: false,
        since_line: None,
        until_line: None,
        pricing_snapshot: Some(pricing_snapshot),
        commit: Some(commit),
        append_receipt: true,
        json: true,
    }
}
