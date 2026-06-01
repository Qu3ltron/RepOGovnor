use super::*;
use crate::reports::RuntimeFailure;

const PLAN_ID: &str = "PLAN-2026-06-01-version-backlog-governance";

#[test]
fn version_governance_validate_accepts_current_release() {
    let root = repo_root();
    let report = crate::version_check::report(&root).unwrap();

    assert_eq!(report.summary.fail, 0);
    assert!(report.checks.iter().any(|check| {
        check.check_id == "version-completed-plan-covered" && check.status == CheckStatus::Pass
    }));
}

#[test]
fn version_governance_next_and_prerelease_are_deterministic() {
    let root = repo_root();
    let next = crate::version_check::run_command(&root, &args(&["next", PLAN_ID]))
        .expect("next release output should render");

    assert!(next.contains("version=2.1.0"));
    assert!(next.contains("tag=v2.1.0"));
    assert!(next.contains("prerelease_tag=v2.1.0-rc.1"));
    assert!(next.contains("manual_final_release=true"));

    let prerelease =
        crate::version_check::run_command(&root, &args(&["prerelease", PLAN_ID, "--rc", "1"]))
            .expect("prerelease output should render");

    assert!(prerelease.contains("push_branch=git push origin main"));
    assert!(prerelease.contains("push_prerelease_tag=git push origin v2.1.0-rc.1"));
    assert!(!prerelease.contains("push_final"));
    assert!(!prerelease.contains("git push origin v2.1.0\n"));
}

#[test]
fn version_governance_rejects_uncovered_completed_plan() {
    let root = temp_root("version-uncovered");
    write_version_fixture(&root, FixtureMode::Valid);
    let mut registry = fs::read_to_string(root.join("docs/task-registry.toml")).unwrap();
    registry.push_str(
        r#"
[[plans]]
plan_id = "PLAN-2026-06-01-uncovered"
plan_path = "docs/plans/uncovered.md"
plan_hash_sha256 = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
activated_at = "2026-06-01"
status = "active"

[[tasks]]
task_id = "TASK-uncovered"
plan_id = "PLAN-2026-06-01-uncovered"
status = "completed"
title = "Uncovered"
kind = "governance"
source_plan_path = "docs/plans/uncovered.md"
source_plan_hash_sha256 = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
reason = "fixture"
acceptance_proof = "fixture"
created_at = "2026-06-01"
updated_at = "2026-06-01"

[[tasks.targets]]
file = "README.md"
object = "fixture"
required_change = "fixture"
"#,
    );
    fs::write(root.join("docs/task-registry.toml"), registry).unwrap();

    let report = crate::version_check::report(&root).unwrap();

    assert!(report.has_failures());
    assert!(report.checks.iter().any(|check| {
        check.check_id == "version-completed-plan-covered"
            && check.actual == "missing"
            && check.status == CheckStatus::Fail
    }));
}

#[test]
fn version_governance_rejects_stale_version_surface() {
    let root = temp_root("version-stale-surface");
    write_version_fixture(&root, FixtureMode::Valid);
    fs::write(root.join("README.md"), "Current release: `2.0.9`\n").unwrap();

    let report = crate::version_check::report(&root).unwrap();

    assert!(report.has_failures());
    assert!(report.checks.iter().any(|check| {
        check.check_id == "version-surface"
            && check.path == "README.md"
            && check.actual == "version 2.0.9"
    }));
}

#[test]
fn version_governance_rejects_illegal_semver_bump() {
    let root = temp_root("version-illegal-bump");
    write_version_fixture(&root, FixtureMode::IllegalBump);

    let report = crate::version_check::report(&root).unwrap();

    assert!(report.has_failures());
    assert!(report.checks.iter().any(|check| {
        check.check_id == "version-semver-bump" && check.status == CheckStatus::Fail
    }));
}

#[test]
fn version_governance_rejects_missing_final_tag() {
    let root = temp_root("version-final-tag");
    write_version_fixture(&root, FixtureMode::Valid);
    let args = args(&["release-check", "--format", "json"]);

    let error = crate::version_check::run_command(&root, &args)
        .expect_err("missing final tag should fail release-check");

    let RuntimeFailure::Json { payload, .. } = error else {
        panic!("release-check failure should preserve JSON report");
    };
    let value = serde_json::from_str::<serde_json::Value>(&payload).unwrap();
    assert_eq!(value["surface"], "version");
    assert!(value["checks"].as_array().unwrap().iter().any(|check| {
        check["check_id"] == "version-final-tag-head" && check["status"] == "fail"
    }));
}

#[test]
fn version_governance_rejects_final_push_in_prerelease_output() {
    let root = repo_root();
    let output =
        crate::version_check::run_command(&root, &args(&["prerelease", PLAN_ID, "--rc", "1"]))
            .expect("prerelease output should render");

    assert!(!output.contains("git push origin v2.1.0\n"));
    assert!(output.contains("git push origin v2.1.0-rc.1"));
}

