use chrono::Local;
#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;

use crate::metrics::{format_metrics, metrics, receipt_value_hash};
use crate::model::*;
use crate::mutation_hook::verify_mutation_hook;
use crate::reports::RuntimeResult;
use crate::schema::{CliCommand, FailureCode, HookFormat, TaskStatus};
use crate::{install, landing, policy, release_checks, source_limit, status_checks};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

use fs2::FileExt;

// ---------------------------------------------------------------------------
// Re-exports: moved functions remain accessible at crate::runtime::* for
// backward compatibility with tests (use super::*) and external callers.
// ---------------------------------------------------------------------------
pub(crate) use crate::activation::{activate_plan, defer_task, update_task_status};
pub(crate) use crate::registry_io::load_registry;
pub(crate) use crate::verifiers::verify_behaviors;

// ---------------------------------------------------------------------------
// Dispatcher and command handlers
// ---------------------------------------------------------------------------

pub(crate) fn run(mut args: Vec<String>) -> RuntimeResult<String> {
    if args.is_empty() {
        return Err(usage().into());
    }
    let command = CliCommand::from_str(&args.remove(0)).map_err(|_| usage())?;
    let root = Path::new(".");
    match command {
        CliCommand::Validate => {
            if !args.is_empty() {
                return Err(usage().into());
            }
            let report = validate_all(root)?;
            Ok(format!(
                "task registry validate ok: {} plans, {} tasks, {} manifests",
                report.registry_plan_count, report.registry_task_count, report.manifest_count
            ))
        }
        CliCommand::Activate => {
            if args.len() != 1 {
                return Err(usage().into());
            }
            let plan_path = args.first().ok_or_else(usage)?;
            activate_plan(root, plan_path)?;
            Ok(format!("PLAN_ACTIVATE {plan_path} ok"))
        }
        CliCommand::Status => {
            if args.len() != 2 {
                return Err(usage().into());
            }
            let task_id = args.first().ok_or_else(usage)?;
            let status = args.get(1).ok_or_else(usage)?;
            update_task_status(root, task_id, status)?;
            Ok(format!("TASK_STATUS {task_id} {status} ok"))
        }
        CliCommand::Defer => {
            if args.len() != 3 {
                return Err(usage().into());
            }
            let task_id = args.first().ok_or_else(usage)?;
            let basis = args.get(1).ok_or_else(usage)?;
            let reactivation = args.get(2).ok_or_else(usage)?;
            defer_task(root, task_id, basis, reactivation)?;
            Ok(format!("TASK_DEFER {task_id} ok"))
        }
        CliCommand::Report => {
            if args.len() != 1 {
                return Err(usage().into());
            }
            let plan_id = args.first().ok_or_else(usage)?;
            let report = report_plan(root, plan_id)?;
            Ok(format_report(&report))
        }
        CliCommand::ReviewerReport => Ok(crate::reviewer_report::run(root, &args)?),
        CliCommand::VersionCheck => crate::version_check::run_command(root, &args),
        CliCommand::BacklogCheck => crate::backlog_check::run_command(root, &args),
        CliCommand::ArchiveCompleted => {
            if !args.is_empty() {
                return Err(usage().into());
            }
            archive_completed(root)?;
            Ok("TASK_ARCHIVE_COMPLETED ok".to_string())
        }
        CliCommand::VerifyBehaviors => {
            if args.len() > 1 {
                return Err(usage().into());
            }
            let filter = args.first().map(String::as_str);
            let count = verify_behaviors(root, filter)?;
            Ok(format!("TASK_VERIFY_BEHAVIORS ok: {count} confirmations"))
        }
        CliCommand::VerifyLanding => Ok(landing::run_command(root, &args)?),
        CliCommand::VerifyChain => crate::verify_chain::run_verify_chain(root, &args),
        CliCommand::VerifyMutationHook => {
            let format = parse_hook_format(&args)?;
            verify_mutation_hook(root, format)?;
            Ok("TASK_VERIFY_MUTATION_HOOK ok".to_string())
        }
        CliCommand::ModelAttributionCheck => crate::model_attribution::run_command(root, &args),
        CliCommand::CostEvidenceCheck => crate::cost_evidence::run_command(root, &args),
        CliCommand::CostIngest => crate::cost_ingest::run_command(root, &args),
        CliCommand::Metrics => {
            let json = match args.as_slice() {
                [] => false,
                [flag, format] if flag == "--format" && format == "json" => true,
                _ => return Err(usage().into()),
            };
            let report = metrics(root)?;
            if json {
                let output = serde_json::to_string_pretty(&report)
                    .map_err(|error| format!("serialize metrics report: {error}"))?;
                if report.receipt_chain_breaks > 0 {
                    Err(crate::reports::RuntimeFailure::json(
                        FailureCode::DiagnosticReport,
                        output,
                    ))
                } else {
                    Ok(output)
                }
            } else {
                Ok(format_metrics(&report))
            }
        }
        CliCommand::SourceLimit => source_limit::run_command(root, &args),
        CliCommand::ReleaseCheck => release_checks::run_command(root, &args),
        CliCommand::Install => install_command(root, &args),
        CliCommand::StatusCheck => status_checks::run_command(root, &args),
        CliCommand::Usage => Err(usage().into()),
    }
}

