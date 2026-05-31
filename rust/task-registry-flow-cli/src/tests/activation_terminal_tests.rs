use super::*;

#[test]
fn activation_terminal_allows_idempotent_reactivation() {
    let root = temp_root("activation-terminal-completed-idempotent");
    seed_repo(&root);
    fs::write(root.join("docs/plans/sample.md"), sample_plan("true")).unwrap();
    activate_plan(&root, "docs/plans/sample.md").unwrap();
    update_task_status(&root, "TASK-2026-05-30-sample-001", "completed").unwrap();
    let before = load_registry(&root).unwrap().tasks.remove(0);

    activate_plan(&root, "docs/plans/sample.md").unwrap();
    let after = load_registry(&root).unwrap().tasks.remove(0);

    assert_eq!(after.status, TaskStatus::Completed);
    assert_eq!(after.updated_at, before.updated_at);
    assert_eq!(
        after.source_plan_hash_sha256,
        before.source_plan_hash_sha256
    );
}

#[test]
fn activation_terminal_rejects_completed_task_manifest_rewrite() {
    let root = temp_root("activation-terminal-completed-rewrite");
    seed_repo(&root);
    fs::write(root.join("docs/plans/sample.md"), sample_plan("true")).unwrap();
    activate_plan(&root, "docs/plans/sample.md").unwrap();
    update_task_status(&root, "TASK-2026-05-30-sample-001", "completed").unwrap();
    let rewritten = sample_plan("true").replace(
        "acceptance_proof = \"Behavior B-001-sample: true\"",
        "acceptance_proof = \"Behavior B-001-sample rewritten: true\"",
    );
    fs::write(root.join("docs/plans/sample.md"), rewritten).unwrap();

    let error = activate_plan(&root, "docs/plans/sample.md")
        .expect_err("completed task provenance must be immutable");

    assert!(
        error.contains("terminal and cannot be rewritten"),
        "{error}"
    );
}

#[test]
fn activation_terminal_rejects_cancelled_task_manifest_rewrite() {
    let root = temp_root("activation-terminal-cancelled-rewrite");
    seed_repo(&root);
    fs::write(root.join("docs/plans/sample.md"), sample_plan("true")).unwrap();
    activate_plan(&root, "docs/plans/sample.md").unwrap();
    update_task_status(&root, "TASK-2026-05-30-sample-001", "cancelled").unwrap();
    let rewritten = sample_plan("true").replace(
        "title = \"Sample task\"",
        "title = \"Sample task rewritten\"",
    );
    fs::write(root.join("docs/plans/sample.md"), rewritten).unwrap();

    let error = activate_plan(&root, "docs/plans/sample.md")
        .expect_err("cancelled task provenance must be immutable");

    assert!(
        error.contains("terminal and cannot be rewritten"),
        "{error}"
    );
}

#[test]
fn activation_terminal_rejects_cancelled_task_stale_source_hash() {
    let root = temp_root("activation-terminal-cancelled-stale-hash");
    seed_repo(&root);
    fs::write(root.join("docs/plans/sample.md"), sample_plan("true")).unwrap();
    activate_plan(&root, "docs/plans/sample.md").unwrap();
    update_task_status(&root, "TASK-2026-05-30-sample-001", "cancelled").unwrap();
    let mut registry = load_registry(&root).unwrap();
    registry.tasks[0].source_plan_hash_sha256 = "0".repeat(64);
    crate::registry_io::save_registry(&root, &registry).unwrap();

    let error = validate_all(&root).expect_err("cancelled task stale source hash must fail");

    assert!(
        error.contains("source_plan_hash_sha256 mismatch"),
        "{error}"
    );
}
