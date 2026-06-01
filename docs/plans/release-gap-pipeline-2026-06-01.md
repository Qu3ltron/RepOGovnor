# Release Gap Pipeline Closure Contract

## Approved Scope
This closure establishes the next-version gap pipeline and closes the release-claim drift found during the current audit.

In scope:
- Add release-version coverage for Markdown release claim surfaces.
- Update stale v2 release documentation from `2.0.1` to the current `2.0.2` release.
- Add an honest gap pipeline document that separates verified current implementation from known remaining gaps.

Out of scope:
- Publishing a new tag or GitHub release.
- Closing every documented remaining gap in one run.
- Adding compatibility shims for old v1 or v0.x layouts.

## Phased Required Change Checklist
### Phase 0: Activation and safety
- [ ] `[NEW]` `docs/plans/release-gap-pipeline-2026-06-01.md` - `Task Manifest`: activate this contract before implementation edits.
- [ ] `[VERIFY]` `docs/task-registry.toml` - `registry`: activation and landing keep the registry valid.

### Phase 1: Release claim drift closure
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/schema.rs` - `VersionFileFormat`: add a typed Markdown-line version-file format.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/release_checks.rs` - `extract_version`: extract versions from Markdown claim lines declared in `REQUIREMENTS.toml`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/release_source_tests.rs` - `release_source_rejects_stale_markdown_version_file`: prove stale Markdown release claims fail release-version checks.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/mod.rs` - `seed_release_repo`: seed Markdown version fixtures.
- [ ] `[MODIFY]` `REQUIREMENTS.toml` - `release_source.version_files`: declare `README.md` and `docs/releases/v2.md` as version-bearing release surfaces.
- [ ] `[MODIFY]` `docs/releases/v2.md` - `Release version`: state `2.0.2`.

### Phase 2: Gap pipeline artifact
- [ ] `[NEW]` `docs/gap-pipeline.md` - `gap pipeline`: document verified claims, remaining gaps, drain order, and activation protocol.

### Phase 3: Verification and handoff
- [ ] `[VERIFY]` `scripts/release-version-check.sh` - `version consistency`: passes with Markdown release surfaces included.
- [ ] `[VERIFY]` `.codex/scripts/task-registry source-limit check` - `line budget`: passes.
- [ ] `[VERIFY]` `.codex/scripts/task-registry validate` - `registry`: passes.

## Per-Gap Success Criteria
### GAP-001: Markdown release claims are outside the version gate
- Current failure: `docs/releases/v2.md` still says `Release version: 2.0.1` while canonical release files say `2.0.2`, and `scripts/release-version-check.sh` does not inspect that Markdown claim.
- Good behavior: Markdown release claim lines declared in `REQUIREMENTS.toml` must match `[release_source].version`.
- Forbidden behavior: release-version checks pass while `README.md` or `docs/releases/v2.md` carries a stale current-release value.
- Files involved: `rust/task-registry-flow-cli/src/schema.rs`, `rust/task-registry-flow-cli/src/release_checks.rs`, `rust/task-registry-flow-cli/src/tests/release_source_tests.rs`, `rust/task-registry-flow-cli/src/tests/mod.rs`, `REQUIREMENTS.toml`, `docs/releases/v2.md`.
- Positive test: `bash scripts/release-version-check.sh`
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_source_tests::release_source_rejects_stale_markdown_version_file -- --nocapture`
- Domain/API/UI: release-source diagnostics gain a typed version-file format; no public CLI command signature changes.
- Runtime: stale Markdown release claims fail release checks.

### GAP-002: Remaining gaps are spread across roadmap prose instead of a drainable pipeline
- Current failure: current and planned gaps exist in `ROADMAP.md`, `VISION.md`, and release docs, but there is no single document that separates verified implementation from remaining gaps and gives a drain protocol.
- Good behavior: `docs/gap-pipeline.md` lists verified current claims, remaining gaps, evidence, priority, and reactivation conditions for future closure contracts.
- Forbidden behavior: the document claims the system is fully complete or that governance checks prove product correctness.
- Files involved: `docs/gap-pipeline.md`.
- Positive test: `rg -n "GP-001|GP-002|GP-003|GP-004|Drain protocol" docs/gap-pipeline.md`
- Negative test: typed `not_contains` verifiers reject "no remaining gaps" and "governance checks prove product correctness".
- Domain/API/UI: documentation only.
- Runtime: N/A; the artifact guides future governed closures.

## Validation Plan
Focused:
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_source_tests::release_source_rejects_stale_markdown_version_file -- --nocapture`
- `bash scripts/release-version-check.sh`
- `.codex/scripts/task-registry source-limit check`
- `.codex/scripts/task-registry validate`