fn usage() -> String {
    "usage: task-registry-flow {validate|activate <docs/plans/file.md>|status <task_id> <status>|defer <task_id> <basis> <reactivation>|report <plan_id>|reviewer-report [--format text|markdown]|version-check {validate|next <plan_id>|prerelease <plan_id> --rc <n>|release-check} [--format json]|backlog-check [--format json]|archive-completed|verify-behaviors [plan_id|task_id]|verify-landing [--plan-id <plan_id>] --changed-files <path>...|verify-chain [--format json] [--repair]|verify-mutation-hook [--format codex|antigravity|cursor|claude]|model-attribution-check [--format json]|cost-evidence-check [--format json]|cost-ingest codex-transcript (--latest|--transcript-path <path>) --pricing-snapshot <path> --commit <sha|HEAD> [--since-line <n>] [--until-line <n>] [--append-receipt] [--format json]|metrics|source-limit check|source-limit plan|release-check {required|version|tracked|all} [--format json]|install plan [--format json]|status-check [--format json]}".to_string()
}

fn install_command(root: &Path, args: &[String]) -> RuntimeResult<String> {
    let json = match args {
        [subcommand] if subcommand == "plan" => false,
        [subcommand, flag, format]
            if subcommand == "plan" && flag == "--format" && format == "json" =>
        {
            true
        }
        _ => return Err("usage: task-registry-flow install plan [--format json]".into()),
    };
    let manifest_body = fs::read_to_string(root.join("MANIFEST.toml"))
        .map_err(|error| format!("read MANIFEST.toml: {error}"))?;
    let manifest_policy = policy::parse_manifest_policy(&manifest_body)?;
    install::validate_action_vocabulary(&manifest_policy.install_policy.action_vocabulary)?;
    let report = install::action_report("MANIFEST.toml", "aligned")?;
    if json {
        Ok(serde_json::to_string(&report)
            .map_err(|error| format!("serialize install report: {error}"))?)
    } else {
        Ok(format!("{}: {}", report.path, report.action))
    }
}

fn validate_all(root: &Path) -> Result<ValidationReport> {
    source_limit::check(root)?;
    let registry = load_registry(root)?;
    let manifests = discover_manifests(root)?;
    crate::validation::validate_registry_with_manifests(root, &registry, &manifests)?;
    Ok(ValidationReport {
        registry_plan_count: registry.plans.len(),
        registry_task_count: registry.tasks.len(),
        manifest_count: manifests.len(),
    })
}

fn archive_completed(root: &Path) -> Result<()> {
    let registry = load_registry(root)?;
    crate::registry_io::save_registry(root, &registry)
}

fn parse_hook_format(args: &[String]) -> Result<HookFormat> {
    if args.is_empty() {
        return Ok(HookFormat::Antigravity);
    }
    if args.len() == 2 && args[0] == "--format" {
        return HookFormat::from_str(&args[1]);
    }
    Err(usage())
}

// ---------------------------------------------------------------------------
// Plan reporting
// ---------------------------------------------------------------------------

