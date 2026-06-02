use crate::metrics::receipt_value_hash;
use crate::model::{EVENTS_PATH, EventRecord, Result};
use crate::reports::{RuntimeFailure, RuntimeResult};
use crate::schema::{
    CheckReport, CliCommand, CostEvidence, CostEvidenceStatus, Diagnostic, FailureCode,
    ReportSurface, UsageContribution,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub(crate) fn run_command(root: &Path, args: &[String]) -> RuntimeResult<String> {
    let json = match args {
        [] => false,
        [flag, format] if flag == "--format" && format == "json" => true,
        _ => return Err("usage: task-registry-flow cost-evidence-check [--format json]".into()),
    };
    let report = check(root)?;
    if json {
        let output = report.to_json()?;
        if report.has_failures() {
            Err(RuntimeFailure::json(FailureCode::DiagnosticReport, output))
        } else {
            Ok(output)
        }
    } else if report.has_failures() {
        Err(format_text_report(&report).into())
    } else {
        Ok(format_text_report(&report))
    }
}

pub(crate) fn check(root: &Path) -> Result<CheckReport> {
    let mut checks = Vec::new();
    let mut seen_event_digests = BTreeSet::new();
    let mut measured_ranges = BTreeMap::<String, Vec<(usize, usize)>>::new();
    let events_path = root.join(EVENTS_PATH);
    if !events_path.is_file() {
        checks.push(no_receipts_check());
        return CheckReport::new(ReportSurface::CostEvidence, checks);
    }

    let body = fs::read_to_string(&events_path)
        .map_err(|error| format!("read {}: {error}", events_path.display()))?;
    for (index, line) in body.lines().enumerate() {
        let line_number = index + 1;
        if line.trim().is_empty() {
            continue;
        }
        let value = match serde_json::from_str::<Value>(line) {
            Ok(value) => value,
            Err(error) => {
                checks.push(Diagnostic::fail(
                    format!("cost-evidence-json-{line_number}"),
                    ReportSurface::CostEvidence,
                    EVENTS_PATH,
                    "valid receipt JSON",
                    format!("line {line_number}: {error}"),
                    "repair or archive malformed receipt events",
                ));
                continue;
            }
        };
        if receipt_value_hash(&value).is_err() {
            checks.push(Diagnostic::fail(
                format!("cost-evidence-hashable-{line_number}"),
                ReportSurface::CostEvidence,
                EVENTS_PATH,
                "hashable receipt event",
                format!("line {line_number} is not hashable"),
                "repair malformed receipt event fields",
            ));
            continue;
        }
        let has_cost_evidence = value.get("cost_evidence").is_some();
        let event = match serde_json::from_value::<EventRecord>(value) {
            Ok(event) => event,
            Err(error) if has_cost_evidence => {
                checks.push(Diagnostic::fail(
                    format!("cost-evidence-schema-{line_number}"),
                    ReportSurface::CostEvidence,
                    EVENTS_PATH,
                    "cost evidence receipt matches runtime schema",
                    format!("line {line_number}: {error}"),
                    "write cost evidence using the canonical receipt schema",
                ));
                continue;
            }
            Err(_) => continue,
        };
        let Some(cost_evidence) = &event.cost_evidence else {
            continue;
        };
        checks.extend(validate_cost_evidence(
            root,
            cost_evidence,
            line_number,
            &mut seen_event_digests,
            &mut measured_ranges,
        ));
    }

    if checks.is_empty() {
        checks.push(no_receipts_check());
    }
    CheckReport::new(ReportSurface::CostEvidence, checks)
}

fn validate_cost_evidence(
    root: &Path,
    evidence: &CostEvidence,
    line_number: usize,
    seen_event_digests: &mut BTreeSet<String>,
    measured_ranges: &mut BTreeMap<String, Vec<(usize, usize)>>,
) -> Vec<Diagnostic> {
    let path = format!(
        "{}:{}",
        evidence.attribution_target.kind, evidence.attribution_target.id
    );
    let mut missing = Vec::new();
    require_nonempty(
        &mut missing,
        "evidence_source",
        Some(&evidence.evidence_source),
    );
    require_nonempty(
        &mut missing,
        "attribution_target.id",
        Some(&evidence.attribution_target.id),
    );

    match evidence.status {
        CostEvidenceStatus::Measured => {
            require_nonempty(&mut missing, "provider", evidence.provider.as_ref());
            require_nonempty(&mut missing, "model_slug", evidence.model_slug.as_ref());
            require_nonempty(
                &mut missing,
                "measurement_timestamp",
                evidence.measurement_timestamp.as_ref(),
            );
            if evidence.usage.is_none() {
                missing.push("usage");
            }
            if let Some(pricing) = &evidence.pricing {
                require_nonempty(&mut missing, "pricing.source", Some(&pricing.source));
                require_nonempty(&mut missing, "pricing.version", Some(&pricing.version));
                require_nonempty(&mut missing, "pricing.currency", Some(&pricing.currency));
                require_nonempty(
                    &mut missing,
                    "pricing.service_tier",
                    Some(&pricing.service_tier),
                );
                require_nonempty(
                    &mut missing,
                    "pricing.snapshot_path",
                    Some(&pricing.snapshot_path),
                );
                require_nonempty(
                    &mut missing,
                    "pricing.snapshot_sha256",
                    Some(&pricing.snapshot_sha256),
                );
                if evidence
                    .usage
                    .as_ref()
                    .and_then(|usage| usage.reasoning_tokens)
                    .is_some_and(|tokens| tokens > 0)
                {
                    require_nonempty(
                        &mut missing,
                        "pricing.reasoning_token_policy",
                        pricing.reasoning_token_policy.as_ref(),
                    );
                }
            } else {
                missing.push("pricing");
            }
            if let Some(amount) = &evidence.amount {
                require_nonempty(&mut missing, "amount.currency", Some(&amount.currency));
            } else {
                missing.push("amount");
            }
            if evidence.pricing_rates.is_none() {
                missing.push("pricing_rates");
            }
            if evidence.usage_contributions.is_empty() {
                missing.push("usage_contributions");
            }
            for contribution in &evidence.usage_contributions {
                require_nonempty(
                    &mut missing,
                    "usage_contributions.source_sha256",
                    Some(&contribution.source_sha256),
                );
                require_nonempty(
                    &mut missing,
                    "usage_contributions.session_id",
                    Some(&contribution.session_id),
                );
                require_nonempty(
                    &mut missing,
                    "usage_contributions.selected_event_digest_sha256",
                    Some(&contribution.selected_event_digest_sha256),
                );
                if contribution.model_context_line == 0 {
                    missing.push("usage_contributions.model_context_line");
                }
                if contribution.turn_ids.is_empty() {
                    missing.push("usage_contributions.turn_ids");
                }
            }
            if !missing.is_empty() {
                return vec![Diagnostic::fail(
                    format!("cost-evidence-measured-{line_number}"),
                    ReportSurface::CostEvidence,
                    path,
                    "measured cost evidence includes all required canonical replay fields",
                    format!("missing {}", missing.join(", ")),
                    "record measured cost only from structured usage, transcript hash, event digest, and pricing evidence",
                )];
            }
            if let Some(failure) = validate_measured_replay(
                root,
                evidence,
                line_number,
                seen_event_digests,
                measured_ranges,
            ) {
                vec![failure]
            } else {
                vec![Diagnostic::pass(
                    format!("cost-evidence-measured-{line_number}"),
                    ReportSurface::CostEvidence,
                    path,
                    "measured cost evidence is replayable, priced, hash-matched, and non-overlapping",
                )]
            }
        }
        CostEvidenceStatus::Estimated => {
            require_nonempty(
                &mut missing,
                "estimation_method",
                evidence.estimation_method.as_ref(),
            );
            if missing.is_empty() {
                vec![Diagnostic::warn(
                    format!("cost-evidence-estimated-{line_number}"),
                    ReportSurface::CostEvidence,
                    path,
                    "estimated cost evidence is explicitly labeled",
                )]
            } else {
                vec![Diagnostic::fail(
                    format!("cost-evidence-estimated-{line_number}"),
                    ReportSurface::CostEvidence,
                    path,
                    "estimated cost evidence includes an estimation method",
                    format!("missing {}", missing.join(", ")),
                    "label estimated cost with explicit assumptions",
                )]
            }
        }
        CostEvidenceStatus::Unmeasured => {
            require_nonempty(
                &mut missing,
                "unmeasured_reason",
                evidence.unmeasured_reason.as_ref(),
            );
            if evidence.amount.is_some() {
                missing.push("amount_absent");
            }
            if missing.is_empty() {
                vec![Diagnostic::warn(
                    format!("cost-evidence-unmeasured-{line_number}"),
                    ReportSurface::CostEvidence,
                    path,
                    "cost evidence is explicitly unmeasured",
                )]
            } else {
                vec![Diagnostic::fail(
                    format!("cost-evidence-unmeasured-{line_number}"),
                    ReportSurface::CostEvidence,
                    path,
                    "unmeasured cost evidence includes a reason and no amount",
                    format!("invalid {}", missing.join(", ")),
                    "do not attach a cost amount to unmeasured evidence",
                )]
            }
        }
    }
}

fn validate_measured_replay(
    root: &Path,
    evidence: &CostEvidence,
    line_number: usize,
    seen_event_digests: &mut BTreeSet<String>,
    measured_ranges: &mut BTreeMap<String, Vec<(usize, usize)>>,
) -> Option<Diagnostic> {
    let path = format!(
        "{}:{}",
        evidence.attribution_target.kind, evidence.attribution_target.id
    );
    let usage = evidence.usage.as_ref()?;
    let rates = evidence.pricing_rates.as_ref()?;
    let amount = evidence.amount.as_ref()?;
    let pricing = evidence.pricing.as_ref()?;
    if amount.currency != pricing.currency {
        return Some(fail_replay(
            line_number,
            path,
            format!(
                "amount currency {} does not match pricing currency {}",
                amount.currency, pricing.currency
            ),
        ));
    }
    let cached = usage.cached_input_tokens.unwrap_or(0);
    let expected_amount = match crate::cost_pricing::credit_micros(
        usage.input_tokens,
        cached,
        usage.output_tokens,
        usage.reasoning_tokens.unwrap_or(0),
        rates,
        pricing.reasoning_token_policy.as_deref(),
    ) {
        Ok(amount) => amount,
        Err(error) => return Some(fail_replay(line_number, path, error)),
    };
    if expected_amount != amount.amount_micros {
        return Some(fail_replay(
            line_number,
            path,
            format!(
                "amount {} does not match recomputed amount {}",
                amount.amount_micros, expected_amount
            ),
        ));
    }
    let pricing_path = root.join(&pricing.snapshot_path);
    match file_sha256(&pricing_path) {
        Ok(hash) if hash == pricing.snapshot_sha256 => {}
        Ok(hash) => {
            return Some(fail_replay(
                line_number,
                path,
                format!(
                    "pricing snapshot hash {hash} does not match {}",
                    pricing.snapshot_sha256
                ),
            ));
        }
        Err(error) => return Some(fail_replay(line_number, path, error)),
    }
    for contribution in &evidence.usage_contributions {
        if let Some(failure) = validate_contribution_replay(
            root,
            evidence,
            line_number,
            contribution,
            seen_event_digests,
            measured_ranges,
        ) {
            return Some(failure);
        }
    }
    None
}

fn validate_contribution_replay(
    root: &Path,
    evidence: &CostEvidence,
    line_number: usize,
    contribution: &UsageContribution,
    seen_event_digests: &mut BTreeSet<String>,
    measured_ranges: &mut BTreeMap<String, Vec<(usize, usize)>>,
) -> Option<Diagnostic> {
    let target_path = format!(
        "{}:{}",
        evidence.attribution_target.kind, evidence.attribution_target.id
    );
    let Some(pricing_identity) = pricing_identity(evidence) else {
        return Some(fail_replay(
            line_number,
            target_path,
            "missing pricing identity for measured replay".to_string(),
        ));
    };
    let digest_identity = format!(
        "{}:{}",
        contribution.selected_event_digest_sha256, pricing_identity
    );
    if !seen_event_digests.insert(digest_identity) {
        return Some(fail_replay(
            line_number,
            target_path,
            format!(
                "duplicate selected event digest {}",
                contribution.selected_event_digest_sha256
            ),
        ));
    }
    let source_path = Path::new(&contribution.source_path);
    let source_path = if source_path.is_absolute() {
        source_path.to_path_buf()
    } else {
        root.join(source_path)
    };
    match selected_event_digest(&source_path, contribution.start_line, contribution.end_line) {
        Ok((digest, event_count))
            if digest == contribution.selected_event_digest_sha256
                && digest == contribution.source_sha256
                && event_count == contribution.event_count => {}
        Ok((digest, event_count)) => {
            return Some(fail_replay(
                line_number,
                target_path,
                format!(
                    "selected event digest {digest} count {event_count} does not match receipt source digest {}, selected digest {}, count {}",
                    contribution.source_sha256,
                    contribution.selected_event_digest_sha256,
                    contribution.event_count
                ),
            ));
        }
        Err(error) => return Some(fail_replay(line_number, target_path, error)),
    }
    let key = format!(
        "{}:{}:{}:{}:{}",
        contribution.session_id,
        contribution.model_slug,
        evidence.attribution_target.kind,
        evidence.attribution_target.id,
        pricing_identity,
    );
    let ranges = measured_ranges.entry(key).or_default();
    if ranges
        .iter()
        .any(|(start, end)| contribution.start_line <= *end && contribution.end_line >= *start)
    {
        return Some(fail_replay(
            line_number,
            target_path,
            "overlapping measured token range".to_string(),
        ));
    }
    ranges.push((contribution.start_line, contribution.end_line));
    None
}

fn pricing_identity(evidence: &CostEvidence) -> Option<String> {
    let pricing = evidence.pricing.as_ref()?;
    let rates = evidence.pricing_rates.as_ref()?;
    let amount = evidence.amount.as_ref()?;
    Some(format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        evidence.provider.as_deref().unwrap_or(""),
        evidence.model_slug.as_deref().unwrap_or(""),
        pricing.source,
        pricing.version,
        pricing.currency,
        pricing.service_tier,
        pricing.snapshot_path,
        pricing.snapshot_sha256,
        rates.input_micros_per_million,
        rates.cached_input_micros_per_million,
        rates.output_micros_per_million,
        amount.currency,
        amount.amount_micros,
    ))
}

