use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::activation::complete_tasks_from_landing;
use crate::model::{EVENTS_PATH, REGISTRY_ARCHIVE_DIR, REGISTRY_PATH, RegistryTask, Result};
use crate::registry_io::load_registry;
use crate::runtime::{discover_manifests, normalize_relative_path};
use crate::schema::{MutationScope, TaskStatus};
use crate::validation::validate_registry_with_manifests;
use crate::verifiers::verify_behaviors;

#[derive(Debug, Default, PartialEq, Eq)]
struct LandingArgs {
    plan_id: Option<String>,
    changed_files: Vec<String>,
}

pub(crate) fn run_command(root: &Path, args: &[String]) -> Result<String> {
    let args = parse_args(args)?;
    let changed_files = normalize_changed_files(&args.changed_files)?;
    let task_bound_files = changed_files
        .iter()
        .filter(|path| !is_registry_generated_path(path))
        .cloned()
        .collect::<Vec<_>>();
    if task_bound_files.is_empty() {
        return Err("verify-landing requires at least one non-registry changed file".to_string());
    }

    let registry = load_registry(root)?;
    validate_registry_with_manifests(root, &registry, &discover_manifests(root)?)?;
    let selected =
        select_landing_tasks(&registry.tasks, &task_bound_files, args.plan_id.as_deref())?;
    let task_ids = selected
        .iter()
        .map(|task| task.task_id.clone())
        .collect::<Vec<_>>();

    for task_id in &task_ids {
        verify_behaviors(root, Some(task_id))?;
    }
    complete_tasks_from_landing(root, &task_ids, &changed_files)?;
    Ok(format!(
        "TASK_VERIFY_LANDING ok: {} task(s) completed",
        task_ids.len()
    ))
}

fn parse_args(args: &[String]) -> Result<LandingArgs> {
    let mut parsed = LandingArgs::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--plan-id" => {
                index += 1;
                let Some(plan_id) = args.get(index) else {
                    return Err(usage());
                };
                if plan_id.starts_with("--") {
                    return Err(usage());
                }
                parsed.plan_id = Some(plan_id.to_string());
            }
            "--changed-files" => {
                index += 1;
                while index < args.len() {
                    if args[index] == "--plan-id" {
                        index -= 1;
                        break;
                    }
                    if args[index].starts_with("--") {
                        return Err(usage());
                    }
                    parsed.changed_files.push(args[index].to_string());
                    index += 1;
                }
            }
            _ => return Err(usage()),
        }
        index += 1;
    }
    if parsed.changed_files.is_empty() {
        return Err(usage());
    }
    Ok(parsed)
}

fn usage() -> String {
    "usage: task-registry-flow verify-landing [--plan-id <plan_id>] --changed-files <path>..."
        .to_string()
}

fn normalize_changed_files(paths: &[String]) -> Result<Vec<String>> {
    let mut normalized = paths
        .iter()
        .map(|path| normalize_relative_path(path))
        .collect::<Result<Vec<_>>>()?;
    normalized.sort();
    normalized.dedup();
    Ok(normalized)
}

fn is_registry_generated_path(path: &str) -> bool {
    path == REGISTRY_PATH
        || path == EVENTS_PATH
        || path.starts_with(&format!("{REGISTRY_ARCHIVE_DIR}/"))
}

fn select_landing_tasks<'a>(
    tasks: &'a [RegistryTask],
    changed_files: &[String],
    plan_id: Option<&str>,
) -> Result<Vec<&'a RegistryTask>> {
    let mut selected = BTreeMap::new();
    for changed_file in changed_files {
        let matches = matching_tasks(tasks, changed_file, plan_id)?;
        match matches.as_slice() {
            [] => {
                return Err(format!(
                    "{changed_file} is not bound to an active task target"
                ));
            }
            [task] => {
                selected.insert(task.task_id.as_str(), *task);
            }
            _ => {
                let task_ids = matches
                    .iter()
                    .map(|task| task.task_id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(format!(
                    "{changed_file} maps to multiple active task targets: {task_ids}"
                ));
            }
        }
    }
    if selected.is_empty() {
        return Err("verify-landing selected no active task targets".to_string());
    }
    let plan_ids = selected
        .values()
        .map(|task| task.plan_id.as_str())
        .collect::<BTreeSet<_>>();
    if plan_id.is_none() && plan_ids.len() > 1 {
        return Err(
            "changed files map to multiple active plans; pass --plan-id to select one".to_string(),
        );
    }
    Ok(selected.values().copied().collect())
}

fn matching_tasks<'a>(
    tasks: &'a [RegistryTask],
    changed_file: &str,
    plan_id: Option<&str>,
) -> Result<Vec<&'a RegistryTask>> {
    let mut matches = Vec::new();
    for task in tasks {
        if task.status != TaskStatus::Active {
            continue;
        }
        if let Some(plan_id) = plan_id
            && task.plan_id != plan_id
        {
            continue;
        }
        if task_target_matches(task, changed_file)? {
            matches.push(task);
        }
    }
    Ok(matches)
}

fn task_target_matches(task: &RegistryTask, changed_file: &str) -> Result<bool> {
    for target in &task.targets {
        let scope = MutationScope::from_task_target(&target.file)
            .map_err(|error| format!("{} invalid target: {error}", task.task_id))?;
        if scope.allows(changed_file) {
            return Ok(true);
        }
    }
    Ok(false)
}
