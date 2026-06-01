# Policy Compliance Cost Posture Gap Closure Contract

## Approved Scope

This contract aligns public product posture with the approved direction:
RepOGovnor targets engineering policy compliance for agent-assisted repos.
It also records token spend and cost-per-commit as first-class future evidence
goals without claiming shipped cost accounting.

In scope:
- Public positioning in README, vision, roadmap, and gap pipeline docs.
- A concise engineering-policy-compliance direction doc.
- Runtime schema doc drift for known diagnostic surfaces.
- Release-source and Nix package inclusion for the new public doc.
- Honest token/cost direction with measured, estimated, and unmeasured states.

Out of scope:
- Implementing a policy engine command.
- Implementing cost-ledger runtime commands.
- Regulatory certification, external attestation, or remote telemetry.
- Compatibility shims for old governance layouts.

## Phased Required Change Checklist

### Phase 0: Activation and safety
- [ ] `[NEW]` `docs/plans/policy-compliance-cost-posture-2026-06-01.md` - `closure_contract`: create and activate this contract before implementation.
- [ ] `[VERIFY]` `docs/plans/policy-compliance-cost-posture-2026-06-01.md` - `PLAN_ACTIVATE`: `.codex/scripts/task-registry activate docs/plans/policy-compliance-cost-posture-2026-06-01.md`.

### Phase 1: Public posture alignment
- [ ] `[MODIFY]` `README.md` - `product_positioning`: lead with engineering policy compliance for agent-assisted repos, supported agents, honest proof boundaries, and separated consumer vs source-release checks.
- [ ] `[MODIFY]` `VISION.md` - `product_direction`: describe configurable engineering policy input, compliance artifact output, current evidence substrate, and future cost evidence.
- [ ] `[MODIFY]` `ROADMAP.md` - `policy_engine_roadmap`: replace stale v2 state and add policy engine, compliance artifact, waiver lifecycle, and token/cost ledger milestones.
- [ ] `[NEW]` `docs/engineering-policy-compliance.md` - `direction_doc`: define engineering-policy compliance, current primitives, intended artifact, cost evidence direction, and explicit non-claims.
- [ ] `[MODIFY]` `docs/gap-pipeline.md` - `current_evidence`: update stale release evidence and add policy/cost gaps without claiming shipped runtime features.

### Phase 2: Release and schema alignment
- [ ] `[MODIFY]` `docs/runtime-schemas.md` - `known_surfaces`: include `version` and `backlog`, and describe future compliance/cost artifacts as direction only.
- [ ] `[MODIFY]` `REQUIREMENTS.toml` - `release_source.required`: add `docs/engineering-policy-compliance.md`.
- [ ] `[MODIFY]` `package.nix` - `asset_docs`: package `docs/engineering-policy-compliance.md`.

### Phase 3: Verification and landing
- [ ] `[VERIFY]` `README.md` - `stale_public_claims`: `! rg -n '2\.0\.0|2\.0\.2|Codex, Cursor, and Antigravity' README.md VISION.md ROADMAP.md docs --glob '*.md' --glob '!docs/plans/**' --glob '!docs/task-registry/**'`.
- [ ] `[VERIFY]` `docs/engineering-policy-compliance.md` - `cost_honesty`: `rg -n 'measured|estimated|unmeasured|must not guess' docs/engineering-policy-compliance.md README.md VISION.md ROADMAP.md`.
- [ ] `[VERIFY]` `docs/runtime-schemas.md` - `surface_doc`: `rg -n 'version.*backlog|backlog.*version' docs/runtime-schemas.md`.
- [ ] `[VERIFY]` `REQUIREMENTS.toml` - `release_doc_required`: `rg -n 'docs/engineering-policy-compliance.md' REQUIREMENTS.toml package.nix`.
- [ ] `[VERIFY]` `source_limit`: `.codex/scripts/task-registry source-limit check`.
- [ ] `[VERIFY]` `registry_validation`: `.codex/scripts/task-registry validate`.
- [ ] `[VERIFY]` `release_posture`: `.codex/scripts/task-registry version-check validate && .codex/scripts/task-registry backlog-check && .codex/scripts/task-registry status-check --format json && scripts/status.sh --release-source`.

## Per-Gap Success Criteria

