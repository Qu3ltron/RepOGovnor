use crate::metrics::receipt_value_hash;
use crate::model::{EVENTS_PATH, EventRecord, Result};
use crate::reports::{RuntimeFailure, RuntimeResult};
use crate::schema::{
    CheckReport, CliCommand, Diagnostic, EventOutcome, FailureCode, HookFormat,
    ModelIdentityStatus, ReportSurface,
};
use serde_json::Value;
use std::fs;
use std::path::Path;

pub(crate) fn run_command(root: &Path, args: &[String]) -> RuntimeResult<String> {
    let json = match args {
        [] => false,
        [flag, format] if flag == "--format" && format == "json" => true,
        _ => {
            return Err("usage: task-registry-flow model-attribution-check [--format json]".into());
        }
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
        checks.push(Diagnostic::pass(
            "model-attribution-receipts-present",
            ReportSurface::ModelAttribution,
            EVENTS_PATH,
            "no mutation attribution receipts yet",
        ));
        return CheckReport::new(ReportSurface::ModelAttribution, checks);
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
                    format!("model-attribution-json-{line_number}"),
                    ReportSurface::ModelAttribution,
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
                format!("model-attribution-hashable-{line_number}"),
                ReportSurface::ModelAttribution,
                EVENTS_PATH,
                "hashable receipt event",
                format!("line {line_number} is not hashable"),
                "repair malformed receipt event fields",
            ));
            continue;
        }
        let Ok(event) = serde_json::from_value::<EventRecord>(value) else {
            continue;
        };
        let Some(mutation_attribution) = &event.mutation_attribution else {
            continue;
        };
        let path = mutation_attribution
            .target_paths
            .first()
            .cloned()
            .unwrap_or_else(|| EVENTS_PATH.to_string());
        let Some(agent) = &event.agent_model_attribution else {
            checks.push(Diagnostic::fail(
                format!("model-attribution-agent-{line_number}"),
                ReportSurface::ModelAttribution,
                path,
                "mutation attribution receipt includes agent model attribution",
                "missing agent_model_attribution",
                "record provider-neutral agent model attribution with mutation attribution",
            ));
            continue;
        };
        match (mutation_attribution.hook_format, agent.identity_status) {
            (HookFormat::Codex, ModelIdentityStatus::Measured) => {
                if agent.model_slug.as_deref().unwrap_or("").trim().is_empty()
                    || agent.session_id.as_deref().unwrap_or("").trim().is_empty()
                    || agent.turn_id.as_deref().unwrap_or("").trim().is_empty()
                    || agent.tool_use_id.as_deref().unwrap_or("").trim().is_empty()
                {
                    checks.push(Diagnostic::fail(
                        format!("model-attribution-codex-measured-{line_number}"),
                        ReportSurface::ModelAttribution,
                        path,
                        "measured Codex mutation attribution includes model/session/turn/tool_use",
                        "measured Codex receipt is missing required identity evidence",
                        "record all Codex common and PreToolUse/PostToolUse identity fields",
                    ));
                } else {
                    checks.push(Diagnostic::pass(
                        format!("model-attribution-codex-measured-{line_number}"),
                        ReportSurface::ModelAttribution,
                        path,
                        "measured Codex mutation attribution includes required identity evidence",
                    ));
                }
            }
            (HookFormat::Codex, ModelIdentityStatus::Unmeasured)
                if event.outcome == EventOutcome::MutationDenied =>
            {
                checks.push(Diagnostic::warn(
                    format!("model-attribution-codex-denied-unmeasured-{line_number}"),
                    ReportSurface::ModelAttribution,
                    path,
                    "denied Codex mutation lacked measured identity evidence",
                ));
            }
            (_, ModelIdentityStatus::Unmeasured) => {
                checks.push(Diagnostic::warn(
                    format!("model-attribution-unmeasured-{line_number}"),
                    ReportSurface::ModelAttribution,
                    path,
                    "mutation attribution remains unmeasured for this adapter",
                ));
            }
            (_, ModelIdentityStatus::Measured) => {
                checks.push(Diagnostic::pass(
                    format!("model-attribution-measured-{line_number}"),
                    ReportSurface::ModelAttribution,
                    path,
                    "measured mutation attribution recorded",
                ));
            }
        }
    }

    if checks.is_empty() {
        checks.push(Diagnostic::pass(
            "model-attribution-receipts-present",
            ReportSurface::ModelAttribution,
            EVENTS_PATH,
            "no mutation attribution receipts yet",
        ));
    }
    CheckReport::new(ReportSurface::ModelAttribution, checks)
}

fn format_text_report(report: &CheckReport) -> String {
    format!(
        "model attribution check: pass={}, warn={}, fail={}, skip={}",
        report.summary.pass, report.summary.warn, report.summary.fail, report.summary.skip
    )
}

#[allow(dead_code)]
fn _command_marker() -> CliCommand {
    CliCommand::ModelAttributionCheck
}
