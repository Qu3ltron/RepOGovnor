use super::*;
use crate::mutation_hook::{inspect_hook_payload, target_allows, verify_mutation_payload};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn activates_and_completes_behavior_backed_task() {
    let root = temp_root("activate");
    seed_repo(&root);
    fs::write(root.join("docs/plans/sample.md"), sample_plan("true")).unwrap();

    activate_plan(&root, "docs/plans/sample.md").unwrap();
    update_task_status(&root, "TASK-2026-05-30-sample-001", "completed").unwrap();
    let report = report_plan(&root, "PLAN-2026-05-30-sample").unwrap();

    assert_eq!(report.completed, 1);
    assert_eq!(report.remaining, 0);
}

#[test]
fn completion_runs_behavior_confirmation() {
    let root = temp_root("confirm-fail");
    seed_repo(&root);
    fs::write(root.join("docs/plans/sample.md"), sample_plan("false")).unwrap();
    activate_plan(&root, "docs/plans/sample.md").unwrap();

    let error = update_task_status(&root, "TASK-2026-05-30-sample-001", "completed")
        .expect_err("completion must fail");

    assert!(error.contains("confirmation failed"), "{error}");
}

#[test]
fn deferred_requires_basis_and_reactivation() {
    let root = temp_root("defer");
    seed_repo(&root);
    fs::write(root.join("docs/plans/sample.md"), sample_plan("true")).unwrap();
    activate_plan(&root, "docs/plans/sample.md").unwrap();

    let error = update_task_status(&root, "TASK-2026-05-30-sample-001", "deferred")
        .expect_err("deferred status must require TASK_DEFER");
    assert!(error.contains("TASK_DEFER"), "{error}");
}

#[test]
fn mutation_hook_denies_unbound_implementation_path() {
    let root = temp_root("hook");
    seed_repo(&root);
    fs::write(root.join("docs/plans/sample.md"), sample_plan("true")).unwrap();
    activate_plan(&root, "docs/plans/sample.md").unwrap();
    let registry = load_registry(&root).unwrap();
    let targets = registry
        .tasks
        .iter()
        .flat_map(|task| task.targets.iter().map(|target| target.file.as_str()))
        .collect::<Vec<_>>();

    assert!(target_allows("src/lib.rs", &targets));
    assert!(!target_allows("src/other.rs", &targets));
}

#[test]
fn hook_allows_governance_write_with_unactivated_manifest() {
    let root = temp_root("hook-governance-unactivated");
    seed_repo(&root);
    fs::write(root.join("docs/plans/sample.md"), sample_plan("true")).unwrap();
    let validation_error = validate_all(&root).expect_err("unactivated plan must fail validate");
    assert!(
        validation_error.contains("must be activated in registry"),
        "{validation_error}"
    );
    let payload = serde_json::json!({
        "tool_name": "apply_patch",
        "tool_input": {
            "command": "*** Begin Patch\n*** Update File: docs/plans/sample.md\n@@\n old\n*** End Patch\n"
        }
    });

    verify_mutation_payload(&root, &payload.to_string()).unwrap();
}

#[test]
fn hook_allows_activation_command_with_unactivated_manifest() {
    let root = temp_root("hook-activate-unactivated");
    seed_repo(&root);
    fs::write(root.join("docs/plans/sample.md"), sample_plan("true")).unwrap();
    let payload = serde_json::json!({
        "tool_name": "exec_command",
        "tool_input": {
            "command": ".codex/scripts/task-registry activate docs/plans/sample.md"
        }
    });

    verify_mutation_payload(&root, &payload.to_string()).unwrap();
}

#[test]
fn hook_denies_unbound_implementation_when_unactivated_manifest_exists() {
    let root = temp_root("hook-deny-unactivated-target");
    seed_repo(&root);
    fs::write(root.join("docs/plans/sample.md"), sample_plan("true")).unwrap();
    let payload = serde_json::json!({
        "toolCall": {
            "name": "edit_file",
            "args": {
                "TargetFile": "src/lib.rs",
                "CodeEdit": "change"
            }
        }
    });

    let error = verify_mutation_payload(&root, &payload.to_string())
        .expect_err("unactivated manifest targets must not authorize implementation writes");
    assert!(
        error.contains("src/lib.rs is not bound to an active registry task target"),
        "{error}"
    );
}

#[test]
fn hook_denies_malformed_uncertain_and_outside_payloads() {
    let root = temp_root("hook-deny-invalid");
    seed_repo(&root);

    let malformed = verify_mutation_payload(&root, "{not json").expect_err("malformed JSON fails");
    assert!(malformed.contains("parse hook JSON"), "{malformed}");

    let uncertain = serde_json::json!({
        "toolCall": {
            "name": "edit_file",
            "args": {
                "CodeEdit": "change without target"
            }
        }
    });
    let uncertain_error = verify_mutation_payload(&root, &uncertain.to_string())
        .expect_err("uncertain write payload must fail");
    assert!(
        uncertain_error.contains("did not expose a deterministic target path"),
        "{uncertain_error}"
    );

    let outside = serde_json::json!({
        "toolCall": {
            "name": "edit_file",
            "args": {
                "TargetFile": "/tmp/outside-governance-plugin.rs",
                "CodeEdit": "change"
            }
        }
    });
    let outside_error =
        verify_mutation_payload(&root, &outside.to_string()).expect_err("outside path must fail");
    assert!(
        outside_error.contains("outside the repo root"),
        "{outside_error}"
    );
}

