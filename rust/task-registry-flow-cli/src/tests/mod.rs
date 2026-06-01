use super::*;
use crate::mutation_hook::{
    inspect_hook_payload, target_allows, verify_mutation_payload_for_format,
};
use crate::schema::{
    BehaviorPolarity, CheckReport, CheckStatus, CliCommand, Diagnostic, DiagnosticSeverity,
    EventOutcome, HookFormat, InstallAction, MutationScope, ReportSurface, TaskKind, TaskStatus,
    VerifierType,
};
use std::env;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};

mod activation_terminal_tests;
mod archive_tests;
mod hook_command_tests;
mod landing_tests;
mod metrics_tests;
mod receipt_chain_tests;
mod release_source_tests;
mod reviewer_report_tests;
mod security_tests;
mod state_transition_tests;
mod status_check_tests;
mod typed_runtime_surface_tests;
mod verify_chain_tests;

#[test]
fn activates_and_lands_behavior_backed_task() {
    let root = temp_root("activate");
    seed_repo(&root);
    fs::write(root.join("docs/plans/sample.md"), sample_plan("true")).unwrap();

    activate_plan(&root, "docs/plans/sample.md").unwrap();
    crate::landing::run_command(&root, &changed_files_args(&["src/lib.rs"])).unwrap();
    let report = report_plan(&root, "PLAN-2026-05-30-sample").unwrap();

    assert_eq!(report.completed, 1);
    assert_eq!(report.remaining, 0);
}

#[test]
fn activation_accepts_comprehensive_phased_v2_contract() {
    let root = temp_root("activation-contract-accepts");
    seed_repo(&root);
    fs::write(root.join("docs/plans/sample.md"), sample_plan("true")).unwrap();

    activate_plan(&root, "docs/plans/sample.md").unwrap();
    let registry = load_registry(&root).unwrap();

    assert_eq!(registry.plans.len(), 1);
    assert_eq!(registry.tasks.len(), 1);
}

#[test]
fn activation_rejects_plan_missing_required_sections() {
    let plan = sample_plan("true").replace(
        "## Phased Required Change Checklist",
        "## Required Change Checklist",
    );

    assert_activation_error(plan, "missing ## Phased Required Change Checklist");
}

#[test]
fn activation_rejects_plan_with_tbd_or_placeholders() {
    let plan = sample_plan("true").replace(
        "Exercise sample task registry behavior.",
        "Exercise sample task registry behavior.\n\nUnresolved marker: TBD",
    );

    assert_activation_error(plan, "unresolved placeholder token");
}

#[test]
fn activation_rejects_v2_behavior_missing_gap_id() {
    let plan = sample_plan("true").replacen("gap_id = \"GAP-SAMPLE\"\n", "", 1);

    assert_activation_error(plan, "requires gap_id");
}

#[test]
fn activation_rejects_v2_behavior_missing_polarity() {
    let plan = sample_plan("true").replacen("polarity = \"positive\"\n", "", 1);

    assert_activation_error(plan, "requires polarity");
}

#[test]
fn activation_rejects_gap_without_negative_behavior() {
    let plan =
        sample_plan("true").replacen("polarity = \"negative\"", "polarity = \"validation\"", 1);

    assert_activation_error(plan, "requires positive and negative behavior coverage");
}

#[test]
fn activation_rejects_implementation_task_with_only_validation_behavior() {
    let validation_behavior = r#"
[[behaviors]]
behavior_id = "B-003-validation-only"
gap_id = "GAP-VALIDATION"
polarity = "validation"
title = "Validation only"
given = "A completed implementation"
when = "Full validation runs"
then = "The repository gates pass"
confirmation = "true"

[[behaviors.verifiers]]
type = "command"
command = "true"
expected_exit = 0

"#;
    let plan = sample_plan("true")
        .replace("[[tasks]]", &format!("{validation_behavior}[[tasks]]"))
        .replace("kind = \"test\"", "kind = \"schema\"")
        .replace(
            "behavior_ids = [\"B-001-sample\", \"B-002-sample-negative\"]",
            "behavior_ids = [\"B-003-validation-only\"]",
        );

    assert_activation_error(plan, "validation-only proof");
}

#[test]
fn activation_rejects_broad_or_wildcard_targets() {
    let plan = sample_plan("true").replace("file = \"src/lib.rs\"", "file = \"src/*\"");

    assert_activation_error(plan, "glob metacharacters");
}

