use super::*;

#[test]
fn landing_completes_changed_file_tasks() {
    let root = temp_root("landing-completes");
    seed_repo(&root);
    fs::write(root.join("docs/plans/sample.md"), sample_plan("true")).unwrap();
    activate_plan(&root, "docs/plans/sample.md").unwrap();
    update_task_status(&root, "TASK-2026-05-30-sample-001", "active").unwrap();

    let output = crate::landing::run_command(
        &root,
        &[
            "--plan-id".to_string(),
            "PLAN-2026-05-30-sample".to_string(),
            "--changed-files".to_string(),
            "src/lib.rs".to_string(),
            REGISTRY_PATH.to_string(),
        ],
    )
    .unwrap();
    let registry = load_registry(&root).unwrap();
    let task = registry
        .tasks
        .iter()
        .find(|task| task.task_id == "TASK-2026-05-30-sample-001")
        .unwrap();

    assert!(output.contains("TASK_VERIFY_LANDING ok"), "{output}");
    assert_eq!(task.status, TaskStatus::Completed);
    assert_eq!(
        task.completion_verified_by.as_deref(),
        Some("verify-landing")
    );
    assert!(
        task.completion_changed_files
            .iter()
            .any(|path| path == "src/lib.rs")
    );
    assert_eq!(
        report_plan(&root, "PLAN-2026-05-30-sample")
            .unwrap()
            .completed,
        1
    );
}

#[test]
fn landing_rejects_direct_completed_status() {
    let root = temp_root("landing-direct-status");
    seed_repo(&root);
    fs::write(root.join("docs/plans/sample.md"), sample_plan("true")).unwrap();
    activate_plan(&root, "docs/plans/sample.md").unwrap();
    update_task_status(&root, "TASK-2026-05-30-sample-001", "active").unwrap();

    let error = update_task_status(&root, "TASK-2026-05-30-sample-001", "completed")
        .expect_err("direct completed status must fail");
    let task = load_registry(&root).unwrap().tasks.remove(0);

    assert!(error.contains("verify-landing-owned"), "{error}");
    assert_eq!(task.status, TaskStatus::Active);
}

#[test]
fn landing_rejects_unbound_or_registry_only_changes() {
    let root = temp_root("landing-rejects-unbound");
    seed_repo(&root);
    fs::write(root.join("docs/plans/sample.md"), sample_plan("true")).unwrap();
    activate_plan(&root, "docs/plans/sample.md").unwrap();
    update_task_status(&root, "TASK-2026-05-30-sample-001", "active").unwrap();

    let unbound = crate::landing::run_command(&root, &changed_files_args(&["src/other.rs"]))
        .expect_err("unbound changed files must fail");
    let registry_only = crate::landing::run_command(&root, &changed_files_args(&[REGISTRY_PATH]))
        .expect_err("registry-only changed files must fail");
    let task = load_registry(&root).unwrap().tasks.remove(0);

    assert!(
        unbound.contains("not bound to an active task target"),
        "{unbound}"
    );
    assert!(
        registry_only.contains("requires at least one non-registry changed file"),
        "{registry_only}"
    );
    assert_eq!(task.status, TaskStatus::Active);
}

#[test]
fn landing_rejects_planned_targets_before_verifiers() {
    let root = temp_root("landing-rejects-planned");
    seed_repo(&root);
    fs::write(root.join("docs/plans/sample.md"), sample_plan("false")).unwrap();
    activate_plan(&root, "docs/plans/sample.md").unwrap();
    defer_task(
        &root,
        "TASK-2026-05-30-sample-001",
        "review fixture",
        "reactivate as planned fixture",
    )
    .unwrap();
    update_task_status(&root, "TASK-2026-05-30-sample-001", "planned").unwrap();

    let error = crate::landing::run_command(&root, &changed_files_args(&["src/lib.rs"]))
        .expect_err("planned targets must fail before verifiers run");
    let task = load_registry(&root).unwrap().tasks.remove(0);

    assert!(
        error.contains("not bound to an active task target"),
        "{error}"
    );
    assert!(!error.contains("confirmation failed"), "{error}");
    assert_eq!(task.status, TaskStatus::Planned);
    assert!(task.completion_verified_by.is_none());
    assert!(task.completion_verified_at.is_none());
    assert!(task.completion_changed_files.is_empty());
}
