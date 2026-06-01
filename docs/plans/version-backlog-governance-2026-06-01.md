# Version And Backlog Governance Gap Closure Contract

## Approved Scope

Port the useful certification-gpt versioning and backlog governance patterns into RepOGovnor without automatic final-release publication.

In scope:

- Add executable Governed SemVer checks for RepOGovnor release surfaces, roadmap coverage, prerelease commands, and final tag validation.
- Add executable backlog checks for `docs/gap-pipeline.md`.
- Update public docs, release-source policy, runtime docs, and version surfaces for `2.1.0`.
- Commit the governed bucket and create only the prerelease tag locally if validation passes.

Out of scope:

- Automatic final release push, final GitHub release publication, or final `v2.1.0` tag creation.
- Copying certification-gpt product-specific backlog gaps or transcript/lab behavior.
- Backward compatibility shims.

## Phased Required Change Checklist

### Phase 0: Activation and safety

- [ ] `[NEW]` `docs/plans/version-backlog-governance-2026-06-01.md` - `closure_contract`: activate this scope before implementation edits; proof is `PLAN_ACTIVATE`.
- [ ] `[MODIFY]` `docs/task-registry.toml` - `task_registry`: track this plan and complete through landing; proof is `TASK_REPORT`.

### Phase 1: Version governance

- [ ] `[NEW]` `docs/versioning.md` - `version_governance`: document Governed SemVer, prerelease automation, and manual final release; proof is `version_governance_validate_accepts_current_release`.
- [ ] `[NEW]` `docs/version-roadmap.toml` - `version_roadmap`: declare `2.1.0`, covered plan ids, `v2.1.0-rc.1`, and manual final release policy; proof is `version_governance_validate_accepts_current_release`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/version_check.rs` - `version_check`: add `validate`, `next`, `prerelease`, and `release-check`; proof is `version_governance_`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/tests/version_check_tests.rs` - `version_tests`: prove positive and negative version behavior; proof is `cargo test ... version_governance_`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/schema.rs` - `cli_command`: add `version-check`; proof is `cli_json_envelope_all_commands`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/runtime.rs` - `dispatcher`: route `version-check`; proof is `version_governance_next_and_prerelease_are_deterministic`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/main.rs` and `rust/task-registry-flow-cli/src/tests/mod.rs` - `module_registration`: register version check module and tests; proof is `cargo test ... version_governance_`.

### Phase 2: Backlog governance

- [ ] `[NEW]` `rust/task-registry-flow-cli/src/backlog_check.rs` - `backlog_check`: validate the drainable gap pipeline and forbidden overclaims; proof is `backlog_check_accepts_current_gap_pipeline`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/tests/backlog_check_tests.rs` - `backlog_tests`: prove missing fields and overclaims fail; proof is `cargo test ... backlog_check_`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/schema.rs` - `cli_command`: add `backlog-check`; proof is `cli_json_envelope_all_commands`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/runtime.rs` - `dispatcher`: route `backlog-check`; proof is `backlog_check_json_failure_preserves_report`.
- [ ] `[MODIFY]` `docs/gap-pipeline.md` - `negative_non_claims`: add executable required fields and non-claims; proof is `backlog_check_accepts_current_gap_pipeline`.

### Phase 3: Release surfaces and docs

- [ ] `[MODIFY]` `VERSION` - `release_version`: set `2.1.0`; proof is `version_governance_validate_accepts_current_release`.
- [ ] `[MODIFY]` `plugin.json` - `release_version`: set `2.1.0`; proof is `release-check version`.
- [ ] `[MODIFY]` `.codex-plugin/plugin.json` - `release_version`: set `2.1.0`; proof is `release-check version`.
- [ ] `[MODIFY]` `MANIFEST.toml` - `plugin_version`: set `2.1.0`; proof is `release-check version`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/Cargo.toml` - `package.version`: set `2.1.0`; proof is `release-check version`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/Cargo.lock` - `package.version`: set `2.1.0`; proof is `release-check version`.
- [ ] `[MODIFY]` `package.nix` - `package.version`: set `2.1.0`; proof is `release-check version`.
- [ ] `[MODIFY]` `README.md`, `docs/releases/v2.md`, `docs/runtime-schemas.md`, `CHANGELOG.md`, `AGENTS.md`, `GEMINI.md`, `CLAUDE.md`, `templates/AGENTS.md.template`, `templates/GEMINI.md.template`, `templates/CLAUDE.md.template` - `docs`: describe version/backlog commands and prerelease-only automation; proof is `version_governance_validate_accepts_current_release`.
- [ ] `[MODIFY]` `REQUIREMENTS.toml` - `release_source`: add new Rust/doc files and version surfaces; proof is `release-check all`.