#[test]
fn landing_runs_behavior_confirmation() {
    let root = temp_root("confirm-fail");
    seed_repo(&root);
    fs::write(root.join("docs/plans/sample.md"), sample_plan("false")).unwrap();
    activate_plan(&root, "docs/plans/sample.md").unwrap();
    update_task_status(&root, "TASK-2026-05-30-sample-001", "active").unwrap();

    let error = crate::landing::run_command(&root, &changed_files_args(&["src/lib.rs"]))
        .expect_err("landing must fail");

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
fn schema_unknown_runtime_values_fail() {
    assert!(HookFormat::from_str("legacy").is_err());
    assert!(TaskKind::from_str("misc").is_err());
    assert!(VerifierType::from_str("shell prose").is_err());
    assert!(BehaviorPolarity::from_str("mixed").is_err());
    assert!(CheckStatus::from_str("maybe").is_err());
    assert_eq!(
        InstallAction::from_str("replace-symlink").unwrap().as_str(),
        "replace-symlink"
    );
}

#[test]
fn schema_event_serializes_versioned_receipt() {
    let event = EventRecord::new(
        "2026-05-30T00:00:00Z".to_string(),
        CliCommand::Validate,
        EventOutcome::Ok,
        7,
        "schema test".to_string(),
    );
    let value = serde_json::to_value(event).unwrap();

    assert_eq!(value["schema_version"], 2);
    assert_eq!(value["command"], "validate");
    assert_eq!(value["outcome"], "ok");
    assert_eq!(value["subject"]["kind"], "command");
    assert_eq!(value["summary"], "schema test");
}

#[test]
fn receipt_read_only_commands_do_not_mutate_events() {
    assert!(!crate::receipts::should_record(CliCommand::Validate, false));
    assert!(!crate::receipts::should_record(CliCommand::Metrics, false));
    assert!(!crate::receipts::should_record(
        CliCommand::ReleaseCheck,
        false
    ));
    assert!(!crate::receipts::should_record(
        CliCommand::SourceLimit,
        false
    ));
    assert!(!crate::receipts::should_record(
        CliCommand::VerifyBehaviors,
        false
    ));
}

#[test]
fn receipt_record_flag_required_for_validation_receipt() {
    assert!(!crate::receipts::should_record(CliCommand::Validate, false));
    assert!(crate::receipts::should_record(CliCommand::Validate, true));
    assert!(crate::receipts::should_record(CliCommand::Activate, false));
}

#[test]
fn receipt_v2_serializes_typed_subjects() {
    let event = EventRecord::mutation_denied(
        "2026-05-31T00:00:00Z".to_string(),
        3,
        "src/lib.rs".to_string(),
        "not bound".to_string(),
    );
    let value = serde_json::to_value(event).unwrap();

    assert_eq!(value["schema_version"], 2);
    assert_eq!(value["subject"]["kind"], "mutation-target");
    assert_eq!(value["mutation_denial"]["path"], "src/lib.rs");
    assert_eq!(value["mutation_denial"]["reason"], "not bound");
    let verifier = crate::verifiers::verifier_result(
        "B-001",
        VerifierType::Command,
        CheckStatus::Pass,
        "docs/plans/sample.md",
        "exit 0",
        "exit 0",
    );
    let verifier_value = serde_json::to_value(verifier).unwrap();
    assert_eq!(verifier_value["subject"]["kind"], "verifier-target");
}

#[test]
fn mutation_scope_exact_file_rejects_prefix_collision() {
    let targets = vec!["src/lib.rs"];

    assert!(target_allows("src/lib.rs", &targets));
    assert!(!target_allows("src/lib.rs.bak", &targets));
}

#[test]
fn mutation_scope_directory_tree_is_explicit() {
    let targets = vec!["fixtures/"];

    assert!(target_allows("fixtures/input.json", &targets));
    assert!(!target_allows("fixtures", &targets));
    assert!(!target_allows("fixtures-other/input.json", &targets));
}

#[test]
fn mutation_scope_broad_targets_fail_closed() {
    assert!(MutationScope::from_task_target(".").is_err());
    assert!(MutationScope::from_task_target("src/").is_err());
    assert!(MutationScope::from_task_target(".codex/").is_err());
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

    verify_mutation_payload_for_format(&root, HookFormat::Codex, &payload.to_string()).unwrap();
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

    verify_mutation_payload_for_format(&root, HookFormat::Codex, &payload.to_string()).unwrap();
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

    let error =
        verify_mutation_payload_for_format(&root, HookFormat::Antigravity, &payload.to_string())
            .expect_err("unactivated manifest targets must not authorize implementation writes");
    assert!(
        error.contains("src/lib.rs is not bound to an active registry task target"),
        "{error}"
    );
}

#[test]
fn hook_denies_runtime_governance_write_without_active_target() {
    let root = temp_root("hook-deny-runtime-governance");
    seed_repo(&root);
    let payload = serde_json::json!({
        "tool_name": "apply_patch",
        "tool_input": {
            "command": "*** Begin Patch\n*** Update File: .codex/config.toml\n@@\n old\n*** End Patch\n"
        }
    });

    let error = verify_mutation_payload_for_format(&root, HookFormat::Codex, &payload.to_string())
        .expect_err("runtime governance config writes require task-bound authorization");

    assert!(
        error.contains(".codex/config.toml is not bound to an active registry task target"),
        "{error}"
    );
}

#[test]
fn hook_allows_runtime_governance_write_with_active_target() {
    let root = temp_root("hook-allow-runtime-governance");
    seed_repo(&root);
    let plan = sample_plan("true").replace("src/lib.rs", ".codex/config.toml");
    fs::create_dir_all(root.join(".codex")).unwrap();
    fs::write(root.join("docs/plans/sample.md"), plan).unwrap();
    activate_plan(&root, "docs/plans/sample.md").unwrap();
    let payload = serde_json::json!({
        "tool_name": "apply_patch",
        "tool_input": {
            "command": "*** Begin Patch\n*** Update File: .codex/config.toml\n@@\n old\n*** End Patch\n"
        }
    });

    verify_mutation_payload_for_format(&root, HookFormat::Codex, &payload.to_string()).unwrap();
}

#[test]
fn verifier_typed_file_content_and_json_checks_pass() {
    let root = temp_root("verifier-pass");
    seed_repo(&root);
    fs::write(root.join("docs/plans/verifier.md"), verifier_plan("false")).unwrap();
    fs::write(root.join("README.md"), "License: MIT\n").unwrap();
    fs::write(root.join("report.json"), r#"{"ok":true}"#).unwrap();
    fs::write(
        root.join("report.schema.json"),
        r#"{"type":"object","required":["ok"],"properties":{"ok":{"const":true}}}"#,
    )
    .unwrap();

    activate_plan(&root, "docs/plans/verifier.md").unwrap();
    let count = verify_behaviors(&root, Some("TASK-2026-05-30-verifier-001")).unwrap();

    assert_eq!(count, 2);
}

#[test]
fn verifier_unknown_type_fails_manifest() {
    let root = temp_root("verifier-unknown");
    seed_repo(&root);
    fs::write(
        root.join("docs/plans/verifier.md"),
        verifier_plan("false").replace("type = \"file_exists\"", "type = \"unknown\""),
    )
    .unwrap();

    let error = activate_plan(&root, "docs/plans/verifier.md")
        .expect_err("unknown verifier type must fail activation");

    assert!(
        error.contains("unknown variant") || error.contains("unknown"),
        "{error}"
    );
}

#[test]
fn activation_rejects_legacy_v1_manifest() {
    let root = temp_root("manifest-v1-active");
    seed_repo(&root);
    fs::write(
        root.join("docs/plans/sample.md"),
        sample_plan("true").replace("schema_version = 2", "schema_version = 1"),
    )
    .unwrap();

    let error = activate_plan(&root, "docs/plans/sample.md")
        .expect_err("new activations must require manifest schema v2");

    assert!(error.contains("schema_version must be 2"), "{error}");
}

#[test]
fn activation_rejects_v2_manifest_without_typed_verifiers() {
    let root = temp_root("manifest-v2-no-verifiers");
    seed_repo(&root);
    let plan = sample_plan("true").replace(
        r#"
[[behaviors.verifiers]]
type = "command"
command = "true"
expected_exit = 0
"#,
        "",
    );
    fs::write(root.join("docs/plans/sample.md"), plan).unwrap();

    let error = activate_plan(&root, "docs/plans/sample.md")
        .expect_err("schema v2 behaviors must require typed verifiers");

    assert!(
        error.contains("requires typed [[behaviors.verifiers]] entries"),
        "{error}"
    );
}

#[test]
fn validation_rejects_completed_legacy_v1_manifest() {
    let root = temp_root("manifest-v1-completed");
    seed_repo(&root);
    let plan = sample_plan("true")
        .replace("schema_version = 2", "schema_version = 1")
        .replace(
            r#"
[[behaviors.verifiers]]
type = "command"
command = "true"
expected_exit = 0
"#,
            "",
        );
    fs::write(root.join("docs/plans/sample.md"), &plan).unwrap();
    let plan_hash = normalized_hash(&plan);
    fs::write(
        root.join(REGISTRY_PATH),
        format!(
            r#"
schema_version = 1
registry_id = "test-task-registry"
registry_authority = "docs/task-registry.toml"
activation_skill = "task-registry-flow"
hash_algorithm = "sha256(normalized plan text: CRLF/CR converted to LF, trailing whitespace stripped from each line, exactly one final newline)"
status_vocabulary = ["planned", "active", "blocked", "deferred", "completed", "cancelled"]
archive_paths = []

[[plans]]
plan_id = "PLAN-2026-05-30-sample"
plan_path = "docs/plans/sample.md"
plan_hash_sha256 = "{}"
activated_at = "2026-05-30"
status = "completed"

[[tasks]]
task_id = "TASK-2026-05-30-sample-001"
plan_id = "PLAN-2026-05-30-sample"
status = "completed"
title = "Sample task"
kind = "test"
source_plan_path = "docs/plans/sample.md"
source_plan_hash_sha256 = "{}"
reason = "Exercise task registry behavior"
acceptance_proof = "Behavior B-001-sample: true"
created_at = "2026-05-30"
updated_at = "2026-05-30"
behavior_ids = ["B-001-sample"]

[[tasks.targets]]
file = "src/lib.rs"
object = "sample_task"
required_change = "Update the sample task."
"#,
            plan_hash, plan_hash
        ),
    )
    .unwrap();

    let error = validate_all(&root).expect_err("completed v1 manifest must fail validation");
    assert!(error.contains("schema_version must be 2"), "{error}");
}

#[test]
fn validation_accepts_migrated_v2_historical_manifests() {
    let root = temp_root("manifest-v2-completed");
    seed_repo(&root);
    let plan = sample_plan("true");
    fs::write(root.join("docs/plans/sample.md"), &plan).unwrap();
    let manifest = parse_manifest_from_body("docs/plans/sample.md", &plan).unwrap();
    fs::write(
        root.join(REGISTRY_PATH),
        format!(
            r#"
schema_version = 1
registry_id = "test-task-registry"
registry_authority = "docs/task-registry.toml"
activation_skill = "task-registry-flow"
hash_algorithm = "sha256(normalized plan text: CRLF/CR converted to LF, trailing whitespace stripped from each line, exactly one final newline)"
status_vocabulary = ["planned", "active", "blocked", "deferred", "completed", "cancelled"]
archive_paths = []

[[plans]]
plan_id = "PLAN-2026-05-30-sample"
plan_path = "docs/plans/sample.md"
plan_hash_sha256 = "{}"
activated_at = "2026-05-30"
status = "completed"

[[tasks]]
task_id = "TASK-2026-05-30-sample-001"
plan_id = "PLAN-2026-05-30-sample"
status = "completed"
title = "Sample task"
kind = "test"
source_plan_path = "docs/plans/sample.md"
source_plan_hash_sha256 = "{}"
reason = "Exercise task registry behavior"
acceptance_proof = "Behavior B-001-sample: true"
created_at = "2026-05-30"
updated_at = "2026-05-30"
behavior_ids = ["B-001-sample"]

[[tasks.targets]]
file = "src/lib.rs"
object = "sample_task"
required_change = "Update the sample task."
"#,
            manifest.plan_hash_sha256, manifest.plan_hash_sha256
        ),
    )
    .unwrap();

    validate_all(&root).unwrap();
}

#[test]
fn verifier_failed_assertion_returns_structured_error() {
    let root = temp_root("verifier-fail");
    seed_repo(&root);
    fs::write(root.join("docs/plans/verifier.md"), verifier_plan("false")).unwrap();
    fs::write(root.join("README.md"), "all rights reserved\n").unwrap();
    fs::write(root.join("report.json"), r#"{"ok":true}"#).unwrap();
    fs::write(
        root.join("report.schema.json"),
        r#"{"type":"object","required":["ok"],"properties":{"ok":{"const":true}}}"#,
    )
    .unwrap();
    activate_plan(&root, "docs/plans/verifier.md").unwrap();

    let error = verify_behaviors(&root, Some("TASK-2026-05-30-verifier-001"))
        .expect_err("contains verifier must fail");

    assert!(error.contains("verifier failed"), "{error}");
    assert!(error.contains("README.md"), "{error}");
}

#[test]
fn diagnostics_reject_missing_failure_fields() {
    let diagnostic = Diagnostic {
        check_id: "release-file-present".to_string(),
        surface: ReportSurface::ReleaseSource,
        path: "".to_string(),
        severity: DiagnosticSeverity::Error,
        status: CheckStatus::Fail,
        expected: "file present".to_string(),
        actual: "missing".to_string(),
        remediation: "restore file".to_string(),
    };

    let error = CheckReport::new(ReportSurface::ReleaseSource, vec![diagnostic])
        .expect_err("empty diagnostic path must fail");

    assert!(error.contains("path"), "{error}");
}

#[test]
fn diagnostics_mixed_report_records_failures() {
    let report = CheckReport::new(
        ReportSurface::ReleaseSource,
        vec![
            Diagnostic::pass(
                "release-file-present",
                ReportSurface::ReleaseSource,
                "README.md",
                "file present",
            ),
            Diagnostic::fail(
                "release-file-present",
                ReportSurface::ReleaseSource,
                "MISSING.md",
                "file present",
                "missing",
                "restore file",
            ),
        ],
    )
    .unwrap();

    assert!(report.has_failures());
    assert_eq!(report.summary.pass, 1);
    assert_eq!(report.summary.fail, 1);
}

#[test]
fn release_schema_reports_required_and_version_failures() {
    let root = temp_root("release-schema");
    seed_release_repo(&root);
    fs::write(root.join("plugin.json"), r#"{"version":"2.0.1"}"#).unwrap();

    let report = release_checks::report(&root, release_checks::Mode::All).unwrap();

    assert!(report.has_failures());
    assert!(
        report
            .checks
            .iter()
            .any(|check| { check.path == "plugin.json" && check.actual.contains("2.0.1") })
    );
}

#[test]
fn release_schema_rejects_unknown_check_id() {
    let root = temp_root("release-schema-check-id");
    seed_release_repo(&root);
    let requirements = fs::read_to_string(root.join("REQUIREMENTS.toml"))
        .unwrap()
        .replace("\"release-file-present\"", "\"unknown-check\"");
    fs::write(root.join("REQUIREMENTS.toml"), requirements).unwrap();

    let report = release_checks::report(&root, release_checks::Mode::Required).unwrap();

    assert!(report.has_failures());
    assert!(
        report
            .checks
            .iter()
            .any(|check| check.path == "unknown-check")
    );
}

#[cfg(unix)]
#[test]
fn release_schema_reports_executable_failures() {
    let root = temp_root("release-schema-executable");
    seed_release_repo(&root);
    set_executable(&root.join("scripts/test-install-modes.sh"), false);

    let report = release_checks::report(&root, release_checks::Mode::Required).unwrap();

    assert!(report.has_failures());
    assert!(report.checks.iter().any(|check| {
        check.check_id == "release-file-executable"
            && check.path == "scripts/test-install-modes.sh"
            && check.status == CheckStatus::Fail
            && check.actual == "not executable"
    }));
}

#[test]
fn release_schema_rejects_undeclared_executable_scripts() {
    let root = temp_root("release-schema-undeclared-executable");
    seed_release_repo(&root);
    let requirements = fs::read_to_string(root.join("REQUIREMENTS.toml"))
        .unwrap()
        .replace(
            "executable = [\"scripts/test-install-modes.sh\"]",
            "executable = []",
        );
    fs::write(root.join("REQUIREMENTS.toml"), requirements).unwrap();

    let report = release_checks::report(&root, release_checks::Mode::Required).unwrap();

    assert!(report.has_failures());
    assert!(report.checks.iter().any(|check| {
        check.check_id == "release-script-executable-undeclared"
            && check.path == "scripts/test-install-modes.sh"
            && check.actual == "missing executable policy"
    }));
}

#[test]
fn release_schema_rejects_undeclared_rust_sources() {
    let root = temp_root("release-schema-undeclared-rust-source");
    seed_release_repo(&root);
    fs::create_dir_all(root.join("rust/task-registry-flow-cli/src/tests")).unwrap();
    fs::write(root.join("rust/task-registry-flow-cli/src/metrics.rs"), "").unwrap();
    fs::write(
        root.join("rust/task-registry-flow-cli/src/tests/metrics_tests.rs"),
        "",
    )
    .unwrap();

    let report = release_checks::report(&root, release_checks::Mode::Required).unwrap();

    assert!(report.has_failures());
    assert!(report.checks.iter().any(|check| {
        check.check_id == "release-rust-source-undeclared"
            && check.path == "rust/task-registry-flow-cli/src/metrics.rs"
            && check.actual == "missing release policy"
    }));
    assert!(report.checks.iter().any(|check| {
        check.check_id == "release-rust-source-undeclared"
            && check.path == "rust/task-registry-flow-cli/src/tests/metrics_tests.rs"
            && check.actual == "missing release policy"
    }));
}

#[test]
fn release_schema_reports_platform_executable_semantics() {
    let root = temp_root("release-schema-executable-platform");
    seed_release_repo(&root);

    let report = release_checks::report(&root, release_checks::Mode::Required).unwrap();

    assert!(report.checks.iter().any(|check| {
        check.check_id == "release-executable-platform"
            && check.path == "scripts/test-install-modes.sh"
            && check.actual.contains("executable")
    }));
}

#[test]
fn release_schema_rejects_unknown_requirement_fields() {
    let root = temp_root("release-schema-unknown-field");
    seed_release_repo(&root);
    let mut requirements = fs::read_to_string(root.join("REQUIREMENTS.toml")).unwrap();
    requirements.push_str("\nunknown_runtime = true\n");
    fs::write(root.join("REQUIREMENTS.toml"), requirements).unwrap();

    let error = release_checks::report(&root, release_checks::Mode::All)
        .expect_err("unknown release schema fields must fail");

    assert!(error.contains("unknown field"), "{error}");
}

#[test]
fn release_schema_rejects_unknown_version_file_format() {
    let root = temp_root("release-schema-version-format");
    seed_release_repo(&root);
    let requirements = fs::read_to_string(root.join("REQUIREMENTS.toml"))
        .unwrap()
        .replace("format = \"plain\"", "format = \"bogus\"");
    fs::write(root.join("REQUIREMENTS.toml"), requirements).unwrap();

    let error = release_checks::report(&root, release_checks::Mode::Version)
        .expect_err("unknown version file format must fail at schema parse");

    assert!(error.contains("unknown variant"), "{error}");
}

#[test]
fn hook_denies_malformed_uncertain_and_outside_payloads() {
    let root = temp_root("hook-deny-invalid");
    seed_repo(&root);

    let malformed = verify_mutation_payload_for_format(&root, HookFormat::Codex, "{not json")
        .expect_err("malformed JSON fails");
    assert!(malformed.contains("parse hook JSON"), "{malformed}");

    let uncertain = serde_json::json!({
        "toolCall": {
            "name": "edit_file",
            "args": {
                "CodeEdit": "change without target"
            }
        }
    });
    let uncertain_error =
        verify_mutation_payload_for_format(&root, HookFormat::Antigravity, &uncertain.to_string())
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
        verify_mutation_payload_for_format(&root, HookFormat::Antigravity, &outside.to_string())
            .expect_err("outside path must fail");
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
fn hook_format_typed_payloads_emit_responses() {
    let codex = serde_json::json!({
        "tool_name": "apply_patch",
        "tool_input": {
            "command": "*** Begin Patch\n*** Update File: src/main.rs\n@@\n old\n*** End Patch\n"
        }
    });
    let cursor = serde_json::json!({
        "toolCall": {
            "name": "edit_file",
            "args": {
                "TargetFile": "src/lib.rs",
                "CodeEdit": "change"
            }
        }
    });

    assert_eq!(HookFormat::Codex.as_str(), "codex");
    assert_eq!(HookFormat::Cursor.as_str(), "cursor");
    assert_eq!(inspect_hook_payload(&codex).paths, vec!["src/main.rs"]);
    assert_eq!(inspect_hook_payload(&cursor).paths, vec!["src/lib.rs"]);
}

#[test]
fn hook_rejects_format_mismatch_and_uncertain_paths() {
    let root = temp_root("hook-format-negative");
    seed_repo(&root);
    let uncertain = serde_json::json!({
        "toolCall": {
            "name": "edit_file",
            "args": {
                "CodeEdit": "change without target"
            }
        }
    });

    let error =
        verify_mutation_payload_for_format(&root, HookFormat::Antigravity, &uncertain.to_string())
            .expect_err("uncertain write must fail");

    assert!(error.contains("deterministic target path"), "{error}");
    let mismatch = serde_json::json!({"tool_name": "apply_patch"});
    let mismatch_error =
        verify_mutation_payload_for_format(&root, HookFormat::Antigravity, &mismatch.to_string())
            .expect_err("format mismatch must fail");
    assert!(
        mismatch_error.contains("requires toolCall"),
        "{mismatch_error}"
    );
    assert!(HookFormat::from_str("legacy").is_err());
}

#[test]
fn installer_plan_emits_typed_actions() {
    let report = crate::install::action_report(".agents/skills/task-registry-flow", "create")
        .expect("known installer action");
    assert_eq!(report.action, InstallAction::Create);
    assert_eq!(
        InstallAction::from_str("create").unwrap().as_str(),
        "create"
    );
    assert_eq!(
        InstallAction::from_str("replace-symlink").unwrap().as_str(),
        "replace-symlink"
    );
}

#[test]
fn installer_rejects_unknown_config_and_dry_run_mutation() {
    assert!(InstallAction::from_str("would-mutate").is_err());
    assert!(InstallAction::from_str("unknown-action").is_err());
    assert!(crate::install::assert_dry_run_unchanged("before", "before").is_ok());
    assert!(crate::install::assert_dry_run_unchanged("before", "after").is_err());
}

#[test]
fn cli_json_envelope_all_commands() {
    let report =
        crate::reports::success_json(CliCommand::Metrics, "metrics active=0 completed=1", false)
            .unwrap();
    let value = serde_json::from_str::<serde_json::Value>(&report).unwrap();

    assert_eq!(value["schema_version"], 2);
    assert_eq!(value["command"], "metrics");
    assert_eq!(value["status"], "pass");
    assert_eq!(
        CliCommand::from_str("metrics").unwrap(),
        CliCommand::Metrics
    );
    assert_eq!(
        CliCommand::from_str("release-check").unwrap(),
        CliCommand::ReleaseCheck
    );
    assert_eq!(
        CliCommand::from_str("source-limit").unwrap(),
        CliCommand::SourceLimit
    );
}

#[test]
fn cli_json_error_reports_actual_command() {
    let output =
        crate::cli::failure_json_for_test(CliCommand::Validate, false, "unexpected trailing args");
    let value = serde_json::from_str::<serde_json::Value>(&output).unwrap();

    assert_eq!(value["schema_version"], 2);
    assert_eq!(value["command"], "validate");
    assert_eq!(value["status"], "fail");
    assert_eq!(value["receipt_recorded"], false);
}

#[test]
fn cli_json_error_reports_receipt_state() {
    let output =
        crate::cli::failure_json_for_test(CliCommand::Validate, true, "unexpected trailing args");
    let value = serde_json::from_str::<serde_json::Value>(&output).unwrap();

    assert_eq!(value["command"], "validate");
    assert_eq!(value["receipt_recorded"], true);
    assert_ne!(value["command"], "usage");
}

#[test]
fn cli_rejects_unknown_format_and_trailing_args() {
    assert!(HookFormat::from_str("unknown").is_err());
    let root = temp_root("source-limit-extra-format");
    let args = vec![
        "check".to_string(),
        "--format".to_string(),
        "bogus".to_string(),
    ];

    let error = source_limit::run_command(root.as_path(), &args)
        .expect_err("unknown source-limit format must fail");
    let error = error.to_string();

    assert!(
        error.contains("usage: task-registry-flow source-limit"),
        "{error}"
    );
}

#[test]
fn module_split_preserves_registry_behaviors() {
    let root = temp_root("module-split");
    seed_repo(&root);
    fs::write(root.join("docs/plans/sample.md"), sample_plan("true")).unwrap();

    activate_plan(&root, "docs/plans/sample.md").unwrap();
    crate::landing::run_command(&root, &changed_files_args(&["src/lib.rs"])).unwrap();

    assert_eq!(
        report_plan(&root, "PLAN-2026-05-30-sample")
            .unwrap()
            .completed,
        1
    );
}

#[test]
fn policy_loads_contract_sections_and_action_vocabulary() {
    let policy = crate::policy::parse_manifest_policy(include_str!("../../../../MANIFEST.toml"))
        .expect("manifest policy should be typed");

    assert!(
        policy
            .install_policy
            .action_vocabulary
            .iter()
            .any(|value| value == "replace-symlink")
    );
    assert!(
        crate::install::validate_action_vocabulary(&policy.install_policy.action_vocabulary)
            .is_ok()
    );
}

#[test]
fn policy_rejects_unknown_fields_and_action_mismatch() {
    assert!(InstallAction::from_str("unknown-action").is_err());
    assert!(
        crate::policy::parse_manifest_policy(
            r#"[install_policy]
action_vocabulary = ["create", "unknown-action"]
dry_run_prefix = "would-"
stale_absent = []
"#
        )
        .is_err()
    );
    assert!(
        crate::policy::parse_manifest_policy(
            r#"[install_policy]
action_vocabulary = ["create"]
dry_run_prefix = "would-"
stale_absent = []
unknown = true
"#
        )
        .is_err()
    );
}

#[test]
fn negative_tests_parse_json_contracts() {
    let report = CheckReport::new(
        ReportSurface::Migration,
        vec![Diagnostic::fail(
            "task-manifest-schema-version",
            ReportSurface::Manifest,
            "docs/plans/legacy.md",
            "schema_version 2",
            "schema_version 1",
            "migrate the Task Manifest",
        )],
    )
    .unwrap();
    let value = serde_json::from_str::<serde_json::Value>(&report.to_json().unwrap()).unwrap();

    assert_eq!(
        value["checks"][0]["check_id"],
        "task-manifest-schema-version"
    );
    assert_eq!(value["checks"][0]["status"], "fail");
}

#[test]
fn source_limit_json_success_exits_zero() {
    let root = temp_root("source-limit-json-success");
    let args = vec![
        "check".to_string(),
        "--format".to_string(),
        "json".to_string(),
    ];

    let output = source_limit::run_command(root.as_path(), &args).unwrap();
    let value = serde_json::from_str::<serde_json::Value>(&output).unwrap();

    assert_eq!(value["surface"], "source-limit");
    assert_eq!(value["summary"]["fail"], 0);
}

#[test]
fn source_limit_json_failure_preserves_diagnostics() {
    let root = temp_root("source-limit-json-failure");
    let long_file = root.join("src/large.rs");
    fs::create_dir_all(long_file.parent().unwrap()).unwrap();
    fs::write(&long_file, "fn item() {}\n".repeat(SOURCE_LINE_LIMIT + 1)).unwrap();
    let args = vec![
        "check".to_string(),
        "--format".to_string(),
        "json".to_string(),
    ];

    let error = source_limit::run_command(root.as_path(), &args)
        .expect_err("over-limit JSON check must fail");

    let crate::reports::RuntimeFailure::Json {
        payload: output, ..
    } = error
    else {
        panic!("source-limit JSON failure must preserve raw JSON");
    };
    let value = serde_json::from_str::<serde_json::Value>(&output).unwrap();
    assert_eq!(value["surface"], "source-limit");
    assert_eq!(value["summary"]["fail"], 1);
    assert_eq!(value["checks"][0]["check_id"], "source-limit");
    assert_eq!(value["checks"][0]["path"], "src/large.rs");
    assert_eq!(value["checks"][0]["status"], "fail");
}

#[test]
fn release_check_json_success_exits_zero() {
    let root = temp_root("release-json-success");
    seed_release_repo(&root);
    let args = vec![
        "all".to_string(),
        "--format".to_string(),
        "json".to_string(),
    ];

    let output = release_checks::run_command(root.as_path(), &args).unwrap();
    let value = serde_json::from_str::<serde_json::Value>(&output).unwrap();

    assert_eq!(value["surface"], "release-source");
    assert_eq!(value["summary"]["fail"], 0);
}

#[test]
fn release_check_json_failure_preserves_report() {
    let root = temp_root("release-json-failure");
    seed_release_repo(&root);
    fs::remove_file(root.join("VERSION")).unwrap();
    let args = vec![
        "all".to_string(),
        "--format".to_string(),
        "json".to_string(),
    ];

    let error = release_checks::run_command(root.as_path(), &args)
        .expect_err("failing release JSON check must fail");

    let crate::reports::RuntimeFailure::Json {
        payload: output, ..
    } = error
    else {
        panic!("release-check JSON failure must preserve raw JSON");
    };
    let value = serde_json::from_str::<serde_json::Value>(&output).unwrap();
    assert_eq!(value["surface"], "release-source");
    assert!(value["summary"]["fail"].as_u64().unwrap() >= 1);
    assert!(value["checks"].as_array().unwrap().iter().any(|check| {
        check["check_id"] == "release-file-present"
            && check["path"] == "VERSION"
            && check["status"] == "fail"
    }));
    assert!(!output.starts_with("task-registry-flow error:"));
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

    let json_plan = source_limit::plan(root.as_path(), Some("src/large.rs"), true).unwrap();
    let value = serde_json::from_str::<serde_json::Value>(&json_plan).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["surface"], "source-limit-plan");
    assert_eq!(value["plans"][0]["path"], "src/large.rs");
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
    let error = error.to_string();

    assert!(
        error.contains("usage: task-registry-flow source-limit"),
        "{error}"
    );
}

fn assert_activation_error(plan: String, expected: &str) {
    let root = temp_root("activation-contract-rejects");
    seed_repo(&root);
    fs::write(root.join("docs/plans/sample.md"), plan).unwrap();

    let error = activate_plan(&root, "docs/plans/sample.md").expect_err("activation must fail");
    assert!(error.contains(expected), "{error}");

    let registry = load_registry(&root).unwrap();
    assert!(registry.plans.is_empty());
    assert!(registry.tasks.is_empty());
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

fn changed_files_args(paths: &[&str]) -> Vec<String> {
    let mut args = vec!["--changed-files".to_string()];
    args.extend(paths.iter().map(|path| path.to_string()));
    args
}

fn sample_plan(command: &str) -> String {
    format!(
        r#"# Sample Plan

## Approved Scope

Exercise sample task registry behavior.

## Phased Required Change Checklist

- [ ] `[MODIFY]` `src/lib.rs` - update the sample task; acceptance proof: sample behaviors pass.

## Per-Gap Success Criteria

### GAP-SAMPLE

- Behavioral: Given a seeded registry when the sample task completes then linked behavior verifiers pass.
- Data/schema/provenance: Registry activation records the sample task hash.
- Runtime: N/A; unit tests prove behavior.

## Validation Plan

Focused:
- {command}

Full:
- {command}

## Walkthrough Evidence

- Registry report for the sample plan.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-05-30-sample"

[[behaviors]]
behavior_id = "B-001-sample"
gap_id = "GAP-SAMPLE"
polarity = "positive"
title = "Sample behavior"
given = "A seeded registry"
when = "The task completes"
then = "The confirmation passes"
confirmation = "{command}"

[[behaviors.verifiers]]
type = "command"
command = "{command}"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-sample-negative"
gap_id = "GAP-SAMPLE"
polarity = "negative"
title = "Sample forbidden behavior remains absent"
given = "A seeded registry"
when = "The task completes"
then = "No forbidden fixture is created"
confirmation = "test ! -e forbidden.txt"

[[behaviors.verifiers]]
type = "file_absent"
path = "forbidden.txt"

[[tasks]]
task_id = "TASK-2026-05-30-sample-001"
title = "Sample task"
status = "planned"
kind = "test"
reason = "Exercise task registry behavior"
acceptance_proof = "Behavior B-001-sample: {command}"
behavior_ids = ["B-001-sample", "B-002-sample-negative"]
[[tasks.targets]]
file = "src/lib.rs"
object = "sample_task"
required_change = "Update the sample task."
```
"#
    )
}

fn verifier_plan(command: &str) -> String {
    format!(
        r#"# Verifier Plan

## Approved Scope

Exercise typed verifier behavior.

## Phased Required Change Checklist

- [ ] `[MODIFY]` `README.md` - update verifier fixture; acceptance proof: typed verifier behaviors pass.

## Per-Gap Success Criteria

### GAP-VERIFIER

- Behavioral: Given verifier fixtures when behavior verification runs then typed file, content, and JSON assertions pass.
- Data/schema/provenance: Manifest behavior rows include positive and negative verifier metadata.
- Runtime: N/A; unit tests prove behavior.

## Validation Plan

Focused:
- {command}

Full:
- {command}

## Walkthrough Evidence

- Behavior verifier output for the verifier plan.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-05-30-verifier"

[[behaviors]]
behavior_id = "B-001-verifier"
gap_id = "GAP-VERIFIER"
polarity = "positive"
title = "Verifier behavior"
given = "A seeded registry and verifier files"
when = "The task verifies behavior"
then = "Typed verifiers prove file, content, absence, and JSON behavior"
confirmation = "{command}"

[[behaviors.verifiers]]
type = "file_exists"
path = "README.md"

[[behaviors.verifiers]]
type = "contains"
path = "README.md"
needle = "License: MIT"

[[behaviors.verifiers]]
type = "not_contains"
path = "README.md"
needle = "all rights reserved"

[[behaviors.verifiers]]
type = "json_schema"
path = "report.json"
schema_path = "report.schema.json"

[[behaviors]]
behavior_id = "B-002-verifier-negative"
gap_id = "GAP-VERIFIER"
polarity = "negative"
title = "Verifier forbidden content remains absent"
given = "A seeded registry and verifier files"
when = "The task verifies behavior"
then = "Forbidden content is absent"
confirmation = "test ! -e forbidden.txt"

[[behaviors.verifiers]]
type = "file_absent"
path = "forbidden.txt"

[[tasks]]
task_id = "TASK-2026-05-30-verifier-001"
title = "Verifier task"
status = "planned"
kind = "validation"
reason = "Exercise typed behavior verifiers"
acceptance_proof = "Behavior B-001-verifier passes."
behavior_ids = ["B-001-verifier", "B-002-verifier-negative"]
[[tasks.targets]]
file = "README.md"
object = "verifier_doc"
required_change = "Update verifier fixture."
```
"#
    )
}

fn seed_release_repo(root: &Path) {
    fs::write(root.join("VERSION"), "2.0.0\n").unwrap();
    fs::write(root.join("README.md"), "Current release: `2.0.0`\n").unwrap();
    fs::create_dir_all(root.join("docs/releases")).unwrap();
    fs::write(
        root.join("docs/releases/v2.md"),
        "# V2 Release Checklist\n\nRelease version: `2.0.0`\n",
    )
    .unwrap();
    fs::write(root.join("plugin.json"), r#"{"version":"2.0.0"}"#).unwrap();
    fs::write(
        root.join("MANIFEST.toml"),
        r#"schema_version = 1
plugin_version = "2.0.0"
"#,
    )
    .unwrap();
    fs::create_dir_all(root.join("rust/task-registry-flow-cli")).unwrap();
    fs::write(
        root.join("rust/task-registry-flow-cli/Cargo.toml"),
        r#"[package]
version = "2.0.0"
"#,
    )
    .unwrap();
    fs::create_dir_all(root.join("scripts")).unwrap();
    fs::write(
        root.join("scripts/test-install-modes.sh"),
        "#!/usr/bin/env bash\n",
    )
    .unwrap();
    set_executable(&root.join("scripts/test-install-modes.sh"), true);
    fs::write(
        root.join("REQUIREMENTS.toml"),
        r#"
schema_version = 1
plugin_name = "agent-governance"

[tracked_for_ci]
required = ["README.md"]

[release_source]
version = "2.0.0"
required = ["VERSION", "README.md", "docs/releases/v2.md", "plugin.json", "MANIFEST.toml", "rust/task-registry-flow-cli/Cargo.toml", "scripts/test-install-modes.sh"]
executable = ["scripts/test-install-modes.sh"]
stale_absent = ["hooks.json"]
check_ids = ["release-file-present", "release-file-executable", "release-script-executable-undeclared", "release-executable-platform", "release-rust-source-undeclared", "stale-path-absent", "release-version-consistent"]

[[release_source.version_files]]
path = "VERSION"
format = "plain"

[[release_source.version_files]]
path = "plugin.json"
format = "json"
key = "version"

[[release_source.version_files]]
path = "README.md"
format = "markdown-line"
key = "Current release:"

[[release_source.version_files]]
path = "docs/releases/v2.md"
format = "markdown-line"
key = "Release version:"

[[release_source.version_files]]
path = "MANIFEST.toml"
format = "toml"
key = "plugin_version"

[[release_source.version_files]]
path = "rust/task-registry-flow-cli/Cargo.toml"
format = "toml"
key = "package.version"
"#,
    )
    .unwrap();
}

#[cfg(unix)]
fn set_executable(path: &Path, executable: bool) {
    let metadata = fs::metadata(path).unwrap();
    let mut permissions = metadata.permissions();
    let mut mode = permissions.mode();
    if executable {
        mode |= 0o755;
    } else {
        mode &= !0o111;
    }
    permissions.set_mode(mode);
    fs::set_permissions(path, permissions).unwrap();
}

#[cfg(not(unix))]
fn set_executable(_path: &Path, _executable: bool) {}

fn temp_root(label: &str) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let pid = std::process::id() as u64;
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = env::temp_dir().join(format!("task-registry-flow-{label}-{pid}-{seq}"));
    fs::create_dir_all(&root).unwrap();
    root
}
