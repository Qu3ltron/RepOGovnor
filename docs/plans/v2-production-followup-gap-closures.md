# V2 Production Follow-Up Gap Closures

## Approved Scope

Close the two production-blocking review gaps for v2:

- Legacy v0.3 skill symlinks must be replaced by native v2 skill directories during installer upgrades.
- Release audit must run source-limit checks from the repository root, regardless of caller working directory.

Out of scope:

- Reintroducing `--overlay`.
- Supporting legacy skill symlinks as a valid v2 state.
- Publishing, tagging, or committing the v2 release.
- Changing consumer install semantics beyond rejecting stale native skill projection symlinks.

Primitive change gate: N/A. This is installer migration and release-governance behavior; it does not change runtime primitives, persistence schemas, APIs, queues, or provider contracts.

## Required Change Checklist

- [MODIFY] `scripts/render-from-config.sh` - treat skill destination symlinks as replaceable before alignment checks. Acceptance proof: `bash scripts/test-install-modes.sh`.
- [MODIFY] `scripts/status.sh` - fail strict status when native skill projection paths are symlinks. Acceptance proof: `bash scripts/test-install-modes.sh` and `AGENT_GOVERNANCE_ALLOW_DIRTY_RELEASE_CHECK=1 scripts/status.sh --release-source`.
- [MODIFY] `scripts/test-install-modes.sh` - add dry-run, merge, force, and negative migration assertions for legacy skill symlinks. Acceptance proof: `bash scripts/test-install-modes.sh`.
- [MODIFY] `scripts/release-audit.sh` - anchor execution at repo root before source-limit and remove ignored `--root` argument. Acceptance proof: `bash scripts/test-release-readiness.sh audit`.
- [MODIFY] `rust/task-registry-flow-cli/src/source_limit.rs` - reject unexpected `source-limit check` arguments. Acceptance proof: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`.
- [MODIFY] `rust/task-registry-flow-cli/src/tests.rs` - add negative unit coverage for unexpected `source-limit check` args. Acceptance proof: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`.
- [MODIFY] `scripts/test-release-readiness.sh` - add nested release-audit positive and over-limit root-file negative tests. Acceptance proof: `bash scripts/test-release-readiness.sh audit`.
- [MODIFY] `docs/releases/v2.md` - document cwd-independent release audit behavior. Acceptance proof: `bash scripts/test-release-readiness.sh artifacts`.
- [MODIFY] `CHANGELOG.md` - record follow-up production blocker fixes. Acceptance proof: `bash scripts/test-release-readiness.sh artifacts`.

## Per-Gap Success Criteria

### Legacy Skill Symlink Replacement

Good behavior:

- A v0.3 workspace with `.agents/skills/<skill>` symlinks upgrades to native v2 directories.
- `--merge` and `--force` leave `.agents/skills/gap-closure-contract` and `.agents/skills/task-registry-flow` as real directories, not symlinks.
- Dry-run reports replacement without changing filesystem state.
- Strict status rejects symlinked native skill projections.
- `PROJECT.md` is preserved only from real native directories, not from symlink targets.

Negative behavior that must fail tests:

- Symlink path reports `aligned`.
- Symlink path reports `preserve-drift`.
- Symlink survives `--merge` or `--force`.
- Symlink target `PROJECT.md` is copied as if it were the native projection.

### Release Audit Root Anchoring

Good behavior:

- `scripts/release-audit.sh` behaves the same from repo root and nested directories.
- Source-limit scans the git repo root, not `$PWD`.
- Over-limit root files fail audit even when audit starts from a nested directory.
- Invalid `source-limit check` trailing arguments fail fast.

Negative behavior that must fail tests:

- Nested audit misses an over-limit root file.
- `source-limit check --root <path>` succeeds.
- Audit passes because it scanned the wrong directory.

## Validation Plan

Focused gates:

```bash
bash scripts/test-install-modes.sh
bash scripts/test-release-readiness.sh audit
cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml
```

Full gates:

```bash
bash scripts/test-release-readiness.sh all
scripts/release-audit.sh
AGENT_GOVERNANCE_ALLOW_DIRTY_RELEASE_CHECK=1 scripts/status.sh --release-source
cargo run --quiet --manifest-path rust/task-registry-flow-cli/Cargo.toml -- validate
cargo run --quiet --manifest-path rust/task-registry-flow-cli/Cargo.toml -- report PLAN-2026-05-30-v2-production-followup-gap-closures
agy plugin validate .
git diff --check
```

