# V2 Release Readiness

## Approved Scope

Prepare the plugin source repository for a v2 release:

- Align release metadata on version `2.0.0`.
- Add release documentation, license artifact, and source-package readiness gates.
- Add deterministic release audit and version consistency scripts.
- Split plugin-source release readiness from consumer-installed workspace status.
- Keep consumer strict status intact.

Out of scope:

- Publishing a GitHub release, marketplace release, or tag.
- Reintroducing `--overlay` or legacy compatibility shims.
- Changing consumer install semantics except for release-readiness validation.

License default: all rights reserved unless the repository owner later chooses a different license.
Audit default: missing `cargo-audit` or `cargo-deny` blocks production unless `AGENT_GOVERNANCE_ALLOW_AUDIT_TOOL_WAIVER=1` is explicitly set.

Primitive change gate: N/A. This is release governance and package validation; it does not change runtime product primitives.

## Required Change Checklist

- [MODIFY] `plugin.json` - set plugin package version to `2.0.0`.
- [MODIFY] `.codex-plugin/plugin.json` - set Codex plugin version to `2.0.0`.
- [MODIFY] `MANIFEST.toml` - set plugin version to `2.0.0` and remove stale install-mode wording.
- [MODIFY] `rust/task-registry-flow-cli/Cargo.toml` - set CLI package version to `2.0.0`.
- [MODIFY] `rust/task-registry-flow-cli/Cargo.lock` - refresh local package version.
- [NEW] `rust/task-registry-flow-cli/deny.toml` - define explicit release dependency license, bans, advisory, and source policy.
- [NEW] `VERSION` - single-source release version text.
- [NEW] `LICENSE` - all-rights-reserved license artifact.
- [NEW] `CHANGELOG.md` - v2 release notes, breaking changes, and validation proof.
- [NEW] `docs/releases/v2.md` - release checklist and readiness criteria.
- [NEW] `scripts/release-version-check.sh` - positive and negative version consistency gate.
- [NEW] `scripts/release-audit.sh` - dependency and audit gate with explicit waiver behavior.
- [NEW] `scripts/test-release-readiness.sh` - release-readiness positive and negative test suite.
- [MODIFY] `scripts/status.sh` - add `--release-source` mode for plugin-source release readiness.
- [MODIFY] `.github/workflows/ci.yml` - run release scripts and status mode.
- [MODIFY] `README.md` - document v2 release artifacts and source-vs-consumer status.
- [MODIFY] `REQUIREMENTS.toml` - document release-source required artifacts.
- [MODIFY] `rules/tracked-for-ci.md` - clarify consumer vs plugin-source release tracking.

## Per-Gap Success Criteria

### Version Metadata

Good behavior:

- `VERSION`, `plugin.json`, `.codex-plugin/plugin.json`, `MANIFEST.toml`, and Rust `Cargo.toml` all report `2.0.0`.
- Version drift fails release checks.
- Missing `VERSION` fails release checks.

### Release Artifacts

Good behavior:

- `CHANGELOG.md`, `LICENSE`, and `docs/releases/v2.md` exist.
- README links point to existing release files.
- Release docs mention v2, breaking changes, migration notes, validation gates, audit policy, and known limitations.

### Source Readiness Status

Good behavior:

- `scripts/status.sh --release-source` validates plugin package surfaces without requiring consumer `.codex/`, `.agents/`, `.cursor/`, `AGENTS.md`, `GEMINI.md`, or `plugins/agent-governance`.
- Consumer `scripts/status.sh --strict` remains unchanged and still fails missing consumer install artifacts.
- Release-source mode fails dirty or untracked release-critical files.

### Audit Gate

Good behavior:

- Required Rust gates pass.
- Duplicate dependencies fail if present.
- Dependency licenses, source registries, bans, and advisories are evaluated by an explicit `cargo-deny` policy.
- Missing `cargo-audit` or `cargo-deny` fails without explicit waiver.
- Waiver mode is visible and non-default.

