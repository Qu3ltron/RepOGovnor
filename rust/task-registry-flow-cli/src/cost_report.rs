use crate::model::{EVENTS_PATH, EventRecord, Result};
use crate::reports::{RuntimeFailure, RuntimeResult};
use crate::schema::{CostEvidenceStatus, FailureCode};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize)]
pub(crate) struct CostReport {
    pub(crate) schema_version: u8,
    pub(crate) measured_targets: usize,
    pub(crate) unmeasured_targets: usize,
    pub(crate) invalid_events: usize,
    pub(crate) entries: Vec<CostReportEntry>,
}

#[derive(Debug, Serialize)]
pub(crate) struct CostReportEntry {
    pub(crate) target_kind: String,
    pub(crate) target_id: String,
    pub(crate) status: String,
    pub(crate) provider: Option<String>,
    pub(crate) model_slug: Option<String>,
    pub(crate) pricing_version: Option<String>,
    pub(crate) service_tier: Option<String>,
    pub(crate) currency: Option<String>,
    pub(crate) measured_amount_micros: Option<u64>,
    pub(crate) receipt_count: usize,
    pub(crate) unmeasured_reasons: Vec<String>,
}

pub(crate) fn run_command(root: &Path, args: &[String]) -> RuntimeResult<String> {
    let json = match args {
        [] => false,
        [flag, format] if flag == "--format" && format == "json" => true,
        _ => return Err("usage: task-registry-flow cost-report [--format json]".into()),
    };
    let report = report(root)?;
    if json {
        let output = serde_json::to_string_pretty(&report)
            .map_err(|error| format!("serialize cost report: {error}"))?;
        if report.invalid_events > 0 {
            Err(RuntimeFailure::json(FailureCode::DiagnosticReport, output))
        } else {
            Ok(output)
        }
    } else {
        Ok(format_text_report(&report))
    }
}

pub(crate) fn report(root: &Path) -> Result<CostReport> {
    let events_path = root.join(EVENTS_PATH);
    if !events_path.is_file() {
        return Ok(CostReport {
            schema_version: 1,
            measured_targets: 0,
            unmeasured_targets: 0,
            invalid_events: 0,
            entries: Vec::new(),
        });
    }
    let body = fs::read_to_string(&events_path)
        .map_err(|error| format!("read {}: {error}", events_path.display()))?;
    let mut invalid_events = 0;
    let mut entries = BTreeMap::<String, CostReportEntry>::new();
    for line in body.lines().filter(|line| !line.trim().is_empty()) {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            invalid_events += 1;
            continue;
        };
        if value.get("cost_evidence").is_none() {
            continue;
        }
        let Ok(event) = serde_json::from_value::<EventRecord>(value) else {
            invalid_events += 1;
            continue;
        };
        let Some(evidence) = event.cost_evidence else {
            continue;
        };
        let status = evidence.status.as_str().to_string();
        let pricing = evidence.pricing.as_ref();
        let amount = evidence.amount.as_ref();
        let key = format!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            evidence.attribution_target.kind,
            evidence.attribution_target.id,
            status,
            evidence.provider.as_deref().unwrap_or(""),
            evidence.model_slug.as_deref().unwrap_or(""),
            pricing
                .map(|pricing| pricing.version.as_str())
                .unwrap_or("")
        );
        let entry = entries.entry(key).or_insert_with(|| CostReportEntry {
            target_kind: evidence.attribution_target.kind.to_string(),
            target_id: evidence.attribution_target.id.clone(),
            status: status.clone(),
            provider: evidence.provider.clone(),
            model_slug: evidence.model_slug.clone(),
            pricing_version: pricing.map(|pricing| pricing.version.clone()),
            service_tier: pricing.map(|pricing| pricing.service_tier.clone()),
            currency: amount
                .map(|amount| amount.currency.clone())
                .or_else(|| pricing.map(|pricing| pricing.currency.clone())),
            measured_amount_micros: if evidence.status == CostEvidenceStatus::Measured {
                Some(0)
            } else {
                None
            },
            receipt_count: 0,
            unmeasured_reasons: Vec::new(),
        });
        entry.receipt_count += 1;
        if evidence.status == CostEvidenceStatus::Measured
            && let Some(amount) = amount
            && let Some(total) = &mut entry.measured_amount_micros
        {
            *total = total.saturating_add(amount.amount_micros);
        }
        if evidence.status == CostEvidenceStatus::Unmeasured
            && let Some(reason) = evidence.unmeasured_reason
            && !entry.unmeasured_reasons.contains(&reason)
        {
            entry.unmeasured_reasons.push(reason);
        }
    }
    let entries = entries.into_values().collect::<Vec<_>>();
    let measured_targets = entries
        .iter()
        .filter(|entry| entry.status == CostEvidenceStatus::Measured.as_str())
        .count();
    let unmeasured_targets = entries
        .iter()
        .filter(|entry| entry.status == CostEvidenceStatus::Unmeasured.as_str())
        .count();
    Ok(CostReport {
        schema_version: 1,
        measured_targets,
        unmeasured_targets,
        invalid_events,
        entries,
    })
}

fn format_text_report(report: &CostReport) -> String {
    format!(
        "cost report: measured_targets={}, unmeasured_targets={}, invalid_events={}",
        report.measured_targets, report.unmeasured_targets, report.invalid_events
    )
}