fn fail_replay(line_number: usize, path: String, actual: String) -> Diagnostic {
    Diagnostic::fail(
        format!("cost-evidence-replay-{line_number}"),
        ReportSurface::CostEvidence,
        path,
        "measured cost evidence replays from transcript and pricing sources",
        actual,
        "recreate measured cost evidence from canonical ingestion sources or record unmeasured evidence",
    )
}

fn file_sha256(path: &Path) -> Result<String> {
    let body = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(&body)))
}

fn selected_event_digest(
    path: &Path,
    start_line: usize,
    end_line: usize,
) -> Result<(String, usize)> {
    let body =
        fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut event_count = 0;
    for (index, line) in body.lines().enumerate() {
        let line_number = index + 1;
        if line_number < start_line || line_number > end_line || line.trim().is_empty() {
            continue;
        }
        let value = serde_json::from_str::<Value>(line)
            .map_err(|error| format!("parse transcript line {line_number}: {error}"))?;
        if value.get("type").and_then(Value::as_str) == Some("event_msg")
            && value.pointer("/payload/type").and_then(Value::as_str) == Some("token_count")
        {
            hasher.update(line_number.to_string().as_bytes());
            hasher.update(b":");
            hasher.update(line.as_bytes());
            hasher.update(b"\n");
            event_count += 1;
        }
    }
    if event_count == 0 {
        return Err("selected transcript range contains no token_count events".to_string());
    }
    Ok((format!("{:x}", hasher.finalize()), event_count))
}

fn require_nonempty<'a>(missing: &mut Vec<&'a str>, name: &'a str, value: Option<&String>) {
    if value.map(|value| value.trim().is_empty()).unwrap_or(true) {
        missing.push(name);
    }
}

fn no_receipts_check() -> Diagnostic {
    Diagnostic::pass(
        "cost-evidence-receipts-present",
        ReportSurface::CostEvidence,
        EVENTS_PATH,
        "no cost evidence receipts yet",
    )
}

fn format_text_report(report: &CheckReport) -> String {
    format!(
        "cost evidence check: pass={}, warn={}, fail={}, skip={}",
        report.summary.pass, report.summary.warn, report.summary.fail, report.summary.skip
    )
}

#[allow(dead_code)]
fn _command_marker() -> CliCommand {
    CliCommand::CostEvidenceCheck
}
