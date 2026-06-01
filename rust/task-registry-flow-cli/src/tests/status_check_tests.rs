use super::*;
use crate::schema::ReportSurface;

#[test]
fn status_check_reports_marker_skill_hook_ci_facts() {
    let root = temp_root("status-marker-facts");
    write_marker_docs(&root);
    let report = crate::status_checks::report(
        ReportSurface::Status,
        vec![
            crate::status_checks::marker_check(&root, "AGENTS.md"),
            crate::status_checks::native_skill_check(".agents/skills/task-registry-flow", true),
        ],
    )
    .unwrap();

    assert_eq!(report.summary.pass, 2);
    assert!(!report.has_failures());
}

#[test]
fn status_check_json_success_exits_zero() {
    let root = temp_root("status-json-success");
    create_native_agent_skills(&root);
    write_marker_docs(&root);
    let args = vec!["--format".to_string(), "json".to_string()];

    let output = crate::status_checks::run_command(&root, &args).unwrap();
    let value = serde_json::from_str::<serde_json::Value>(&output).unwrap();

    assert_eq!(value["surface"], "status");
    assert_eq!(value["summary"]["fail"], 0);
    assert!(value["checks"].as_array().unwrap().iter().any(|check| {
        check["check_id"] == "governance-marker"
            && check["path"] == "AGENTS.md"
            && check["status"] == "pass"
    }));
    for path in [
        ".agents/skills/gap-closure-contract",
        ".agents/skills/task-registry-flow",
    ] {
        assert!(value["checks"].as_array().unwrap().iter().any(|check| {
            check["check_id"] == "native-skill"
                && check["path"] == path
                && check["status"] == "pass"
        }));
    }
}

