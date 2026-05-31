use serde::{Deserialize, Serialize};

use crate::schema::{BehaviorPolarity, BehaviorVerifier, TaskKind, TaskStatus};

pub(crate) const REGISTRY_PATH: &str = "docs/task-registry.toml";
pub(crate) const REGISTRY_ARCHIVE_DIR: &str = "docs/task-registry/archive";
pub(crate) const EVENTS_PATH: &str = "docs/task-registry/events.jsonl";
pub(crate) const PLAN_DIR: &str = "docs/plans";
pub(crate) const SOURCE_LINE_LIMIT: usize = 1600;
pub(crate) const ARCHIVE_COMPLETED_PLAN_CHUNK_SIZE: usize = 8;
pub(crate) const ACTIVE_TARGET_STATUSES: &[TaskStatus] = &[TaskStatus::Planned, TaskStatus::Active];
pub(crate) const PLAN_BOOTSTRAP_PREFIX: &str = "docs/plans/";
pub(crate) const EXTERNAL_BLOCKER_INDICATORS: &[&str] = &[
    "runtime",
    "provider",
    "service",
    "external",
    "authorization",
    "credential",
    "data",
    "remote check",
    "evaluation",
];

pub(crate) type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TaskRegistry {
    pub(crate) schema_version: i64,
    pub(crate) registry_id: String,
    pub(crate) registry_authority: String,
    pub(crate) activation_skill: String,
    pub(crate) hash_algorithm: String,
    pub(crate) status_vocabulary: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) archive_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) plans: Vec<RegistryPlan>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) tasks: Vec<RegistryTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TaskRegistryArchive {
    pub(crate) schema_version: i64,
    pub(crate) registry_id: String,
    pub(crate) archive_id: String,
    pub(crate) archive_authority: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) plans: Vec<RegistryPlan>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) tasks: Vec<RegistryTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RegistryPlan {
    pub(crate) plan_id: String,
    pub(crate) plan_path: String,
    pub(crate) plan_hash_sha256: String,
    pub(crate) activated_at: String,
    pub(crate) status: TaskStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RegistryTask {
    pub(crate) task_id: String,
    pub(crate) plan_id: String,
    pub(crate) status: TaskStatus,
    pub(crate) title: String,
    pub(crate) kind: TaskKind,
    pub(crate) source_plan_path: String,
    pub(crate) source_plan_hash_sha256: String,
    pub(crate) reason: String,
    pub(crate) acceptance_proof: String,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) behavior_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) deferral_governance_basis: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) reactivation_condition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) closure_plan_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) targets: Vec<TaskTarget>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) blockers: Vec<TaskBlocker>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) projected_steps: Vec<ProjectedStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct TaskTarget {
    pub(crate) file: String,
    pub(crate) object: String,
    pub(crate) required_change: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TaskBlocker {
    pub(crate) blocker_id: String,
    pub(crate) blocked_object: String,
    pub(crate) blocked_change: String,
    pub(crate) current_state: String,
    pub(crate) unblock_condition: String,
    pub(crate) evidence_required: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectedStep {
    pub(crate) step_id: String,
    pub(crate) status: TaskStatus,
    pub(crate) file: String,
    pub(crate) object: String,
    pub(crate) required_change: String,
    pub(crate) blocked_by: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PlanManifest {
    pub(crate) schema_version: i64,
    pub(crate) plan_id: String,
    #[serde(default)]
    pub(crate) behaviors: Vec<Behavior>,
    #[serde(default)]
    pub(crate) tasks: Vec<ManifestTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Behavior {
    pub(crate) behavior_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) gap_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) polarity: Option<BehaviorPolarity>,
    pub(crate) title: String,
    pub(crate) given: String,
    pub(crate) when: String,
    pub(crate) then: String,
    pub(crate) confirmation: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) verifiers: Vec<BehaviorVerifier>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ManifestTask {
    pub(crate) task_id: String,
    pub(crate) title: String,
    pub(crate) status: TaskStatus,
    pub(crate) kind: TaskKind,
    pub(crate) reason: String,
    pub(crate) acceptance_proof: String,
    pub(crate) behavior_ids: Vec<String>,
    #[serde(default)]
    pub(crate) deferral_governance_basis: Option<String>,
    #[serde(default)]
    pub(crate) reactivation_condition: Option<String>,
    #[serde(default)]
    pub(crate) targets: Vec<TaskTarget>,
    #[serde(default)]
    pub(crate) blockers: Vec<TaskBlocker>,
    #[serde(default)]
    pub(crate) projected_steps: Vec<ProjectedStep>,
}

#[derive(Debug)]
pub(crate) struct ActivatedManifest {
    pub(crate) plan_path: String,
    pub(crate) plan_hash_sha256: String,
    pub(crate) plan_body: String,
    pub(crate) manifest: PlanManifest,
}

#[derive(Debug)]
pub(crate) struct ValidationReport {
    pub(crate) registry_plan_count: usize,
    pub(crate) registry_task_count: usize,
    pub(crate) manifest_count: usize,
}

#[derive(Debug)]
pub(crate) struct PlanReport {
    pub(crate) plan_id: String,
    pub(crate) completed: usize,
    pub(crate) deferred: usize,
    pub(crate) blocked: usize,
    pub(crate) cancelled: usize,
    pub(crate) remaining: usize,
    pub(crate) deferred_or_blocked: Vec<(String, String, String)>,
}

#[derive(Debug, Serialize)]
pub(crate) struct MetricsReport {
    pub(crate) plans: usize,
    pub(crate) tasks: usize,
    pub(crate) planned: usize,
    pub(crate) active: usize,
    pub(crate) blocked: usize,
    pub(crate) deferred: usize,
    pub(crate) completed: usize,
    pub(crate) cancelled: usize,
    pub(crate) manifests: usize,
    pub(crate) events: usize,
    pub(crate) failed_events: usize,
    pub(crate) mutation_denials: usize,
    pub(crate) malformed_events: usize,
    pub(crate) chained_events: usize,
    pub(crate) unchained_events: usize,
    pub(crate) receipt_chain_breaks: usize,
}

pub(crate) type EventRecord = crate::schema::ReceiptEvent;
