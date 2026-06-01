# Reviewer Markdown Format Gap Closure Contract

## Approved Scope

Close the local PR-oriented part of GP-002 by adding a Markdown output mode to
`reviewer-report`. In scope: `reviewer-report --format markdown`, tests for
Markdown output and unsupported formats, CLI usage/docs updates, and gap
pipeline evidence.

Out of scope: GitHub API calls, hosted review services, telemetry, remote PR
posting, or changing the default text output.

## Phased Required Change Checklist

### Phase 0: Activation and safety
- [ ] `[NEW]` `docs/plans/reviewer-markdown-format-2026-06-01.md` - `closure_contract`: declare GP-002 scope, behaviors, validation, and task targets.
- [ ] `[VERIFY]` `.codex/scripts/task-registry activate docs/plans/reviewer-markdown-format-2026-06-01.md` - `PLAN_ACTIVATE`: activate exact task targets before implementation edits.

### Phase 1: Reviewer output
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/reviewer_report.rs` - `markdown_format`: accept `--format markdown`, render PR-oriented Markdown, reject unknown formats, and preserve default text output.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/runtime.rs` - `usage`: advertise the optional Markdown format.

### Phase 2: Behavior tests
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/reviewer_report_tests.rs` - `markdown_format_tests`: assert Markdown output contains summary/proof boundary and unknown formats fail closed.

### Phase 3: Docs and pipeline
- [ ] `[MODIFY]` `README.md` - `daily_workflow`: document local Markdown handoff usage.
- [ ] `[MODIFY]` `docs/runtime-schemas.md` - `Reviewer Report`: document text and Markdown modes.
- [ ] `[MODIFY]` `docs/gap-pipeline.md` - `GP-002`: update with local PR-format evidence and future remote-integration reactivation.
- [ ] `[VERIFY]` `.codex/scripts/task-registry verify-landing --plan-id PLAN-2026-06-01-reviewer-markdown-format --changed-files rust/task-registry-flow-cli/src/reviewer_report.rs rust/task-registry-flow-cli/src/runtime.rs rust/task-registry-flow-cli/src/tests/reviewer_report_tests.rs README.md docs/runtime-schemas.md docs/gap-pipeline.md` - `TASK_VERIFY_LANDING`: land through behavior verification.

## Per-Gap Success Criteria

### GP-002 Local PR-Oriented Reviewer Handoff
- Current failure: `reviewer-report` is pasteable text but lacks an explicit PR-oriented format; the gap pipeline still points to optional PR formatting or integration.
- Good behavior: Given `reviewer-report --format markdown`, output is Markdown with a summary list, proof boundary section, active targets, landed files, and blocked/deferred work suitable for manual PR paste.
- Forbidden behavior: Unknown formats must fail closed; the command must not post to GitHub, call a network API, or imply remote telemetry.
- Files involved: `rust/task-registry-flow-cli/src/reviewer_report.rs`, `rust/task-registry-flow-cli/src/runtime.rs`, `rust/task-registry-flow-cli/src/tests/reviewer_report_tests.rs`, `README.md`, `docs/runtime-schemas.md`, `docs/gap-pipeline.md`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml reviewer_report_markdown_formats_pr_handoff`
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml reviewer_report_rejects_unknown_format`
- Data/schema/provenance: Output remains local stdout only and derives from the local registry/metrics surfaces.
- Runtime: Default `reviewer-report` text output remains accepted.

## Validation Plan

Focused:
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml reviewer_report`
- `.codex/scripts/task-registry source-limit check`

Full:
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `.codex/scripts/task-registry validate`
- `bash scripts/test-release-readiness.sh all`

## Source File Limit

Expected line impact is small and remains below the 1600-line cap. Validate with
`.codex/scripts/task-registry source-limit check` before landing.

## Walkthrough Evidence

- Focused reviewer-report tests pass.
- `reviewer-report --format markdown` works from the live repo.
- `TASK_VERIFY_LANDING` completes the task.
- Registry report, metrics, validation, source-limit check, and receipt-chain verification pass.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-06-01-reviewer-markdown-format"

[[behaviors]]
behavior_id = "B-2026-06-01-reviewer-markdown-positive"
gap_id = "GP-002"
polarity = "positive"
title = "Reviewer report renders Markdown handoff"
given = "A local registry with landed, active, blocked, or deferred task evidence"
when = "reviewer-report --format markdown runs"
then = "The report emits Markdown sections for summary, proof boundary, active targets, landed files, and blocked/deferred work"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml reviewer_report_markdown_formats_pr_handoff"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml reviewer_report_markdown_formats_pr_handoff"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-06-01-reviewer-markdown-negative"
gap_id = "GP-002"
polarity = "negative"
title = "Reviewer report rejects unsupported formats"
given = "A reviewer-report invocation with an unsupported format"
when = "the command runs"
then = "The command fails closed with usage instead of silently selecting another format"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml reviewer_report_rejects_unknown_format"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml reviewer_report_rejects_unknown_format"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-06-01-reviewer-markdown-format-001"
status = "planned"
title = "Add local Markdown reviewer handoff"
kind = "implementation"
reason = "GP-002 needs PR-oriented local reviewer output without remote integration."
acceptance_proof = "Behaviors B-2026-06-01-reviewer-markdown-positive and B-2026-06-01-reviewer-markdown-negative pass."
behavior_ids = [
  "B-2026-06-01-reviewer-markdown-positive",
  "B-2026-06-01-reviewer-markdown-negative",
]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/reviewer_report.rs"
object = "markdown_format"
required_change = "Accept --format markdown, render local Markdown handoff, reject unknown formats, preserve default text."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/runtime.rs"
object = "usage"
required_change = "Advertise reviewer-report optional Markdown format."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/reviewer_report_tests.rs"
object = "markdown_format_tests"
required_change = "Assert Markdown output and unknown format rejection."

[[tasks.targets]]
file = "README.md"
object = "daily_workflow"
required_change = "Document local Markdown reviewer handoff usage."

[[tasks.targets]]
file = "docs/runtime-schemas.md"
object = "Reviewer Report"
required_change = "Document text and Markdown output modes."

[[tasks.targets]]
file = "docs/gap-pipeline.md"
object = "GP-002"
required_change = "Update GP-002 with local PR-format evidence and future remote integration boundary."
```
