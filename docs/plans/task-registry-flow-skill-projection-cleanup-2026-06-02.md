# Task Registry Flow Skill Projection Cleanup Contract

## Approved Scope
Close the commit-review issue found in the uncommitted skill projection: `skills/task-registry-flow/SKILL.md` duplicated the cost-evidence command paragraph and drifted from the active `.agents` projection for `verify-landing` completion semantics. Scope is limited to aligning the packaged skill projection with `.agents/skills/task-registry-flow/SKILL.md` and adding this cleanup plan to version coverage. No runtime behavior or API changes are in scope.

## Phased Required Change Checklist
### Phase 0: Activation
- [x] `[NEW]` `docs/plans/task-registry-flow-skill-projection-cleanup-2026-06-02.md` - `Task Manifest`: define projection-alignment and duplicate-text negative coverage.
- [ ] `[VERIFY]` `docs/task-registry.toml` - `PLAN_ACTIVATE`: activate this plan before editing projection files.

### Phase 1: Projection cleanup
- [ ] `[MODIFY]` `skills/task-registry-flow/SKILL.md` - `Task Registry Flow`: match `.agents/skills/task-registry-flow/SKILL.md`, including `TASK_VERIFY_LANDING`, non-terminal `TASK_STATUS`, and one cost-evidence command paragraph.
- [ ] `[MODIFY]` `docs/version-roadmap.toml` - `releases[2.1.0].covered_plan_ids`: add `PLAN-2026-06-02-task-registry-flow-skill-projection-cleanup`.

### Phase 2: Verification and handoff
- [ ] `[VERIFY]` `skills/task-registry-flow/SKILL.md` - `projection diff`: `diff -u .agents/skills/task-registry-flow/SKILL.md skills/task-registry-flow/SKILL.md` exits zero.
- [ ] `[VERIFY]` `skills/task-registry-flow/SKILL.md` - `duplicate guard`: exactly one cost-evidence command paragraph exists and completion text names `verify-landing`.
- [ ] `[VERIFY]` `docs/version-roadmap.toml` - `version-check validate`: no completed plan is left uncovered.

## Per-Gap Success Criteria
### GAP-001: Packaged Task-Registry Skill Projection Drift
- Current failure: the packaged `skills/task-registry-flow/SKILL.md` contains duplicated cost-evidence prose and stale completed-status instructions.
- Good behavior: Given the current `.agents` task-registry skill projection, when the packaged skill is compared, then the files match exactly.
- Forbidden behavior: The packaged skill duplicates cost-evidence prose, omits `TASK_VERIFY_LANDING`, or tells agents to complete tasks through `TASK_STATUS`.
- Files involved: `skills/task-registry-flow/SKILL.md`, `docs/version-roadmap.toml`.
- Positive test: `diff -u .agents/skills/task-registry-flow/SKILL.md skills/task-registry-flow/SKILL.md`.
- Negative test: `python3 -c "from pathlib import Path; text=Path('skills/task-registry-flow/SKILL.md').read_text(); assert text.count('Cost evidence commands are also registry-owned') == 1; assert 'TASK_VERIFY_LANDING' in text; assert 'verify-landing --plan-id' in text; assert '--changed-files' in text; assert 'Do not set ' in text and 'completed' in text and 'TASK_STATUS' in text"`.
- Domain/API/UI: N/A; this is packaging/projection documentation.
- Runtime: N/A; no runtime behavior changes.

## Validation Plan
Focused:
- `diff -u .agents/skills/task-registry-flow/SKILL.md skills/task-registry-flow/SKILL.md`
- `python3 -c "from pathlib import Path; text=Path('skills/task-registry-flow/SKILL.md').read_text(); assert text.count('Cost evidence commands are also registry-owned') == 1; assert 'TASK_VERIFY_LANDING' in text; assert 'verify-landing --plan-id' in text; assert '--changed-files' in text; assert 'Do not set ' in text and 'completed' in text and 'TASK_STATUS' in text"`
- `.codex/scripts/task-registry version-check validate --format json`
- `.codex/scripts/task-registry source-limit check`