### GAP-001: Public docs do not state the central engineering-policy direction
- Current failure: README and vision frame the product mostly as agent workflow governance, while the approved direction is engineering policy compliance for agent-assisted repos.
- Good behavior: Given the public docs, when a maintainer reads the first page and vision, then RepOGovnor is presented as evaluating declared engineering policy and producing local compliance evidence.
- Forbidden behavior: Public docs must not imply regulatory certification, external attestation, or product-correctness proof.
- Files involved: `README.md`, `VISION.md`, `ROADMAP.md`, `docs/engineering-policy-compliance.md`, `docs/gap-pipeline.md`.
- Positive test: `rg -n 'engineering policy compliance|declared engineering policy|compliance artifact' README.md VISION.md ROADMAP.md docs/engineering-policy-compliance.md`.
- Negative test: `! rg -n 'is a regulatory certification|provides external attestation|proves product correctness' README.md VISION.md ROADMAP.md docs/engineering-policy-compliance.md`.
- Domain/API/UI: Public docs only; no runtime API changes.
- Runtime: N/A; this is posture documentation.

### GAP-002: Token spend direction could become a false precision claim
- Current failure: Cost per commit is an approved direction, but current runtime does not capture structured token usage receipts.
- Good behavior: Given the cost direction, when docs describe cost metrics, then they require structured evidence and classify values as measured, estimated, or unmeasured.
- Forbidden behavior: Docs must not claim reliable cost per commit is currently shipped or inferable without provider/model usage evidence and pricing snapshots.
- Files involved: `README.md`, `VISION.md`, `ROADMAP.md`, `docs/engineering-policy-compliance.md`.
- Positive test: `rg -n 'measured|estimated|unmeasured|cost per commit' README.md VISION.md ROADMAP.md docs/engineering-policy-compliance.md`.
- Negative test: `! rg -n 'cost per commit is available|reliable cost per commit today|automatically calculates token spend' README.md VISION.md ROADMAP.md docs/engineering-policy-compliance.md`.
- Domain/API/UI: Public docs only; no runtime API changes.
- Runtime: N/A; future cost ledger is not implemented in this scope.

### GAP-003: Public release docs and package surfaces drift from actual capabilities
- Current failure: Stale version claims remain, Claude Code support is underclaimed, `version` and `backlog` surfaces are missing from the known-surface list, and a new public doc would be unavailable unless declared and packaged.
- Good behavior: Given the current repo, when release-source docs are checked, then version evidence is `2.1.0`, supported agent docs include Claude Code, schema docs list `version` and `backlog`, and the new direction doc is release-required and packaged.
- Forbidden behavior: Consumer install docs must not direct generic consumer repos to run plugin-source release checks as install validation.
- Files involved: `README.md`, `VISION.md`, `ROADMAP.md`, `docs/gap-pipeline.md`, `docs/runtime-schemas.md`, `REQUIREMENTS.toml`, `package.nix`.
- Positive test: `rg -n 'Claude Code|docs/engineering-policy-compliance.md|version.*backlog|backlog.*version' README.md VISION.md ROADMAP.md docs/runtime-schemas.md REQUIREMENTS.toml package.nix`.
- Negative test: `! rg -n '2\.0\.0|2\.0\.2|Codex, Cursor, and Antigravity' README.md VISION.md ROADMAP.md docs --glob '*.md' --glob '!docs/plans/**' --glob '!docs/task-registry/**'`.
- Domain/API/UI: Public docs and package assets only.
- Runtime: Release-source validation must remain green.

## Validation Plan

Focused:
- `rg -n 'engineering policy compliance|declared engineering policy|compliance artifact' README.md VISION.md ROADMAP.md docs/engineering-policy-compliance.md`
- `! rg -n '2\.0\.0|2\.0\.2|Codex, Cursor, and Antigravity' README.md VISION.md ROADMAP.md docs --glob '*.md' --glob '!docs/plans/**' --glob '!docs/task-registry/**'`
- `rg -n 'measured|estimated|unmeasured|cost per commit' README.md VISION.md ROADMAP.md docs/engineering-policy-compliance.md`
- `! rg -n 'cost per commit is available|reliable cost per commit today|automatically calculates token spend' README.md VISION.md ROADMAP.md docs/engineering-policy-compliance.md`
- `rg -n 'docs/engineering-policy-compliance.md' REQUIREMENTS.toml package.nix`
- `.codex/scripts/task-registry source-limit check`
- `.codex/scripts/task-registry validate`