Full:
- `cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `bash scripts/test-release-readiness.sh all`

## Source File Limit
Expected impact is small. New and modified files must remain under 1600 lines. Run `.codex/scripts/task-registry source-limit check` before landing.

## Walkthrough Evidence
- Contract activation output.
- Focused negative test output for stale Markdown version claims.
- Release version check output showing `README.md` and `docs/releases/v2.md`.
- Source-limit and registry validation output.
- Task report and metrics output.

## Task Manifest
```toml
schema_version = 2
plan_id = "PLAN-2026-06-01-release-gap-pipeline"

[[behaviors]]
behavior_id = "B-001-markdown-version-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Markdown release claims match the release version"
given = "Release claim files are declared as version-bearing surfaces"
when = "the release version checker runs"
then = "README.md and docs/releases/v2.md report the canonical release version"
confirmation = "bash scripts/release-version-check.sh"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/release-version-check.sh"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-markdown-version-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Stale Markdown release claims fail the release gate"
given = "A release fixture has a stale Markdown release claim"
when = "release version checks run"
then = "the stale Markdown claim is reported as a release-version failure"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_source_tests::release_source_rejects_stale_markdown_version_file -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_source_tests::release_source_rejects_stale_markdown_version_file -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-003-gap-pipeline-positive"
gap_id = "GAP-002"
polarity = "positive"
title = "Gap pipeline lists remaining drainable gaps"
given = "The next-version audit needs a durable pipeline"
when = "the gap pipeline document is inspected"
then = "it lists gap ids, evidence, priorities, and a drain protocol"
confirmation = "rg -n \"GP-001|GP-002|GP-003|GP-004|Drain protocol\" docs/gap-pipeline.md"

[[behaviors.verifiers]]
type = "command"
command = "rg -n \"GP-001|GP-002|GP-003|GP-004|Drain protocol\" docs/gap-pipeline.md"
expected_exit = 0

[[behaviors]]
behavior_id = "B-004-gap-pipeline-negative"
gap_id = "GAP-002"
polarity = "negative"
title = "Gap pipeline avoids false completion claims"
given = "The gap pipeline document is public-facing project evidence"
when = "the document is checked for overclaiming"
then = "it does not claim there are no remaining gaps or that governance proves product correctness"
confirmation = "typed not_contains verifiers for forbidden completion claims"

[[behaviors.verifiers]]
type = "not_contains"
path = "docs/gap-pipeline.md"
needle = "no remaining gaps"

[[behaviors.verifiers]]
type = "not_contains"
path = "docs/gap-pipeline.md"
needle = "governance checks prove product correctness"

[[tasks]]
task_id = "TASK-2026-06-01-release-gap-pipeline-001"
behavior_ids = [
  "B-001-markdown-version-positive",
  "B-002-markdown-version-negative",
]
status = "planned"
title = "Gate Markdown release claim versions"
kind = "implementation"
reason = "Release readiness must catch stale public release claims, not only machine metadata files."
acceptance_proof = "Behaviors B-001-markdown-version-positive and B-002-markdown-version-negative."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/schema.rs"
object = "VersionFileFormat"
required_change = "Add a typed Markdown-line release version format."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/release_checks.rs"
object = "extract_version"
required_change = "Extract release versions from configured Markdown claim lines."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/release_source_tests.rs"
object = "release_source_rejects_stale_markdown_version_file"
required_change = "Add a negative fixture test for stale Markdown release claims."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/mod.rs"
object = "seed_release_repo"
required_change = "Seed Markdown release version fixtures."

[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release_source.version_files"
required_change = "Declare README.md and docs/releases/v2.md as version-bearing release surfaces."

[[tasks.targets]]
file = "docs/releases/v2.md"
object = "Release version"
required_change = "Set the documented release version to 2.0.2."

[[tasks]]
task_id = "TASK-2026-06-01-release-gap-pipeline-002"
behavior_ids = [
  "B-003-gap-pipeline-positive",
  "B-004-gap-pipeline-negative",
]
status = "planned"
title = "Document the remaining gap pipeline"
kind = "documentation"
reason = "The project needs a durable, honest backlog that can be drained through future governed closure contracts."
acceptance_proof = "Behaviors B-003-gap-pipeline-positive and B-004-gap-pipeline-negative."

[[tasks.targets]]
file = "docs/gap-pipeline.md"
object = "gap_pipeline"
required_change = "Document verified claims, remaining gaps, evidence, priorities, and drain protocol."
```