### Phase 4: Validation and handoff

- [ ] `[VERIFY]` `rust/task-registry-flow-cli/Cargo.toml` - `focused_tests`: run version and backlog tests; proof is focused validation.
- [ ] `[VERIFY]` `.codex/scripts/task-registry` - `full_gates`: run source-limit, validate, verify-chain, full cargo tests, clippy, and release readiness; proof is full validation.
- [ ] `[VERIFY]` `docs/task-registry.toml` - `landing`: run `verify-landing`, report, metrics, and archive if needed; proof is no active, blocked, or deferred work.

## Per-Gap Success Criteria

### GAP-001: Version authority misses post-tag governed work

- Current failure: `release-check version` passes at `2.0.2` even when `HEAD` is ahead of `v2.0.2` with completed governed work.
- Good behavior: Given completed post-cutover plans, when `version-check validate` runs, then every completed plan is covered by a roadmap release entry.
- Forbidden behavior: A completed post-cutover plan is omitted from release coverage while validation passes.
- Files involved: `rust/task-registry-flow-cli/src/version_check.rs`, `docs/version-roadmap.toml`, `docs/versioning.md`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml version_governance_validate_accepts_current_release -- --nocapture`
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml version_governance_rejects_uncovered_completed_plan -- --nocapture`
- Domain/API/UI: CLI surface adds `version-check`; no hosted release publication.
- Runtime: Final release remains manual; prerelease output may include branch and `vX.Y.Z-rc.N` push commands only.

### GAP-002: Backlog pipeline is prose-guarded instead of executable

- Current failure: `docs/gap-pipeline.md` is useful but not checked for required fields, reactivation conditions, or forbidden product-proof overclaims.
- Good behavior: Given the current backlog document, when `backlog-check` runs, then required fields and negative non-claims are present.
- Forbidden behavior: Missing reactivation conditions, no negative non-claims, or claims of product correctness/fleet governance pass validation.
- Files involved: `rust/task-registry-flow-cli/src/backlog_check.rs`, `docs/gap-pipeline.md`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml backlog_check_accepts_current_gap_pipeline -- --nocapture`
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml backlog_check_rejects_missing_fields_and_overclaims -- --nocapture`
- Domain/API/UI: CLI surface adds `backlog-check`; no public service.
- Runtime: Backlog proof is local source validation only.

## Validation Plan

Focused:

- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml version_governance_ -- --nocapture`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml backlog_check_ -- --nocapture`
- `.codex/scripts/task-registry verify-behaviors PLAN-2026-06-01-version-backlog-governance`
- `.codex/scripts/task-registry source-limit check`

Full:

- `cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings`
- `.codex/scripts/task-registry validate`
- `.codex/scripts/task-registry verify-chain --format json`
- `bash scripts/test-release-readiness.sh all`

## Source File Limit

Expected impact: new Rust modules and tests avoid growing `runtime.rs` and `tests/mod.rs` near the 1600-line limit. Run `.codex/scripts/task-registry source-limit check` before landing.

## Walkthrough Evidence

- Focused positive and negative tests pass.
- `version-check validate` passes on `2.1.0`.
- `version-check prerelease PLAN-2026-06-01-version-backlog-governance --rc 1` prints only prerelease push commands.
- `backlog-check --format json` passes.
- Full validation and release-readiness gates pass.
- Final report shows no blocked, deferred, planned, or active tasks for this plan.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-06-01-version-backlog-governance"