## Validation Plan

Focused tests:

- `scripts/release-version-check.sh`
- `bash scripts/test-release-readiness.sh`
- `AGENT_GOVERNANCE_ALLOW_AUDIT_TOOL_WAIVER=1 scripts/release-audit.sh`
- `scripts/release-audit.sh`

Full gates:

- `bash -n scripts/install-to-workspace.sh scripts/render-from-config.sh scripts/status.sh scripts/test-install-modes.sh scripts/test-release-readiness.sh scripts/release-version-check.sh scripts/release-audit.sh scripts/pre-tool-use-gap-closure.sh templates/.codex/scripts/task-registry.template templates/.cursor/hooks/gap-closure-gate.sh.template templates/tools/agent-governance/pre-tool-use-gap-closure.sh.template`
- `bash scripts/test-install-modes.sh`
- `bash scripts/test-release-readiness.sh`
- `cargo run --locked --quiet --manifest-path rust/task-registry-flow-cli/Cargo.toml -- source-limit check`
- `cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings`
- `agy plugin validate .`
- `cargo run --quiet --manifest-path rust/task-registry-flow-cli/Cargo.toml -- validate`
- `cargo run --quiet --manifest-path rust/task-registry-flow-cli/Cargo.toml -- metrics`
- `git diff --check`

Release final gates after commit:

- `git status --short` is empty.
- `scripts/status.sh --release-source` passes.
- `scripts/release-audit.sh` passes without waiver.

## Walkthrough Evidence

Capture:

- Plan activation output.
- Version consistency output.
- Release-readiness negative test output summary.
- Audit output, including whether waiver was used.
- Final status-source result or its remaining dirty-tree blocker.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-05-30-v2-release-readiness"

[[behaviors]]
behavior_id = "B-2026-05-30-v2-version-consistency"
gap_id = "GAP-2026-05-30-v2-version-consistency"
polarity = "positive"
title = "V2 versions are consistent"
given = "A v2 release package"
when = "The version consistency script runs"
then = "All version-bearing files agree on 2.0.0 and drift/missing version files fail"
confirmation = "bash scripts/test-release-readiness.sh version"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-release-readiness.sh version"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-30-v2-release-artifacts"
gap_id = "GAP-2026-05-30-v2-release-artifacts"
polarity = "positive"
title = "V2 release artifacts exist"
given = "A v2 release package"
when = "The release readiness test runs"
then = "Release notes, license artifact, and v2 release docs exist and are linked"
confirmation = "bash scripts/test-release-readiness.sh artifacts"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-release-readiness.sh artifacts"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-30-v2-source-status"
gap_id = "GAP-2026-05-30-v2-source-status"
polarity = "positive"
title = "Plugin-source status is distinct from consumer status"
given = "The plugin source repository"
when = "Release-source and consumer strict status checks are evaluated"
then = "Release-source checks package readiness without requiring consumer install artifacts, while consumer strict status remains strict"
confirmation = "bash scripts/test-release-readiness.sh status"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-release-readiness.sh status"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-30-v2-audit-gate"
gap_id = "GAP-2026-05-30-v2-audit-gate"
polarity = "positive"
title = "V2 audit gate is deterministic"
given = "The Rust task-registry CLI package"
when = "The release audit script runs"
then = "Required Rust gates pass, duplicate dependency checks run, and missing audit tools require an explicit waiver"
confirmation = "bash scripts/test-release-readiness.sh audit"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-release-readiness.sh audit"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-05-30-v2-release-readiness-001"
title = "Align v2 version metadata"
status = "planned"
kind = "release"
reason = "Release metadata is still pre-v2."
acceptance_proof = "Behavior B-2026-05-30-v2-version-consistency passes."
behavior_ids = ["B-2026-05-30-v2-version-consistency"]
[[tasks.targets]]
file = "plugin.json"
object = "package_version"
required_change = "Set version to 2.0.0."
[[tasks.targets]]
file = ".codex-plugin/plugin.json"
object = "codex_plugin_version"
required_change = "Set version to 2.0.0."
[[tasks.targets]]
file = "MANIFEST.toml"
object = "plugin_version"
required_change = "Set plugin_version to 2.0.0 and remove stale mode wording."
[[tasks.targets]]
file = "rust/task-registry-flow-cli/Cargo.toml"
object = "rust_cli_version"
required_change = "Set package version to 2.0.0."
[[tasks.targets]]
file = "VERSION"
object = "release_version_file"
required_change = "Add 2.0.0 release version file."
[[tasks.targets]]
file = "scripts/release-version-check.sh"
object = "version_consistency_gate"
required_change = "Add version drift and missing file checks."