Failure evidence:

- Any surviving `.agents/skills/<skill>` symlink means the migration gap remains open.
- Any nested audit that misses a root over-limit file means the release audit gap remains open.
- Any task still planned, active, deferred, or blocked means production closure is incomplete.

## Walkthrough Evidence

Capture:

- Plan activation output.
- `bash scripts/test-install-modes.sh` result proving legacy symlink replacement.
- `bash scripts/test-release-readiness.sh audit` result proving nested audit behavior and negative missing-tool behavior.
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml` result proving CLI argument rejection.
- Final task registry report.

## Task Manifest

```toml
schema_version = 1
plan_id = "PLAN-2026-05-30-v2-production-followup-gap-closures"

[[behaviors]]
behavior_id = "B-2026-05-30-v2-followup-skill-symlink-replacement"
title = "Legacy skill symlinks are replaced by native directories"
given = "A v0.3 workspace with .agents skill symlinks pointing at .cursor skill directories"
when = "The installer runs in dry-run, merge, and force modes"
then = "Dry-run reports replacement without mutation, and merge/force replace symlinks with native v2 skill directories"
confirmation = "bash scripts/test-install-modes.sh"

[[behaviors]]
behavior_id = "B-2026-05-30-v2-followup-release-audit-root"
title = "Release audit scans the repository root from any caller directory"
given = "The plugin source repository or a temp copy with a root-level source-limit violation"
when = "scripts/release-audit.sh is launched from a nested directory"
then = "The normal repo passes and the temp repo with a root-level violation fails with that file named"
confirmation = "bash scripts/test-release-readiness.sh audit"

[[tasks]]
task_id = "TASK-2026-05-30-v2-followup-001"
title = "Replace legacy skill symlinks with native v2 directories"
status = "planned"
kind = "migration"
reason = "Legacy v0.3 skill symlinks can be treated as aligned and survive v2 installs."
acceptance_proof = "Behavior B-2026-05-30-v2-followup-skill-symlink-replacement passes."
behavior_ids = ["B-2026-05-30-v2-followup-skill-symlink-replacement"]
[[tasks.targets]]
file = "scripts/render-from-config.sh"
object = "sync_skill"
required_change = "Detect symlink destinations before directory alignment and replace them as stale migration artifacts."
[[tasks.targets]]
file = "scripts/status.sh"
object = "skill_diff_hint"
required_change = "Fail strict status for symlinked native skill projection paths."
[[tasks.targets]]
file = "scripts/test-install-modes.sh"
object = "legacy_skill_symlink_migration_tests"
required_change = "Add dry-run, merge, force, and negative assertions for legacy skill symlink replacement."

[[tasks]]
task_id = "TASK-2026-05-30-v2-followup-002"
title = "Anchor release audit source-limit checks at repo root"
status = "planned"
kind = "release"
reason = "release-audit.sh can run source-limit against a subdirectory because the CLI ignored the extra --root argument."
acceptance_proof = "Behavior B-2026-05-30-v2-followup-release-audit-root passes."
behavior_ids = ["B-2026-05-30-v2-followup-release-audit-root"]
[[tasks.targets]]
file = "scripts/release-audit.sh"
object = "repo_root_execution"
required_change = "cd to repo root before source-limit and remove ignored --root argument."
[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/source_limit.rs"
object = "source_limit_check_args"
required_change = "Reject unexpected arguments for source-limit check."
[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests.rs"
object = "source_limit_check_arg_tests"
required_change = "Add negative unit test proving extra check args fail."
[[tasks.targets]]
file = "scripts/test-release-readiness.sh"
object = "nested_release_audit_tests"
required_change = "Add nested positive audit test and over-limit root-file negative test."
[[tasks.targets]]
file = "docs/releases/v2.md"
object = "release_audit_policy"
required_change = "Document cwd-independent release audit behavior."
[[tasks.targets]]
file = "CHANGELOG.md"
object = "v2_followup_notes"
required_change = "Record production-blocker fixes for skill symlinks and release audit root scanning."
```