[[behaviors]]
behavior_id = "VBG-B-001-version-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Version governance validates current release coverage"
given = "RepOGovnor release surfaces and roadmap are aligned at 2.1.0"
when = "version-check validate runs"
then = "version surfaces, roadmap, semver, and completed plan coverage pass"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml version_governance_validate_accepts_current_release -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml version_governance_validate_accepts_current_release -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "VBG-B-002-version-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Version governance rejects drift and final auto-push"
given = "A fixture has stale surfaces, uncovered completed plans, or final push output"
when = "version governance checks run"
then = "the forbidden state fails closed"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml version_governance_rejects_ -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml version_governance_rejects_ -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "VBG-B-003-backlog-positive"
gap_id = "GAP-002"
polarity = "positive"
title = "Backlog check validates current gap pipeline"
given = "docs/gap-pipeline.md contains required fields and negative non-claims"
when = "backlog-check runs"
then = "the backlog report passes"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml backlog_check_accepts_current_gap_pipeline -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml backlog_check_accepts_current_gap_pipeline -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "VBG-B-004-backlog-negative"
gap_id = "GAP-002"
polarity = "negative"
title = "Backlog check rejects missing fields and overclaims"
given = "A fixture backlog omits required fields or claims unsupported proof"
when = "backlog-check runs"
then = "the backlog report fails"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml backlog_check_rejects_ -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml backlog_check_rejects_ -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "VBG-B-005-full-validation"
gap_id = "GAP-VALIDATION"
polarity = "validation"
title = "Version and backlog governance pass full release gates"
given = "The closure is implemented"
when = "Full validation runs"
then = "format, tests, clippy, registry, chain, and release readiness pass"
confirmation = "cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check && cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml && cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings && .codex/scripts/task-registry validate && .codex/scripts/task-registry verify-chain --format json && bash scripts/test-release-readiness.sh all"

[[behaviors.verifiers]]
type = "command"
command = "cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check && cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml && cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings && .codex/scripts/task-registry validate && .codex/scripts/task-registry verify-chain --format json && bash scripts/test-release-readiness.sh all"
expected_exit = 0

[[behaviors]]
behavior_id = "VBG-B-006-plan-activation-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Plan activation remains registry-valid"
given = "The closure contract has been activated"
when = "task registry validation runs"
then = "the active plan and task manifest remain valid"
confirmation = ".codex/scripts/task-registry validate"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry validate"
expected_exit = 0

[[behaviors]]
behavior_id = "VBG-B-007-release-surface-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Release surfaces carry version and backlog governance"
given = "The public release and agent surfaces have been updated"
when = "release-source and version checks run"
then = "release-source files and version governance pass"
confirmation = ".codex/scripts/task-registry release-check all --format json && .codex/scripts/task-registry version-check validate"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry release-check all --format json && .codex/scripts/task-registry version-check validate"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-06-01-VBG-001"
status = "planned"
title = "Activate version and backlog governance plan"
kind = "governance"
reason = "The approved scope needs task-bound provenance before implementation edits."
acceptance_proof = "Behavior VBG-B-006-plan-activation-positive."
behavior_ids = ["VBG-B-006-plan-activation-positive"]

[[tasks.targets]]
file = "docs/plans/version-backlog-governance-2026-06-01.md"
object = "closure_contract"
required_change = "Capture approved scope, behaviors, validation, and task targets."

[[tasks.targets]]
file = "docs/task-registry.toml"
object = "task_registry"
required_change = "Activate and complete this plan through registry CLI."

[[tasks]]
task_id = "TASK-2026-06-01-VBG-002"
status = "planned"
title = "Implement version governance"
kind = "implementation"
reason = "Release authority must catch post-tag governed work and forbid automatic final release."
acceptance_proof = "Behaviors VBG-B-001-version-positive and VBG-B-002-version-negative."
behavior_ids = ["VBG-B-001-version-positive", "VBG-B-002-version-negative"]

[[tasks.targets]]
file = "docs/versioning.md"
object = "version_governance"
required_change = "Document Governed SemVer, prerelease automation, and manual final release."

[[tasks.targets]]
file = "docs/version-roadmap.toml"
object = "version_roadmap"
required_change = "Declare 2.1.0, covered plans, rc tag, final tag, and manual final release policy."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/version_check.rs"
object = "version_check"
required_change = "Implement validate, next, prerelease, and release-check."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/version_check_tests.rs"
object = "version_check_tests"
required_change = "Add positive and negative version governance tests."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/schema.rs"
object = "cli_command"
required_change = "Add version-check and backlog-check command variants."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/runtime.rs"
object = "dispatcher"
required_change = "Route version-check and backlog-check commands."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/main.rs"
object = "module_registration"
required_change = "Register version_check and backlog_check modules."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/mod.rs"
object = "test_module_registration"
required_change = "Register version_check_tests and backlog_check_tests modules."