#[test]
fn hook_payload_extracts_native_paths() {
    let agy = serde_json::json!({
        "toolCall": {
            "name": "edit_file",
            "args": {
                "TargetFile": "src/lib.rs",
                "CodeEdit": "change"
            }
        }
    });
    let codex = serde_json::json!({
        "tool_name": "apply_patch",
        "tool_input": {
            "command": "*** Begin Patch\n*** Update File: src/main.rs\n@@\n old\n*** End Patch\n"
        }
    });

    assert_eq!(inspect_hook_payload(&agy).paths, vec!["src/lib.rs"]);
    assert_eq!(inspect_hook_payload(&codex).paths, vec!["src/main.rs"]);
}

#[test]
fn hook_payload_flags_uncertain_write_tools() {
    let payload = serde_json::json!({
        "toolCall": {
            "name": "edit_file",
            "args": {
                "CodeEdit": "change without target"
            }
        }
    });

    assert!(inspect_hook_payload(&payload).uncertain_write());
}

#[test]
fn metrics_counts_local_receipts() {
    let root = temp_root("metrics");
    seed_repo(&root);
    append_event(
        &root,
        EventRecord {
            timestamp: timestamp(),
            command: "validate".to_string(),
            outcome: "ok".to_string(),
            duration_ms: 1,
            detail: "test".to_string(),
        },
    )
    .unwrap();

    let report = metrics(&root).unwrap();

    assert_eq!(report.events, 1);
}

#[test]
fn source_limit_reports_and_plans_violations() {
    let root = temp_root("source-limit");
    let long_file = root.join("src/large.rs");
    fs::create_dir_all(long_file.parent().unwrap()).unwrap();
    fs::write(&long_file, "fn item() {}\n".repeat(SOURCE_LINE_LIMIT + 1)).unwrap();

    let error = source_limit::check(root.as_path()).expect_err("over-limit file must fail");
    assert!(error.contains("src/large.rs"), "{error}");

    let plan = source_limit::plan(root.as_path(), Some("src/large.rs"), false).unwrap();
    assert!(plan.contains("Split plan for `src/large.rs`"), "{plan}");
    assert!(plan.contains("module_support_part_002.rs"), "{plan}");
}

#[test]
fn source_limit_includes_extensionless_scripts() {
    let root = temp_root("source-limit-script");
    let script = root.join("scripts/task-registry");
    fs::create_dir_all(script.parent().unwrap()).unwrap();
    fs::write(
        &script,
        "#!/usr/bin/env bash\n".repeat(SOURCE_LINE_LIMIT + 1),
    )
    .unwrap();

    let error = source_limit::check(root.as_path()).expect_err("extensionless script must fail");
    assert!(error.contains("scripts/task-registry"), "{error}");
}

#[test]
fn source_limit_includes_extensionless_config() {
    let root = temp_root("source-limit-config");
    fs::write(
        root.join(".gitignore"),
        "target/\n".repeat(SOURCE_LINE_LIMIT + 1),
    )
    .unwrap();

    let error = source_limit::check(root.as_path()).expect_err("extensionless config must fail");
    assert!(error.contains(".gitignore"), "{error}");
}

#[test]
fn source_limit_check_rejects_unexpected_args() {
    let root = temp_root("source-limit-extra-args");
    let args = vec![
        "check".to_string(),
        "--root".to_string(),
        root.display().to_string(),
    ];

    let error = source_limit::run_command(root.as_path(), &args)
        .expect_err("source-limit check must reject trailing args");

    assert!(
        error.contains("usage: task-registry-flow source-limit"),
        "{error}"
    );
}

fn seed_repo(root: &Path) {
    fs::create_dir_all(root.join("docs/plans")).unwrap();
    fs::create_dir_all(root.join("docs/task-registry/archive")).unwrap();
    fs::write(
        root.join(REGISTRY_PATH),
        r#"
schema_version = 1
registry_id = "test-task-registry"
registry_authority = "docs/task-registry.toml"
activation_skill = "task-registry-flow"
hash_algorithm = "sha256(normalized plan text: CRLF/CR converted to LF, trailing whitespace stripped from each line, exactly one final newline)"
status_vocabulary = ["planned", "active", "blocked", "deferred", "completed", "cancelled"]
archive_paths = []
plans = []
tasks = []
"#,
    )
    .unwrap();
}

fn sample_plan(command: &str) -> String {
    format!(
        r#"# Sample Plan

## Task Manifest

```toml
schema_version = 1
plan_id = "PLAN-2026-05-30-sample"

[[behaviors]]
behavior_id = "B-001-sample"
title = "Sample behavior"
given = "A seeded registry"
when = "The task completes"
then = "The confirmation passes"
confirmation = "{command}"

[[tasks]]
task_id = "TASK-2026-05-30-sample-001"
title = "Sample task"
status = "planned"
kind = "test"
reason = "Exercise task registry behavior"
acceptance_proof = "Behavior B-001-sample: {command}"
behavior_ids = ["B-001-sample"]
[[tasks.targets]]
file = "src/lib.rs"
object = "sample_task"
required_change = "Update the sample task."
```
"#
    )
}

fn temp_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = env::temp_dir().join(format!("task-registry-flow-{label}-{unique}"));
    fs::create_dir_all(&root).unwrap();
    root
}
