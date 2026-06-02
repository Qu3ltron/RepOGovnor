use super::*;
#[cfg(unix)]
use std::os::unix::fs::symlink;

fn assert_governed_source_omission(path: &str) {
    let root = temp_root("release-source-governed-omission");
    seed_release_repo(&root);
    let full_path = root.join(path);
    fs::create_dir_all(full_path.parent().unwrap()).unwrap();
    fs::write(&full_path, "governed source\n").unwrap();

    let report = release_checks::report(&root, release_checks::Mode::Required).unwrap();

    assert!(report.checks.iter().any(|check| {
        check.check_id == "release-governed-source-undeclared"
            && check.path == path
            && check.status == CheckStatus::Fail
            && check.actual == "missing release policy"
    }));
}

#[test]
fn release_source_rejects_undeclared_nix_package_file() {
    assert_governed_source_omission("flake.nix");
}

#[test]
fn release_source_rejects_undeclared_nixos_module() {
    assert_governed_source_omission("modules/nixos/agent-governance-auto-update.nix");
}

#[test]
fn release_source_rejects_undeclared_claude_template() {
    assert_governed_source_omission("templates/CLAUDE.md.template");
}

#[test]
fn release_source_rejects_undeclared_hook_template() {
    assert_governed_source_omission(
        "templates/tools/agent-governance/pre-tool-use-gap-closure.sh.template",
    );
}

#[test]
fn release_source_rejects_undeclared_claude_skill() {
    assert_governed_source_omission(".claude/skills/run-governance-plugin/SKILL.md");
}

#[cfg(unix)]
#[test]
fn release_source_rejects_required_symlink() {
    let root = temp_root("release-source-required-symlink");
    seed_release_repo(&root);
    fs::remove_file(root.join("README.md")).unwrap();
    symlink("/tmp/agent-governance-readme", root.join("README.md")).unwrap();

    let report = release_checks::report(&root, release_checks::Mode::Required).unwrap();

    assert!(report.has_failures());
    assert!(report.checks.iter().any(|check| {
        check.check_id == "release-file-present"
            && check.path == "README.md"
            && check.status == CheckStatus::Fail
            && check.actual == "symlink"
    }));
}

#[test]
fn release_source_rejects_stale_markdown_version_file() {
    let root = temp_root("release-source-markdown-version");
    seed_release_repo(&root);
    fs::write(
        root.join("docs/releases/v2.md"),
        "# V2 Release Checklist\n\nRelease version: `2.0.1`\n",
    )
    .unwrap();

    let report = release_checks::report(&root, release_checks::Mode::Version).unwrap();

    assert!(report.has_failures());
    assert!(report.checks.iter().any(|check| {
        check.check_id == "release-version-consistent"
            && check.path == "docs/releases/v2.md"
            && check.status == CheckStatus::Fail
            && check.actual == "version 2.0.1"
    }));
}

#[test]
fn public_cost_evidence_allows_sanitized_paths() {
    let root = temp_root("release-source-public-cost-sanitized");
    seed_release_repo(&root);
    fs::create_dir_all(root.join("docs/task-registry")).unwrap();
    declare_cost_events_release_source(&root);
    fs::write(
        root.join("docs/task-registry/events.jsonl"),
        r#"{"cost_evidence":{"usage_contributions":[{"source_path":"docs/private-cost-redacted.jsonl"}]}}"#,
    )
    .unwrap();

    let report = release_checks::report(&root, release_checks::Mode::Required).unwrap();

    assert!(report.checks.iter().any(|check| {
        check.check_id == "release-private-cost-evidence-absent"
            && check.status == CheckStatus::Pass
    }));
}

#[test]
fn public_cost_evidence_rejects_private_transcript_paths() {
    let root = temp_root("release-source-private-cost-path");
    seed_release_repo(&root);
    fs::create_dir_all(root.join("docs/task-registry")).unwrap();
    declare_cost_events_release_source(&root);
    fs::write(
        root.join("docs/task-registry/events.jsonl"),
        "/home/hasnamuss/.codex/sessions/2026/05/31/private.jsonl\n",
    )
    .unwrap();

    let report = release_checks::report(&root, release_checks::Mode::Required).unwrap();

    assert!(report.checks.iter().any(|check| {
        check.check_id == "release-private-cost-evidence-absent"
            && check.status == CheckStatus::Fail
            && check.actual.contains("/home/hasnamuss/.codex/sessions/")
    }));
}

fn declare_cost_events_release_source(root: &Path) {
    let path = root.join("REQUIREMENTS.toml");
    let body = fs::read_to_string(&path).unwrap();
    fs::write(
        path,
        body.replace(
            r#"required = ["VERSION", "README.md","#,
            r#"required = ["VERSION", "README.md", "docs/task-registry/events.jsonl","#,
        ),
    )
    .unwrap();
}
