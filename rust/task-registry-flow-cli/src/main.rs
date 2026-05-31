use chrono::Local;
mod model;
mod mutation_hook;
mod plan_contract;
mod release_checks;
mod schema;
mod source_limit;
#[cfg(test)]
mod tests;

use crate::model::*;
use crate::mutation_hook::verify_mutation_hook;
use crate::schema::{
    BehaviorVerifier, CliCommand, EventOutcome, HookFormat, MutationScope, TaskStatus,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::time::Instant;

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let command = args
        .first()
        .and_then(|value| CliCommand::from_str(value).ok())
        .unwrap_or(CliCommand::Usage);
    let started = Instant::now();
    let result = run(args);
    let duration_ms = started.elapsed().as_millis();
    match result {
        Ok(detail) => {
            let _ = append_event(
                Path::new("."),
                EventRecord::new(
                    timestamp(),
                    command,
                    EventOutcome::Ok,
                    duration_ms,
                    truncate_detail(&detail),
                ),
            );
        }
        Err(error) => {
            let _ = append_event(
                Path::new("."),
                EventRecord::new(
                    timestamp(),
                    command,
                    EventOutcome::Error,
                    duration_ms,
                    truncate_detail(&error),
                ),
            );
            eprintln!("task-registry-flow error: {error}");
            std::process::exit(1);
        }
    }
}

fn run(mut args: Vec<String>) -> Result<String> {
    if args.is_empty() {
        return Err(usage());
    }
    let command = CliCommand::from_str(&args.remove(0)).map_err(|_| usage())?;
    let root = Path::new(".");
    match command {
        CliCommand::Validate => {
            let report = validate_all(root)?;
            println!(
                "task registry validate ok: {} plans, {} tasks, {} manifests",
                report.registry_plan_count, report.registry_task_count, report.manifest_count
            );
            Ok("validate".to_string())
        }
        CliCommand::Activate => {
            let plan_path = args.first().ok_or_else(usage)?;
            activate_plan(root, plan_path)?;
            println!("PLAN_ACTIVATE {plan_path} ok");
            Ok(format!("activate {plan_path}"))
        }
        CliCommand::Status => {
            let task_id = args.first().ok_or_else(usage)?;
            let status = args.get(1).ok_or_else(usage)?;
            update_task_status(root, task_id, status)?;
            println!("TASK_STATUS {task_id} {status} ok");
            Ok(format!("status {task_id} {status}"))
        }
        CliCommand::Defer => {
            let task_id = args.first().ok_or_else(usage)?;
            let basis = args.get(1).ok_or_else(usage)?;
            let reactivation = args.get(2).ok_or_else(usage)?;
            defer_task(root, task_id, basis, reactivation)?;
            println!("TASK_DEFER {task_id} ok");
            Ok(format!("defer {task_id}"))
        }
        CliCommand::Report => {
            let plan_id = args.first().ok_or_else(usage)?;
            let report = report_plan(root, plan_id)?;
            let formatted = format_report(&report);
            println!("{formatted}");
            Ok(format!("report {plan_id}"))
        }
        CliCommand::ArchiveCompleted => {
            archive_completed(root)?;
            println!("TASK_ARCHIVE_COMPLETED ok");
            Ok("archive-completed".to_string())
        }
        CliCommand::VerifyBehaviors => {
            let filter = args.first().map(String::as_str);
            let count = verify_behaviors(root, filter)?;
            println!("TASK_VERIFY_BEHAVIORS ok: {count} confirmations");
            Ok(format!("verify-behaviors {count}"))
        }
        CliCommand::VerifyMutationHook => {
            let format = parse_hook_format(&args)?;
            verify_mutation_hook(root, format)?;
            println!("TASK_VERIFY_MUTATION_HOOK ok");
            Ok("verify-mutation-hook".to_string())
        }
        CliCommand::Metrics => {
            let report = metrics(root)?;
            let formatted = format_metrics(&report);
            println!("{formatted}");
            Ok("metrics".to_string())
        }
        CliCommand::SourceLimit => source_limit::run_command(root, &args),
        CliCommand::ReleaseCheck => release_checks::run_command(root, &args),
        CliCommand::Usage => Err(usage()),
    }
}

fn usage() -> String {
    "usage: task-registry-flow {validate|activate <docs/plans/file.md>|status <task_id> <status>|defer <task_id> <basis> <reactivation>|report <plan_id>|archive-completed|verify-behaviors [plan_id|task_id]|verify-mutation-hook [--format codex|antigravity|cursor]|metrics|source-limit check|source-limit plan|release-check {required|version|tracked|all} [--format json]}".to_string()
}

fn validate_all(root: &Path) -> Result<ValidationReport> {
    source_limit::check(root)?;
    let registry = load_registry(root)?;
    let manifests = discover_manifests(root)?;
    validate_registry_with_manifests(root, &registry, &manifests)?;
    Ok(ValidationReport {
        registry_plan_count: registry.plans.len(),
        registry_task_count: registry.tasks.len(),
        manifest_count: manifests.len(),
    })
}

