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
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use sha2::{Digest, Sha256};

#[derive(Debug, Serialize)]
pub(crate) struct CostIngestReport {
    pub(crate) schema_version: u8,
    pub(crate) provider: String,
    pub(crate) evidence_source: String,
    pub(crate) transcript_path: String,
    pub(crate) session_id: String,
    pub(crate) service_tier: String,
    pub(crate) pricing_snapshot_path: String,
    pub(crate) target: CostAttributionTarget,
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
    schema_version: u8,
    provider: String,
    surface: String,
    currency: String,
    source_url: String,
    retrieved_at: String,
    effective_from: String,
    version: String,
    service_tier: Option<String>,
    #[serde(default)]
    reasoning_token_policy: Option<String>,
    #[serde(default)]
    service_tiers: Vec<PricingTier>,
    #[serde(default)]
    models: Vec<PricingModel>,
}

#[derive(Debug, Deserialize)]
struct PricingModel {
    model_slug: String,
    input_micros_per_million: u64,
    cached_input_micros_per_million: u64,
    output_micros_per_million: u64,
}

#[derive(Debug, Deserialize)]
struct PricingTier {
    name: String,
    models: Vec<PricingModel>,
}

#[derive(Debug, Default)]
struct UsageAggregate {
    input_tokens: u64,
    cached_input_tokens: u64,
    output_tokens: u64,
    reasoning_tokens: u64,
    start_line: usize,
    end_line: usize,
    model_context_line: usize,
    event_count: usize,
    turn_ids: BTreeSet<String>,
    token_event_lines: Vec<(usize, String)>,
}

pub(crate) fn run_command(root: &Path, args: &[String]) -> RuntimeResult<String> {
    if matches!(args, [flag] if flag == "--help" || flag == "help")
        || matches!(args, [subcommand, flag] if subcommand == "codex-transcript" && (flag == "--help" || flag == "help"))
    {
        return Ok(help());
    }
    let request = IngestRequest::parse(args)?;
    let report = ingest(root, &request)?;
    let output = if request.json {
        serde_json::to_string_pretty(&report)
            .map_err(|error| format!("serialize cost ingest report: {error}"))?
    } else {
        format!(
            "cost ingest: {} measured model(s), {} receipt(s) appended, {} {}",
            report.measured.len(),
            report.receipts_appended,
            report.target.kind,
            report.target.id
        )
    };
    Ok(output)
}

