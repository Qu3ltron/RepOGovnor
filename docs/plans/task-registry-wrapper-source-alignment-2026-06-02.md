# Task Registry Wrapper Source Alignment Contract

## Approved Scope
Close the release-gate gap where `.codex/scripts/task-registry` invokes the stale nested `plugins/agent-governance` checkout instead of this repo's canonical Rust CLI. Scope is limited to aligning the wrapper to `rust/task-registry-flow-cli/Cargo.toml`, isolating default cargo target directories by source root so stale artifacts are not reused across manifests, hardening the status check so stale nested wrapper targets fail directly, and covering the plan in version governance.

## Phased Required Change Checklist
### Phase 0: Activation
- [x] `[NEW]` `docs/plans/task-registry-wrapper-source-alignment-2026-06-02.md` - `Task Manifest`: define wrapper-source positive coverage and stale nested path negative coverage.
- [ ] `[VERIFY]` `docs/task-registry.toml` - `PLAN_ACTIVATE`: activate this plan before editing the wrapper and status gate.

### Phase 1: Wrapper source alignment
- [ ] `[MODIFY]` `.codex/scripts/task-registry` - `manifest`: point at `${root}/rust/task-registry-flow-cli/Cargo.toml`.
- [ ] `[MODIFY]` `.codex/scripts/task-registry` - `CARGO_TARGET_DIR`: default to a source-keyed target directory.
- [ ] `[MODIFY]` `scripts/status.sh` - `task_registry` and `Task registry artifacts`: use a source-keyed target directory, require the root CLI manifest path, and reject the stale nested plugin manifest path.
- [ ] `[MODIFY]` `docs/version-roadmap.toml` - `releases[2.1.0].covered_plan_ids`: add `PLAN-2026-06-02-task-registry-wrapper-source-alignment`.

### Phase 2: Verification and handoff
- [ ] `[VERIFY]` `.codex/scripts/task-registry` - `wrapper source guard`: wrapper uses the root Rust CLI manifest and omits `plugins/agent-governance/rust/task-registry-flow-cli/Cargo.toml`.
- [ ] `[VERIFY]` `.codex/scripts/task-registry` - `release-check`: `.codex/scripts/task-registry release-check all --format json` exits zero with the current source.
- [ ] `[VERIFY]` `scripts/status.sh` - `release-source`: `scripts/status.sh --release-source` exits zero on a clean worktree.
- [ ] `[VERIFY]` `docs/version-roadmap.toml` - `version-check validate`: no completed plan is left uncovered.

## Per-Gap Success Criteria
### GAP-001: Repo-Local Task Registry Wrapper Uses Stale Nested Source
- Current failure: the clean release-source status gate calls `.codex/scripts/task-registry`, which points at `plugins/agent-governance/rust/task-registry-flow-cli/Cargo.toml` and can validate stale nested code or stale shared cargo artifacts instead of this repo's canonical Rust CLI.
- Good behavior: Given the repo-local wrapper, when the wrapper is inspected or used for release checks, then it runs `rust/task-registry-flow-cli/Cargo.toml` from the repo root through a source-keyed target dir and validates the current source tree.
- Forbidden behavior: The wrapper references `plugins/agent-governance/rust/task-registry-flow-cli/Cargo.toml`, shares a default target dir across source roots, release-source status accepts a nested wrapper source, or release-check fails because the wrapper sees stale check ids.
- Files involved: `.codex/scripts/task-registry`, `scripts/status.sh`, `docs/version-roadmap.toml`.
- Positive test: `grep -F 'manifest="${root}/rust/task-registry-flow-cli/Cargo.toml"' .codex/scripts/task-registry`.
- Negative test: `! grep -F 'plugins/agent-governance/rust/task-registry-flow-cli/Cargo.toml' .codex/scripts/task-registry`.
- Behavioral test: `.codex/scripts/task-registry release-check all --format json`.
- Release test: `scripts/status.sh --release-source`.
- Domain/API/UI: N/A; this is governance runtime wiring.
- Runtime: The repo-local CLI wrapper now executes current root source for all registry commands.

