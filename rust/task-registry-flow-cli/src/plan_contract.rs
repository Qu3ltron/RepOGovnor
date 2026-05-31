use std::collections::{BTreeMap, BTreeSet};

use crate::model::{PlanManifest, Result};
use crate::schema::{BehaviorPolarity, TaskKind, TaskStatus};

const REQUIRED_SECTIONS: &[&str] = &[
    "Approved Scope",
    "Phased Required Change Checklist",
    "Per-Gap Success Criteria",
    "Validation Plan",
    "Walkthrough Evidence",
    "Task Manifest",
];

const PLACEHOLDER_TOKENS: &[&str] = &["TBD", "TODO", "???"];

pub(crate) fn validate_activation_contract(
    plan_path: &str,
    plan_body: &str,
    manifest: &PlanManifest,
) -> Result<()> {
    if manifest.schema_version != 2 {
        return Ok(());
    }
    validate_required_sections(plan_path, plan_body)?;
    reject_placeholders(plan_path, plan_body)?;
    validate_gap_behavior_coverage(plan_path, manifest)?;
    validate_task_behavior_links(plan_path, manifest)?;
    Ok(())
}

pub(crate) fn validate_registry_contract(
    plan_path: &str,
    plan_body: &str,
    manifest: &PlanManifest,
    status: TaskStatus,
) -> Result<()> {
    if manifest.schema_version != 2
        || matches!(status, TaskStatus::Completed | TaskStatus::Cancelled)
    {
        return Ok(());
    }
    validate_activation_contract(plan_path, plan_body, manifest)
}

fn validate_required_sections(plan_path: &str, plan_body: &str) -> Result<()> {
    for section in REQUIRED_SECTIONS {
        let heading = format!("## {section}");
        let count = plan_body
            .lines()
            .filter(|line| line.trim() == heading)
            .count();
        match count {
            1 => {}
            0 => return Err(format!("{plan_path} missing ## {section}")),
            _ => return Err(format!("{plan_path} has multiple ## {section} sections")),
        }
    }
    Ok(())
}

fn reject_placeholders(plan_path: &str, plan_body: &str) -> Result<()> {
    for (index, line) in plan_body.lines().enumerate() {
        let trimmed = line.trim();
        for token in PLACEHOLDER_TOKENS {
            if trimmed.contains(token) {
                return Err(format!(
                    "{plan_path} line {} contains unresolved placeholder token {token}",
                    index + 1
                ));
            }
        }
        if contains_angle_placeholder(trimmed) {
            return Err(format!(
                "{plan_path} line {} contains unresolved angle-bracket placeholder",
                index + 1
            ));
        }
    }
    Ok(())
}

fn contains_angle_placeholder(line: &str) -> bool {
    let Some(start) = line.find('<') else {
        return false;
    };
    let Some(end) = line[start + 1..].find('>') else {
        return false;
    };
    let candidate = &line[start + 1..start + 1 + end];
    !candidate.trim().is_empty()
        && candidate
            .chars()
            .all(|value| value.is_ascii_alphanumeric() || matches!(value, '-' | '_' | ' '))
}

fn validate_gap_behavior_coverage(plan_path: &str, manifest: &PlanManifest) -> Result<()> {
    let mut coverage: BTreeMap<&str, BTreeSet<BehaviorPolarity>> = BTreeMap::new();
    for behavior in &manifest.behaviors {
        let gap_id = behavior.gap_id.as_deref().ok_or_else(|| {
            format!(
                "{} behavior {} requires gap_id",
                manifest.plan_id, behavior.behavior_id
            )
        })?;
        if gap_id.trim().is_empty() {
            return Err(format!(
                "{} behavior {} requires non-empty gap_id",
                manifest.plan_id, behavior.behavior_id
            ));
        }
        let polarity = behavior.polarity.ok_or_else(|| {
            format!(
                "{} behavior {} requires polarity",
                manifest.plan_id, behavior.behavior_id
            )
        })?;
        if polarity != BehaviorPolarity::Validation {
            coverage.entry(gap_id).or_default().insert(polarity);
        }
    }
    for (gap_id, polarities) in coverage {
        if !polarities.contains(&BehaviorPolarity::Positive)
            || !polarities.contains(&BehaviorPolarity::Negative)
        {
            return Err(format!(
                "{plan_path} gap {gap_id} requires positive and negative behavior coverage"
            ));
        }
    }
    Ok(())
}

fn validate_task_behavior_links(plan_path: &str, manifest: &PlanManifest) -> Result<()> {
    let behavior_polarity = manifest
        .behaviors
        .iter()
        .map(|behavior| (behavior.behavior_id.as_str(), behavior.polarity))
        .collect::<BTreeMap<_, _>>();
    for task in &manifest.tasks {
        if !task_kind_requires_gap_proof(task.kind) {
            continue;
        }
        let has_gap_behavior = task.behavior_ids.iter().any(|behavior_id| {
            behavior_polarity
                .get(behavior_id.as_str())
                .and_then(|polarity| *polarity)
                .is_some_and(|polarity| polarity != BehaviorPolarity::Validation)
        });
        if !has_gap_behavior {
            return Err(format!(
                "{plan_path} task {} requires positive or negative behavior proof, not validation-only proof",
                task.task_id
            ));
        }
    }
    Ok(())
}

fn task_kind_requires_gap_proof(kind: TaskKind) -> bool {
    matches!(
        kind,
        TaskKind::Authorization
            | TaskKind::Governance
            | TaskKind::Implementation
            | TaskKind::Migration
            | TaskKind::Release
            | TaskKind::Schema
    )
}