[[tasks]]
task_id = "TASK-2026-06-01-VBG-003"
status = "planned"
title = "Implement backlog governance"
kind = "implementation"
reason = "The drainable gap pipeline needs executable required-field and overclaim checks."
acceptance_proof = "Behaviors VBG-B-003-backlog-positive and VBG-B-004-backlog-negative."
behavior_ids = ["VBG-B-003-backlog-positive", "VBG-B-004-backlog-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/backlog_check.rs"
object = "backlog_check"
required_change = "Validate gap pipeline required fields, reactivation conditions, and negative non-claims."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/backlog_check_tests.rs"
object = "backlog_check_tests"
required_change = "Add positive and negative backlog governance tests."

[[tasks.targets]]
file = "docs/gap-pipeline.md"
object = "negative_non_claims"
required_change = "Add executable required fields and negative non-claims."

[[tasks]]
task_id = "TASK-2026-06-01-VBG-004"
status = "planned"
title = "Update release and agent surfaces"
kind = "release"
reason = "New governance commands and release coverage must be public and package-tracked."
acceptance_proof = "Behavior VBG-B-007-release-surface-positive."
behavior_ids = ["VBG-B-007-release-surface-positive"]

[[tasks.targets]]
file = "VERSION"
object = "release_version"
required_change = "Set release version to 2.1.0."

[[tasks.targets]]
file = "plugin.json"
object = "release_version"
required_change = "Set release version to 2.1.0."

[[tasks.targets]]
file = ".codex-plugin/plugin.json"
object = "release_version"
required_change = "Set release version to 2.1.0."

[[tasks.targets]]
file = "MANIFEST.toml"
object = "plugin_version"
required_change = "Set plugin version and package new files."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/Cargo.toml"
object = "package_version"
required_change = "Set package version to 2.1.0."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/Cargo.lock"
object = "package_version"
required_change = "Set package version to 2.1.0."

[[tasks.targets]]
file = "package.nix"
object = "package_version"
required_change = "Set package version and package new docs."

[[tasks.targets]]
file = "README.md"
object = "version_backlog_docs"
required_change = "Document version-check, backlog-check, and prerelease-only automation."

[[tasks.targets]]
file = "docs/releases/v2.md"
object = "release_docs"
required_change = "Set release version and describe 2.1.0 governance commands."

[[tasks.targets]]
file = "docs/runtime-schemas.md"
object = "runtime_docs"
required_change = "Document version-check and backlog-check outputs."

[[tasks.targets]]
file = "CHANGELOG.md"
object = "release_notes"
required_change = "Add 2.1.0 release notes."

[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release_source"
required_change = "Track new docs, Rust modules, and tests as release-source files."

[[tasks.targets]]
file = "AGENTS.md"
object = "agent_version_policy"
required_change = "Add prerelease-only automation and manual final release policy."

[[tasks.targets]]
file = "GEMINI.md"
object = "agent_version_policy"
required_change = "Mirror prerelease-only automation and manual final release policy."

[[tasks.targets]]
file = "CLAUDE.md"
object = "agent_version_policy"
required_change = "Mirror prerelease-only automation and manual final release policy."

[[tasks.targets]]
file = "templates/AGENTS.md.template"
object = "agent_version_policy"
required_change = "Template prerelease-only automation and manual final release policy."

[[tasks.targets]]
file = "templates/GEMINI.md.template"
object = "agent_version_policy"
required_change = "Template prerelease-only automation and manual final release policy."

[[tasks.targets]]
file = "templates/CLAUDE.md.template"
object = "agent_version_policy"
required_change = "Template prerelease-only automation and manual final release policy."

[[tasks]]
task_id = "TASK-2026-06-01-VBG-005"
status = "planned"
title = "Validate and land version and backlog governance"
kind = "validation"
reason = "This shared governance change must pass behavior, registry, chain, and release-readiness gates."
acceptance_proof = "Behavior VBG-B-005-full-validation and final TASK_REPORT."
behavior_ids = ["VBG-B-005-full-validation"]

[[tasks.targets]]
file = "docs/task-registry.toml"
object = "landing_report"
required_change = "Complete tasks through verify-landing, report, metrics, and archive if needed."
```