fn activate_plan(root: &Path, plan_path: &str) -> Result<()> {
    let manifest = load_manifest(root, plan_path)?;
    validate_manifest_for_activation(&manifest.manifest)?;
    plan_contract::validate_activation_contract(
        &manifest.plan_path,
        &manifest.plan_body,
        &manifest.manifest,
    )?;
    let mut registry = load_registry(root)?;
    validate_registry_root(&registry)?;
    let today = today();

    if let Some(existing) = registry
        .plans
        .iter()
        .find(|plan| plan.plan_id == manifest.manifest.plan_id && plan.plan_path != plan_path)
    {
        return Err(format!(
            "{} already activated from {}",
            existing.plan_id, existing.plan_path
        ));
    }

    match registry
        .plans
        .iter_mut()
        .find(|plan| plan.plan_id == manifest.manifest.plan_id)
    {
        Some(plan) => {
            plan.plan_hash_sha256 = manifest.plan_hash_sha256.clone();
            plan.activated_at = today.clone();
        }
        None => registry.plans.push(RegistryPlan {
            plan_id: manifest.manifest.plan_id.clone(),
            plan_path: plan_path.to_string(),
            plan_hash_sha256: manifest.plan_hash_sha256.clone(),
            activated_at: today.clone(),
            status: TaskStatus::Active,
        }),
    }

    let manifest_task_ids = manifest
        .manifest
        .tasks
        .iter()
        .map(|task| task.task_id.as_str())
        .collect::<BTreeSet<_>>();
    for task in &registry.tasks {
        if task.source_plan_path == manifest.plan_path
            && !manifest_task_ids.contains(task.task_id.as_str())
            && task.status != TaskStatus::Cancelled
        {
            return Err(format!(
                "{} would be removed by activation; cancel it explicitly first",
                task.task_id
            ));
        }
    }

    for manifest_task in &manifest.manifest.tasks {
        let existing_index = registry
            .tasks
            .iter()
            .position(|task| task.task_id == manifest_task.task_id);
        if let Some(index) = existing_index {
            let existing = registry.tasks[index].clone();
            if existing.plan_id != manifest.manifest.plan_id
                || existing.source_plan_path != manifest.plan_path
            {
                return Err(format!(
                    "{} already belongs to {} / {}",
                    existing.task_id, existing.plan_id, existing.source_plan_path
                ));
            }
            registry.tasks[index] =
                build_registry_task(manifest_task, &manifest, Some(&existing), &today);
        } else {
            registry
                .tasks
                .push(build_registry_task(manifest_task, &manifest, None, &today));
        }
    }
    refresh_plan_status(&mut registry, &manifest.manifest.plan_id);
    validate_registry_with_manifests(root, &registry, &discover_manifests(root)?)?;
    save_registry(root, &registry)
}

fn update_task_status(root: &Path, task_id: &str, status: &str) -> Result<()> {
    let status = parse_status(status, task_id)?;
    if status == TaskStatus::Deferred {
        return Err("use TASK_DEFER instead of TASK_STATUS for deferred tasks".to_string());
    }
    if status == TaskStatus::Completed {
        verify_behaviors(root, Some(task_id))?;
    }
    let mut registry = load_registry(root)?;
    let plan_id = {
        let task = registry
            .tasks
            .iter_mut()
            .find(|task| task.task_id == task_id)
            .ok_or_else(|| format!("missing task_id {task_id}"))?;
        task.status = status;
        task.updated_at = today();
        task.plan_id.clone()
    };
    refresh_plan_status(&mut registry, &plan_id);
    validate_registry_with_manifests(root, &registry, &discover_manifests(root)?)?;
    save_registry(root, &registry)
}

fn defer_task(root: &Path, task_id: &str, basis: &str, reactivation: &str) -> Result<()> {
    reject_empty("deferral_governance_basis", basis)?;
    reject_empty("reactivation_condition", reactivation)?;
    let mut registry = load_registry(root)?;
    let plan_id = {
        let task = registry
            .tasks
            .iter_mut()
            .find(|task| task.task_id == task_id)
            .ok_or_else(|| format!("missing task_id {task_id}"))?;
        task.status = TaskStatus::Deferred;
        task.deferral_governance_basis = Some(basis.to_string());
        task.reactivation_condition = Some(reactivation.to_string());
        task.updated_at = today();
        validate_deferred_task(task)?;
        task.plan_id.clone()
    };
    refresh_plan_status(&mut registry, &plan_id);
    validate_registry_with_manifests(root, &registry, &discover_manifests(root)?)?;
    save_registry(root, &registry)
}

fn archive_completed(root: &Path) -> Result<()> {
    let registry = load_registry(root)?;
    save_registry(root, &registry)
}

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

