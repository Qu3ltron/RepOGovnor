use crate::model::{EventRecord, Result};
use crate::reports::RuntimeResult;
use crate::runtime::{append_event, timestamp};
use crate::schema::{
    CliCommand, CostAttributionKind, CostEvidence, CostEvidenceStatus, EventOutcome,
};
use std::path::Path;
use std::str::FromStr;

pub(crate) fn run_command(root: &Path, args: &[String]) -> RuntimeResult<String> {
    if matches!(args, [flag] if flag == "--help" || flag == "help") {
        return Ok(help());
    }
    let request = RecordRequest::parse(args)?;
    let target = crate::cost_targets::resolve(
        root,
        Some(request.target_kind),
        Some(request.target_id.as_str()),
    )?;
    let mut event = EventRecord::new(
        timestamp(),
        CliCommand::CostRecord,
        EventOutcome::Ok,
        0,
        format!("unmeasured cost evidence for {} {}", target.kind, target.id),
    );
    event.cost_evidence = Some(CostEvidence {
        status: CostEvidenceStatus::Unmeasured,
        evidence_source: request.evidence_source,
        attribution_target: target,
        provider: request.provider,
        model_slug: request.model_slug,
        usage: None,
        pricing: None,
        amount: None,
        pricing_rates: None,
        usage_contributions: Vec::new(),
        measurement_timestamp: None,
        estimation_method: None,
        unmeasured_reason: Some(request.reason),
        boundary_session_id: request.boundary_session_id,
        boundary_turn_id: request.boundary_turn_id,
        boundary_tool_use_id: request.boundary_tool_use_id,
    });
    append_event(root, event)?;
    if request.json {
        Ok(
            "{\"schema_version\":1,\"status\":\"recorded\",\"cost_status\":\"unmeasured\"}"
                .to_string(),
        )
    } else {
        Ok("cost record: unmeasured receipt appended".to_string())
    }
}

struct RecordRequest {
    target_kind: CostAttributionKind,
    target_id: String,
    reason: String,
    evidence_source: String,
    provider: Option<String>,
    model_slug: Option<String>,
    boundary_session_id: Option<String>,
    boundary_turn_id: Option<String>,
    boundary_tool_use_id: Option<String>,
    json: bool,
}

impl RecordRequest {
    fn parse(args: &[String]) -> Result<Self> {
        if args.first().map(String::as_str) != Some("unmeasured") {
            return Err(usage());
        }
        let mut target_kind = None;
        let mut target_id = None;
        let mut reason = None;
        let mut evidence_source = "manual-cost-record".to_string();
        let mut provider = None;
        let mut model_slug = None;
        let mut boundary_session_id = None;
        let mut boundary_turn_id = None;
        let mut boundary_tool_use_id = None;
        let mut json = false;
        let mut iter = args.iter().skip(1);
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--target-kind" => {
                    target_kind = Some(CostAttributionKind::from_str(
                        iter.next().ok_or_else(usage)?,
                    )?)
                }
                "--target-id" => target_id = Some(iter.next().ok_or_else(usage)?.to_string()),
                "--reason" => reason = Some(iter.next().ok_or_else(usage)?.to_string()),
                "--evidence-source" => evidence_source = iter.next().ok_or_else(usage)?.to_string(),
                "--provider" => provider = Some(iter.next().ok_or_else(usage)?.to_string()),
                "--model" | "--model-slug" => {
                    model_slug = Some(iter.next().ok_or_else(usage)?.to_string())
                }
                "--boundary-session-id" => {
                    boundary_session_id = Some(iter.next().ok_or_else(usage)?.to_string())
                }
                "--boundary-turn-id" => {
                    boundary_turn_id = Some(iter.next().ok_or_else(usage)?.to_string())
                }
                "--boundary-tool-use-id" => {
                    boundary_tool_use_id = Some(iter.next().ok_or_else(usage)?.to_string())
                }
                "--amount" => {
                    return Err("unmeasured cost records must not include --amount".to_string());
                }
                "--format" => {
                    if iter.next().map(String::as_str) != Some("json") {
                        return Err(usage());
                    }
                    json = true;
                }
                _ => return Err(usage()),
            }
        }
        let target_kind = target_kind.ok_or_else(|| "missing --target-kind <kind>".to_string())?;
        let target_id = target_id.ok_or_else(|| "missing --target-id <id>".to_string())?;
        let reason = reason
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "missing --reason <why-unmeasured>".to_string())?;
        Ok(Self {
            target_kind,
            target_id,
            reason,
            evidence_source,
            provider,
            model_slug,
            boundary_session_id,
            boundary_turn_id,
            boundary_tool_use_id,
            json,
        })
    }
}

fn usage() -> String {
    "usage: task-registry-flow cost-record unmeasured --target-kind <kind> --target-id <id> --reason <why-unmeasured> [--provider <name>] [--model <slug>] [--boundary-session-id <id>] [--boundary-turn-id <id>] [--boundary-tool-use-id <id>] [--format json]".to_string()
}

fn help() -> String {
    [
        "usage: task-registry-flow cost-record unmeasured --target-kind <kind> --target-id <id> --reason <why-unmeasured> [--provider <name>] [--model <slug>] [--boundary-session-id <id>] [--boundary-turn-id <id>] [--boundary-tool-use-id <id>] [--format json]",
        "",
        "Record explicit unmeasured cost evidence when measured provider usage cannot be published.",
    ]
    .join("\n")
}
