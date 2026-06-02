use crate::model::{EVENTS_PATH, EventRecord, MetricsReport, Result};
use crate::runtime::{discover_manifests, load_registry};
use crate::schema::{EventOutcome, ModelIdentityStatus, TaskStatus};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub(crate) fn metrics(root: &Path) -> Result<MetricsReport> {
    let registry = load_registry(root)?;
    let manifests = discover_manifests(root)?;
    let mut report = MetricsReport {
        plans: registry.plans.len(),
        tasks: registry.tasks.len(),
        planned: 0,
        active: 0,
        blocked: 0,
        deferred: 0,
        completed: 0,
        cancelled: 0,
        manifests: manifests.len(),
        events: 0,
        failed_events: 0,
        mutation_denials: 0,
        model_attributed_mutation_events: 0,
        model_unmeasured_mutation_events: 0,
        malformed_events: 0,
        chained_events: 0,
        unchained_events: 0,
        receipt_chain_breaks: 0,
    };
    for task in &registry.tasks {
        match task.status {
            TaskStatus::Planned => report.planned += 1,
            TaskStatus::Active => report.active += 1,
            TaskStatus::Blocked => report.blocked += 1,
            TaskStatus::Deferred => report.deferred += 1,
            TaskStatus::Completed => report.completed += 1,
            TaskStatus::Cancelled => report.cancelled += 1,
        }
    }
    count_receipts(root, &mut report)?;
    Ok(report)
}

fn count_receipts(root: &Path, report: &mut MetricsReport) -> Result<()> {
    let events_path = root.join(EVENTS_PATH);
    if !events_path.is_file() {
        return Ok(());
    }
    let body = fs::read_to_string(&events_path)
        .map_err(|error| format!("read {}: {error}", events_path.display()))?;
    let mut previous_hash = None;
    for line in body.lines().filter(|line| !line.trim().is_empty()) {
        report.events += 1;
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            report.malformed_events += 1;
            report.failed_events += 1;
            previous_hash = None;
            continue;
        };
        let Ok(current_hash) = receipt_value_hash(&value) else {
            report.malformed_events += 1;
            report.failed_events += 1;
            previous_hash = None;
            continue;
        };
        match serde_json::from_value::<EventRecord>(value.clone()) {
            Ok(event) => {
                if event.outcome == EventOutcome::Error {
                    report.failed_events += 1;
                }
                if event.outcome == EventOutcome::MutationDenied {
                    report.mutation_denials += 1;
                }
                if event.mutation_attribution.is_some() {
                    match event
                        .agent_model_attribution
                        .as_ref()
                        .map(|attribution| attribution.identity_status)
                    {
                        Some(ModelIdentityStatus::Measured) => {
                            report.model_attributed_mutation_events += 1;
                        }
                        _ => {
                            report.model_unmeasured_mutation_events += 1;
                        }
                    }
                }
                if let Some(event_hash) = &event.event_hash_sha256 {
                    report.chained_events += 1;
                    if event_hash != &current_hash
                        || event.previous_event_hash_sha256 != previous_hash
                    {
                        report.receipt_chain_breaks += 1;
                        report.failed_events += 1;
                    }
                } else {
                    report.unchained_events += 1;
                    report.receipt_chain_breaks += 1;
                    report.failed_events += 1;
                }
                previous_hash = Some(current_hash);
            }
            Err(_) => {
                report.malformed_events += 1;
                report.failed_events += 1;
                previous_hash = Some(current_hash);
            }
        }
    }
    Ok(())
}

pub(crate) fn format_metrics(report: &MetricsReport) -> String {
    format!(
        "Task registry metrics: plans={}, tasks={}, manifests={}, planned={}, active={}, completed={}, deferred={}, blocked={}, cancelled={}, events={}, failed_events={}, mutation_denials={}, model_attributed_mutation_events={}, model_unmeasured_mutation_events={}, malformed_events={}, chained_events={}, unchained_events={}, receipt_chain_breaks={}",
        report.plans,
        report.tasks,
        report.manifests,
        report.planned,
        report.active,
        report.completed,
        report.deferred,
        report.blocked,
        report.cancelled,
        report.events,
        report.failed_events,
        report.mutation_denials,
        report.model_attributed_mutation_events,
        report.model_unmeasured_mutation_events,
        report.malformed_events,
        report.chained_events,
        report.unchained_events,
        report.receipt_chain_breaks
    )
}

pub(crate) fn receipt_value_hash(value: &Value) -> Result<String> {
    let mut canonical = value.clone();
    if let Some(object) = canonical.as_object_mut() {
        object.remove("event_hash_sha256");
    }
    let body = serde_json::to_string(&canonical)
        .map_err(|error| format!("serialize receipt for hash: {error}"))?;
    Ok(format!("{:x}", Sha256::digest(body.as_bytes())))
}