fn report_plan(root: &Path, plan_id: &str) -> Result<PlanReport> {
    let registry = load_registry(root)?;
    if !registry.plans.iter().any(|plan| plan.plan_id == plan_id) {
        return Err(format!("missing plan_id {plan_id}"));
    }
    let mut report = PlanReport {
        plan_id: plan_id.to_string(),
        completed: 0,
        deferred: 0,
        blocked: 0,
        cancelled: 0,
        remaining: 0,
        deferred_or_blocked: Vec::new(),
    };
    for task in registry.tasks.iter().filter(|task| task.plan_id == plan_id) {
        match task.status {
            TaskStatus::Completed => report.completed += 1,
            TaskStatus::Deferred => {
                report.deferred += 1;
                report.deferred_or_blocked.push((
                    task.task_id.clone(),
                    task.title.clone(),
                    task.deferral_governance_basis
                        .clone()
                        .unwrap_or_else(|| task.reason.clone()),
                ));
            }
            TaskStatus::Blocked => {
                report.blocked += 1;
                report.deferred_or_blocked.push((
                    task.task_id.clone(),
                    task.title.clone(),
                    task.reason.clone(),
                ));
            }
            TaskStatus::Cancelled => report.cancelled += 1,
            _ => report.remaining += 1,
        }
    }
    Ok(report)
}

fn format_report(report: &PlanReport) -> String {
    let mut lines = vec![format!(
        "Task registry: {} completed, {} deferred, {} blocked for {}.",
        report.completed, report.deferred, report.blocked, report.plan_id
    )];
    if report.cancelled > 0 || report.remaining > 0 {
        lines.push(format!(
            "Additional status: {} cancelled, {} planned/active remaining.",
            report.cancelled, report.remaining
        ));
    }
    for (task_id, title, reason) in &report.deferred_or_blocked {
        lines.push(format!("- {task_id}: {title} -- {reason}"));
    }
    lines.join("\n")
}

// ---------------------------------------------------------------------------
// Manifest discovery and loading
// ---------------------------------------------------------------------------

pub(crate) fn discover_manifests(root: &Path) -> Result<Vec<ActivatedManifest>> {
    let mut manifests = Vec::new();
    let plans_root = root.join(PLAN_DIR);
    if !plans_root.exists() {
        return Ok(manifests);
    }
    let mut markdown_files = Vec::new();
    collect_markdown_files(&plans_root, &mut markdown_files)?;
    markdown_files.sort();
    for path in markdown_files {
        let relative = relative_path(root, &path)?;
        let body = fs::read_to_string(&path)
            .map_err(|error| format!("read {}: {error}", path.display()))?;
        if body.lines().any(|line| line.trim() == "## Task Manifest") {
            manifests.push(parse_manifest_from_body(&relative, &body)?);
        }
    }
    Ok(manifests)
}

