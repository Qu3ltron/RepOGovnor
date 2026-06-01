# Adoption Documentation Closure Contract

## Approved Scope
Close the documentation portion of `GP-001` by adding first-run and migration guidance that public users can follow without inferring the workflow from scattered release notes.

In scope:
- Add a concise v2 migration guide.
- Add a minimal example workflow for plan -> activate -> edit -> land -> report.
- Link both documents from README and package them as release assets.
- Update `docs/gap-pipeline.md` to record the closed documentation evidence and keep installer-message polish as future work.

Out of scope:
- Changing installer output.
- Adding a full sample repository.
- Supporting legacy compatibility shims or removed v2 paths.

## Phased Required Change Checklist
### Phase 0: Activation and safety
- [ ] `[NEW]` `docs/plans/adoption-docs-2026-06-01.md` - `Task Manifest`: activate this contract before documentation edits.
- [ ] `[VERIFY]` `docs/task-registry.toml` - `registry`: activation and landing keep validation green.

### Phase 1: Adoption docs
- [ ] `[NEW]` `docs/migration-v2.md` - `migration_guide`: document v0.x/v1 to v2 migration steps, validation, and hard-cut boundaries.
- [ ] `[NEW]` `docs/example-workflow.md` - `example_workflow`: document the minimal daily workflow with concrete commands.
- [ ] `[MODIFY]` `README.md` - `Files users should know`: link the new adoption docs.
- [ ] `[MODIFY]` `docs/gap-pipeline.md` - `GP-001`: record that migration and example docs now exist, with installer output still pending.

### Phase 2: Release packaging
- [ ] `[MODIFY]` `REQUIREMENTS.toml` - `release_source.required`: require the new adoption docs.
- [ ] `[MODIFY]` `package.nix` - `runtime docs assets`: install the new adoption docs under `share/agent-governance/docs`.

### Phase 3: Verification and handoff
- [ ] `[VERIFY]` `scripts/status.sh` - `release-source`: release-source passes with new docs required.
- [ ] `[VERIFY]` `.codex/scripts/task-registry source-limit check` - `line budget`: passes.
- [ ] `[VERIFY]` `.codex/scripts/task-registry validate` - `registry`: passes.

## Per-Gap Success Criteria
### GAP-001: First-run public adoption guidance is too dense
- Current failure: README has install commands and ROADMAP names the adoption gap, but there is no concise migration guide or minimal workflow example in packaged docs.
- Good behavior: public users can read direct migration and daily workflow docs from the repo and packaged asset root.
- Forbidden behavior: docs imply legacy `--overlay` or removed v1 paths are supported.
- Files involved: `docs/migration-v2.md`, `docs/example-workflow.md`, `README.md`, `docs/gap-pipeline.md`, `REQUIREMENTS.toml`, `package.nix`.
- Positive test: `rg -n "docs/migration-v2.md|docs/example-workflow.md|--merge|verify-landing|report" README.md docs/migration-v2.md docs/example-workflow.md package.nix REQUIREMENTS.toml`
- Negative test: typed `not_contains` verifiers reject `--overlay` in the new adoption docs.
- Domain/API/UI: documentation and packaged assets only.
- Runtime: N/A; no runtime behavior changes.

## Validation Plan
Focused:
- `rg -n "docs/migration-v2.md|docs/example-workflow.md|--merge|verify-landing|report" README.md docs/migration-v2.md docs/example-workflow.md package.nix REQUIREMENTS.toml`
- `scripts/status.sh --release-source`
- `.codex/scripts/task-registry source-limit check`
- `.codex/scripts/task-registry validate`

Full:
- `bash scripts/test-release-readiness.sh all`

## Source File Limit
Expected impact is small. New docs and modified files must remain below 1600 lines.

## Walkthrough Evidence
- Contract activation output.
- Focused adoption-doc `rg` output.
- Release-source, source-limit, registry validation, and receipt-chain output.
- Task report and metrics output.

## Task Manifest
```toml
schema_version = 2
plan_id = "PLAN-2026-06-01-adoption-docs"

[[behaviors]]
behavior_id = "B-001-adoption-docs-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Public adoption docs cover migration and workflow"
given = "The adoption docs are added"
when = "the docs and release packaging are inspected"
then = "migration, example workflow, merge install, landing, report, and release asset references exist"
confirmation = "rg -n \"docs/migration-v2.md|docs/example-workflow.md|--merge|verify-landing|report\" README.md docs/migration-v2.md docs/example-workflow.md package.nix REQUIREMENTS.toml"

[[behaviors.verifiers]]
type = "command"
command = "rg -n \"docs/migration-v2.md|docs/example-workflow.md|--merge|verify-landing|report\" README.md docs/migration-v2.md docs/example-workflow.md package.nix REQUIREMENTS.toml"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-adoption-docs-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Adoption docs do not revive removed overlay paths"
given = "The migration and workflow docs are public v2 guidance"
when = "removed installer mode guidance is checked"
then = "the new adoption docs do not recommend --overlay"
confirmation = "typed not_contains verifiers for --overlay in new adoption docs"

[[behaviors.verifiers]]
type = "not_contains"
path = "docs/migration-v2.md"
needle = "--overlay"

[[behaviors.verifiers]]
type = "not_contains"
path = "docs/example-workflow.md"
needle = "--overlay"

[[tasks]]
task_id = "TASK-2026-06-01-adoption-docs-001"
behavior_ids = [
  "B-001-adoption-docs-positive",
  "B-002-adoption-docs-negative",
]
status = "planned"
title = "Add packaged adoption guidance"
kind = "documentation"
reason = "Public users need a concise migration guide and minimal workflow example before broader next-version release work."
acceptance_proof = "Behaviors B-001-adoption-docs-positive and B-002-adoption-docs-negative."

[[tasks.targets]]
file = "docs/migration-v2.md"
object = "migration_guide"
required_change = "Add concise v2 migration guide for existing workspaces."

[[tasks.targets]]
file = "docs/example-workflow.md"
object = "example_workflow"
required_change = "Add minimal plan to landing workflow example."

[[tasks.targets]]
file = "README.md"
object = "Files users should know"
required_change = "Link the new adoption docs."

[[tasks.targets]]
file = "docs/gap-pipeline.md"
object = "GP-001"
required_change = "Record closed documentation evidence and leave installer output as pending."

[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release_source.required"
required_change = "Require the new adoption docs as release-source files."

[[tasks.targets]]
file = "package.nix"
object = "runtime docs assets"
required_change = "Install the new adoption docs in the packaged asset root."
```
