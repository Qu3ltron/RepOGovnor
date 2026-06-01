# CI Nix Release Readiness Gap Closure Contract

## Approved Scope

Close the release blocker where GitHub CI runs `scripts/test-release-readiness.sh`
without Nix installed, then fails before the Nix package gate can prove release
artifacts.

In scope:

- Install Nix in `.github/workflows/ci.yml` before release readiness runs.
- Keep the release readiness script strict; do not skip or soften Nix package
  validation when Nix is missing.
- Capture local release and registry validation evidence.

Out of scope:

- Moving or rewriting the existing `v2` tag.
- Dropping the stale stash before release is verified green.
- Adding fallback compatibility paths for non-Nix release validation.

## Phased Required Change Checklist

### Phase 0: Activation

- [ ] `[NEW]` `docs/plans/ci-nix-release-readiness-2026-05-31.md` - `closure contract`: define CI Nix release gate scope, success criteria, validation, and manifest; acceptance proof is `PLAN_ACTIVATE docs/plans/ci-nix-release-readiness-2026-05-31.md`.

### Phase 1: Workflow implementation

- [ ] `[MODIFY]` `.github/workflows/ci.yml` - `Release readiness behavior prerequisites`: install Nix before `bash scripts/test-release-readiness.sh`; acceptance proof is behaviors `B-2026-05-31-ci-nix-G01-positive` and `B-2026-05-31-ci-nix-G01-negative`.

### Phase 2: Validation and handoff

- [ ] `[VERIFY]` `.github/workflows/ci.yml` - `release gate validation`: run the workflow-order verifier, local Nix package gate, release readiness, final release-source status, source-limit, registry validation, and receipt-chain verification; acceptance proof is behavior `B-2026-05-31-ci-nix-G02-validation`.

## Per-Gap Success Criteria

### GAP-001: CI release readiness runs without Nix installed

- Current failure: GitHub CI reaches `Release readiness behavior`, invokes
  `scripts/test-release-readiness.sh`, and fails at `nix build` with
  `nix: command not found`.
- Good behavior: Given a fresh GitHub Ubuntu runner, when CI reaches release
  readiness, then Nix is already installed and the strict `nix-package` gate can
  run.
- Forbidden behavior: The workflow must not run release readiness before Nix is
  installed, and the release script must not skip `nix build` when Nix is
  missing.
- Files involved: `.github/workflows/ci.yml`.
- Positive test: `awk '/install-nix-action@v31.10.6/{n=1} /Release readiness behavior/{exit n?0:1} END{if(!n) exit 1}' .github/workflows/ci.yml`
- Negative test: `! awk '/Release readiness behavior/{r=1} /install-nix-action@v31.10.6/ && r{exit 0} END{exit 1}' .github/workflows/ci.yml`
- Data/schema/provenance: No runtime schema change.
- Runtime: GitHub CI has Nix before release readiness executes.

## Validation Plan

Focused:

- `awk '/install-nix-action@v31.10.6/{n=1} /Release readiness behavior/{exit n?0:1} END{if(!n) exit 1}' .github/workflows/ci.yml`
- `! awk '/Release readiness behavior/{r=1} /install-nix-action@v31.10.6/ && r{exit 0} END{exit 1}' .github/workflows/ci.yml`
- `bash scripts/test-release-readiness.sh nix-package`
- `.codex/scripts/task-registry source-limit check`

Full:

- `bash scripts/test-release-readiness.sh all`
- `AGENT_GOVERNANCE_FINAL_RELEASE=1 scripts/status.sh --release-source` after landing and commit
- `.codex/scripts/task-registry validate`
- `.codex/scripts/task-registry verify-chain --format json`
- `.codex/scripts/task-registry report PLAN-2026-05-31-ci-nix-release-readiness`
- `.codex/scripts/task-registry metrics`

## Documentation and Release Sync

- Runtime docs: N/A; this closes a CI environment prerequisite, not a runtime
  contract change.
- Release docs: N/A; release already requires Nix package validation.
- Workflow sync: `.github/workflows/ci.yml` must install Nix before release
  readiness.

## Source File Limit