## Validation Plan
Focused:
- `grep -F 'manifest="${root}/rust/task-registry-flow-cli/Cargo.toml"' .codex/scripts/task-registry`
- `! grep -F 'plugins/agent-governance/rust/task-registry-flow-cli/Cargo.toml' .codex/scripts/task-registry`
- `.codex/scripts/task-registry release-check all --format json`
- `.codex/scripts/task-registry version-check validate --format json`
- `.codex/scripts/task-registry source-limit check`

Full:
- `.codex/scripts/task-registry validate`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `scripts/status.sh --release-source`
- `bash scripts/test-release-readiness.sh all`

## Walkthrough Evidence
- Wrapper source guard exits zero.
- Repo-local `.codex/scripts/task-registry release-check all --format json` exits zero.
- Release-source status exits zero after commit.
- Task report and metrics are captured after landing.

## Source File Limit
The edited files remain below the 1600-line source limit. Run `.codex/scripts/task-registry source-limit check` before completion.

## Task Manifest
```toml
schema_version = 2
plan_id = "PLAN-2026-06-02-task-registry-wrapper-source-alignment"

[[behaviors]]
behavior_id = "B-TR-WRAPPER-001-root-source-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Repo-local task registry wrapper uses the root Rust CLI"
given = "the repo-local .codex task-registry wrapper"
when = "the wrapper source is inspected"
then = "the manifest path targets rust/task-registry-flow-cli/Cargo.toml at the repo root"
confirmation = "grep -F 'manifest=\"${root}/rust/task-registry-flow-cli/Cargo.toml\"' .codex/scripts/task-registry"

[[behaviors.verifiers]]
type = "command"
command = "grep -F 'manifest=\"${root}/rust/task-registry-flow-cli/Cargo.toml\"' .codex/scripts/task-registry"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TR-WRAPPER-002-stale-source-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Nested plugin checkout path is rejected from the wrapper"
given = "the repo-local .codex task-registry wrapper"
when = "the stale nested manifest path is searched"
then = "the path is absent"
confirmation = "! grep -F 'plugins/agent-governance/rust/task-registry-flow-cli/Cargo.toml' .codex/scripts/task-registry"

[[behaviors.verifiers]]
type = "command"
command = "! grep -F 'plugins/agent-governance/rust/task-registry-flow-cli/Cargo.toml' .codex/scripts/task-registry"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TR-WRAPPER-003-release-check-current-source"
gap_id = "GAP-001"
polarity = "positive"
title = "Repo-local wrapper validates current release-source checks"
given = "the wrapper points at current root source"
when = "release-check runs through .codex/scripts/task-registry"
then = "schema-backed release checks pass"
confirmation = ".codex/scripts/task-registry release-check all --format json"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry release-check all --format json"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TR-WRAPPER-004-version-coverage"
gap_id = "GAP-001"
polarity = "validation"
title = "Wrapper source alignment is covered by the release roadmap"
given = "the wrapper source alignment plan exists in the task registry"
when = "version governance validates the current release roadmap"
then = "version checks report zero failures"
confirmation = ".codex/scripts/task-registry version-check validate --format json"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry version-check validate --format json"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-06-02-task-registry-wrapper-source-alignment-001"
status = "planned"
title = "Align repo-local task-registry wrapper to root CLI source"
kind = "governance"
reason = "Release gates must validate the current root CLI source instead of a stale nested plugin checkout."
acceptance_proof = "Behaviors B-TR-WRAPPER-001-root-source-positive, B-TR-WRAPPER-002-stale-source-negative, B-TR-WRAPPER-003-release-check-current-source, and B-TR-WRAPPER-004-version-coverage pass."
behavior_ids = ["B-TR-WRAPPER-001-root-source-positive", "B-TR-WRAPPER-002-stale-source-negative", "B-TR-WRAPPER-003-release-check-current-source", "B-TR-WRAPPER-004-version-coverage"]

[[tasks.targets]]
file = ".codex/scripts/task-registry"
object = "repo-local task-registry wrapper manifest path"
required_change = "Point the wrapper at rust/task-registry-flow-cli/Cargo.toml under the repo root."

[[tasks.targets]]
file = "scripts/status.sh"
object = "Task registry artifacts checks"
required_change = "Require the root CLI manifest path and reject the stale nested plugin manifest path."

[[tasks.targets]]
file = "docs/version-roadmap.toml"
object = "releases[2.1.0].covered_plan_ids"
required_change = "Add PLAN-2026-06-02-task-registry-wrapper-source-alignment."
```