pub(crate) fn ingest(root: &Path, request: &IngestRequest) -> Result<CostIngestReport> {
    let transcript_path = request
        .transcript_path
        .clone()
        .ok_or_else(|| "missing --transcript-path <path>".to_string())?;
    let session_id = request
        .session_id
        .clone()
        .ok_or_else(|| "missing --session-id <id>".to_string())?;
    if let Some(boundary_session_id) = &request.boundary_session_id
        && boundary_session_id != &session_id
    {
        return Err(format!(
            "boundary session id {boundary_session_id} does not match requested session id {session_id}"
        ));
    }
    let target =
        crate::cost_targets::resolve(root, request.target_kind, request.target_id.as_deref())?;
    let pricing_snapshot_path = request
        .pricing_snapshot
        .clone()
        .ok_or_else(|| "missing --pricing-snapshot <path>".to_string())?;
    let pricing_snapshot_full_path = root.join(&pricing_snapshot_path);
    let pricing_snapshot_sha256 = file_sha256(&pricing_snapshot_full_path)?;
    let pricing = load_pricing_snapshot(root, &pricing_snapshot_path)?;
    let selected_tier = select_service_tier(&pricing, request.service_tier.as_deref())?;
    let pricing_models = pricing_models_for_tier(&pricing, &selected_tier)?;
    let aggregates = parse_codex_transcript(
        &transcript_path,
        &session_id,
        request.since_line,
        request.until_line,
    )?;
    if aggregates.is_empty() {
        return Err(format!(
            "no Codex token_count usage events found in {}",
            transcript_path.display()
        ));
    }

    let mut measured = Vec::new();
    for (model_slug, aggregate) in aggregates {
        let selected_event_digest = selected_event_digest(&aggregate.token_event_lines);
        if let Some(boundary_turn_id) = &request.boundary_turn_id
            && !aggregate.turn_ids.contains(boundary_turn_id)
        {
            return Err(format!(
                "boundary turn id {boundary_turn_id} not found in selected transcript range"
            ));
        }
        let rates = pricing_models
            .iter()
            .find(|model| model.model_slug.eq_ignore_ascii_case(&model_slug))
            .ok_or_else(|| format!("pricing snapshot missing model {model_slug}"))?;
        let pricing_rates = CostPricingRates {
            input_micros_per_million: rates.input_micros_per_million,
            cached_input_micros_per_million: rates.cached_input_micros_per_million,
            output_micros_per_million: rates.output_micros_per_million,
        };
        let amount_micros = crate::cost_pricing::credit_micros(
            aggregate.input_tokens,
            aggregate.cached_input_tokens,
            aggregate.output_tokens,
            aggregate.reasoning_tokens,
            &pricing_rates,
            pricing.reasoning_token_policy.as_deref(),
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
                source_sha256: selected_event_digest.clone(),
                start_line: aggregate.start_line,
                end_line: aggregate.end_line,
                event_count: aggregate.event_count,
                model_slug,
                model_context_line: aggregate.model_context_line,
                session_id: session_id.clone(),
                selected_event_digest_sha256: selected_event_digest,
                turn_ids: aggregate.turn_ids.into_iter().collect(),
                tool_use_ids: request
                    .boundary_tool_use_id
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>(),
            },
        });
    }

    let mut receipts_appended = 0;
    if request.append_receipt {
        for evidence in &measured {
            if !receipt_exists(
                root,
                &pricing,
                &selected_tier,
                &pricing_snapshot_path,
                &pricing_snapshot_sha256,
                &target,
                evidence,
            )? {
                append_event(
                    root,
                    cost_receipt(
                        &pricing,
                        &selected_tier,
                        &pricing_snapshot_path,
                        &pricing_snapshot_sha256,
                        &target,
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
        session_id,
        service_tier: selected_tier,
        pricing_snapshot_path,
        target,
        receipts_appended,
        measured,
    })
}

#[derive(Debug)]
pub(crate) struct IngestRequest {
    transcript_path: Option<PathBuf>,
    session_id: Option<String>,
    since_line: Option<usize>,
    until_line: Option<usize>,
    pricing_snapshot: Option<String>,
    service_tier: Option<String>,
    target_kind: Option<CostAttributionKind>,
    target_id: Option<String>,
    boundary_session_id: Option<String>,
    boundary_turn_id: Option<String>,
    boundary_tool_use_id: Option<String>,
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
            session_id: None,
            since_line: None,
            until_line: None,
            pricing_snapshot: None,
            service_tier: None,
            target_kind: None,
            target_id: None,
            boundary_session_id: None,
            boundary_turn_id: None,
            boundary_tool_use_id: None,
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
                "--latest" => return Err("unsupported selector --latest; provide --transcript-path, --session-id, --since-line, and --until-line".to_string()),
                "--session-id" => request.session_id = Some(iter.next().ok_or_else(usage)?.to_string()),
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
                "--service-tier" => request.service_tier = Some(iter.next().ok_or_else(usage)?.to_string()),
                "--commit" => return Err("unsupported legacy flag --commit; use --target-kind commit --target-id <sha|HEAD>".to_string()),
                "--target-kind" => {
                    let value = iter.next().ok_or_else(usage)?;
                    request.target_kind = Some(CostAttributionKind::from_str(value)?);
                }
                "--target-id" => request.target_id = Some(iter.next().ok_or_else(usage)?.to_string()),
                "--boundary-session-id" => request.boundary_session_id = Some(iter.next().ok_or_else(usage)?.to_string()),
                "--boundary-turn-id" => request.boundary_turn_id = Some(iter.next().ok_or_else(usage)?.to_string()),
                "--boundary-tool-use-id" => request.boundary_tool_use_id = Some(iter.next().ok_or_else(usage)?.to_string()),
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
        if request.transcript_path.is_none() {
            return Err("missing --transcript-path <path>".to_string());
        }
        if request.session_id.is_none() {
            return Err("missing --session-id <id>".to_string());
        }
        if request.target_kind.is_none() {
            return Err("missing --target-kind <kind>".to_string());
        }
        if request.target_id.is_none() {
            return Err("missing --target-id <id>".to_string());
        }
        if request.pricing_snapshot.is_none() {
            return Err("missing --pricing-snapshot <path>".to_string());
        }
        if request.since_line.is_none() || request.until_line.is_none() {
            return Err("provide bounded --since-line <n> and --until-line <n>".to_string());
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
    "usage: task-registry-flow cost-ingest codex-transcript --transcript-path <path> --session-id <id> --since-line <n> --until-line <n> --pricing-snapshot <path> [--service-tier <tier>] --target-kind <kind> --target-id <id> [--boundary-session-id <id>] [--boundary-turn-id <id>] [--boundary-tool-use-id <id>] [--append-receipt] [--format json]".to_string()
}

fn help() -> String {
    [
        "usage: task-registry-flow cost-ingest codex-transcript --transcript-path <path> --session-id <id> --since-line <n> --until-line <n> --pricing-snapshot <path> [--service-tier <tier>] --target-kind <kind> --target-id <id> [--boundary-session-id <id>] [--boundary-turn-id <id>] [--boundary-tool-use-id <id>] [--append-receipt] [--format json]",
        "",
        "Ingest measured Codex token spend from a bounded local transcript slice.",
        "",
        "Required evidence:",
        "  --transcript-path <path>      Local Codex JSONL transcript path",
        "  --session-id <id>            Codex session id expected in the transcript",
        "  --since-line <n>             First line included in the bounded scan",
        "  --until-line <n>             Last line included in the bounded scan",
        "  --pricing-snapshot <path>    Governed pricing TOML relative to repo root",
        "  --service-tier <tier>         Explicit pricing tier; defaults only when snapshot declares one tier",
        "  --target-kind <kind>         commit, plan, task, verifier-run, landing-attempt, retry, release-cycle, or session",
        "  --target-id <id>             Explicit governed target id",
        "  --boundary-session-id <id>   Optional mutation boundary session id for coverage checks",
        "  --boundary-turn-id <id>      Optional mutation boundary turn id; must appear in selected range",
        "  --boundary-tool-use-id <id>  Optional mutation boundary tool-use id for coverage checks",
        "",
        "Options:",
        "  --append-receipt             Append measured cost evidence to docs/task-registry/events.jsonl",
        "  --format json                Emit a JSON report",
    ]
    .join("\n")
}

fn load_pricing_snapshot(root: &Path, path: &str) -> Result<PricingSnapshot> {
    let full_path = root.join(path);
    let body = fs::read_to_string(&full_path)
        .map_err(|error| format!("read pricing snapshot {}: {error}", full_path.display()))?;
    let snapshot = toml::from_str::<PricingSnapshot>(&body)
        .map_err(|error| format!("parse pricing snapshot {}: {error}", full_path.display()))?;
    if snapshot.schema_version != 1
        || snapshot.provider.trim().is_empty()
        || snapshot.surface != "codex"
        || snapshot.currency != "CREDITS"
        || snapshot.source_url.trim().is_empty()
        || snapshot.retrieved_at.trim().is_empty()
        || snapshot.effective_from.trim().is_empty()
        || snapshot.version.trim().is_empty()
        || snapshot.models.is_empty() && snapshot.service_tiers.is_empty()
    {
        return Err("pricing snapshot missing required provenance or rates".to_string());
    }
    let mut seen = BTreeSet::new();
    for (tier, model) in snapshot
        .models
        .iter()
        .map(|model| ("default", model))
        .chain(snapshot.service_tiers.iter().flat_map(|tier| {
            tier.models
                .iter()
                .map(move |model| (tier.name.as_str(), model))
        }))
    {
        if !seen.insert(format!("{tier}:{}", model.model_slug.to_ascii_lowercase())) {
            return Err(format!(
                "pricing snapshot duplicate model {} in tier {tier}",
                model.model_slug,
            ));
        }
        if model.input_micros_per_million == 0
            || model.cached_input_micros_per_million == 0
            || model.output_micros_per_million == 0
        {
            return Err(format!(
                "pricing snapshot model {} has zero rate",
                model.model_slug
            ));
        }
    }
    Ok(snapshot)
}

fn select_service_tier(snapshot: &PricingSnapshot, requested: Option<&str>) -> Result<String> {
    if let Some(tier) = requested.map(str::trim).filter(|value| !value.is_empty()) {
        if snapshot
            .service_tier
            .as_deref()
            .is_some_and(|default| default == tier)
            || snapshot
                .service_tiers
                .iter()
                .any(|entry| entry.name == tier)
        {
            return Ok(tier.to_string());
        }
        return Err(format!("pricing snapshot missing service tier {tier}"));
    }
    if let Some(default) = snapshot
        .service_tier
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Ok(default.to_string());
    }
    if snapshot.service_tiers.len() == 1 {
        return Ok(snapshot.service_tiers[0].name.clone());
    }
    Err("missing --service-tier <tier> for multi-tier pricing snapshot".to_string())
}

fn pricing_models_for_tier<'a>(
    snapshot: &'a PricingSnapshot,
    selected_tier: &str,
) -> Result<&'a [PricingModel]> {
    if let Some(tier) = snapshot
        .service_tiers
        .iter()
        .find(|tier| tier.name == selected_tier)
    {
        return Ok(tier.models.as_slice());
    }
    if snapshot
        .service_tier
        .as_deref()
        .is_some_and(|default| default == selected_tier)
        && !snapshot.models.is_empty()
    {
        return Ok(&snapshot.models);
    }
    Err(format!(
        "pricing snapshot missing service tier {selected_tier}"
    ))
}

