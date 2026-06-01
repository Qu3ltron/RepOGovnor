use super::*;

#[test]
fn reviewer_report_summarizes_handoff_and_proof_boundaries() {
    let root = temp_root("reviewer-report");
    seed_repo(&root);
    fs::create_dir_all(root.join("src")).unwrap();

    fs::write(
        root.join("docs/plans/landed.md"),
        reviewer_plan(
            "PLAN-2026-06-01-reviewer-landed",
            "TASK-2026-06-01-reviewer-landed-001",
            "B-001-reviewer-landed",
            "B-002-reviewer-landed-negative",
            "src/lib.rs",
        ),
    )
    .unwrap();
    activate_plan(&root, "docs/plans/landed.md").unwrap();
    crate::landing::run_command(&root, &changed_files_args(&["src/lib.rs"])).unwrap();

    fs::write(
        root.join("docs/plans/blocked.md"),
        reviewer_plan(
            "PLAN-2026-06-01-reviewer-blocked",
            "TASK-2026-06-01-reviewer-blocked-001",
            "B-001-reviewer-blocked",
            "B-002-reviewer-blocked-negative",
            "src/blocked.rs",
        ),
    )
    .unwrap();
    activate_plan(&root, "docs/plans/blocked.md").unwrap();
    update_task_status(&root, "TASK-2026-06-01-reviewer-blocked-001", "blocked").unwrap();

    fs::write(
        root.join("docs/plans/deferred.md"),
        reviewer_plan(
            "PLAN-2026-06-01-reviewer-deferred",
            "TASK-2026-06-01-reviewer-deferred-001",
            "B-001-reviewer-deferred",
            "B-002-reviewer-deferred-negative",
            "src/deferred.rs",
        ),
    )
    .unwrap();
    activate_plan(&root, "docs/plans/deferred.md").unwrap();
    defer_task(
        &root,
        "TASK-2026-06-01-reviewer-deferred-001",
        "reviewer handoff fixture",
        "reactivate when reviewer report fixture changes",
    )
    .unwrap();

    let report = crate::reviewer_report::render(&root).unwrap();

    assert!(report.contains("Reviewer report"));
    assert!(report.contains("Active plans: 2"), "{report}");
    assert!(report.contains("Landed tasks: 1"), "{report}");
    assert!(report.contains("Changed targets: 1"), "{report}");
    assert!(report.contains("changed src/lib.rs"), "{report}");
    assert!(report.contains("Blocked/deferred: 2"), "{report}");
    assert!(
        report.contains("TASK-2026-06-01-reviewer-blocked-001 [blocked]"),
        "{report}"
    );
    assert!(
        report.contains("TASK-2026-06-01-reviewer-deferred-001 [deferred]"),
        "{report}"
    );
    assert!(
        report.contains("Receipts: Task registry metrics"),
        "{report}"
    );
    assert!(
        report.contains("governance proof is not product correctness proof"),
        "{report}"
    );
}

#[test]
fn reviewer_report_rejects_unexpected_args() {
    let root = temp_root("reviewer-report-args");
    seed_repo(&root);

    let error = crate::reviewer_report::run(&root, &["--bogus".to_string()])
        .expect_err("unexpected reviewer-report args fail");

    assert!(error.contains("usage: task-registry-flow reviewer-report [--format text|markdown]"));
}

#[test]
fn reviewer_report_markdown_formats_pr_handoff() {
    let root = temp_root("reviewer-report-markdown");
    seed_repo(&root);
    fs::create_dir_all(root.join("src")).unwrap();

    fs::write(
        root.join("docs/plans/landed.md"),
        reviewer_plan(
            "PLAN-2026-06-01-reviewer-markdown",
            "TASK-2026-06-01-reviewer-markdown-001",
            "B-001-reviewer-markdown",
            "B-002-reviewer-markdown-negative",
            "src/lib.rs",
        ),
    )
    .unwrap();
    activate_plan(&root, "docs/plans/landed.md").unwrap();
    crate::landing::run_command(&root, &changed_files_args(&["src/lib.rs"])).unwrap();

    let report =
        crate::reviewer_report::run(&root, &["--format".to_string(), "markdown".to_string()])
            .unwrap();

    assert!(report.contains("# Reviewer Report"), "{report}");
    assert!(report.contains("## Summary"), "{report}");
    assert!(report.contains("- Active plans: 0"), "{report}");
    assert!(report.contains("- Landed tasks: 1"), "{report}");
    assert!(report.contains("## Proof Boundary"), "{report}");
    assert!(
        report.contains("governance proof is not product correctness proof"),
        "{report}"
    );
    assert!(report.contains("## Landed Changed Files"), "{report}");
    assert!(report.contains("changed `src/lib.rs`"), "{report}");
}

#[test]
fn reviewer_report_rejects_unknown_format() {
    let root = temp_root("reviewer-report-unknown-format");
    seed_repo(&root);

    let error = crate::reviewer_report::run(&root, &["--format".to_string(), "json".to_string()])
        .expect_err("unknown reviewer-report format fails");

    assert!(error.contains("usage: task-registry-flow reviewer-report [--format text|markdown]"));
}

fn reviewer_plan(
    plan_id: &str,
    task_id: &str,
    positive_behavior: &str,
    negative_behavior: &str,
    target_file: &str,
) -> String {
    sample_plan("true")
        .replace("PLAN-2026-05-30-sample", plan_id)
        .replace("TASK-2026-05-30-sample-001", task_id)
        .replace("B-001-sample", positive_behavior)
        .replace("B-002-sample-negative", negative_behavior)
        .replace("src/lib.rs", target_file)
}
