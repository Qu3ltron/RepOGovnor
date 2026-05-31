use std::collections::BTreeSet;
use std::path::Path;

use crate::model::{ActivatedManifest, ManifestTask, RegistryPlan, RegistryTask, Result};
use crate::plan_contract;
use crate::registry_io::{load_registry, save_registry};
use crate::runtime::{discover_manifests, load_manifest, today};
use crate::schema::TaskStatus;
use crate::validation::{
    parse_status, reject_empty, validate_deferred_task, validate_manifest_for_activation,
    validate_registry_root, validate_registry_with_manifests, validate_transition,
};
use crate::verifiers::verify_behaviors;

pub(crate) fn activate_plan(root: &Path, plan_path: &str) -> Result<()> {
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

pub(crate) fn update_task_status(root: &Path, task_id: &str, status: &str) -> Result<()> {
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
        validate_transition(task.status, status)?;
        task.status = status;
        task.updated_at = today();
        task.plan_id.clone()
    };
    refresh_plan_status(&mut registry, &plan_id);
    validate_registry_with_manifests(root, &registry, &discover_manifests(root)?)?;
    save_registry(root, &registry)
}

pub(crate) fn defer_task(
    root: &Path,
    task_id: &str,
    basis: &str,
    reactivation: &str,
) -> Result<()> {
    reject_empty("deferral_governance_basis", basis)?;
    reject_empty("reactivation_condition", reactivation)?;
    let mut registry = load_registry(root)?;
    let plan_id = {
        let task = registry
            .tasks
            .iter_mut()
            .find(|task| task.task_id == task_id)
            .ok_or_else(|| format!("missing task_id {task_id}"))?;
        validate_transition(task.status, TaskStatus::Deferred)?;
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

fn build_registry_task(
    task: &ManifestTask,
    manifest: &ActivatedManifest,
    existing: Option<&RegistryTask>,
    today: &str,
) -> RegistryTask {
    RegistryTask {
        task_id: task.task_id.clone(),
        plan_id: manifest.manifest.plan_id.clone(),
        status: existing
            .map(|task| task.status)
            .unwrap_or(TaskStatus::Active),
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

fn refresh_plan_status(registry: &mut crate::model::TaskRegistry, plan_id: &str) {
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