fn verify_behaviors(root: &Path, filter: Option<&str>) -> Result<usize> {
    let registry = load_registry(root)?;
    let manifests = discover_manifests(root)?;
    let behavior_map = behavior_map(&manifests)?;
    let mut count = 0usize;

    for task in &registry.tasks {
        if matches!(filter, Some(value) if value != task.task_id && value != task.plan_id) {
            continue;
        }
        if task.status == TaskStatus::Cancelled {
            continue;
        }
        for behavior_id in &task.behavior_ids {
            let behavior = behavior_map.get(behavior_id).ok_or_else(|| {
                format!("{} references missing behavior {behavior_id}", task.task_id)
            })?;
            run_behavior_verifiers(root, behavior, behavior_id)?;
            count += 1;
        }
    }
    Ok(count)
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

fn metrics(root: &Path) -> Result<MetricsReport> {
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
        malformed_events: 0,
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
    let events_path = root.join(EVENTS_PATH);
    if events_path.is_file() {
        let body = fs::read_to_string(&events_path)
            .map_err(|error| format!("read {}: {error}", events_path.display()))?;
        for line in body.lines().filter(|line| !line.trim().is_empty()) {
            report.events += 1;
            match serde_json::from_str::<EventRecord>(line) {
                Ok(event) => {
                    if event.outcome == EventOutcome::Error {
                        report.failed_events += 1;
                    }
                    if event.outcome == EventOutcome::MutationDenied {
                        report.mutation_denials += 1;
                    }
                }
                Err(_) => {
                    report.malformed_events += 1;
                    report.failed_events += 1;
                }
            }
        }
    }
    Ok(report)
}

fn format_metrics(report: &MetricsReport) -> String {
    format!(
        "Task registry metrics: plans={}, tasks={}, manifests={}, planned={}, active={}, completed={}, deferred={}, blocked={}, cancelled={}, events={}, failed_events={}, mutation_denials={}, malformed_events={}",
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
        report.malformed_events
    )
}

pub(crate) fn load_registry(root: &Path) -> Result<TaskRegistry> {
    let mut registry = load_current_registry(root)?;
    let archives = load_registry_archives(root, &registry.archive_paths)?;
    for archive in archives {
        registry.plans.extend(archive.plans);
        registry.tasks.extend(archive.tasks);
    }
    validate_registry_root(&registry)?;
    Ok(registry)
}

fn load_current_registry(root: &Path) -> Result<TaskRegistry> {
    let path = root.join(REGISTRY_PATH);
    let body =
        fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
    toml::from_str(&body).map_err(|error| format!("parse {}: {error}", path.display()))
}

fn load_registry_archives(
    root: &Path,
    archive_paths: &[String],
) -> Result<Vec<TaskRegistryArchive>> {
    let mut archives = Vec::new();
    for archive_path in archive_paths {
        validate_archive_path(archive_path)?;
        let body = fs::read_to_string(root.join(archive_path))
            .map_err(|error| format!("read {archive_path}: {error}"))?;
        let archive = toml::from_str::<TaskRegistryArchive>(&body)
            .map_err(|error| format!("parse {archive_path}: {error}"))?;
        validate_archive_shape(archive_path, &archive)?;
        archives.push(archive);
    }
    Ok(archives)
}

fn save_registry(root: &Path, registry: &TaskRegistry) -> Result<()> {
    validate_registry_root(registry)?;
    let (main_registry, archives) = split_registry_for_archives(registry);
    write_registry_archives(root, &archives)?;
    let path = root.join(REGISTRY_PATH);
    let mut body = toml::to_string_pretty(&main_registry)
        .map_err(|error| format!("serialize {}: {error}", path.display()))?;
    if !body.ends_with('\n') {
        body.push('\n');
    }
    fs::write(&path, body).map_err(|error| format!("write {}: {error}", path.display()))
}

fn split_registry_for_archives(
    registry: &TaskRegistry,
) -> (TaskRegistry, Vec<(String, TaskRegistryArchive)>) {
    let completed_plan_ids = registry
        .plans
        .iter()
        .filter(|plan| plan_is_archive_eligible(plan, &registry.tasks))
        .map(|plan| plan.plan_id.clone())
        .collect::<BTreeSet<_>>();
    let mut main_registry = registry.clone();
    main_registry.plans = registry
        .plans
        .iter()
        .filter(|plan| !completed_plan_ids.contains(&plan.plan_id))
        .cloned()
        .collect();
    main_registry.tasks = registry
        .tasks
        .iter()
        .filter(|task| !completed_plan_ids.contains(&task.plan_id))
        .cloned()
        .collect();

    let completed_plans = registry
        .plans
        .iter()
        .filter(|plan| completed_plan_ids.contains(&plan.plan_id))
        .map(archived_completed_plan)
        .collect::<Vec<_>>();
    let mut archives = Vec::new();
    for (index, chunk) in completed_plans
        .chunks(ARCHIVE_COMPLETED_PLAN_CHUNK_SIZE)
        .enumerate()
    {
        let archive_id = format!("completed-{:03}", index + 1);
        let archive_path = format!("{REGISTRY_ARCHIVE_DIR}/{archive_id}.toml");
        let chunk_plan_ids = chunk
            .iter()
            .map(|plan| plan.plan_id.as_str())
            .collect::<BTreeSet<_>>();
        let tasks = registry
            .tasks
            .iter()
            .filter(|task| chunk_plan_ids.contains(task.plan_id.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        archives.push((
            archive_path.clone(),
            TaskRegistryArchive {
                schema_version: registry.schema_version,
                registry_id: registry.registry_id.clone(),
                archive_id,
                archive_authority: archive_path,
                plans: chunk.to_vec(),
                tasks,
            },
        ));
    }
    main_registry.archive_paths = archives
        .iter()
        .map(|(archive_path, _)| archive_path.clone())
        .collect();
    (main_registry, archives)
}

fn write_registry_archives(root: &Path, archives: &[(String, TaskRegistryArchive)]) -> Result<()> {
    let archive_dir = root.join(REGISTRY_ARCHIVE_DIR);
    fs::create_dir_all(&archive_dir)
        .map_err(|error| format!("create {REGISTRY_ARCHIVE_DIR}: {error}"))?;
    for entry in fs::read_dir(&archive_dir)
        .map_err(|error| format!("read {REGISTRY_ARCHIVE_DIR}: {error}"))?
        .flatten()
    {
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if file_name.starts_with("completed-") && file_name.ends_with(".toml") {
            fs::remove_file(&path)
                .map_err(|error| format!("remove {}: {error}", path.display()))?;
        }
    }
    for (archive_path, archive) in archives {
        let mut body = toml::to_string_pretty(archive)
            .map_err(|error| format!("serialize {archive_path}: {error}"))?;
        if !body.ends_with('\n') {
            body.push('\n');
        }
        fs::write(root.join(archive_path), body)
            .map_err(|error| format!("write {archive_path}: {error}"))?;
    }
    Ok(())
}

fn plan_is_archive_eligible(plan: &RegistryPlan, tasks: &[RegistryTask]) -> bool {
    if plan.status == TaskStatus::Completed || plan.status == TaskStatus::Cancelled {
        return true;
    }
    let plan_tasks = tasks
        .iter()
        .filter(|task| task.plan_id == plan.plan_id)
        .collect::<Vec<_>>();
    !plan_tasks.is_empty()
        && plan_tasks.iter().all(|task| {
            task.status == TaskStatus::Completed || task.status == TaskStatus::Cancelled
        })
}

fn archived_completed_plan(plan: &RegistryPlan) -> RegistryPlan {
    let mut archived = plan.clone();
    if archived.status != TaskStatus::Cancelled {
        archived.status = TaskStatus::Completed;
    }
    archived
}

fn validate_registry_with_manifests(
    root: &Path,
    registry: &TaskRegistry,
    manifests: &[ActivatedManifest],
) -> Result<()> {
    validate_registry_root(registry)?;
    let mut plans_by_id = BTreeMap::new();
    for plan in &registry.plans {
        validate_plan(root, plan)?;
        if plans_by_id.insert(plan.plan_id.clone(), plan).is_some() {
            return Err(format!("duplicate plan_id {}", plan.plan_id));
        }
    }

    let mut tasks_by_id = BTreeMap::new();
    for task in &registry.tasks {
        if tasks_by_id.insert(task.task_id.clone(), task).is_some() {
            return Err(format!("duplicate task_id {}", task.task_id));
        }
        validate_task(task, &plans_by_id)?;
    }

    let mut manifest_plan_ids = BTreeMap::new();
    let mut manifest_task_ids = BTreeMap::new();
    for manifest in manifests {
        if let Some(previous) = manifest_plan_ids.insert(
            manifest.manifest.plan_id.as_str(),
            manifest.plan_path.as_str(),
        ) {
            return Err(format!(
                "duplicate manifest plan_id {} in {} and {}",
                manifest.manifest.plan_id, previous, manifest.plan_path
            ));
        }
        let registry_plan = plans_by_id.get(&manifest.manifest.plan_id).ok_or_else(|| {
            format!(
                "{} from {} must be activated in registry",
                manifest.manifest.plan_id, manifest.plan_path
            )
        })?;
        validate_manifest_for_registry(&manifest.manifest, registry_plan.status)?;
        plan_contract::validate_registry_contract(
            &manifest.plan_path,
            &manifest.plan_body,
            &manifest.manifest,
            registry_plan.status,
        )?;
        if registry_plan.plan_path != manifest.plan_path {
            return Err(format!(
                "{} registry path {} does not match manifest path {}",
                manifest.manifest.plan_id, registry_plan.plan_path, manifest.plan_path
            ));
        }
        if registry_plan.plan_hash_sha256 != manifest.plan_hash_sha256 {
            return Err(format!(
                "{} registry hash is stale",
                manifest.manifest.plan_id
            ));
        }
        for manifest_task in &manifest.manifest.tasks {
            if let Some(previous) = manifest_task_ids
                .insert(manifest_task.task_id.as_str(), manifest.plan_path.as_str())
            {
                return Err(format!(
                    "duplicate manifest task_id {} in {} and {}",
                    manifest_task.task_id, previous, manifest.plan_path
                ));
            }
            let registry_task = tasks_by_id.get(&manifest_task.task_id).ok_or_else(|| {
                format!(
                    "{} from {} must be activated in registry",
                    manifest_task.task_id, manifest.plan_path
                )
            })?;
            if registry_task.plan_id != manifest.manifest.plan_id {
                return Err(format!("{} registry plan mismatch", registry_task.task_id));
            }
            if registry_task.source_plan_path != manifest.plan_path {
                return Err(format!(
                    "{} registry source path mismatch",
                    registry_task.task_id
                ));
            }
            if registry_task.source_plan_hash_sha256 != manifest.plan_hash_sha256 {
                return Err(format!(
                    "{} registry source_plan_hash_sha256 mismatch",
                    registry_task.task_id
                ));
            }
        }
    }

    for task in &registry.tasks {
        if task.source_plan_path.starts_with(PLAN_DIR)
            && !manifest_task_ids.contains_key(task.task_id.as_str())
            && task.status != TaskStatus::Cancelled
            && manifests
                .iter()
                .any(|manifest| manifest.plan_path == task.source_plan_path)
        {
            return Err(format!(
                "{} claims manifest source {} but is absent from that manifest",
                task.task_id, task.source_plan_path
            ));
        }
    }
    Ok(())
}

fn validate_registry_root(registry: &TaskRegistry) -> Result<()> {
    if registry.schema_version != 1 {
        return Err("registry schema_version must be 1".to_string());
    }
    reject_empty("registry_id", &registry.registry_id)?;
    if registry.registry_authority != REGISTRY_PATH {
        return Err(format!("registry_authority must be {REGISTRY_PATH}"));
    }
    if registry.activation_skill != "task-registry-flow" {
        return Err("activation_skill must be task-registry-flow".to_string());
    }
    for status in TaskStatus::variants() {
        if !registry
            .status_vocabulary
            .iter()
            .any(|candidate| candidate == status)
        {
            return Err(format!("status_vocabulary missing {status}"));
        }
    }
    let mut archive_paths = BTreeSet::new();
    for archive_path in &registry.archive_paths {
        validate_archive_path(archive_path)?;
        if !archive_paths.insert(archive_path.as_str()) {
            return Err(format!("duplicate archive_path {archive_path}"));
        }
    }
    Ok(())
}

fn validate_archive_shape(path: &str, archive: &TaskRegistryArchive) -> Result<()> {
    if archive.schema_version != 1 {
        return Err(format!("{path} schema_version must be 1"));
    }
    reject_empty("registry_id", &archive.registry_id)?;
    reject_empty("archive_id", &archive.archive_id)?;
    if archive.archive_authority != path {
        return Err(format!("{path} archive_authority must match archive path"));
    }
    Ok(())
}

fn validate_archive_path(path: &str) -> Result<()> {
    if !path.starts_with("docs/task-registry/archive/") || !path.ends_with(".toml") {
        return Err(format!(
            "archive path must be under docs/task-registry/archive and end in .toml: {path}"
        ));
    }
    Ok(())
}

fn validate_plan(root: &Path, plan: &RegistryPlan) -> Result<()> {
    validate_plan_path(&plan.plan_path)?;
    reject_empty("plan_hash_sha256", &plan.plan_hash_sha256)?;
    assert_sha256("plan_hash_sha256", &plan.plan_hash_sha256)?;
    let path = root.join(&plan.plan_path);
    if !path.is_file() {
        return Err(format!("missing plan file {}", plan.plan_path));
    }
    let expected_hash = normalized_file_hash(&path)?;
    if plan.plan_hash_sha256 != expected_hash {
        return Err(format!("{} hash mismatch", plan.plan_id));
    }
    Ok(())
}

fn validate_task(task: &RegistryTask, plans_by_id: &BTreeMap<String, &RegistryPlan>) -> Result<()> {
    let plan = plans_by_id
        .get(&task.plan_id)
        .ok_or_else(|| format!("{} references missing plan {}", task.task_id, task.plan_id))?;
    if task.source_plan_path != plan.plan_path {
        return Err(format!("{} source_plan_path mismatch", task.task_id));
    }
    if task.status != TaskStatus::Cancelled && task.source_plan_hash_sha256 != plan.plan_hash_sha256
    {
        return Err(format!("{} source_plan_hash_sha256 mismatch", task.task_id));
    }
    for (field, value) in [
        ("task_id", &task.task_id),
        ("title", &task.title),
        ("reason", &task.reason),
        ("acceptance_proof", &task.acceptance_proof),
        ("created_at", &task.created_at),
        ("updated_at", &task.updated_at),
    ] {
        reject_empty(field, value)?;
    }
    assert_sha256("source_plan_hash_sha256", &task.source_plan_hash_sha256)?;
    validate_targets(&task.task_id, &task.targets)?;
    if task.behavior_ids.is_empty() {
        return Err(format!("{} requires behavior_ids", task.task_id));
    }
    for behavior_id in &task.behavior_ids {
        reject_empty("behavior_ids", behavior_id)?;
    }
    if task.status == TaskStatus::Deferred {
        validate_deferred_task(task)?;
    }
    Ok(())
}

fn validate_manifest(manifest: &PlanManifest) -> Result<()> {
    if !matches!(manifest.schema_version, 1 | 2) {
        return Err("Task Manifest schema_version must be 1 or 2".to_string());
    }
    reject_empty("manifest plan_id", &manifest.plan_id)?;
    if manifest.behaviors.is_empty() {
        return Err(format!("{} has no behaviors", manifest.plan_id));
    }
    if manifest.tasks.is_empty() {
        return Err(format!("{} has no tasks", manifest.plan_id));
    }
    let mut behavior_ids = BTreeSet::new();
    let require_verifiers = manifest.schema_version >= 2;
    for behavior in &manifest.behaviors {
        validate_behavior(behavior, require_verifiers)?;
        if !behavior_ids.insert(behavior.behavior_id.as_str()) {
            return Err(format!("duplicate behavior_id {}", behavior.behavior_id));
        }
    }
    let mut task_ids = BTreeSet::new();
    for task in &manifest.tasks {
        if !task_ids.insert(task.task_id.as_str()) {
            return Err(format!(
                "{} duplicate task_id {}",
                manifest.plan_id, task.task_id
            ));
        }
        validate_manifest_task(task, &behavior_ids)?;
    }
    Ok(())
}

fn validate_manifest_for_activation(manifest: &PlanManifest) -> Result<()> {
    validate_manifest(manifest)?;
    if manifest.schema_version != 2 {
        return Err(format!(
            "{} uses Task Manifest schema_version {}; new activations require schema_version 2",
            manifest.plan_id, manifest.schema_version
        ));
    }
    Ok(())
}

fn validate_manifest_for_registry(manifest: &PlanManifest, status: TaskStatus) -> Result<()> {
    validate_manifest(manifest)?;
    if manifest.schema_version == 1
        && !matches!(status, TaskStatus::Completed | TaskStatus::Cancelled)
    {
        return Err(format!(
            "{} uses legacy Task Manifest schema_version 1; only completed/cancelled archived evidence may remain v1",
            manifest.plan_id
        ));
    }
    Ok(())
}

fn validate_behavior(behavior: &Behavior, require_verifiers: bool) -> Result<()> {
    for (field, value) in [
        ("behavior_id", &behavior.behavior_id),
        ("title", &behavior.title),
        ("given", &behavior.given),
        ("when", &behavior.when),
        ("then", &behavior.then),
        ("confirmation", &behavior.confirmation),
    ] {
        reject_empty(field, value)?;
    }
    if require_verifiers && behavior.verifiers.is_empty() {
        return Err(format!(
            "{} requires typed [[behaviors.verifiers]] entries",
            behavior.behavior_id
        ));
    }
    if require_verifiers {
        reject_empty(
            "gap_id",
            behavior
                .gap_id
                .as_deref()
                .ok_or_else(|| format!("{} requires gap_id", behavior.behavior_id))?,
        )?;
        if behavior.polarity.is_none() {
            return Err(format!("{} requires polarity", behavior.behavior_id));
        }
    }
    for verifier in &behavior.verifiers {
        verifier
            .validate()
            .map_err(|error| format!("{} invalid verifier: {error}", behavior.behavior_id))?;
    }
    Ok(())
}

fn validate_manifest_task(task: &ManifestTask, behavior_ids: &BTreeSet<&str>) -> Result<()> {
    for (field, value) in [
        ("task_id", &task.task_id),
        ("title", &task.title),
        ("reason", &task.reason),
        ("acceptance_proof", &task.acceptance_proof),
    ] {
        reject_empty(field, value)?;
    }
    if task.behavior_ids.is_empty() {
        return Err(format!("{} requires behavior_ids", task.task_id));
    }
    for behavior_id in &task.behavior_ids {
        if !behavior_ids.contains(behavior_id.as_str()) {
            return Err(format!(
                "{} references unknown behavior_id {}",
                task.task_id, behavior_id
            ));
        }
    }
    validate_targets(&task.task_id, &task.targets)?;
    if task.status == TaskStatus::Deferred {
        if empty_option(&task.deferral_governance_basis)
            || empty_option(&task.reactivation_condition)
        {
            return Err(format!(
                "{} deferred without deferral_governance_basis/reactivation_condition",
                task.task_id
            ));
        }
        if manifest_task_requires_decomposition(task) {
            validate_blocker_decomposition(&task.task_id, &task.blockers, &task.projected_steps)?;
        }
    }
    Ok(())
}

fn validate_targets(task_id: &str, targets: &[TaskTarget]) -> Result<()> {
    if targets.is_empty() {
        return Err(format!("{task_id} requires at least one target"));
    }
    for target in targets {
        for (field, value) in [
            ("file", &target.file),
            ("object", &target.object),
            ("required_change", &target.required_change),
        ] {
            reject_empty(field, value)?;
        }
        if is_broad_object(&target.object) {
            return Err(format!(
                "{task_id} target object is too broad: {}",
                target.object
            ));
        }
        normalize_relative_path(&target.file)?;
        MutationScope::from_task_target(&target.file)
            .map_err(|error| format!("{task_id} invalid mutation target: {error}"))?;
    }
    Ok(())
}

fn validate_deferred_task(task: &RegistryTask) -> Result<()> {
    if empty_option(&task.deferral_governance_basis) || empty_option(&task.reactivation_condition) {
        return Err(format!(
            "{} deferred without deferral_governance_basis/reactivation_condition",
            task.task_id
        ));
    }
    if task_requires_decomposition(task) {
        validate_blocker_decomposition(&task.task_id, &task.blockers, &task.projected_steps)?;
    }
    Ok(())
}

fn validate_blocker_decomposition(
    task_id: &str,
    blockers: &[TaskBlocker],
    projected_steps: &[ProjectedStep],
) -> Result<()> {
    if blockers.is_empty() || projected_steps.is_empty() {
        return Err(format!(
            "{task_id} deferred blocker requires blockers and projected_steps"
        ));
    }
    let mut blocker_ids = BTreeSet::new();
    for blocker in blockers {
        for (field, value) in [
            ("blocker_id", &blocker.blocker_id),
            ("blocked_object", &blocker.blocked_object),
            ("blocked_change", &blocker.blocked_change),
            ("current_state", &blocker.current_state),
            ("unblock_condition", &blocker.unblock_condition),
            ("evidence_required", &blocker.evidence_required),
        ] {
            reject_empty(field, value)?;
        }
        if !blocker_ids.insert(blocker.blocker_id.as_str()) {
            return Err(format!(
                "{} duplicate blocker_id {}",
                task_id, blocker.blocker_id
            ));
        }
    }
    for step in projected_steps {
        for (field, value) in [
            ("step_id", step.step_id.as_str()),
            ("status", step.status.as_str()),
            ("file", step.file.as_str()),
            ("object", step.object.as_str()),
            ("required_change", step.required_change.as_str()),
            ("blocked_by", step.blocked_by.as_str()),
        ] {
            reject_empty(field, value)?;
        }
        for blocker_id in step
            .blocked_by
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            if !blocker_ids.contains(blocker_id) {
                return Err(format!(
                    "{} projected_step {} references missing blocker {}",
                    task_id, step.step_id, blocker_id
                ));
            }
        }
    }
    Ok(())
}

fn build_registry_task(
    task: &ManifestTask,
    manifest: &ActivatedManifest,
    existing: Option<&RegistryTask>,
    today: &str,
) -> RegistryTask {
    RegistryTask {
        task_id: task.task_id.clone(),
        plan_id: manifest.manifest.plan_id.clone(),
        status: existing.map(|task| task.status).unwrap_or(task.status),
        title: task.title.clone(),
        kind: task.kind,
        source_plan_path: manifest.plan_path.clone(),
        source_plan_hash_sha256: manifest.plan_hash_sha256.clone(),
        reason: task.reason.clone(),
        acceptance_proof: task.acceptance_proof.clone(),
        created_at: existing
            .map(|task| task.created_at.clone())
            .unwrap_or_else(|| today.to_string()),
        updated_at: today.to_string(),
        behavior_ids: task.behavior_ids.clone(),
        deferral_governance_basis: task
            .deferral_governance_basis
            .clone()
            .or_else(|| existing.and_then(|task| task.deferral_governance_basis.clone())),
        reactivation_condition: task
            .reactivation_condition
            .clone()
            .or_else(|| existing.and_then(|task| task.reactivation_condition.clone())),
        closure_plan_id: existing.and_then(|task| task.closure_plan_id.clone()),
        targets: task.targets.clone(),
        blockers: if task.blockers.is_empty() {
            existing
                .map(|task| task.blockers.clone())
                .unwrap_or_default()
        } else {
            task.blockers.clone()
        },
        projected_steps: if task.projected_steps.is_empty() {
            existing
                .map(|task| task.projected_steps.clone())
                .unwrap_or_default()
        } else {
            task.projected_steps.clone()
        },
    }
}

fn refresh_plan_status(registry: &mut TaskRegistry, plan_id: &str) {
    let tasks = registry
        .tasks
        .iter()
        .filter(|task| task.plan_id == plan_id)
        .cloned()
        .collect::<Vec<_>>();
    if let Some(plan) = registry
        .plans
        .iter_mut()
        .find(|plan| plan.plan_id == plan_id)
    {
        plan.status = derive_plan_status(&tasks);
    }
}

fn derive_plan_status(tasks: &[RegistryTask]) -> TaskStatus {
    if !tasks.is_empty()
        && tasks
            .iter()
            .all(|task| task.status == TaskStatus::Completed)
    {
        TaskStatus::Completed
    } else if !tasks.is_empty()
        && tasks
            .iter()
            .all(|task| task.status == TaskStatus::Cancelled)
    {
        TaskStatus::Cancelled
    } else {
        TaskStatus::Active
    }
}

fn discover_manifests(root: &Path) -> Result<Vec<ActivatedManifest>> {
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

fn load_manifest(root: &Path, plan_path: &str) -> Result<ActivatedManifest> {
    validate_plan_path(plan_path)?;
    let path = root.join(plan_path);
    let body = fs::read_to_string(&path).map_err(|error| format!("read {plan_path}: {error}"))?;
    parse_manifest_from_body(plan_path, &body)
}

fn parse_manifest_from_body(plan_path: &str, body: &str) -> Result<ActivatedManifest> {
    let manifest_text = task_manifest_table(plan_path, body)?;
    let manifest = toml::from_str::<PlanManifest>(&manifest_text)
        .map_err(|error| format!("parse Task Manifest in {plan_path}: {error}"))?;
    validate_manifest(&manifest)?;
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

fn behavior_map(manifests: &[ActivatedManifest]) -> Result<BTreeMap<String, Behavior>> {
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

fn run_behavior_verifiers(root: &Path, behavior: &Behavior, behavior_id: &str) -> Result<()> {
    if behavior.verifiers.is_empty() {
        return Err(format!(
            "{behavior_id} requires typed [[behaviors.verifiers]] entries"
        ));
    }
    for verifier in &behavior.verifiers {
        verifier
            .validate()
            .map_err(|error| format!("invalid verifier for {behavior_id}: {error}"))?;
        run_behavior_verifier(root, verifier, behavior_id)?;
    }
    Ok(())
}

fn run_behavior_verifier(
    root: &Path,
    verifier: &BehaviorVerifier,
    behavior_id: &str,
) -> Result<()> {
    match verifier {
        BehaviorVerifier::Command {
            command,
            expected_exit,
        } => run_confirmation(command, *expected_exit, behavior_id),
        BehaviorVerifier::FileExists { path } => {
            let path = verifier_path(path)?;
            if root.join(&path).is_file() {
                Ok(())
            } else {
                Err(format!(
                    "verifier failed for {behavior_id}: expected file {path} to exist"
                ))
            }
        }
        BehaviorVerifier::FileAbsent { path } => {
            let path = verifier_path(path)?;
            if !root.join(&path).exists() {
                Ok(())
            } else {
                Err(format!(
                    "verifier failed for {behavior_id}: expected {path} to be absent"
                ))
            }
        }
        BehaviorVerifier::Contains { path, needle }
        | BehaviorVerifier::NotContains { path, needle } => {
            let path = verifier_path(path)?;
            let body = fs::read_to_string(root.join(&path))
                .map_err(|error| format!("read verifier file {path}: {error}"))?;
            let contains = body.contains(needle);
            if matches!(verifier, BehaviorVerifier::Contains { .. }) && contains {
                return Ok(());
            }
            if matches!(verifier, BehaviorVerifier::NotContains { .. }) && !contains {
                return Ok(());
            }
            Err(format!(
                "verifier failed for {behavior_id}: {} {path} needle {:?}",
                verifier.verifier_type(),
                needle
            ))
        }
        BehaviorVerifier::JsonValid { path } => {
            let path = verifier_path(path)?;
            let body = fs::read_to_string(root.join(&path))
                .map_err(|error| format!("read verifier JSON {path}: {error}"))?;
            serde_json::from_str::<Value>(&body).map_err(|error| {
                format!("verifier failed for {behavior_id}: invalid JSON {path}: {error}")
            })?;
            Ok(())
        }
        BehaviorVerifier::JsonSchema { path, schema_path } => {
            let path = verifier_path(path)?;
            let schema_path = verifier_path(schema_path)?;
            let body = fs::read_to_string(root.join(&path))
                .map_err(|error| format!("read verifier JSON {path}: {error}"))?;
            let schema_body = fs::read_to_string(root.join(&schema_path))
                .map_err(|error| format!("read verifier JSON schema {schema_path}: {error}"))?;
            let instance = serde_json::from_str::<Value>(&body).map_err(|error| {
                format!("verifier failed for {behavior_id}: invalid JSON {path}: {error}")
            })?;
            let schema = serde_json::from_str::<Value>(&schema_body).map_err(|error| {
                format!(
                    "verifier failed for {behavior_id}: invalid JSON schema {schema_path}: {error}"
                )
            })?;
            let validator = jsonschema::validator_for(&schema).map_err(|error| {
                format!(
                    "verifier failed for {behavior_id}: invalid JSON schema {schema_path}: {error}"
                )
            })?;
            validator.validate(&instance).map_err(|error| {
                format!("verifier failed for {behavior_id}: JSON {path} does not match {schema_path}: {error}")
            })
        }
    }
}

fn verifier_path(path: &str) -> Result<String> {
    normalize_relative_path(path)
}

fn run_confirmation(command: &str, expected_exit: i32, behavior_id: &str) -> Result<()> {
    let status = Command::new("bash")
        .arg("-lc")
        .arg(command)
        .status()
        .map_err(|error| format!("run confirmation for {behavior_id}: {error}"))?;
    let actual = status.code().unwrap_or(1);
    if actual == expected_exit {
        Ok(())
    } else {
        Err(format!(
            "confirmation failed for {behavior_id}: expected exit {expected_exit}, actual {actual}"
        ))
    }
}

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

pub(crate) fn append_event(root: &Path, event: EventRecord) -> Result<()> {
    let path = root.join(EVENTS_PATH);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("create {}: {error}", parent.display()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|error| format!("open {}: {error}", path.display()))?;
    let line =
        serde_json::to_string(&event).map_err(|error| format!("serialize event: {error}"))?;
    writeln!(file, "{line}").map_err(|error| format!("write {}: {error}", path.display()))
}

fn normalized_file_hash(path: &Path) -> Result<String> {
    let body =
        fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    Ok(normalized_hash(&body))
}

fn normalized_hash(body: &str) -> String {
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

fn validate_plan_path(path: &str) -> Result<()> {
    let plan_path = normalize_relative_path(path)?;
    if !plan_path.starts_with(&format!("{PLAN_DIR}/")) {
        return Err(format!("plan path must be under {PLAN_DIR}: {path}"));
    }
    if !plan_path.ends_with(".md") {
        return Err(format!("plan path must be a markdown file: {path}"));
    }
    Ok(())
}

fn parse_status(status: &str, object: &str) -> Result<TaskStatus> {
    TaskStatus::from_str(status).map_err(|error| format!("{object} {error}"))
}

fn assert_sha256(field: &str, value: &str) -> Result<()> {
    if value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(format!("{field} must be a 64-character SHA-256 hex string"))
    }
}

fn reject_empty(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        Err(format!("{field} must not be empty"))
    } else {
        Ok(())
    }
}

fn empty_option(value: &Option<String>) -> bool {
    value
        .as_ref()
        .map(|value| value.trim().is_empty())
        .unwrap_or(true)
}

fn task_requires_decomposition(task: &RegistryTask) -> bool {
    let text = format!(
        "{} {} {}",
        task.reason,
        task.deferral_governance_basis.clone().unwrap_or_default(),
        task.reactivation_condition.clone().unwrap_or_default()
    )
    .to_ascii_lowercase();
    EXTERNAL_BLOCKER_INDICATORS
        .iter()
        .any(|indicator| text.contains(indicator))
}

fn manifest_task_requires_decomposition(task: &ManifestTask) -> bool {
    let text = format!(
        "{} {} {}",
        task.reason,
        task.deferral_governance_basis.clone().unwrap_or_default(),
        task.reactivation_condition.clone().unwrap_or_default()
    )
    .to_ascii_lowercase();
    EXTERNAL_BLOCKER_INDICATORS
        .iter()
        .any(|indicator| text.contains(indicator))
}

fn is_broad_object(object: &str) -> bool {
    matches!(
        object.trim().to_ascii_lowercase().as_str(),
        "backend" | "frontend" | "docs" | "tests" | "code" | "implementation"
    )
}

fn today() -> String {
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
