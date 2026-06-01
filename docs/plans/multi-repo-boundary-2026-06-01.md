# Multi-Repo Boundary Gap Closure Contract

## Approved Scope

Close GP-004 by documenting the current multi-repo consumption boundary:
RepOGovnor is repo-local governance, not a fleet status service. In scope:
add a packaged `docs/multi-repo.md`, link it from README, include it in
release-source/package surfaces, add release-readiness assertions, and update
the gap pipeline evidence.

Out of scope: hosted dashboards, cross-repo telemetry, central aggregation,
automatic fleet policy sync, compatibility shims, or remote state.

## Phased Required Change Checklist

### Phase 0: Activation and safety
- [ ] `[NEW]` `docs/plans/multi-repo-boundary-2026-06-01.md` - `closure_contract`: declare GP-004 scope, behaviors, validation, and task targets.
- [ ] `[VERIFY]` `.codex/scripts/task-registry activate docs/plans/multi-repo-boundary-2026-06-01.md` - `PLAN_ACTIVATE`: activate exact task targets before implementation edits.

### Phase 1: Public boundary doc
- [ ] `[NEW]` `docs/multi-repo.md` - `multi_repo_boundary`: document one-install-per-repo, pinned plugin version, per-repo validation, no aggregation, and reactivation triggers.
- [ ] `[MODIFY]` `README.md` - `files_users_should_know`: link the multi-repo boundary doc.

### Phase 2: Release/package surfaces
- [ ] `[MODIFY]` `REQUIREMENTS.toml` - `release_required_files`: include `docs/multi-repo.md`.
- [ ] `[MODIFY]` `package.nix` - `package_docs`: install `docs/multi-repo.md` under `share/agent-governance/docs`.
- [ ] `[MODIFY]` `scripts/test-release-readiness.sh` - `multi_repo_doc_checks`: assert doc content and package asset presence.

### Phase 3: Gap pipeline
- [ ] `[MODIFY]` `docs/gap-pipeline.md` - `GP-004`: update with current manual-boundary evidence and future reactivation conditions.
- [ ] `[VERIFY]` `.codex/scripts/task-registry verify-landing --plan-id PLAN-2026-06-01-multi-repo-boundary --changed-files docs/multi-repo.md README.md REQUIREMENTS.toml package.nix scripts/test-release-readiness.sh docs/gap-pipeline.md` - `TASK_VERIFY_LANDING`: land through behavior verification.

## Per-Gap Success Criteria

### GP-004 Multi-Repo Governance Boundary
- Current failure: README says the plugin is portable and local-first, but it does not clearly say how teams with several repos should consume it or what remains manual.
- Good behavior: Public docs state one install per repo, pin the plugin version per repo, run per-repo validation, compare reviewer reports manually, and avoid claiming central aggregation.
- Forbidden behavior: Docs must not claim hosted fleet status, remote sync, cross-repo telemetry, or automatic multi-repo policy aggregation.
- Files involved: `docs/multi-repo.md`, `README.md`, `REQUIREMENTS.toml`, `package.nix`, `scripts/test-release-readiness.sh`, `docs/gap-pipeline.md`.
- Positive test: `bash -c 'rg -n "one install per repo|No fleet aggregator|reviewer-report" docs/multi-repo.md README.md && rg -n "docs/multi-repo.md" REQUIREMENTS.toml package.nix scripts/test-release-readiness.sh'`
- Negative test: `bash -c '! rg -n "fleet dashboard included|central aggregator included|automatic policy aggregation included" docs/multi-repo.md README.md'`
- Data/schema/provenance: Release-source and Nix package surfaces include the new doc.
- Runtime: `bash scripts/test-release-readiness.sh all` proves packaged doc presence.

## Validation Plan

Focused:
- `bash -c 'rg -n "one install per repo|No fleet aggregator|reviewer-report" docs/multi-repo.md README.md && rg -n "docs/multi-repo.md" REQUIREMENTS.toml package.nix scripts/test-release-readiness.sh'`
- `bash -c '! rg -n "fleet dashboard|remote sync|telemetry|central aggregator|automatic policy aggregation" docs/multi-repo.md README.md'`
- `.codex/scripts/task-registry source-limit check`