The workflow edit adds a small CI step and the new plan remains below the
1600-line hard limit. Run `.codex/scripts/task-registry source-limit check`
before landing.

## Walkthrough Evidence

- Workflow order verifier exits 0.
- `bash scripts/test-release-readiness.sh nix-package` exits 0.
- `bash scripts/test-release-readiness.sh all` exits 0.
- `AGENT_GOVERNANCE_FINAL_RELEASE=1 scripts/status.sh --release-source` exits 0 after landing and commit.
- `.codex/scripts/task-registry verify-chain --format json` reports an intact
  receipt chain.
- GitHub CI rerun for `main` passes after push.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-05-31-ci-nix-release-readiness"

[[behaviors]]
behavior_id = "B-2026-05-31-ci-nix-G01-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "CI installs Nix before release readiness"
given = "the GitHub CI workflow"
when = "release readiness behavior is reached"
then = "a pinned Nix install action appears earlier in the workflow"
confirmation = "awk '/install-nix-action@v31.10.6/{n=1} /Release readiness behavior/{exit n?0:1} END{if(!n) exit 1}' .github/workflows/ci.yml"

[[behaviors.verifiers]]
type = "command"
command = "awk '/install-nix-action@v31.10.6/{n=1} /Release readiness behavior/{exit n?0:1} END{if(!n) exit 1}' .github/workflows/ci.yml"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-ci-nix-G01-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Release readiness cannot precede Nix installation"
given = "the GitHub CI workflow"
when = "the workflow is inspected"
then = "there is no release readiness step before the pinned Nix install action"
confirmation = "! awk '/Release readiness behavior/{r=1} /install-nix-action@v31.10.6/ && r{exit 0} END{exit 1}' .github/workflows/ci.yml"

[[behaviors.verifiers]]
type = "command"
command = "! awk '/Release readiness behavior/{r=1} /install-nix-action@v31.10.6/ && r{exit 0} END{exit 1}' .github/workflows/ci.yml"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-ci-nix-G02-validation"
gap_id = "GAP-001"
polarity = "validation"
title = "Release readiness and registry gates pass locally"
given = "Nix is available in the local release environment"
when = "release readiness and registry gates run"
then = "the strict Nix package gate, release source status, registry validation, and receipt chain pass"
confirmation = "bash scripts/test-release-readiness.sh nix-package && bash scripts/test-release-readiness.sh all && .codex/scripts/task-registry source-limit check && .codex/scripts/task-registry validate && .codex/scripts/task-registry verify-chain --format json"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-release-readiness.sh nix-package && bash scripts/test-release-readiness.sh all && .codex/scripts/task-registry source-limit check && .codex/scripts/task-registry validate && .codex/scripts/task-registry verify-chain --format json"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-05-31-ci-nix-001"
status = "active"
behavior_ids = ["B-2026-05-31-ci-nix-G01-positive", "B-2026-05-31-ci-nix-G01-negative"]
title = "Install Nix before CI release readiness"
kind = "implementation"
reason = "GitHub CI release readiness failed because nix was unavailable on the runner."
acceptance_proof = "Behaviors B-2026-05-31-ci-nix-G01-positive and B-2026-05-31-ci-nix-G01-negative: workflow order verifiers pass."

[[tasks.targets]]
file = ".github/workflows/ci.yml"
object = "release_readiness_prerequisites"
required_change = "Install pinned Nix action before Release readiness behavior."

[[tasks]]
task_id = "TASK-2026-05-31-ci-nix-002"
status = "active"
behavior_ids = ["B-2026-05-31-ci-nix-G02-validation"]
title = "Validate release readiness after CI prerequisite fix"
kind = "validation"
reason = "Release handoff requires strict local release, registry, and receipt-chain evidence."
acceptance_proof = "Behavior B-2026-05-31-ci-nix-G02-validation: local release readiness and registry gates pass."

[[tasks.targets]]
file = "docs/plans/ci-nix-release-readiness-2026-05-31.md"
object = "walkthrough_evidence"
required_change = "Capture release validation and receipt-chain evidence for the CI prerequisite closure."
```
