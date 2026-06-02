use crate::model::{EVENTS_PATH, EventRecord, Result};
use crate::reports::{RuntimeFailure, RuntimeResult};
use crate::schema::{
    CheckReport, CheckStatus, CliCommand, CostEvidence, CostEvidenceStatus, Diagnostic,
    FailureCode, ModelIdentityStatus, MutationAttributionDecision, ReportSurface,
};
use serde_json::Value;
use std::fs;
use std::path::Path;

pub(crate) fn run_command(root: &Path, args: &[String]) -> RuntimeResult<String> {
    let json = match args {
        [] => false,
        [flag, format] if flag == "--format" && format == "json" => true,
        _ => return Err("usage: task-registry-flow cost-coverage-check [--format json]".into()),
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
    let events = load_events(root)?;
    let costs = events
        .iter()
        .filter_map(|event| event.cost_evidence.as_ref())
        .collect::<Vec<_>>();
    let mut checks = Vec::new();
    for (index, event) in events.iter().enumerate() {
        let Some(agent) = event.agent_model_attribution.as_ref() else {
            continue;
        };
        let Some(mutation) = event.mutation_attribution.as_ref() else {
            continue;
        };
        if !matches!(
            mutation.decision,
            MutationAttributionDecision::Allowed | MutationAttributionDecision::Observed
        ) {
            continue;
        }
        if agent.identity_status != ModelIdentityStatus::Measured {
            checks.push(Diagnostic::fail(
                format!("cost-coverage-model-unmeasured-{}", index + 1),
                ReportSurface::CostCoverage,
                mutation.target_paths.join(","),
                "mutating agent model identity is measured before cost coverage",
                "mutation model identity unmeasured",
                "repair model attribution or record unmeasured cost against the mutation boundary",
            ));
            continue;
        }
        if costs.iter().any(|cost| cost_matches_agent(cost, agent)) {
            checks.push(Diagnostic::pass(
                format!("cost-coverage-mutation-{}", index + 1),
                ReportSurface::CostCoverage,
                mutation.target_paths.join(","),
                "mutation has measured or explicitly unmeasured cost evidence",
            ));
        } else {
            checks.push(Diagnostic::fail(
                format!("cost-coverage-mutation-{}", index + 1),
                ReportSurface::CostCoverage,
                mutation.target_paths.join(","),
                "mutation has measured or explicitly unmeasured cost evidence",
                "no matching cost receipt for session/turn/tool/model boundary",
                "run cost-ingest for measured usage or cost-record unmeasured with boundary ids",
            ));
        }
    }
    if checks.is_empty() {
        checks.push(Diagnostic::pass(
            "cost-coverage-no-mutations",
            ReportSurface::CostCoverage,
            EVENTS_PATH,
            "no model-attributed mutation receipts require cost coverage",
        ));
    }
    CheckReport::new(ReportSurface::CostCoverage, checks)
}

fn load_events(root: &Path) -> Result<Vec<EventRecord>> {
    let events_path = root.join(EVENTS_PATH);
    if !events_path.is_file() {
        return Ok(Vec::new());
    }
    let body = fs::read_to_string(&events_path)
        .map_err(|error| format!("read {}: {error}", events_path.display()))?;
    let mut events = Vec::new();
    for (index, line) in body.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let value = serde_json::from_str::<Value>(line)
            .map_err(|error| format!("parse receipt line {}: {error}", index + 1))?;
        if value.get("agent_model_attribution").is_none()
            && value.get("mutation_attribution").is_none()
            && value.get("cost_evidence").is_none()
        {
            continue;
        }
        let event = serde_json::from_value::<EventRecord>(value)
            .map_err(|error| format!("parse receipt schema line {}: {error}", index + 1))?;
        events.push(event);
    }
    Ok(events)
}

fn cost_matches_agent(cost: &CostEvidence, agent: &crate::schema::AgentModelAttribution) -> bool {
    if !matches!(
        cost.status,
        CostEvidenceStatus::Measured | CostEvidenceStatus::Unmeasured
    ) {
        return false;
    }
    if let (Some(cost_model), Some(agent_model)) = (&cost.model_slug, &agent.model_slug)
        && cost_model != agent_model
    {
        return false;
    }
    let session_match = match (&cost.boundary_session_id, &agent.session_id) {
        (Some(cost_session), Some(agent_session)) => cost_session == agent_session,
        (None, Some(agent_session)) => cost
            .usage_contributions
            .iter()
            .any(|contribution| contribution.session_id == *agent_session),
        (_, None) => true,
    };
    let turn_match = match (&cost.boundary_turn_id, &agent.turn_id) {
        (Some(cost_turn), Some(agent_turn)) => cost_turn == agent_turn,
        (None, Some(agent_turn)) => cost
            .usage_contributions
            .iter()
            .any(|contribution| contribution.turn_ids.iter().any(|turn| turn == agent_turn)),
        (_, None) => true,
    };
    let tool_match = match (&cost.boundary_tool_use_id, &agent.tool_use_id) {
        (Some(cost_tool), Some(agent_tool)) => cost_tool == agent_tool,
        (None, Some(agent_tool)) => cost.usage_contributions.iter().any(|contribution| {
            contribution
                .tool_use_ids
                .iter()
                .any(|tool| tool == agent_tool)
        }),
        (_, None) => true,
    };
    session_match && turn_match && tool_match
}

fn format_text_report(report: &CheckReport) -> String {
    let status = if report
        .checks
        .iter()
        .any(|check| check.status == CheckStatus::Fail)
    {
        "fail"
    } else {
        "ok"
    };
    format!(
        "cost coverage check: {status}, pass={}, fail={}",
        report.summary.pass, report.summary.fail
    )
}

#[allow(dead_code)]
fn _command_marker() -> CliCommand {
    CliCommand::CostCoverageCheck
}