fn collect_markdown_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).map_err(|error| format!("read {}: {error}", dir.display()))? {
        let entry = entry.map_err(|error| format!("read {} entry: {error}", dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_markdown_files(&path, out)?;
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("md") {
            out.push(path);
        }
    }
    Ok(())
}

pub(crate) fn load_manifest(root: &Path, plan_path: &str) -> Result<ActivatedManifest> {
    use crate::validation::validate_plan_path;
    validate_plan_path(plan_path)?;
    let path = root.join(plan_path);
    let body = fs::read_to_string(&path).map_err(|error| format!("read {plan_path}: {error}"))?;
    parse_manifest_from_body(plan_path, &body)
}

fn parse_manifest_from_body(plan_path: &str, body: &str) -> Result<ActivatedManifest> {
    let manifest_text = task_manifest_table(plan_path, body)?;
    let manifest = toml::from_str::<PlanManifest>(&manifest_text)
        .map_err(|error| format!("parse Task Manifest in {plan_path}: {error}"))?;
    crate::validation::validate_manifest(&manifest)?;
    Ok(ActivatedManifest {
        plan_path: plan_path.to_string(),
        plan_hash_sha256: normalized_hash(body),
        plan_body: body.to_string(),
        manifest,
    })
}

fn task_manifest_table(plan_path: &str, plan_body: &str) -> Result<String> {
    let heading_count = plan_body
        .lines()
        .filter(|line| line.trim() == "## Task Manifest")
        .count();
    if heading_count == 0 {
        return Err(format!("{plan_path} missing ## Task Manifest"));
    }
    if heading_count > 1 {
        return Err(format!("{plan_path} has multiple Task Manifest sections"));
    }
    let manifest_start = plan_body
        .find("## Task Manifest")
        .ok_or_else(|| format!("{plan_path} missing ## Task Manifest"))?;
    let after_heading = &plan_body[manifest_start..];
    let fence_start = after_heading
        .find("```toml")
        .ok_or_else(|| format!("{plan_path} missing ```toml fence in Task Manifest"))?;
    let manifest = &after_heading[fence_start + "```toml".len()..];
    let fence_end = manifest
        .find("```")
        .ok_or_else(|| format!("{plan_path} missing closing fence in Task Manifest"))?;
    Ok(manifest[..fence_end].to_string())
}

pub(crate) fn behavior_map(manifests: &[ActivatedManifest]) -> Result<BTreeMap<String, Behavior>> {
    let mut map = BTreeMap::new();
    for manifest in manifests {
        for behavior in &manifest.manifest.behaviors {
            if map
                .insert(behavior.behavior_id.clone(), behavior.clone())
                .is_some()
            {
                return Err(format!("duplicate behavior_id {}", behavior.behavior_id));
            }
        }
    }
    Ok(map)
}

// ---------------------------------------------------------------------------
// Event / receipt I/O
// ---------------------------------------------------------------------------

pub(crate) fn append_event(root: &Path, mut event: EventRecord) -> Result<()> {
    let path = root.join(EVENTS_PATH);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("create {}: {error}", parent.display()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .append(true)
        .open(&path)
        .map_err(|error| format!("open {}: {error}", path.display()))?;
    file.try_lock_exclusive()
        .map_err(|_| "events file is locked by another process; retry in a moment".to_string())?;
    event.previous_event_hash_sha256 = previous_receipt_hash(&path)?;
    event.event_hash_sha256 = None;
    let value = serde_json::to_value(&event)
        .map_err(|error| format!("serialize event for hash: {error}"))?;
    event.event_hash_sha256 = Some(receipt_value_hash(&value)?);
    let line =
        serde_json::to_string(&event).map_err(|error| format!("serialize event: {error}"))?;
    writeln!(file, "{line}").map_err(|error| format!("write {}: {error}", path.display()))?;
    // fsync after append — ensures the new event is durable before returning.
    file.sync_all()
        .map_err(|error| format!("sync {}: {error}", path.display()))?;
    file.unlock()
        .map_err(|error| format!("unlock {}: {error}", path.display()))
}

fn previous_receipt_hash(path: &Path) -> Result<Option<String>> {
    if !path.is_file() {
        return Ok(None);
    }
    let body =
        fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let Some(line) = body.lines().rev().find(|line| !line.trim().is_empty()) else {
        return Ok(None);
    };
    let value = serde_json::from_str::<Value>(line)
        .map_err(|error| format!("parse previous receipt for hash: {error}"))?;
    receipt_value_hash(&value).map(Some)
}

// ---------------------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------------------

pub(crate) fn normalize_relative_path(path: &str) -> Result<String> {
    if path
        .chars()
        .any(|value| matches!(value, '*' | '?' | '[' | ']' | '{' | '}'))
    {
        return Err(format!("path must not contain glob metacharacters: {path}"));
    }
    let path = path.replace('\\', "/");
    let mut parts = Vec::new();
    for component in Path::new(&path).components() {
        match component {
            Component::Normal(value) => parts.push(value.to_string_lossy().to_string()),
            Component::CurDir => {}
            Component::ParentDir => return Err(format!("path must not contain '..': {path}")),
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!("path must be relative: {path}"));
            }
        }
    }
    if parts.is_empty() {
        return Err("path must not be empty".to_string());
    }
    Ok(parts.join("/"))
}

pub(crate) fn normalized_file_hash(path: &Path) -> Result<String> {
    let body =
        fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    Ok(normalized_hash(&body))
}

pub(crate) fn normalized_hash(body: &str) -> String {
    let body = body.replace("\r\n", "\n").replace('\r', "\n");
    let mut normalized = String::new();
    for line in body.lines() {
        normalized.push_str(line.trim_end());
        normalized.push('\n');
    }
    Sha256::digest(normalized.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(crate) fn today() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

pub(crate) fn timestamp() -> String {
    Local::now().to_rfc3339()
}

pub(crate) fn truncate_detail(detail: &str) -> String {
    const LIMIT: usize = 500;
    let mut result = detail.replace('\n', " ");
    if result.len() > LIMIT {
        result.truncate(LIMIT);
        result.push_str("...");
    }
    result
}

fn relative_path(root: &Path, path: &Path) -> Result<String> {
    path.strip_prefix(root)
        .map_err(|error| format!("strip prefix {}: {error}", path.display()))
        .map(|path| path.to_string_lossy().replace('\\', "/"))
}
