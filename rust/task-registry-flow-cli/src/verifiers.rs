use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

use crate::model::{Behavior, Result};
use crate::registry_io::load_registry;
use crate::runtime::{behavior_map, discover_manifests, normalize_relative_path};
use crate::schema::TaskStatus;
use crate::schema::{BehaviorVerifier, CheckStatus, RuntimeSubject, VerifierResult, VerifierType};

pub(crate) fn verifier_result(
    behavior_id: impl Into<String>,
    verifier_type: VerifierType,
    status: CheckStatus,
    path: impl Into<String>,
    expected: impl Into<String>,
    actual: impl Into<String>,
) -> VerifierResult {
    let path = path.into();
    VerifierResult {
        behavior_id: behavior_id.into(),
        verifier_type,
        status,
        subject: RuntimeSubject::path("verifier-target", path),
        expected: expected.into(),
        actual: actual.into(),
    }
}

pub(crate) fn verify_behaviors(root: &Path, filter: Option<&str>) -> Result<usize> {
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
        } => {
            run_confirmation(command, *expected_exit, behavior_id)?;
            let _ = verifier_result(
                behavior_id,
                verifier.verifier_type(),
                CheckStatus::Pass,
                command,
                format!("exit {expected_exit}"),
                format!("exit {expected_exit}"),
            );
            Ok(())
        }
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
        .arg("-c")
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
