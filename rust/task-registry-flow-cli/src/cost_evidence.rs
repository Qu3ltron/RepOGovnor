use crate::metrics::receipt_value_hash;
use crate::model::{EVENTS_PATH, EventRecord, Result};
use crate::reports::{RuntimeFailure, RuntimeResult};
use crate::schema::{
    CheckReport, CliCommand, CostEvidence, CostEvidenceStatus, Diagnostic, FailureCode,
    ReportSurface,
};
use serde_json::Value;
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
        checks.extend(validate_cost_evidence(cost_evidence, line_number));
    }

    if checks.is_empty() {
        checks.push(no_receipts_check());
    }
    CheckReport::new(ReportSurface::CostEvidence, checks)
}

fn validate_cost_evidence(evidence: &CostEvidence, line_number: usize) -> Vec<Diagnostic> {
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
            if missing.is_empty() {
                vec![Diagnostic::pass(
                    format!("cost-evidence-measured-{line_number}"),
                    ReportSurface::CostEvidence,
                    path,
                    "measured cost evidence includes provider, model, usage, pricing, rates, timestamp, target, evidence source, amount, and contributions",
                )]
            } else {
                vec![Diagnostic::fail(
                    format!("cost-evidence-measured-{line_number}"),
                    ReportSurface::CostEvidence,
                    path,
                    "measured cost evidence includes all required fields and contribution evidence",
                    format!("missing {}", missing.join(", ")),
                    "record measured cost only from structured usage and pricing evidence",
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