#[test]
fn status_check_json_failure_exits_nonzero() {
    let root = temp_root("status-json-failure");
    write_marker_docs(&root);
    let args = vec!["--format".to_string(), "json".to_string()];

    let error = crate::status_checks::run_command(&root, &args)
        .expect_err("missing native skill must fail JSON status");

    let crate::reports::RuntimeFailure::Json {
        payload: output, ..
    } = error
    else {
        panic!("status JSON failure must preserve raw JSON");
    };
    let value = serde_json::from_str::<serde_json::Value>(&output).unwrap();
    assert_eq!(value["surface"], "status");
    assert_eq!(value["summary"]["fail"], 2);
    assert!(
        value["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|check| { check["check_id"] == "native-skill" && check["status"] == "fail" })
    );
}

#[cfg(unix)]
#[test]
fn status_check_json_symlink_failure_exits_nonzero() {
    let root = temp_root("status-json-symlink-failure");
    write_marker_docs(&root);
    let target = root.join(".cursor/skills/task-registry-flow");
    fs::create_dir_all(&target).unwrap();
    fs::create_dir_all(root.join(".agents/skills/gap-closure-contract")).unwrap();
    let link = root.join(".agents/skills/task-registry-flow");
    fs::create_dir_all(link.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let args = vec!["--format".to_string(), "json".to_string()];

    let error = crate::status_checks::run_command(&root, &args)
        .expect_err("legacy skill symlink must fail JSON status");

    let crate::reports::RuntimeFailure::Json {
        payload: output, ..
    } = error
    else {
        panic!("status symlink JSON failure must preserve raw JSON");
    };
    let value = serde_json::from_str::<serde_json::Value>(&output).unwrap();
    assert_eq!(value["surface"], "status");
    assert_eq!(value["summary"]["fail"], 1);
    assert!(value["checks"].as_array().unwrap().iter().any(|check| {
        check["check_id"] == "native-skill"
            && check["path"] == ".agents/skills/task-registry-flow"
            && check["status"] == "fail"
    }));
}

#[test]
fn status_check_json_missing_marker_failure_exits_nonzero() {
    let root = temp_root("status-json-missing-marker-failure");
    create_native_agent_skills(&root);
    fs::write(root.join("AGENTS.md"), "custom agents\n").unwrap();
    fs::write(
        root.join("GEMINI.md"),
        "<!-- agent-governance:begin -->\ncustom gemini\n",
    )
    .unwrap();
    let args = vec!["--format".to_string(), "json".to_string()];

    let error = crate::status_checks::run_command(&root, &args)
        .expect_err("missing or malformed governance markers must fail JSON status");

    let crate::reports::RuntimeFailure::Json {
        payload: output, ..
    } = error
    else {
        panic!("status marker JSON failure must preserve raw JSON");
    };
    let value = serde_json::from_str::<serde_json::Value>(&output).unwrap();
    assert_eq!(value["surface"], "status");
    assert_eq!(value["summary"]["fail"], 2);
    assert!(value["checks"].as_array().unwrap().iter().any(|check| {
        check["check_id"] == "governance-marker"
            && check["path"] == "AGENTS.md"
            && check["actual"] == "missing marker block"
            && check["status"] == "fail"
    }));
    assert!(value["checks"].as_array().unwrap().iter().any(|check| {
        check["check_id"] == "governance-marker"
            && check["path"] == "GEMINI.md"
            && check["actual"] == "begin=1 end=0"
            && check["status"] == "fail"
    }));
}

#[test]
fn status_check_json_rejects_non_block_marker_tokens() {
    let root = temp_root("status-json-non-block-marker-failure");
    create_native_agent_skills(&root);
    fs::write(
        root.join("AGENTS.md"),
        "mentions agent-governance:begin and agent-governance:end in prose\n",
    )
    .unwrap();
    fs::write(
        root.join("GEMINI.md"),
        "<!-- agent-governance:end -->\nmanaged\n<!-- agent-governance:begin -->\n",
    )
    .unwrap();
    let args = vec!["--format".to_string(), "json".to_string()];

    let error = crate::status_checks::run_command(&root, &args)
        .expect_err("prose-only and reversed markers must fail JSON status");

    let crate::reports::RuntimeFailure::Json {
        payload: output, ..
    } = error
    else {
        panic!("status marker JSON failure must preserve raw JSON");
    };
    let value = serde_json::from_str::<serde_json::Value>(&output).unwrap();
    assert_eq!(value["surface"], "status");
    assert_eq!(value["summary"]["fail"], 2);
    assert!(value["checks"].as_array().unwrap().iter().any(|check| {
        check["check_id"] == "governance-marker"
            && check["path"] == "AGENTS.md"
            && check["actual"] == "missing marker block"
            && check["status"] == "fail"
    }));
    assert!(value["checks"].as_array().unwrap().iter().any(|check| {
        check["check_id"] == "governance-marker"
            && check["path"] == "GEMINI.md"
            && check["actual"] == "marker block out of order"
            && check["status"] == "fail"
    }));
}

#[test]
fn status_check_json_rejects_stale_marker_content() {
    let root = temp_root("status-json-stale-marker-failure");
    create_native_agent_skills(&root);
    let stale =
        "<!-- agent-governance:begin -->\nold managed block\n<!-- agent-governance:end -->\n";
    fs::write(root.join("AGENTS.md"), stale).unwrap();
    fs::write(root.join("GEMINI.md"), stale).unwrap();
    let args = vec!["--format".to_string(), "json".to_string()];

    let error = crate::status_checks::run_command(&root, &args)
        .expect_err("stale marker content must fail JSON status");

    let crate::reports::RuntimeFailure::Json {
        payload: output, ..
    } = error
    else {
        panic!("status marker JSON failure must preserve raw JSON");
    };
    let value = serde_json::from_str::<serde_json::Value>(&output).unwrap();
    assert_eq!(value["surface"], "status");
    assert_eq!(value["summary"]["fail"], 2);
    assert!(value["checks"].as_array().unwrap().iter().any(|check| {
        check["check_id"] == "governance-marker"
            && check["path"] == "AGENTS.md"
            && check["actual"] == "stale marker content"
            && check["status"] == "fail"
    }));
    assert!(value["checks"].as_array().unwrap().iter().any(|check| {
        check["check_id"] == "governance-marker"
            && check["path"] == "GEMINI.md"
            && check["actual"] == "stale marker content"
            && check["status"] == "fail"
    }));
}

#[test]
fn status_check_fails_missing_native_skill_projection() {
    let report = crate::status_checks::report(
        ReportSurface::Status,
        vec![crate::status_checks::native_skill_check(
            ".agents/skills/task-registry-flow",
            false,
        )],
    )
    .unwrap();

    assert!(report.has_failures());
    assert_eq!(report.checks[0].check_id, "native-skill");
}

#[cfg(unix)]
#[test]
fn status_check_json_reports_stale_migration_layouts_with_remediation() {
    let root = temp_root("status-json-stale-migration-layouts");
    write_marker_docs(&root);
    fs::create_dir_all(root.join(".cursor/skills")).unwrap();
    fs::create_dir_all(root.join(".agents/skills")).unwrap();
    std::os::unix::fs::symlink(
        "../../.cursor/skills/gap-closure-contract",
        root.join(".agents/skills/gap-closure-contract"),
    )
    .unwrap();
    std::os::unix::fs::symlink(
        "../../.cursor/skills/task-registry-flow",
        root.join(".agents/skills/task-registry-flow"),
    )
    .unwrap();
    for path in [
        "hooks.json",
        ".codex/settings.toml",
        ".codex/hooks/user-plan-approval.toml",
        ".gemini/settings.json",
        "tools/antigravity/pre-tool-use-gap-closure.sh",
    ] {
        let full_path = root.join(path);
        fs::create_dir_all(full_path.parent().unwrap()).unwrap();
        fs::write(full_path, "legacy\n").unwrap();
    }
    let args = vec!["--format".to_string(), "json".to_string()];

    let error = crate::status_checks::run_command(&root, &args)
        .expect_err("stale migration layout must fail JSON status");

    let crate::reports::RuntimeFailure::Json {
        payload: output, ..
    } = error
    else {
        panic!("status stale migration JSON failure must preserve raw JSON");
    };
    let value = serde_json::from_str::<serde_json::Value>(&output).unwrap();
    assert_eq!(value["surface"], "status");
    assert_eq!(value["summary"]["fail"], 7);
    for path in [
        ".agents/skills/gap-closure-contract",
        ".agents/skills/task-registry-flow",
    ] {
        assert!(value["checks"].as_array().unwrap().iter().any(|check| {
            check["check_id"] == "native-skill"
                && check["path"] == path
                && check["status"] == "fail"
                && check["remediation"] == "replace legacy skill symlink with native directory"
        }));
    }
    for path in [
        "hooks.json",
        ".codex/settings.toml",
        ".codex/hooks/user-plan-approval.toml",
        ".gemini/settings.json",
        "tools/antigravity/pre-tool-use-gap-closure.sh",
    ] {
        assert!(value["checks"].as_array().unwrap().iter().any(|check| {
            check["check_id"] == "stale-legacy-path"
                && check["path"] == path
                && check["actual"] == "present"
                && check["status"] == "fail"
                && check["remediation"]
                    .as_str()
                    .unwrap()
                    .contains("install-to-workspace --merge or --force")
        }));
    }
}

fn create_native_agent_skills(root: &Path) {
    for skill in ["gap-closure-contract", "task-registry-flow"] {
        fs::create_dir_all(root.join(".agents/skills").join(skill)).unwrap();
    }
}

fn write_marker_docs(root: &Path) {
    let agents = "<!-- agent-governance:begin -->\n## Agent governance (portable plugin)\n| Registry CLI | `.codex/scripts/task-registry` |\n| Source limit | 1600 lines |\n<!-- agent-governance:end -->\n";
    let gemini = "<!-- agent-governance:begin -->\n## Agent governance (portable plugin)\n- Antigravity hook: `.agents/hooks.json`\n- Source limit: 1600 lines\n<!-- agent-governance:end -->\n";
    fs::write(root.join("AGENTS.md"), agents).unwrap();
    fs::write(root.join("GEMINI.md"), gemini).unwrap();
}