Full:
- `.codex/scripts/task-registry version-check validate`
- `.codex/scripts/task-registry backlog-check`
- `.codex/scripts/task-registry status-check --format json`
- `scripts/status.sh --release-source`
- `bash scripts/test-release-readiness.sh all`

## Source File Limit

Expected impact is documentation-only. Existing target files are far below the
1600-line limit. The new public direction doc must remain concise, and final
verification includes `.codex/scripts/task-registry source-limit check`.

## Walkthrough Evidence

- Plan activation output.
- Focused rg positive and negative checks.
- Source-limit output.
- Registry validation output.
- Version, backlog, status, and release-source validation output.
- `TASK_REPORT` and `TASK_METRICS` after landing.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-06-01-policy-compliance-cost-posture"

[[behaviors]]
behavior_id = "B-001-policy-positioning-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Public docs state engineering policy compliance direction"
given = "The public README, vision, roadmap, and policy compliance doc exist"
when = "The docs are searched for the approved product direction"
then = "They state engineering policy compliance, declared policy input, and compliance artifact output"
confirmation = "rg -n 'engineering policy compliance|declared engineering policy|compliance artifact' README.md VISION.md ROADMAP.md docs/engineering-policy-compliance.md"

[[behaviors.verifiers]]
type = "command"
command = "rg -n 'engineering policy compliance|declared engineering policy|compliance artifact' README.md VISION.md ROADMAP.md docs/engineering-policy-compliance.md"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-policy-positioning-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Public docs avoid certification and correctness overclaims"
given = "The public docs describe compliance direction"
when = "Forbidden overclaims are searched"
then = "The docs do not claim regulatory certification, external attestation, or product correctness proof"
confirmation = "! rg -n 'is a regulatory certification|provides external attestation|proves product correctness' README.md VISION.md ROADMAP.md docs/engineering-policy-compliance.md"

[[behaviors.verifiers]]
type = "command"
command = "! rg -n 'is a regulatory certification|provides external attestation|proves product correctness' README.md VISION.md ROADMAP.md docs/engineering-policy-compliance.md"
expected_exit = 0

[[behaviors]]
behavior_id = "B-003-cost-direction-positive"
gap_id = "GAP-002"
polarity = "positive"
title = "Cost direction requires measured, estimated, and unmeasured states"
given = "The cost evidence direction is documented"
when = "The docs are searched for cost classification"
then = "They name measured, estimated, unmeasured, and cost per commit"
confirmation = "rg -n 'measured|estimated|unmeasured|cost per commit' README.md VISION.md ROADMAP.md docs/engineering-policy-compliance.md"

[[behaviors.verifiers]]
type = "command"
command = "rg -n 'measured|estimated|unmeasured|cost per commit' README.md VISION.md ROADMAP.md docs/engineering-policy-compliance.md"
expected_exit = 0

[[behaviors]]
behavior_id = "B-004-cost-direction-negative"
gap_id = "GAP-002"
polarity = "negative"
title = "Cost direction avoids false precision"
given = "Cost per commit is a future product direction"
when = "False shipped-cost claims are searched"
then = "The docs do not claim reliable automatic token spend calculation today"
confirmation = "! rg -n 'cost per commit is available|reliable cost per commit today|automatically calculates token spend' README.md VISION.md ROADMAP.md docs/engineering-policy-compliance.md"

[[behaviors.verifiers]]
type = "command"
command = "! rg -n 'cost per commit is available|reliable cost per commit today|automatically calculates token spend' README.md VISION.md ROADMAP.md docs/engineering-policy-compliance.md"
expected_exit = 0

[[behaviors]]
behavior_id = "B-005-release-docs-positive"
gap_id = "GAP-003"
polarity = "positive"
title = "Release docs and package surfaces include current capability docs"
given = "Public docs and release manifests are updated"
when = "Capability and packaging references are searched"
then = "Claude Code, engineering-policy-compliance docs, and version/backlog surfaces are present"
confirmation = "rg -n 'Claude Code|docs/engineering-policy-compliance.md|version.*backlog|backlog.*version' README.md VISION.md ROADMAP.md docs/runtime-schemas.md REQUIREMENTS.toml package.nix"

