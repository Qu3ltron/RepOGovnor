use super::*;

#[test]
fn archive_completed_keeps_archives_under_source_limit() {
    let root = temp_root("archive-line-budget");
    seed_repo(&root);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn sample() {}\n").unwrap();

    for index in 0..14 {
        let plan = sample_plan("true")
            .replace(
                "PLAN-2026-05-30-sample",
                &format!("PLAN-2026-06-01-archive-budget-{index:02}"),
            )
            .replace("B-001-sample", &format!("B-001-archive-budget-{index:02}"))
            .replace(
                "B-002-sample-negative",
                &format!("B-002-archive-budget-negative-{index:02}"),
            )
            .replace(
                "TASK-2026-05-30-sample-001",
                &format!("TASK-2026-06-01-archive-budget-{index:02}"),
            );
        let plan_path = format!("docs/plans/archive-budget-{index:02}.md");
        fs::write(root.join(&plan_path), plan).unwrap();
        activate_plan(&root, &plan_path).unwrap();
        crate::landing::run_command(&root, &changed_files_args(&["src/lib.rs"])).unwrap();
    }

    archive_completed(&root).unwrap();

    let registry_body = fs::read_to_string(root.join(REGISTRY_PATH)).unwrap();
    let registry: TaskRegistry = toml::from_str(&registry_body).unwrap();
    assert!(
        registry.archive_paths.len() > 2,
        "fixture must generate multiple archive chunks"
    );

    for archive_path in registry.archive_paths {
        let body = fs::read_to_string(root.join(&archive_path)).unwrap();
        let lines = body.lines().count();
        assert!(
            lines <= SOURCE_LINE_LIMIT,
            "{archive_path} has {lines} lines; limit is {SOURCE_LINE_LIMIT}"
        );
    }
}