enum FixtureMode {
    Valid,
    IllegalBump,
}

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn write_version_fixture(root: &Path, mode: FixtureMode) {
    fs::create_dir_all(root.join(".codex-plugin")).unwrap();
    fs::create_dir_all(root.join("docs/releases")).unwrap();
    fs::create_dir_all(root.join("rust/task-registry-flow-cli")).unwrap();
    fs::write(root.join("VERSION"), "2.1.0\n").unwrap();
    fs::write(root.join("README.md"), "Current release: `2.1.0`\n").unwrap();
    fs::write(
        root.join("docs/releases/v2.md"),
        "# V2\n\nRelease version: `2.1.0`\n",
    )
    .unwrap();
    fs::write(root.join("plugin.json"), r#"{"version":"2.1.0"}"#).unwrap();
    fs::write(
        root.join(".codex-plugin/plugin.json"),
        r#"{"version":"2.1.0"}"#,
    )
    .unwrap();
    fs::write(root.join("MANIFEST.toml"), "plugin_version = \"2.1.0\"\n").unwrap();
    fs::write(
        root.join("rust/task-registry-flow-cli/Cargo.toml"),
        "[package]\nversion = \"2.1.0\"\n",
    )
    .unwrap();
    fs::write(
        root.join("rust/task-registry-flow-cli/Cargo.lock"),
        "[[package]]\nname = \"task-registry-flow-cli\"\nversion = \"2.1.0\"\n",
    )
    .unwrap();
    fs::write(root.join("package.nix"), "version = \"2.1.0\";\n").unwrap();
    fs::write(
        root.join("REQUIREMENTS.toml"),
        "[release_source]\nversion = \"2.1.0\"\n",
    )
    .unwrap();
    fs::write(
        root.join("CHANGELOG.md"),
        "# Changelog\n\n## 2.1.0 - 2026-06-01\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("docs/task-registry/archive")).unwrap();
    fs::write(
        root.join("docs/task-registry/archive/completed-001.toml"),
        archive(),
    )
    .unwrap();
    fs::write(root.join("docs/task-registry.toml"), registry()).unwrap();
    fs::write(
        root.join("docs/version-roadmap.toml"),
        roadmap(matches!(mode, FixtureMode::IllegalBump)),
    )
    .unwrap();
}

fn roadmap(illegal_bump: bool) -> String {
    let bump = if illegal_bump { "patch" } else { "minor" };
    format!(
        r#"schema_version = 1
version_model = "governed_semver"
current_version = "2.1.0"
previous_version = "2.0.2"
release_branch = "main"
remote = "origin"
tag_prefix = "v"
push_policy = "auto_prerelease_manual_release"
cutover_plan_id = "PLAN-2026-06-01-cutover"

[[releases]]
version = "2.1.0"
date = "2026-06-01"
plan_id = "{PLAN_ID}"
bump = "{bump}"
tag = "v2.1.0"
prerelease_tag = "v2.1.0-rc.1"
commit_subject = "feat(governance): add version and backlog checks"
summary = "Add version and backlog governance."
covered_plan_ids = [
  "PLAN-2026-06-01-cutover",
  "{PLAN_ID}",
]
"#
    )
}

fn archive() -> &'static str {
    r#"schema_version = 1
registry_id = "agent-governance-task-registry"
archive_id = "completed-001"
archive_authority = "docs/task-registry/archive/completed-001.toml"

[[plans]]
plan_id = "PLAN-2026-06-01-cutover"
plan_path = "docs/plans/cutover.md"
plan_hash_sha256 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
activated_at = "2026-06-01"
status = "active"

[[tasks]]
task_id = "TASK-cutover"
plan_id = "PLAN-2026-06-01-cutover"
status = "completed"
title = "Cutover"
kind = "governance"
source_plan_path = "docs/plans/cutover.md"
source_plan_hash_sha256 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
reason = "fixture"
acceptance_proof = "fixture"
created_at = "2026-06-01"
updated_at = "2026-06-01"

[[tasks.targets]]
file = "README.md"
object = "fixture"
required_change = "fixture"
"#
}

fn registry() -> &'static str {
    r#"schema_version = 1
registry_id = "agent-governance-task-registry"
registry_authority = "docs/task-registry.toml"
activation_skill = "task-registry-flow"
hash_algorithm = "sha256"
status_vocabulary = ["planned", "active", "blocked", "deferred", "completed", "cancelled"]
archive_paths = ["docs/task-registry/archive/completed-001.toml"]

[[plans]]
plan_id = "PLAN-2026-06-01-version-backlog-governance"
plan_path = "docs/plans/version-backlog-governance-2026-06-01.md"
plan_hash_sha256 = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
activated_at = "2026-06-01"
status = "active"

[[tasks]]
task_id = "TASK-vbg"
plan_id = "PLAN-2026-06-01-version-backlog-governance"
status = "completed"
title = "VBG"
kind = "governance"
source_plan_path = "docs/plans/version-backlog-governance-2026-06-01.md"
source_plan_hash_sha256 = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
reason = "fixture"
acceptance_proof = "fixture"
created_at = "2026-06-01"
updated_at = "2026-06-01"

[[tasks.targets]]
file = "README.md"
object = "fixture"
required_change = "fixture"
"#
}
