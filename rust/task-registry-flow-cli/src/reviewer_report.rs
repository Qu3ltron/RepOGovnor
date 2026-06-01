use crate::metrics::{format_metrics, metrics};
use crate::model::{RegistryTask, Result};
use crate::runtime::load_registry;
use crate::schema::TaskStatus;
use std::path::Path;

const MAX_ACTIVE_TASKS: usize = 8;
const MAX_LANDED_TASKS: usize = 8;
const MAX_CHANGED_FILES_PER_TASK: usize = 8;
const MAX_BLOCKED_OR_DEFERRED_TASKS: usize = 12;
const USAGE: &str = "usage: task-registry-flow reviewer-report [--format text|markdown]";

pub(crate) fn run(root: &Path, args: &[String]) -> Result<String> {
    match args {
        [] => render(root),
        [flag, format] if flag == "--format" && format == "text" => render(root),
        [flag, format] if flag == "--format" && format == "markdown" => render_markdown(root),
        _ => Err(USAGE.to_string()),
    }
}

pub(crate) fn render(root: &Path) -> Result<String> {
    let registry = load_registry(root)?;
    let metrics = metrics(root)?;
    let active_plans = registry
        .plans
        .iter()
        .filter(|plan| plan.status == TaskStatus::Active)
        .count();
    let landed_tasks = registry
        .tasks
        .iter()
        .filter(|task| task.status == TaskStatus::Completed)
        .count();
    let changed_targets = registry
        .tasks
        .iter()
        .map(|task| task.completion_changed_files.len())
        .sum::<usize>();
    let blocked_or_deferred = registry
        .tasks
        .iter()
        .filter(|task| matches!(task.status, TaskStatus::Blocked | TaskStatus::Deferred))
        .collect::<Vec<_>>();

    let mut lines = vec![
        "Reviewer report".to_string(),
        format!("Active plans: {active_plans}"),
        format!("Landed tasks: {landed_tasks}"),
        format!("Changed targets: {changed_targets}"),
        format!("Blocked/deferred: {}", blocked_or_deferred.len()),
        format!("Receipts: {}", format_metrics(&metrics)),
        "Governance proof: approved plans, task targets, behavior verifiers, receipts, and release-source checks.".to_string(),
        "Product correctness proof: project-owned domain tests, code review, security review, and acceptance evidence.".to_string(),
        "Boundary: governance proof is not product correctness proof.".to_string(),
    ];

    append_active_plans(&mut lines, &registry.tasks);
    append_landed_targets(&mut lines, &registry.tasks);
    append_blocked_or_deferred(&mut lines, &blocked_or_deferred);
    Ok(lines.join("\n"))
}

pub(crate) fn render_markdown(root: &Path) -> Result<String> {
    let registry = load_registry(root)?;
    let metrics = metrics(root)?;
    let active_plans = registry
        .plans
        .iter()
        .filter(|plan| plan.status == TaskStatus::Active)
        .count();
    let landed_tasks = registry
        .tasks
        .iter()
        .filter(|task| task.status == TaskStatus::Completed)
        .count();
    let changed_targets = registry
        .tasks
        .iter()
        .map(|task| task.completion_changed_files.len())
        .sum::<usize>();
    let blocked_or_deferred = registry
        .tasks
        .iter()
        .filter(|task| matches!(task.status, TaskStatus::Blocked | TaskStatus::Deferred))
        .collect::<Vec<_>>();

    let mut lines = vec![
        "# Reviewer Report".to_string(),
        String::new(),
        "## Summary".to_string(),
        format!("- Active plans: {active_plans}"),
        format!("- Landed tasks: {landed_tasks}"),
        format!("- Changed targets: {changed_targets}"),
        format!("- Blocked/deferred: {}", blocked_or_deferred.len()),
        format!("- Receipts: {}", format_metrics(&metrics)),
        String::new(),
        "## Proof Boundary".to_string(),
        "- Governance proof: approved plans, task targets, behavior verifiers, receipts, and release-source checks.".to_string(),
        "- Product correctness proof: project-owned domain tests, code review, security review, and acceptance evidence.".to_string(),
        "- Boundary: governance proof is not product correctness proof.".to_string(),
    ];

    append_active_plans_markdown(&mut lines, &registry.tasks);
    append_landed_targets_markdown(&mut lines, &registry.tasks);
    append_blocked_or_deferred_markdown(&mut lines, &blocked_or_deferred);
    Ok(lines.join("\n"))
}

fn append_active_plans(lines: &mut Vec<String>, tasks: &[RegistryTask]) {
    let mut active = tasks
        .iter()
        .filter(|task| matches!(task.status, TaskStatus::Planned | TaskStatus::Active))
        .collect::<Vec<_>>();
    active.sort_by(|left, right| left.task_id.cmp(&right.task_id));
    if active.is_empty() {
        lines.push("Active task targets: none".to_string());
    } else {
        lines.push("Active task targets:".to_string());
        let omitted = active.len().saturating_sub(MAX_ACTIVE_TASKS);
        for task in active.into_iter().take(MAX_ACTIVE_TASKS) {
            lines.push(format!(
                "- {} [{}]: {}",
                task.task_id,
                task.status.as_str(),
                task.title
            ));
            for target in &task.targets {
                lines.push(format!("  target {} -> {}", target.file, target.object));
            }
        }
        if omitted > 0 {
            lines.push(format!("  ... {omitted} more active task(s) omitted"));
        }
    }
}

