# Proof Boundary Documentation Closure Contract

## Approved Scope
Close `GP-005` from `docs/gap-pipeline.md` by making public docs separate governance proof from product correctness proof.

In scope:
- Add a visible proof-boundary section to `README.md`.
- Update `docs/gap-pipeline.md` so `GP-005` records the closure evidence while remaining honest about future reviewer-output work.

Out of scope:
- Implementing a compact reviewer report command.
- Changing runtime verifier semantics.
- Adding compatibility shims or weakening governance requirements.

## Phased Required Change Checklist
### Phase 0: Activation and safety
- [ ] `[NEW]` `docs/plans/proof-boundary-docs-2026-06-01.md` - `Task Manifest`: activate this contract before documentation edits.
- [ ] `[VERIFY]` `docs/task-registry.toml` - `registry`: activation and landing keep validation green.

### Phase 1: Public proof-boundary docs
- [ ] `[MODIFY]` `README.md` - `Proof boundaries`: state what governance checks prove and do not prove.
- [ ] `[MODIFY]` `docs/gap-pipeline.md` - `GP-005`: move the immediate docs gap to closed evidence and keep reviewer-output wording as future follow-up.

### Phase 2: Verification and handoff
- [ ] `[VERIFY]` `.codex/scripts/task-registry source-limit check` - `line budget`: passes.
- [ ] `[VERIFY]` `.codex/scripts/task-registry validate` - `registry`: passes.
- [ ] `[VERIFY]` `.codex/scripts/task-registry verify-chain --format json` - `receipt chain`: passes.

## Per-Gap Success Criteria
### GAP-001: Public docs can overstate governance proof
- Current failure: `docs/gap-pipeline.md` identifies `GP-005`, and README only briefly states that governance does not prove product correctness.
- Good behavior: README explicitly distinguishes governance proof from product correctness proof and tells maintainers which project-owned evidence still matters.
- Forbidden behavior: public docs claim governance checks replace code review, domain tests, security review, or product acceptance.
- Files involved: `README.md`, `docs/gap-pipeline.md`.
- Positive test: `rg -n "Proof boundaries|Governance proof|Product correctness proof|domain tests|code review" README.md docs/gap-pipeline.md`
- Negative test: typed `not_contains` verifiers reject "governance checks replace code review" and "green governance means product correctness".
- Domain/API/UI: documentation only.
- Runtime: N/A; no runtime behavior changes.

## Validation Plan
Focused:
- `rg -n "Proof boundaries|Governance proof|Product correctness proof|domain tests|code review" README.md docs/gap-pipeline.md`
- `.codex/scripts/task-registry source-limit check`
- `.codex/scripts/task-registry validate`

Full:
- `bash scripts/release-version-check.sh`
- `bash scripts/test-release-readiness.sh all`

## Source File Limit
Expected impact is small. `README.md`, `docs/gap-pipeline.md`, and this plan must remain below 1600 lines.

## Walkthrough Evidence
- Contract activation output.
- Focused proof-boundary `rg` output.
- Source-limit, registry validation, and receipt-chain output.
- Task report and metrics output.

## Task Manifest
```toml
schema_version = 2
plan_id = "PLAN-2026-06-01-proof-boundary-docs"

[[behaviors]]
behavior_id = "B-001-proof-boundary-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Public docs separate governance proof from product proof"
given = "The README and gap pipeline are current"
when = "proof-boundary terms are inspected"
then = "the docs name governance proof, product correctness proof, domain tests, and code review"
confirmation = "rg -n \"Proof boundaries|Governance proof|Product correctness proof|domain tests|code review\" README.md docs/gap-pipeline.md"

[[behaviors.verifiers]]
type = "command"
command = "rg -n \"Proof boundaries|Governance proof|Product correctness proof|domain tests|code review\" README.md docs/gap-pipeline.md"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-proof-boundary-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Public docs do not claim governance replaces product review"
given = "The README and gap pipeline are public-facing"
when = "forbidden overclaims are checked"
then = "the docs do not say governance checks replace review or prove product correctness"
confirmation = "typed not_contains verifiers for forbidden proof-boundary claims"

[[behaviors.verifiers]]
type = "not_contains"
path = "README.md"
needle = "governance checks replace code review"

[[behaviors.verifiers]]
type = "not_contains"
path = "README.md"
needle = "green governance means product correctness"

[[behaviors.verifiers]]
type = "not_contains"
path = "docs/gap-pipeline.md"
needle = "governance checks replace code review"

[[behaviors.verifiers]]
type = "not_contains"
path = "docs/gap-pipeline.md"
needle = "green governance means product correctness"

[[tasks]]
task_id = "TASK-2026-06-01-proof-boundary-docs-001"
behavior_ids = [
  "B-001-proof-boundary-positive",
  "B-002-proof-boundary-negative",
]
status = "planned"
title = "Document governance proof boundaries"
kind = "documentation"
reason = "Public users need to know governance checks prove process and provenance, not product correctness."
acceptance_proof = "Behaviors B-001-proof-boundary-positive and B-002-proof-boundary-negative."

[[tasks.targets]]
file = "README.md"
object = "Proof boundaries"
required_change = "Add a visible section separating governance proof from product correctness proof."

[[tasks.targets]]
file = "docs/gap-pipeline.md"
object = "GP-005"
required_change = "Record immediate docs closure evidence and preserve reviewer-output follow-up."
```