fn parse_codex_transcript(
    transcript_path: &Path,
    expected_session_id: &str,
    since_line: Option<usize>,
    until_line: Option<usize>,
) -> Result<BTreeMap<String, UsageAggregate>> {
    let body = fs::read_to_string(transcript_path)
        .map_err(|error| format!("read transcript {}: {error}", transcript_path.display()))?;
    let mut model_slug = None::<String>;
    let mut turn_id = None::<String>;
    let mut model_context_line = None::<usize>;
    let mut session_id = None::<String>;
    let mut aggregates = BTreeMap::<String, UsageAggregate>::new();
    for (index, line) in body.lines().enumerate() {
        let line_number = index + 1;
        if line.trim().is_empty() {
            continue;
        }
        let value = serde_json::from_str::<Value>(line)
            .map_err(|error| format!("parse transcript line {line_number}: {error}"))?;
        if value.get("type").and_then(Value::as_str) == Some("session_meta") {
            if let Some(id) = value
                .pointer("/payload/id")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
            {
                session_id = Some(id.to_string());
            }
            if let Some(provider_model) = value
                .pointer("/payload/model")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
            {
                model_slug = Some(provider_model.to_string());
            }
        }
        if since_line.is_some_and(|since| line_number < since)
            || until_line.is_some_and(|until| line_number > until)
        {
            continue;
        }
        if value.get("type").and_then(Value::as_str) == Some("turn_context") {
            if let Some(turn_model) = value
                .pointer("/payload/model")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
            {
                model_slug = Some(turn_model.to_string());
                model_context_line = Some(line_number);
            }
            turn_id = value
                .pointer("/payload/turn_id")
                .and_then(Value::as_str)
                .map(str::to_string);
            continue;
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
            aggregate.model_context_line = model_context_line.unwrap_or(0);
        }
        if aggregate.model_context_line == 0 {
            return Err(format!(
                "token_count line {line_number} missing model context inside selected range"
            ));
        }
        aggregate.end_line = line_number;
        aggregate.event_count += 1;
        aggregate.input_tokens += input_tokens;
        aggregate.cached_input_tokens += cached_input_tokens;
        aggregate.output_tokens += output_tokens;
        aggregate.reasoning_tokens += reasoning_tokens;
        aggregate
            .token_event_lines
            .push((line_number, line.to_string()));
        if let Some(turn_id) = &turn_id {
            aggregate.turn_ids.insert(turn_id.clone());
        }
    }
    let found_session =
        session_id.ok_or_else(|| "transcript missing session_meta id".to_string())?;
    if found_session != expected_session_id {
        return Err(format!(
            "transcript session id {found_session} does not match requested session id {expected_session_id}"
        ));
    }
    Ok(aggregates)
}