fn append_active_plans_markdown(lines: &mut Vec<String>, tasks: &[RegistryTask]) {
    let mut active = tasks
        .iter()
        .filter(|task| matches!(task.status, TaskStatus::Planned | TaskStatus::Active))
        .collect::<Vec<_>>();
    active.sort_by(|left, right| left.task_id.cmp(&right.task_id));
    lines.push(String::new());
    lines.push("## Active Task Targets".to_string());
    if active.is_empty() {
        lines.push("None".to_string());
    } else {
        let omitted = active.len().saturating_sub(MAX_ACTIVE_TASKS);
        for task in active.into_iter().take(MAX_ACTIVE_TASKS) {
            lines.push(format!(
                "- `{}` (`{}`): {}",
                task.task_id,
                task.status.as_str(),
                task.title
            ));
            for target in &task.targets {
                lines.push(format!(
                    "  - target `{}` -> `{}`",
                    target.file, target.object
                ));
            }
        }
        if omitted > 0 {
            lines.push(format!("- {omitted} more active task(s) omitted"));
        }
    }
}

fn append_landed_targets(lines: &mut Vec<String>, tasks: &[RegistryTask]) {
    let mut landed = tasks
        .iter()
        .filter(|task| {
            task.status == TaskStatus::Completed && !task.completion_changed_files.is_empty()
        })
        .collect::<Vec<_>>();
    landed.sort_by(|left, right| left.task_id.cmp(&right.task_id));
    if landed.is_empty() {
        lines.push("Landed changed files: none".to_string());
    } else {
        lines.push("Landed changed files:".to_string());
        let omitted = landed.len().saturating_sub(MAX_LANDED_TASKS);
        for task in landed.into_iter().take(MAX_LANDED_TASKS) {
            lines.push(format!("- {}: {}", task.task_id, task.title));
            let omitted_files = task
                .completion_changed_files
                .len()
                .saturating_sub(MAX_CHANGED_FILES_PER_TASK);
            for file in task
                .completion_changed_files
                .iter()
                .take(MAX_CHANGED_FILES_PER_TASK)
            {
                lines.push(format!("  changed {file}"));
            }
            if omitted_files > 0 {
                lines.push(format!(
                    "  ... {omitted_files} more changed file(s) omitted"
                ));
            }
        }
        if omitted > 0 {
            lines.push(format!("  ... {omitted} more landed task(s) omitted"));
        }
    }
}

fn append_landed_targets_markdown(lines: &mut Vec<String>, tasks: &[RegistryTask]) {
    let mut landed = tasks
        .iter()
        .filter(|task| {
            task.status == TaskStatus::Completed && !task.completion_changed_files.is_empty()
        })
        .collect::<Vec<_>>();
    landed.sort_by(|left, right| left.task_id.cmp(&right.task_id));
    lines.push(String::new());
    lines.push("## Landed Changed Files".to_string());
    if landed.is_empty() {
        lines.push("None".to_string());
    } else {
        let omitted = landed.len().saturating_sub(MAX_LANDED_TASKS);
        for task in landed.into_iter().take(MAX_LANDED_TASKS) {
            lines.push(format!("- `{}`: {}", task.task_id, task.title));
            let omitted_files = task
                .completion_changed_files
                .len()
                .saturating_sub(MAX_CHANGED_FILES_PER_TASK);
            for file in task
                .completion_changed_files
                .iter()
                .take(MAX_CHANGED_FILES_PER_TASK)
            {
                lines.push(format!("  - changed `{file}`"));
            }
            if omitted_files > 0 {
                lines.push(format!("  - {omitted_files} more changed file(s) omitted"));
            }
        }
        if omitted > 0 {
            lines.push(format!("- {omitted} more landed task(s) omitted"));
        }
    }
}

fn append_blocked_or_deferred(lines: &mut Vec<String>, tasks: &[&RegistryTask]) {
    if tasks.is_empty() {
        lines.push("Blocked/deferred work: none".to_string());
    } else {
        lines.push("Blocked/deferred work:".to_string());
        let omitted = tasks.len().saturating_sub(MAX_BLOCKED_OR_DEFERRED_TASKS);
        for task in tasks.iter().take(MAX_BLOCKED_OR_DEFERRED_TASKS) {
            let reason = task
                .deferral_governance_basis
                .as_deref()
                .unwrap_or(task.reason.as_str());
            lines.push(format!(
                "- {} [{}]: {} -- {}",
                task.task_id,
                task.status.as_str(),
                task.title,
                reason
            ));
        }
        if omitted > 0 {
            lines.push(format!(
                "  ... {omitted} more blocked/deferred task(s) omitted"
            ));
        }
    }
}

fn append_blocked_or_deferred_markdown(lines: &mut Vec<String>, tasks: &[&RegistryTask]) {
    lines.push(String::new());
    lines.push("## Blocked/Deferred Work".to_string());
    if tasks.is_empty() {
        lines.push("None".to_string());
    } else {
        let omitted = tasks.len().saturating_sub(MAX_BLOCKED_OR_DEFERRED_TASKS);
        for task in tasks.iter().take(MAX_BLOCKED_OR_DEFERRED_TASKS) {
            let reason = task
                .deferral_governance_basis
                .as_deref()
                .unwrap_or(task.reason.as_str());
            lines.push(format!(
                "- `{}` (`{}`): {} -- {}",
                task.task_id,
                task.status.as_str(),
                task.title,
                reason
            ));
        }
        if omitted > 0 {
            lines.push(format!("- {omitted} more blocked/deferred task(s) omitted"));
        }
    }
}
