use crate::model::Result;
use crate::registry_io::load_registry;
use crate::schema::{CostAttributionKind, CostAttributionTarget};
use std::path::Path;
use std::process::Command;

pub(crate) fn resolve(
    root: &Path,
    target_kind: Option<CostAttributionKind>,
    target_id: Option<&str>,
) -> Result<CostAttributionTarget> {
    let kind = target_kind.ok_or_else(|| "missing --target-kind <kind>".to_string())?;
    let id = target_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "missing --target-id <id>".to_string())?;
    let resolved_id = match kind {
        CostAttributionKind::Commit => resolve_commit(root, id)?,
        CostAttributionKind::Plan => resolve_plan(root, id)?,
        CostAttributionKind::Task => resolve_task(root, id)?,
        CostAttributionKind::VerifierRun => resolve_verifier(root, id)?,
        CostAttributionKind::LandingAttempt
        | CostAttributionKind::Retry
        | CostAttributionKind::ReleaseCycle
        | CostAttributionKind::Session => id.to_string(),
    };
    Ok(CostAttributionTarget {
        kind,
        id: resolved_id,
    })
}

fn resolve_commit(root: &Path, commit: &str) -> Result<String> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--verify")
        .arg(format!("{commit}^{{commit}}"))
        .current_dir(root)
        .output()
        .map_err(|error| format!("run git rev-parse: {error}"))?;
    if !output.status.success() {
        return Err(format!("unknown governed cost target commit {commit}"));
    }
    let sha = String::from_utf8(output.stdout)
        .map_err(|error| format!("decode git rev-parse output: {error}"))?;
    Ok(sha.trim().to_string())
}

fn resolve_plan(root: &Path, plan_id: &str) -> Result<String> {
    let registry = load_registry(root)?;
    if registry.plans.iter().any(|plan| plan.plan_id == plan_id) {
        Ok(plan_id.to_string())
    } else {
        Err(format!("unknown governed cost target plan {plan_id}"))
    }
}

fn resolve_task(root: &Path, task_id: &str) -> Result<String> {
    let registry = load_registry(root)?;
    if registry.tasks.iter().any(|task| task.task_id == task_id) {
        Ok(task_id.to_string())
    } else {
        Err(format!("unknown governed cost target task {task_id}"))
    }
}

fn resolve_verifier(root: &Path, behavior_id: &str) -> Result<String> {
    let registry = load_registry(root)?;
    if registry
        .tasks
        .iter()
        .any(|task| task.behavior_ids.iter().any(|id| id == behavior_id))
    {
        Ok(behavior_id.to_string())
    } else {
        Err(format!(
            "unknown governed cost target verifier-run {behavior_id}"
        ))
    }
}