fn usage_field(usage: &Value, field: &str, line_number: usize) -> Result<u64> {
    usage
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("token_count line {line_number} missing numeric {field}"))
}

fn cost_receipt(
    pricing: &PricingSnapshot,
    selected_tier: &str,
    pricing_snapshot_path: &str,
    pricing_snapshot_sha256: &str,
    target: &CostAttributionTarget,
    evidence: MeasuredCostEvidence,
) -> EventRecord {
    let model_slug = evidence.model_slug.clone();
    let mut event = EventRecord::new(
        timestamp(),
        CliCommand::CostIngest,
        EventOutcome::Ok,
        0,
        format!(
            "measured Codex cost evidence for {} {} using {model_slug}",
            target.kind, target.id
        ),
    );
    event.cost_evidence = Some(CostEvidence {
        status: CostEvidenceStatus::Measured,
        evidence_source: "codex-transcript-token-count".to_string(),
        attribution_target: target.clone(),
        provider: Some(pricing.provider.clone()),
        model_slug: Some(model_slug),
        usage: Some(evidence.usage),
        pricing: Some(CostPricingSnapshot {
            source: pricing.source_url.clone(),
            version: pricing.version.clone(),
            currency: pricing.currency.clone(),
            service_tier: selected_tier.to_string(),
            snapshot_path: pricing_snapshot_path.to_string(),
            snapshot_sha256: pricing_snapshot_sha256.to_string(),
            reasoning_token_policy: pricing.reasoning_token_policy.clone(),
        }),
        amount: Some(evidence.amount),
        pricing_rates: Some(evidence.pricing_rates),
        usage_contributions: vec![evidence.contribution],
        measurement_timestamp: Some(timestamp()),
        estimation_method: None,
        unmeasured_reason: None,
        boundary_session_id: None,
        boundary_turn_id: None,
        boundary_tool_use_id: None,
    });
    event
}