[[behaviors.verifiers]]
type = "command"
command = "rg -n 'Claude Code|docs/engineering-policy-compliance.md|version.*backlog|backlog.*version' README.md VISION.md ROADMAP.md docs/runtime-schemas.md REQUIREMENTS.toml package.nix"
expected_exit = 0

[[behaviors]]
behavior_id = "B-006-release-docs-negative"
gap_id = "GAP-003"
polarity = "negative"
title = "Stale public version and agent-support claims are removed"
given = "The current release is 2.1.0 and Claude Code is supported"
when = "Stale version and three-agent-only phrases are searched"
then = "No stale 2.0.0, 2.0.2, or three-agent-only phrase remains in public docs"
confirmation = "! rg -n '2\\.0\\.0|2\\.0\\.2|Codex, Cursor, and Antigravity' README.md VISION.md ROADMAP.md docs --glob '*.md' --glob '!docs/plans/**' --glob '!docs/task-registry/**'"

[[behaviors.verifiers]]
type = "command"
command = "! rg -n '2\\.0\\.0|2\\.0\\.2|Codex, Cursor, and Antigravity' README.md VISION.md ROADMAP.md docs --glob '*.md' --glob '!docs/plans/**' --glob '!docs/task-registry/**'"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-06-01-policy-compliance-cost-posture-001"
title = "Align public product posture with engineering policy compliance"
kind = "documentation"
status = "planned"
reason = "Public docs must state the approved central product direction without overclaiming certification or product correctness."
behavior_ids = ["B-001-policy-positioning-positive", "B-002-policy-positioning-negative"]
acceptance_proof = "Behaviors B-001 and B-002 pass their typed verifiers."

[[tasks.targets]]
file = "README.md"
object = "product_positioning"
required_change = "Lead with engineering policy compliance for agent-assisted repos, supported agent surfaces, proof boundaries, and corrected validation guidance."

[[tasks.targets]]
file = "VISION.md"
object = "product_direction"
required_change = "Describe declared engineering policy input, local compliance artifact output, current primitives, and non-claims."

[[tasks.targets]]
file = "ROADMAP.md"
object = "policy_engine_roadmap"
required_change = "Replace stale v2 state and add policy engine, compliance artifact, waiver lifecycle, and cost evidence milestones."

[[tasks.targets]]
file = "docs/gap-pipeline.md"
object = "current_evidence"
required_change = "Update current evidence and remaining gaps for engineering policy compliance direction."

[[tasks]]
task_id = "TASK-2026-06-01-policy-compliance-cost-posture-002"
title = "Document honest token spend and cost evidence direction"
kind = "documentation"
status = "planned"
reason = "Token spend and cost per commit should be first-class direction without false precision."
behavior_ids = ["B-003-cost-direction-positive", "B-004-cost-direction-negative"]
acceptance_proof = "Behaviors B-003 and B-004 pass their typed verifiers."

[[tasks.targets]]
file = "docs/engineering-policy-compliance.md"
object = "cost_evidence"
required_change = "Define honest token/cost evidence requirements and unmeasured gaps."

[[tasks]]
task_id = "TASK-2026-06-01-policy-compliance-cost-posture-003"
title = "Align release docs, schema docs, and packaged assets"
kind = "release"
status = "planned"
reason = "Public release surfaces must match actual supported agents, schema surfaces, current version, and packaged docs."
behavior_ids = ["B-005-release-docs-positive", "B-006-release-docs-negative"]
acceptance_proof = "Behaviors B-005 and B-006 pass their typed verifiers."

[[tasks.targets]]
file = "docs/runtime-schemas.md"
object = "known_surfaces"
required_change = "List version and backlog diagnostic surfaces and describe future compliance/cost artifacts as non-shipped direction."

[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release_source.required"
required_change = "Require docs/engineering-policy-compliance.md as a release-source file."

[[tasks.targets]]
file = "package.nix"
object = "asset_docs"
required_change = "Package docs/engineering-policy-compliance.md in Nix assets."
```
