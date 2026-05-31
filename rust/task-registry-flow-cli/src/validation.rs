use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::str::FromStr;

use crate::model::{
    ActivatedManifest, Behavior, EXTERNAL_BLOCKER_INDICATORS, ManifestTask, PLAN_DIR, PlanManifest,
    ProjectedStep, REGISTRY_PATH, RegistryPlan, RegistryTask, Result, TaskBlocker, TaskRegistry,
    TaskRegistryArchive, TaskTarget,
};
use crate::runtime::{normalize_relative_path, normalized_file_hash};
use crate::schema::{MutationScope, TaskStatus};
use crate::{manifest, plan_contract};

// ---------------------------------------------------------------------------
// Transition validation (added for Gap 2 — structural phase; integrated here
// since validation.rs is the natural home for policy rules)
// ---------------------------------------------------------------------------

fn allowed_transitions_from(status: TaskStatus) -> &'static [TaskStatus] {
    match status {
        TaskStatus::Planned => &[TaskStatus::Active, TaskStatus::Cancelled],
        TaskStatus::Active => &[
            TaskStatus::Blocked,
            TaskStatus::Deferred,
            TaskStatus::Completed,
            TaskStatus::Cancelled,
        ],
        TaskStatus::Blocked => &[TaskStatus::Active, TaskStatus::Cancelled],
        TaskStatus::Deferred => &[TaskStatus::Planned, TaskStatus::Cancelled],
        TaskStatus::Completed | TaskStatus::Cancelled => &[],
    }
}

pub(crate) fn validate_transition(from: TaskStatus, to: TaskStatus) -> Result<()> {
    if from == to {
        return Ok(());
    }
    let allowed = allowed_transitions_from(from);
    if allowed.contains(&to) {
        Ok(())
    } else if allowed.is_empty() {
        Err(format!(
            "cannot transition from {from} to {to}; {from} is a terminal status"
        ))
    } else {
        Err(format!(
            "cannot transition from {from} to {to}; allowed transitions from {from} are: {}",
            allowed
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }
}

// ---------------------------------------------------------------------------
// Registry / manifest validation
// ---------------------------------------------------------------------------

pub(crate) fn validate_registry_with_manifests(
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

pub(crate) fn validate_registry_root(registry: &TaskRegistry) -> Result<()> {
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

pub(crate) fn validate_archive_shape(path: &str, archive: &TaskRegistryArchive) -> Result<()> {
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

pub(crate) fn validate_archive_path(path: &str) -> Result<()> {
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
    if task.source_plan_hash_sha256 != plan.plan_hash_sha256 {
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

pub(crate) fn validate_manifest(manifest: &PlanManifest) -> Result<()> {
    manifest::require_schema_version_v2(manifest.schema_version, "Task Manifest")?;
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

pub(crate) fn validate_manifest_for_activation(manifest: &PlanManifest) -> Result<()> {
    validate_manifest(manifest)?;
    Ok(())
}

fn validate_manifest_for_registry(manifest: &PlanManifest, _status: TaskStatus) -> Result<()> {
    validate_manifest(manifest)?;
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

pub(crate) fn validate_deferred_task(task: &RegistryTask) -> Result<()> {
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

// ---------------------------------------------------------------------------
// Shared validators / predicates
// ---------------------------------------------------------------------------

pub(crate) fn validate_plan_path(path: &str) -> Result<()> {
    let plan_path = normalize_relative_path(path)?;
    if !plan_path.starts_with(&format!("{PLAN_DIR}/")) {
        return Err(format!("plan path must be under {PLAN_DIR}: {path}"));
    }
    if !plan_path.ends_with(".md") {
        return Err(format!("plan path must be a markdown file: {path}"));
    }
    Ok(())
}

pub(crate) fn parse_status(status: &str, object: &str) -> Result<TaskStatus> {
    TaskStatus::from_str(status).map_err(|error| format!("{object} {error}"))
}

fn assert_sha256(field: &str, value: &str) -> Result<()> {
    if value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(format!("{field} must be a 64-character SHA-256 hex string"))
    }
}

pub(crate) fn reject_empty(field: &str, value: &str) -> Result<()> {
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
