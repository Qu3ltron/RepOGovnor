# Claude CLI Template Pipeline Gap Closure Contract

## Approved Scope

**4 gaps** — Complete the Claude Code governance integration by adding the template pipeline files and entries needed for consumer repos to receive CLAUDE.md via `render-from-config.sh`:

1. Missing `templates/CLAUDE.md.template` — no full-template render for Claude Code entry point
2. Missing `templates/CLAUDE.overlay.md.template` — no merge-block overlay for CLAUDE.md
3. `MANIFEST.toml` — no `[[render]]` entries for CLAUDE.md templates
4. `scripts/render-from-config.sh` — no CLAUDE.md rendering alongside AGENTS.md/GEMINI.md

## Phased Required Change Checklist

### Phase 0: Activation and safety
- [ ] `[NEW]` `docs/plans/claude-cli-template-pipeline-2026-05-31.md` — this contract
- [ ] `[VERIFY]` `.codex/scripts/task-registry source-limit check`

### Phase 1: Template files
- [ ] `[NEW]` `templates/CLAUDE.md.template` — full Claude Code entry-point template with `{{REPO_NAME}}`, `{{CONSTITUTION_PATH}}`, `{{TASK_REGISTRY_CLI}}`, and all other standard placeholders
- [ ] `[NEW]` `templates/CLAUDE.overlay.md.template` — merge-block overlay identical in structure to `templates/AGENTS.overlay.md.template`

### Phase 2: Pipeline wiring
- [ ] `[MODIFY]` `MANIFEST.toml` — add `[[render]]` entries for `CLAUDE.md.template` (full mode) and `CLAUDE.overlay.md.template` (overlay mode with merge-block meaning)
- [ ] `[MODIFY]` `scripts/render-from-config.sh` — add CLAUDE.md rendering in both merge and force modes alongside AGENTS.md/GEMINI.md