fn receipt_exists(
    root: &Path,
    pricing: &PricingSnapshot,
    selected_tier: &str,
    pricing_snapshot_path: &str,
    pricing_snapshot_sha256: &str,
    target: &CostAttributionTarget,
    expected: &MeasuredCostEvidence,
) -> Result<bool> {
    let events_path = root.join(EVENTS_PATH);
    if !events_path.is_file() {
        return Ok(false);
    }
    let body = fs::read_to_string(&events_path)
        .map_err(|error| format!("read {}: {error}", events_path.display()))?;
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
        let Some(receipt_pricing) = evidence.pricing.as_ref() else {
            continue;
        };
        let Some(receipt_rates) = evidence.pricing_rates.as_ref() else {
            continue;
        };
        let Some(receipt_amount) = evidence.amount.as_ref() else {
            continue;
        };
        let same_receipt = evidence.status == CostEvidenceStatus::Measured
            && evidence.attribution_target.kind == target.kind
            && evidence.attribution_target.id == target.id
            && evidence.provider.as_deref() == Some(pricing.provider.as_str())
            && evidence.model_slug.as_deref() == Some(expected.model_slug.as_str())
            && receipt_pricing.source == pricing.source_url
            && receipt_pricing.version == pricing.version
            && receipt_pricing.currency == pricing.currency
            && receipt_pricing.service_tier == selected_tier
            && receipt_pricing.snapshot_path == pricing_snapshot_path
            && receipt_pricing.snapshot_sha256 == pricing_snapshot_sha256
            && receipt_pricing.reasoning_token_policy == pricing.reasoning_token_policy
            && receipt_rates.input_micros_per_million
                == expected.pricing_rates.input_micros_per_million
            && receipt_rates.cached_input_micros_per_million
                == expected.pricing_rates.cached_input_micros_per_million
            && receipt_rates.output_micros_per_million
                == expected.pricing_rates.output_micros_per_million
            && receipt_amount.currency == expected.amount.currency
            && receipt_amount.amount_micros == expected.amount.amount_micros
            && contribution.source_kind == expected.contribution.source_kind
            && contribution.source_path == expected.contribution.source_path
            && contribution.start_line == expected.contribution.start_line
            && contribution.end_line == expected.contribution.end_line
            && contribution.event_count == expected.contribution.event_count
            && contribution.model_slug == expected.contribution.model_slug
            && contribution.selected_event_digest_sha256
                == expected.contribution.selected_event_digest_sha256;
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
    target_id: String,
) -> IngestRequest {
    IngestRequest {
        transcript_path: Some(transcript_path),
        session_id: Some("session-cost-tests".to_string()),
        since_line: Some(2),
        until_line: Some(3),
        pricing_snapshot: Some(pricing_snapshot),
        service_tier: None,
        target_kind: Some(CostAttributionKind::Commit),
        target_id: Some(target_id),
        boundary_session_id: None,
        boundary_turn_id: None,
        boundary_tool_use_id: None,
        append_receipt: false,
        json: true,
    }
}

#[cfg(test)]
pub(crate) fn append_request_for_test(
    transcript_path: PathBuf,
    pricing_snapshot: String,
    target_id: String,
) -> IngestRequest {
    IngestRequest {
        transcript_path: Some(transcript_path),
        session_id: Some("session-cost-tests".to_string()),
        since_line: Some(2),
        until_line: Some(3),
        pricing_snapshot: Some(pricing_snapshot),
        service_tier: None,
        target_kind: Some(CostAttributionKind::Commit),
        target_id: Some(target_id),
        boundary_session_id: None,
        boundary_turn_id: None,
        boundary_tool_use_id: None,
        append_receipt: true,
        json: true,
    }
}

fn file_sha256(path: &Path) -> Result<String> {
    let body = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(&body)))
}

fn selected_event_digest(lines: &[(usize, String)]) -> String {
    let mut hasher = Sha256::new();
    for (line_number, line) in lines {
        hasher.update(line_number.to_string().as_bytes());
        hasher.update(b":");
        hasher.update(line.as_bytes());
        hasher.update(b"\n");
    }
    format!("{:x}", hasher.finalize())
}