Full:
- `.codex/scripts/task-registry validate`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `bash scripts/test-release-readiness.sh all`

## Walkthrough Evidence
- Projection diff exits zero.
- Duplicate guard exits zero.
- Version governance reports zero failures.
- Task report and metrics are captured after landing.

## Source File Limit
The edited files remain below the 1600-line source limit. Run `.codex/scripts/task-registry source-limit check` before completion.

## Task Manifest
```toml
schema_version = 2
plan_id = "PLAN-2026-06-02-task-registry-flow-skill-projection-cleanup"

[[behaviors]]
behavior_id = "B-TRF-SKILL-001-projection-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Packaged task-registry skill matches the active agent projection"
given = "the active .agents task-registry-flow skill projection"
when = "the packaged skill projection is compared"
then = "the two files match exactly"
confirmation = "diff -u .agents/skills/task-registry-flow/SKILL.md skills/task-registry-flow/SKILL.md"

[[behaviors.verifiers]]
type = "command"
command = "diff -u .agents/skills/task-registry-flow/SKILL.md skills/task-registry-flow/SKILL.md"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TRF-SKILL-002-duplicate-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Duplicate cost prose and stale completion text stay absent"
given = "the packaged task-registry-flow skill projection"
when = "the duplicate guard inspects the file"
then = "there is one cost-evidence paragraph and completion semantics name verify-landing"
confirmation = "python3 -c \"from pathlib import Path; text=Path('skills/task-registry-flow/SKILL.md').read_text(); assert text.count('Cost evidence commands are also registry-owned') == 1; assert 'TASK_VERIFY_LANDING' in text; assert 'verify-landing --plan-id' in text; assert '--changed-files' in text; assert 'Do not set ' in text and 'completed' in text and 'TASK_STATUS' in text\""

[[behaviors.verifiers]]
type = "command"
command = "python3 -c \"from pathlib import Path; text=Path('skills/task-registry-flow/SKILL.md').read_text(); assert text.count('Cost evidence commands are also registry-owned') == 1; assert 'TASK_VERIFY_LANDING' in text; assert 'verify-landing --plan-id' in text; assert '--changed-files' in text; assert 'Do not set ' in text and 'completed' in text and 'TASK_STATUS' in text\""
expected_exit = 0

[[behaviors]]
behavior_id = "B-TRF-SKILL-003-version-coverage"
gap_id = "GAP-001"
polarity = "validation"
title = "Skill projection cleanup is covered by the release roadmap"
given = "the cleanup plan exists in the task registry"
when = "version governance validates the current release roadmap"
then = "version checks report zero failures"
confirmation = ".codex/scripts/task-registry version-check validate --format json"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry version-check validate --format json"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-06-02-task-registry-flow-skill-projection-cleanup-001"
status = "planned"
title = "Align packaged task-registry-flow skill projection"
kind = "governance"
reason = "Packaged skill projection must not duplicate cost-evidence prose or publish stale completion semantics."
acceptance_proof = "Behaviors B-TRF-SKILL-001-projection-positive, B-TRF-SKILL-002-duplicate-negative, and B-TRF-SKILL-003-version-coverage pass."
behavior_ids = ["B-TRF-SKILL-001-projection-positive", "B-TRF-SKILL-002-duplicate-negative", "B-TRF-SKILL-003-version-coverage"]

[[tasks.targets]]
file = "skills/task-registry-flow/SKILL.md"
object = "Task Registry Flow packaged skill projection"
required_change = "Match .agents/skills/task-registry-flow/SKILL.md and remove duplicate cost-evidence prose."

[[tasks.targets]]
file = "docs/version-roadmap.toml"
object = "releases[2.1.0].covered_plan_ids"
required_change = "Add PLAN-2026-06-02-task-registry-flow-skill-projection-cleanup."
```