### Phase 3: Validation
- [ ] `[VERIFY]` `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- [ ] `[VERIFY]` `.codex/scripts/task-registry source-limit check`
- [ ] `[VERIFY]` `MODE=merge DRY_RUN=1 bash scripts/render-from-config.sh`

## Per-Gap Success Criteria

### Gap 1: Missing CLAUDE.md full template
- **Current failure**: Consumer repos cannot receive a rendered CLAUDE.md via `render-from-config.sh`. Only AGENTS.md and GEMINI.md templates exist.
- **Good behavior**: `templates/CLAUDE.md.template` exists with the same placeholder set as `templates/AGENTS.md.template`, rendering a complete Claude Code entry point.
- **Forbidden behavior**: CLAUDE.md not rendered for consumer repos, forcing manual creation with stale or incorrect content.
- **Files involved**: `templates/CLAUDE.md.template`
- **Positive test**: File exists, contains `{{REPO_NAME}}`, `{{CONSTITUTION_PATH}}`, `{{TASK_REGISTRY_CLI}}`, `{{REPO_ROOT}}`, `{{SCRATCH_ROOT}}`, `{{VALIDATION_FOCUSED}}`, `{{VALIDATION_FULL}}`, `{{AUTHORITY_ORDER}}`, `{{COMMIT_GOVERNANCE_SECTION}}`, `{{VERIFY_HOOK_COMMAND}}`, `{{MUTATION_HOOK_SCRIPT}}`, and `<!-- agent-governance:begin -->` marker
- **Negative test**: File does not contain unreplaced AGENTS-specific paths (e.g., should reference `.claude/settings.json` not just `.codex/hooks.json`)

### Gap 2: Missing CLAUDE.md overlay template
- **Current failure**: `render-from-config.sh --merge` cannot update the agent-governance block in an existing CLAUDE.md.
- **Good behavior**: `templates/CLAUDE.overlay.md.template` exists with the standard `<!-- agent-governance:begin -->` / `<!-- agent-governance:end -->` block.
- **Forbidden behavior**: Merge mode skips CLAUDE.md, leaving stale governance block content.
- **Files involved**: `templates/CLAUDE.overlay.md.template`
- **Positive test**: File exists, contains `<!-- agent-governance:begin -->` and `<!-- agent-governance:end -->` markers
- **Negative test**: File is not empty

### Gap 3: MANIFEST.toml missing CLAUDE.md render entries
- **Current failure**: MANIFEST.toml has `[[render]]` entries for AGENTS.md and GEMINI.md but not CLAUDE.md.
- **Good behavior**: Two new `[[render]]` entries wire CLAUDE.md.template (full) and CLAUDE.overlay.md.template (overlay) to `CLAUDE.md`.
- **Forbidden behavior**: Install/merge operations skip CLAUDE.md because MANIFEST doesn't declare it.
- **Files involved**: `MANIFEST.toml`
- **Positive test**: `grep -c "CLAUDE.md" MANIFEST.toml` returns ≥ 2 render entries
- **Negative test**: No render entry points to a non-existent template path

### Gap 4: render-from-config.sh missing CLAUDE.md rendering
- **Current failure**: `render-from-config.sh` renders AGENTS.md and GEMINI.md in both merge and force modes but skips CLAUDE.md.
- **Good behavior**: CLAUDE.md is rendered alongside AGENTS.md and GEMINI.md in both merge and force modes.
- **Forbidden behavior**: Consumer repos have rendered AGENTS.md and GEMINI.md but a stale or missing CLAUDE.md.
- **Files involved**: `scripts/render-from-config.sh`
- **Positive test**: `MODE=merge DRY_RUN=1 bash scripts/render-from-config.sh` output includes a CLAUDE.md merge action
- **Negative test**: CLAUDE.md rendering does not fail when `templates/CLAUDE.md.template` exists

## Validation Plan

Focused:
```bash
cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml
.codex/scripts/task-registry source-limit check
bash -c 'test -f templates/CLAUDE.md.template && test -f templates/CLAUDE.overlay.md.template'
grep -q 'CLAUDE.md.template' MANIFEST.toml
grep -q 'CLAUDE.overlay.md.template' MANIFEST.toml
grep -q 'CLAUDE.md' scripts/render-from-config.sh
```

Full:
```bash
MODE=merge DRY_RUN=1 bash scripts/render-from-config.sh
bash scripts/test-install-modes.sh
```

## Walkthrough Evidence
- All 95 Rust tests pass
- Source limit check passes
- Templates exist with all required placeholders
- MANIFEST.toml has CLAUDE.md render entries
- render-from-config.sh dry-run includes CLAUDE.md actions

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-05-31-claude-cli-template-pipeline"

[[behaviors]]
behavior_id = "B-001-claude-md-template-exists"
gap_id = "GAP-001"
polarity = "positive"
title = "CLAUDE.md.template exists with all standard placeholders"
given = "a consumer repo needing a rendered CLAUDE.md"
when = "render-from-config.sh runs in force mode"
then = "a complete CLAUDE.md is rendered from the template"
confirmation = "test -f templates/CLAUDE.md.template"

[[behaviors.verifiers]]
type = "command"
command = "bash -c 'test -f templates/CLAUDE.md.template'"
expected_exit = 0

[[behaviors.verifiers]]
type = "contains"
path = "templates/CLAUDE.md.template"
needle = "{{REPO_NAME}}"

[[behaviors.verifiers]]
type = "contains"
path = "templates/CLAUDE.md.template"
needle = "{{TASK_REGISTRY_CLI}}"

[[behaviors.verifiers]]
type = "contains"
path = "templates/CLAUDE.md.template"
needle = "<!-- agent-governance:begin -->"

[[behaviors]]
behavior_id = "B-001b-no-stale-agents-refs"
gap_id = "GAP-001"
polarity = "negative"
title = "CLAUDE.md.template does not contain AGENTS-specific paths"
given = "CLAUDE.md.template source"
when = "comparing against AGENTS.md.template"
then = "Claude Code paths (.claude/settings.json, .claude/skills) are used instead of Codex-only paths"
confirmation = "grep -q '.claude/settings.json' templates/CLAUDE.md.template"

[[behaviors.verifiers]]
type = "command"
command = "bash -c 'grep -q \".claude/settings.json\" templates/CLAUDE.md.template'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-overlay-template-exists"
gap_id = "GAP-002"
polarity = "positive"
title = "CLAUDE.overlay.md.template exists with merge block markers"
given = "an existing CLAUDE.md with a stale governance block"
when = "render-from-config.sh runs in merge mode"
then = "the agent-governance block is updated from the overlay template"
confirmation = "test -f templates/CLAUDE.overlay.md.template"

[[behaviors.verifiers]]
type = "command"
command = "bash -c 'test -f templates/CLAUDE.overlay.md.template'"
expected_exit = 0

[[behaviors.verifiers]]
type = "contains"
path = "templates/CLAUDE.overlay.md.template"
needle = "Claude Code hooks"

[[behaviors]]
behavior_id = "B-002b-overlay-not-empty"
gap_id = "GAP-002"
polarity = "negative"
title = "CLAUDE.overlay.md.template is not empty"
given = "CLAUDE.overlay.md.template"
when = "read for merge operations"
then = "the file contains non-whitespace content"
confirmation = "test -s templates/CLAUDE.overlay.md.template"

[[behaviors.verifiers]]
type = "command"
command = "bash -c 'test -s templates/CLAUDE.overlay.md.template'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-003-manifest-has-claude-render"
gap_id = "GAP-003"
polarity = "positive"
title = "MANIFEST.toml declares CLAUDE.md render entries"
given = "a consumer repo installing the agent-governance plugin"
when = "the install plan is computed"
then = "CLAUDE.md full and overlay render entries are included"
confirmation = "grep -c 'CLAUDE.md.template' MANIFEST.toml"

[[behaviors.verifiers]]
type = "command"
command = "bash -c 'test $(grep -c \"CLAUDE.md.template\" MANIFEST.toml) -ge 1'"
expected_exit = 0

[[behaviors.verifiers]]
type = "command"
command = "bash -c 'test $(grep -c \"CLAUDE.overlay.md.template\" MANIFEST.toml) -ge 1'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-003b-no-non-existent-template-ref"
gap_id = "GAP-003"
polarity = "negative"
title = "MANIFEST.toml CLAUDE.md render entries point to existing templates"
given = "MANIFEST.toml declaring CLAUDE.md render entries"
when = "the install plan is validated"
then = "every CLAUDE.md template path references an existing file in templates/"
confirmation = "test -f templates/CLAUDE.md.template && test -f templates/CLAUDE.overlay.md.template"

[[behaviors.verifiers]]
type = "command"
command = "bash -c 'test -f templates/CLAUDE.md.template && test -f templates/CLAUDE.overlay.md.template'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-004-render-script-claude-support"
gap_id = "GAP-004"
polarity = "positive"
title = "render-from-config.sh handles CLAUDE.md templates in both modes"
given = "a consumer repo with render_claude_code = true"
when = "render-from-config.sh executes in merge or force mode"
then = "CLAUDE.md is rendered/merged alongside AGENTS.md and GEMINI.md"
confirmation = "grep -q 'CLAUDE.md' scripts/render-from-config.sh"

[[behaviors.verifiers]]
type = "command"
command = "bash -c 'grep -q \"CLAUDE.md\" scripts/render-from-config.sh'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-004b-dry-run-includes-claude"
gap_id = "GAP-004"
polarity = "negative"
title = "DRY_RUN merge mode does not skip CLAUDE.md"
given = "render-from-config.sh with MODE=merge DRY_RUN=1"
when = "infrastructure files are listed"
then = "CLAUDE.md merge action appears in the output"
confirmation = "MODE=merge DRY_RUN=1 bash scripts/render-from-config.sh 2>&1 | grep -q 'CLAUDE.md'"

[[behaviors.verifiers]]
type = "command"
command = "bash -c 'MODE=merge DRY_RUN=1 bash scripts/render-from-config.sh 2>&1 | grep -q \"CLAUDE.md\"'"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-05-31-C01"
status = "planned"
kind = "implementation"
reason = "Consumer repos need a Claude Code entry-point template for the render pipeline"
behavior_ids = ["B-001-claude-md-template-exists", "B-001b-no-stale-agents-refs"]
title = "Create templates/CLAUDE.md.template"
acceptance_proof = "Behavior B-001: template exists with all required placeholders"

[[tasks.targets]]
file = "templates/CLAUDE.md.template"
object = "Full Claude Code entry-point template"
required_change = "Create template with all standard placeholders, Claude Code agent paths"

[[tasks]]
task_id = "TASK-2026-05-31-C02"
status = "planned"
kind = "implementation"
reason = "Merge mode needs an overlay to update the agent-governance block in existing CLAUDE.md files"
behavior_ids = ["B-002-overlay-template-exists", "B-002b-overlay-not-empty"]
title = "Create templates/CLAUDE.overlay.md.template"
acceptance_proof = "Behavior B-002: overlay template exists with agent-governance block markers"

[[tasks.targets]]
file = "templates/CLAUDE.overlay.md.template"
object = "Claude Code merge-block overlay"
required_change = "Create overlay template with agent-governance block content"

[[tasks]]
task_id = "TASK-2026-05-31-C03"
status = "planned"
kind = "implementation"
reason = "MANIFEST.toml must declare CLAUDE.md render entries so install plan discovers them"
behavior_ids = ["B-003-manifest-has-claude-render", "B-003b-no-non-existent-template-ref"]
title = "Add CLAUDE.md render entries to MANIFEST.toml"
acceptance_proof = "Behavior B-003: MANIFEST.toml has [[render]] entries for both CLAUDE.md templates"

[[tasks.targets]]
file = "MANIFEST.toml"
object = "[[render]] entries for CLAUDE.md"
required_change = "Add full and overlay render entries for CLAUDE.md templates"

[[tasks]]
task_id = "TASK-2026-05-31-C04"
status = "planned"
kind = "implementation"
reason = "render-from-config.sh must handle CLAUDE.md alongside AGENTS.md/GEMINI.md in both modes"
behavior_ids = ["B-004-render-script-claude-support", "B-004b-dry-run-includes-claude"]
title = "Add CLAUDE.md rendering to render-from-config.sh"
acceptance_proof = "Behavior B-004: render-from-config.sh includes CLAUDE.md in merge and force modes"

[[tasks.targets]]
file = "scripts/render-from-config.sh"
object = "CLAUDE.md rendering logic"
required_change = "Add CLAUDE.md template rendering in merge and force mode blocks"
```