[[tasks]]
task_id = "TASK-2026-05-30-v2-release-readiness-002"
title = "Add v2 release artifacts"
status = "planned"
kind = "release"
reason = "Release notes, license artifact, and v2 release checklist are missing."
acceptance_proof = "Behavior B-2026-05-30-v2-release-artifacts passes."
behavior_ids = ["B-2026-05-30-v2-release-artifacts"]
[[tasks.targets]]
file = "CHANGELOG.md"
object = "v2_release_notes"
required_change = "Add v2 release notes with breaking changes, migration notes, and validation proof."
[[tasks.targets]]
file = "LICENSE"
object = "license_artifact"
required_change = "Add all-rights-reserved license artifact."
[[tasks.targets]]
file = "docs/releases/v2.md"
object = "v2_release_checklist"
required_change = "Add v2 release readiness checklist."
[[tasks.targets]]
file = "README.md"
object = "release_artifact_links"
required_change = "Link v2 release artifacts and clarify release-source checks."

[[tasks]]
task_id = "TASK-2026-05-30-v2-release-readiness-003"
title = "Add plugin-source release status"
status = "planned"
kind = "release"
reason = "Consumer strict status cannot be used as plugin-source release readiness."
acceptance_proof = "Behavior B-2026-05-30-v2-source-status passes."
behavior_ids = ["B-2026-05-30-v2-source-status"]
[[tasks.targets]]
file = "scripts/status.sh"
object = "release_source_mode"
required_change = "Add --release-source mode that checks plugin-source package readiness separately from consumer install posture."
[[tasks.targets]]
file = "scripts/test-release-readiness.sh"
object = "release_status_tests"
required_change = "Add positive and negative tests for release-source and consumer strict status separation."
[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release_source_artifacts"
required_change = "Document plugin-source release artifacts."
[[tasks.targets]]
file = "rules/tracked-for-ci.md"
object = "source_vs_consumer_tracking"
required_change = "Clarify consumer tracked artifacts and plugin-source release artifacts."

[[tasks]]
task_id = "TASK-2026-05-30-v2-release-readiness-004"
title = "Add deterministic release audit gate"
status = "planned"
kind = "release"
reason = "Dependency and security audit tooling was missing or undocumented during the v2 sweep."
acceptance_proof = "Behavior B-2026-05-30-v2-audit-gate passes."
behavior_ids = ["B-2026-05-30-v2-audit-gate"]
[[tasks.targets]]
file = "scripts/release-audit.sh"
object = "release_audit_gate"
required_change = "Add Rust gates, duplicate dependency detection, audit tool checks, and explicit waiver behavior."
[[tasks.targets]]
file = "rust/task-registry-flow-cli/deny.toml"
object = "release_dependency_policy"
required_change = "Add explicit cargo-deny policy for licenses, bans, advisories, and source registries."
[[tasks.targets]]
file = ".github/workflows/ci.yml"
object = "release_ci_gates"
required_change = "Run release readiness tests and audit gate in CI."
[[tasks.targets]]
file = "scripts/test-release-readiness.sh"
object = "audit_negative_tests"
required_change = "Add negative tests for missing audit tools without waiver and positive waiver behavior."
```