Full:
- `.codex/scripts/task-registry validate`
- `bash scripts/test-release-readiness.sh all`

## Source File Limit

Expected line impact is small and remains below the 1600-line cap. Validate with
`.codex/scripts/task-registry source-limit check` before landing.

## Walkthrough Evidence

- Positive and negative doc-boundary checks pass.
- Release readiness proves required file and packaged doc presence.
- `TASK_VERIFY_LANDING` completes the task.
- Registry report, metrics, validation, source-limit check, and receipt-chain verification pass.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-06-01-multi-repo-boundary"

[[behaviors]]
behavior_id = "B-2026-06-01-multi-repo-positive"
gap_id = "GP-004"
polarity = "positive"
title = "Multi-repo manual boundary is documented and packaged"
given = "The public docs and release/package surfaces"
when = "The boundary check runs"
then = "They document one install per repo, no fleet aggregator, reviewer-report usage, and packaged doc presence"
confirmation = "bash -c 'rg -n \"one install per repo|No fleet aggregator|reviewer-report\" docs/multi-repo.md README.md && rg -n \"docs/multi-repo.md\" REQUIREMENTS.toml package.nix scripts/test-release-readiness.sh'"

[[behaviors.verifiers]]
type = "command"
command = "bash -c 'rg -n \"one install per repo|No fleet aggregator|reviewer-report\" docs/multi-repo.md README.md && rg -n \"docs/multi-repo.md\" REQUIREMENTS.toml package.nix scripts/test-release-readiness.sh'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-06-01-multi-repo-negative"
gap_id = "GP-004"
polarity = "negative"
title = "Multi-repo docs do not claim unsupported aggregation"
given = "The public docs"
when = "Unsupported fleet claims are searched"
then = "No hosted fleet dashboard, remote sync, telemetry, central aggregator, or automatic aggregation claim appears"
confirmation = "bash -c '! rg -n \"fleet dashboard included|central aggregator included|automatic policy aggregation included\" docs/multi-repo.md README.md'"

[[behaviors.verifiers]]
type = "command"
command = "bash -c '! rg -n \"fleet dashboard included|central aggregator included|automatic policy aggregation included\" docs/multi-repo.md README.md'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-06-01-multi-repo-release-validation"
gap_id = "GP-004"
polarity = "validation"
title = "Release readiness packages the multi-repo doc"
given = "The release readiness suite"
when = "The full release gate runs"
then = "The multi-repo doc is required and installed as a Nix package asset"
confirmation = "bash scripts/test-release-readiness.sh all"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-release-readiness.sh all"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-06-01-multi-repo-boundary-001"
status = "planned"
title = "Document multi-repo governance boundary"
kind = "documentation"
reason = "GP-004 needs an honest public boundary for teams using several repos."
acceptance_proof = "Behaviors B-2026-06-01-multi-repo-positive and B-2026-06-01-multi-repo-negative pass."
behavior_ids = [
  "B-2026-06-01-multi-repo-positive",
  "B-2026-06-01-multi-repo-negative",
  "B-2026-06-01-multi-repo-release-validation",
]

[[tasks.targets]]
file = "docs/multi-repo.md"
object = "multi_repo_boundary"
required_change = "Document current manual multi-repo consumption model and unsupported aggregation claims."

[[tasks.targets]]
file = "README.md"
object = "files_users_should_know"
required_change = "Link the multi-repo boundary doc."

[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release_required_files"
required_change = "Declare docs/multi-repo.md as a required release file."

[[tasks.targets]]
file = "package.nix"
object = "package_docs"
required_change = "Install docs/multi-repo.md in the package asset root."

[[tasks.targets]]
file = "scripts/test-release-readiness.sh"
object = "multi_repo_doc_checks"
required_change = "Assert multi-repo doc content and package asset presence."

[[tasks.targets]]
file = "docs/gap-pipeline.md"
object = "GP-004"
required_change = "Update GP-004 with manual-boundary evidence and remaining reactivation conditions."
```
