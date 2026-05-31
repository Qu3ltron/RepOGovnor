# CI-Owned Landing Gap Closure Contract

## Approved Scope

Close the plan-to-activation hardening gap surfaced by local comparison against PhoneClaw and certification-gpt.

In scope:

- Add a plugin-owned `verify-landing` task-registry command that receives changed files, proves they map to active task targets, runs linked typed behavior verifiers, and writes completion status from that command path only.
- Forbid direct `status TASK-ID completed`; non-terminal status changes remain available through `status`, and deferral remains available only through `defer`.
- Record `verify-landing` in the CLI command vocabulary and receipt path.
- Update agent-facing docs, templates, and skill projections so completion authority is described as landing-owned rather than agent-owned.

Out of scope:

- Remote CI workflow wiring for a specific consumer repository. Reactivation condition: a consumer repo asks for platform-specific CI integration after this CLI contract exists.
- PhoneClaw-style feature registry generation. Reactivation condition: this plugin adds a portable feature registry contract.

Schema impact:

- The registry task row may gain optional completion evidence fields for new completions. Existing historical rows remain valid without a compatibility adapter.

## Phased Required Change Checklist

### Phase 0: Activation and safety

- [ ] `[NEW]` `docs/plans/ci-owned-landing-2026-05-31.md` - `closure contract`: define scope, success criteria, validation, and task manifest; acceptance proof is `PLAN_ACTIVATE docs/plans/ci-owned-landing-2026-05-31.md`.
- [ ] `[GENERATE]` `docs/task-registry.toml` - `active task registry`: activate this contract through `.codex/scripts/task-registry activate`; acceptance proof is matching plan hash and active task rows.

### Phase 1: Landing command implementation

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/schema.rs` - `CliCommand`: add canonical `verify-landing` opcode; acceptance proof is behavior `B-2026-05-31-landing-G01-positive`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/runtime.rs` - `runtime dispatcher`: route `verify-landing`, update usage, and keep status completion rejected; acceptance proof is behaviors `B-2026-05-31-landing-G01-positive` and `B-2026-05-31-landing-G02-negative`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/activation.rs` - `status transition entrypoint`: reject direct completed status and expose only internal landing completion; acceptance proof is behavior `B-2026-05-31-landing-G02-negative`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/landing.rs` - `landing verifier`: implement changed-file parsing, target binding, verifier execution, registry completion, and plan status refresh; acceptance proof is behaviors `B-2026-05-31-landing-G01-positive` and `B-2026-05-31-landing-G03-negative`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/model.rs` - `RegistryTask completion evidence`: add optional landing-owned completion evidence fields; acceptance proof is behavior `B-2026-05-31-landing-G01-positive`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/receipts.rs` - `receipt command policy`: record `verify-landing` by default; acceptance proof is behavior `B-2026-05-31-landing-G01-positive`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/main.rs` - `module registry`: register the landing module; acceptance proof is behavior `B-2026-05-31-landing-G01-positive`.
- [ ] `[MODIFY]` `REQUIREMENTS.toml` - `release source declaration`: declare the new landing implementation and tests as required release-source files; acceptance proof is behavior `B-2026-05-31-landing-G05-validation`.

### Phase 2: Behavioral tests

- [ ] `[NEW]` `rust/task-registry-flow-cli/src/tests/landing_tests.rs` - `landing tests`: prove successful completion, direct status denial, and unbound or registry-only rejection; acceptance proof is behaviors `B-2026-05-31-landing-G01-positive`, `B-2026-05-31-landing-G02-negative`, and `B-2026-05-31-landing-G03-negative`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/mod.rs` - `test module registry`: include landing tests and update direct completion expectations; acceptance proof is focused cargo test commands.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/activation_terminal_tests.rs` - `terminal activation tests`: complete setup through `verify-landing` semantics instead of direct status completion; acceptance proof is full cargo test.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/state_transition_tests.rs` - `transition tests`: keep transition semantics internal while documenting direct status denial in landing tests; acceptance proof is full cargo test.

### Phase 3: Documentation and projected workflow sync

- [ ] `[MODIFY]` `README.md` - `daily workflow`: replace direct completion status guidance with `verify-landing`; acceptance proof is behavior `B-2026-05-31-landing-G04-positive`.
- [ ] `[MODIFY]` `AGENTS.md` - `gap closure instructions`: state completion is owned by `verify-landing`; acceptance proof is behavior `B-2026-05-31-landing-G04-positive`.
- [ ] `[MODIFY]` `docs/runtime-schemas.md` - `runtime completion contract`: document landing-owned completion and registry-only denial; acceptance proof is behavior `B-2026-05-31-landing-G04-positive`.
- [ ] `[MODIFY]` `.codex/agent-governance.toml` - `task registry command metadata`: add `verify_landing_command`; acceptance proof is behavior `B-2026-05-31-landing-G04-positive`.
- [ ] `[MODIFY]` `templates/.codex/agent-governance.toml.template` - `projected task registry metadata`: add `verify_landing_command`; acceptance proof is behavior `B-2026-05-31-landing-G04-positive`.
- [ ] `[MODIFY]` `scripts/render-from-config.sh` - `config renderer`: preserve canonical verify-landing metadata and docs projection text; acceptance proof is full validation.
- [ ] `[MODIFY]` `templates/AGENTS.md.template` - `Codex projected workflow`: describe landing-owned completion; acceptance proof is behavior `B-2026-05-31-landing-G04-positive`.
- [ ] `[MODIFY]` `templates/CLAUDE.md.template` - `Claude projected workflow`: describe landing-owned completion; acceptance proof is behavior `B-2026-05-31-landing-G04-positive`.
- [ ] `[MODIFY]` `templates/GEMINI.md.template` - `Antigravity projected workflow`: describe landing-owned completion; acceptance proof is behavior `B-2026-05-31-landing-G04-positive`.
- [ ] `[MODIFY]` `.agents/skills/task-registry-flow/SKILL.md` - `Codex skill workflow`: describe `TASK_VERIFY_LANDING`; acceptance proof is behavior `B-2026-05-31-landing-G04-positive`.
- [ ] `[MODIFY]` `.agents/skills/task-registry-flow.md` - `flat Codex skill workflow`: describe `TASK_VERIFY_LANDING`; acceptance proof is behavior `B-2026-05-31-landing-G04-positive`.
- [ ] `[MODIFY]` `.cursor/skills/task-registry-flow/SKILL.md` - `Cursor skill workflow`: describe `TASK_VERIFY_LANDING`; acceptance proof is behavior `B-2026-05-31-landing-G04-positive`.
- [ ] `[MODIFY]` `.claude/skills/task-registry-flow/SKILL.md` - `Claude skill workflow`: describe `TASK_VERIFY_LANDING`; acceptance proof is behavior `B-2026-05-31-landing-G04-positive`.
- [ ] `[MODIFY]` `.agents/skills/task-registry-flow/PROJECT.md` - `Codex skill project extension`: add verify-landing opcode row; acceptance proof is behavior `B-2026-05-31-landing-G04-positive`.
- [ ] `[MODIFY]` `.cursor/skills/task-registry-flow/PROJECT.md` - `Cursor skill project extension`: add verify-landing opcode row; acceptance proof is behavior `B-2026-05-31-landing-G04-positive`.
- [ ] `[MODIFY]` `.claude/skills/task-registry-flow/PROJECT.md` - `Claude skill project extension`: add verify-landing opcode row; acceptance proof is behavior `B-2026-05-31-landing-G04-positive`.

### Phase 4: Final validation and handoff

- [ ] `[VERIFY]` `rust/task-registry-flow-cli/Cargo.toml` - `focused and full gates`: run source-limit, focused landing tests, full cargo test, registry validation, receipt chain, metrics, and task report; acceptance proof is behavior `B-2026-05-31-landing-G05-validation`.

## Per-Gap Success Criteria

### GAP-001: Completion authority still lives on direct status

- Current failure: `status TASK-ID completed` can certify task completion after verifiers pass, so the same actor that implements a task can directly mark it complete.
- Good behavior: Given an active task with changed files bound to its targets, when `verify-landing --plan-id PLAN-ID --changed-files PATHS` runs, then linked typed verifiers pass and only the bound tasks become `completed`.
- Forbidden behavior: Given a task id, when `status TASK-ID completed` is attempted, then the command fails closed and tells the caller to use `verify-landing`.
- Files involved: `rust/task-registry-flow-cli/src/activation.rs`, `rust/task-registry-flow-cli/src/landing.rs`, `rust/task-registry-flow-cli/src/runtime.rs`, `rust/task-registry-flow-cli/src/schema.rs`, `rust/task-registry-flow-cli/src/model.rs`, `rust/task-registry-flow-cli/src/receipts.rs`, `rust/task-registry-flow-cli/src/main.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml landing_completes_changed_file_tasks -- --nocapture`
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml landing_rejects_direct_completed_status -- --nocapture`
- Data/schema/provenance: Completed rows written by `verify-landing` include landing-owned completion evidence; direct status cannot write completed rows.
- Runtime: CLI command exits nonzero for direct completion and zero for valid landing.

### GAP-002: Landing does not reject off-plan or registry-only drift

- Current failure: mutation hooks protect live edits, but there is no first-class landing command that rejects changed files without active task target provenance or rejects registry-only completion edits.
- Good behavior: Given changed files that map to exactly one active plan's task targets, when `verify-landing` runs, then it selects those tasks and runs their behavior verifiers.
- Forbidden behavior: Given registry-only changes or a changed implementation file with no active task target, when `verify-landing` runs, then it fails closed before writing completion.
- Files involved: `rust/task-registry-flow-cli/src/landing.rs`, `rust/task-registry-flow-cli/src/tests/landing_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml landing_completes_changed_file_tasks -- --nocapture`
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml landing_rejects_unbound_or_registry_only_changes -- --nocapture`
- Data/schema/provenance: Registry-only changed-file sets do not satisfy task completion; changed implementation paths must resolve through declared targets.
- Runtime: CLI command exits nonzero for unbound paths and registry-only changes.

### GAP-003: Docs and projections still teach direct completion

- Current failure: README, AGENTS, templates, and task-registry-flow skill projections still describe direct completed status or omit landing-owned completion.
- Good behavior: Given the projected docs and skill files, when searched for `verify-landing`, then each active workflow surface names it as the completion path.
- Forbidden behavior: Given the same surfaces, when searched for direct `status ... completed` guidance, then no active guidance instructs agents to mark completion directly.
- Files involved: `README.md`, `AGENTS.md`, `docs/runtime-schemas.md`, `.codex/agent-governance.toml`, `templates/.codex/agent-governance.toml.template`, `scripts/render-from-config.sh`, `templates/AGENTS.md.template`, `templates/CLAUDE.md.template`, `templates/GEMINI.md.template`, `.agents/skills/task-registry-flow/SKILL.md`, `.agents/skills/task-registry-flow.md`, `.cursor/skills/task-registry-flow/SKILL.md`, `.claude/skills/task-registry-flow/SKILL.md`, `.agents/skills/task-registry-flow/PROJECT.md`, `.cursor/skills/task-registry-flow/PROJECT.md`, `.claude/skills/task-registry-flow/PROJECT.md`.
- Positive test: `rg -n "verify-landing" README.md AGENTS.md docs/runtime-schemas.md templates/AGENTS.md.template templates/CLAUDE.md.template templates/GEMINI.md.template .agents/skills/task-registry-flow/SKILL.md .agents/skills/task-registry-flow.md .cursor/skills/task-registry-flow/SKILL.md .claude/skills/task-registry-flow/SKILL.md .agents/skills/task-registry-flow/PROJECT.md .cursor/skills/task-registry-flow/PROJECT.md .claude/skills/task-registry-flow/PROJECT.md .codex/agent-governance.toml templates/.codex/agent-governance.toml.template`
- Negative test: `! rg -n 'status .*completed|Mark tasks `completed` only' README.md AGENTS.md templates/AGENTS.md.template templates/CLAUDE.md.template .agents/skills/task-registry-flow/SKILL.md .agents/skills/task-registry-flow.md .cursor/skills/task-registry-flow/SKILL.md .claude/skills/task-registry-flow/SKILL.md`
- Data/schema/provenance: Projected governance metadata includes `verify_landing_command`.
- Runtime: N/A; this is agent workflow text and template projection.

## Validation Plan

Focused:

- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml landing_completes_changed_file_tasks -- --nocapture`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml landing_rejects_direct_completed_status -- --nocapture`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml landing_rejects_unbound_or_registry_only_changes -- --nocapture`
- `rg -n "verify-landing" README.md AGENTS.md docs/runtime-schemas.md templates/AGENTS.md.template templates/CLAUDE.md.template templates/GEMINI.md.template .agents/skills/task-registry-flow/SKILL.md .agents/skills/task-registry-flow.md .cursor/skills/task-registry-flow/SKILL.md .claude/skills/task-registry-flow/SKILL.md .agents/skills/task-registry-flow/PROJECT.md .cursor/skills/task-registry-flow/PROJECT.md .claude/skills/task-registry-flow/PROJECT.md .codex/agent-governance.toml templates/.codex/agent-governance.toml.template`
- `! rg -n 'status .*completed|Mark tasks `completed` only' README.md AGENTS.md templates/AGENTS.md.template templates/CLAUDE.md.template .agents/skills/task-registry-flow/SKILL.md .agents/skills/task-registry-flow.md .cursor/skills/task-registry-flow/SKILL.md .claude/skills/task-registry-flow/SKILL.md`
- `.codex/scripts/task-registry source-limit check`

Full:

- `.codex/scripts/task-registry validate`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `bash scripts/test-release-readiness.sh all`
- `.codex/scripts/task-registry verify-chain --format json`
- `.codex/scripts/task-registry metrics`

## Walkthrough Evidence

- `PLAN_ACTIVATE docs/plans/ci-owned-landing-2026-05-31.md` output.
- Focused positive and negative landing test output.
- Documentation positive and negative grep output.
- Full validation output, or exact blocked command and reason.
- `TASK_REPORT PLAN-2026-05-31-ci-owned-landing` output.
- `TASK_METRICS` output.
- `VERIFY_CHAIN` output because activation and landing completion mutate registry receipts.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-05-31-ci-owned-landing"

[[behaviors]]
behavior_id = "B-2026-05-31-landing-G01-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Landing command owns completion"
given = "An active plan with a changed file bound to one task target"
when = "verify-landing runs for that plan and changed file"
then = "linked typed behavior verifiers pass and the selected task becomes completed"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml landing_completes_changed_file_tasks -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml landing_completes_changed_file_tasks -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-landing-G02-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Direct completed status is rejected"
given = "An active task id"
when = "status completed is attempted"
then = "the command fails closed and points to verify-landing"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml landing_rejects_direct_completed_status -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml landing_rejects_direct_completed_status -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-landing-G03-positive"
gap_id = "GAP-002"
polarity = "positive"
title = "Changed files bind to active task targets"
given = "A changed file set containing an active task target"
when = "verify-landing evaluates the changed-file set"
then = "the command selects the bound task and runs its typed behavior verifiers"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml landing_completes_changed_file_tasks -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml landing_completes_changed_file_tasks -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-landing-G03-negative"
gap_id = "GAP-002"
polarity = "negative"
title = "Unbound and registry-only landings are rejected"
given = "Registry-only or off-plan changed files"
when = "verify-landing evaluates the changed-file set"
then = "the command fails before completion status is written"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml landing_rejects_unbound_or_registry_only_changes -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml landing_rejects_unbound_or_registry_only_changes -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-landing-G04-positive"
gap_id = "GAP-003"
polarity = "positive"
title = "Workflow docs name verify-landing"
given = "The active docs, templates, and task-registry-flow skill projections exist"
when = "workflow text is searched"
then = "verify-landing is present and direct completed-status guidance is absent"
confirmation = "rg -n \"verify-landing\" README.md AGENTS.md docs/runtime-schemas.md templates/AGENTS.md.template templates/CLAUDE.md.template templates/GEMINI.md.template .agents/skills/task-registry-flow/SKILL.md .agents/skills/task-registry-flow.md .cursor/skills/task-registry-flow/SKILL.md .claude/skills/task-registry-flow/SKILL.md .agents/skills/task-registry-flow/PROJECT.md .cursor/skills/task-registry-flow/PROJECT.md .claude/skills/task-registry-flow/PROJECT.md .codex/agent-governance.toml templates/.codex/agent-governance.toml.template && ! rg -n 'status .*completed|Mark tasks `completed` only' README.md AGENTS.md templates/AGENTS.md.template templates/CLAUDE.md.template .agents/skills/task-registry-flow/SKILL.md .agents/skills/task-registry-flow.md .cursor/skills/task-registry-flow/SKILL.md .claude/skills/task-registry-flow/SKILL.md"

[[behaviors.verifiers]]
type = "command"
command = "rg -n \"verify-landing\" README.md AGENTS.md docs/runtime-schemas.md templates/AGENTS.md.template templates/CLAUDE.md.template templates/GEMINI.md.template .agents/skills/task-registry-flow/SKILL.md .agents/skills/task-registry-flow.md .cursor/skills/task-registry-flow/SKILL.md .claude/skills/task-registry-flow/SKILL.md .agents/skills/task-registry-flow/PROJECT.md .cursor/skills/task-registry-flow/PROJECT.md .claude/skills/task-registry-flow/PROJECT.md .codex/agent-governance.toml templates/.codex/agent-governance.toml.template"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-landing-G04-negative"
gap_id = "GAP-003"
polarity = "negative"
title = "Workflow docs reject direct completed status guidance"
given = "The active docs, templates, and task-registry-flow skill projections exist"
when = "workflow text is searched for direct completion status instructions"
then = "no active guidance tells agents to use status completed"
confirmation = "! rg -n 'status .*completed|Mark tasks `completed` only' README.md AGENTS.md templates/AGENTS.md.template templates/CLAUDE.md.template .agents/skills/task-registry-flow/SKILL.md .agents/skills/task-registry-flow.md .cursor/skills/task-registry-flow/SKILL.md .claude/skills/task-registry-flow/SKILL.md"

[[behaviors.verifiers]]
type = "command"
command = "! rg -n 'status .*completed|Mark tasks `completed` only' README.md AGENTS.md templates/AGENTS.md.template templates/CLAUDE.md.template .agents/skills/task-registry-flow/SKILL.md .agents/skills/task-registry-flow.md .cursor/skills/task-registry-flow/SKILL.md .claude/skills/task-registry-flow/SKILL.md"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-landing-G05-validation"
gap_id = "GAP-VALIDATION"
polarity = "validation"
title = "Full governance validation passes"
given = "The landing command and workflow docs are implemented"
when = "full project validation runs"
then = "registry validation, cargo tests, release readiness, source limit, receipt chain, and metrics pass"
confirmation = ".codex/scripts/task-registry validate && cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml && bash scripts/test-release-readiness.sh all && .codex/scripts/task-registry verify-chain --format json && .codex/scripts/task-registry metrics"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry validate && cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml && bash scripts/test-release-readiness.sh all && .codex/scripts/task-registry verify-chain --format json && .codex/scripts/task-registry metrics"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-05-31-ci-owned-landing-001"
status = "active"
title = "Implement verify-landing completion authority"
kind = "governance"
reason = "Task completion must be written only through a landing verifier that binds changed files to activated task targets."
acceptance_proof = "Behaviors B-2026-05-31-landing-G01-positive, B-2026-05-31-landing-G02-negative, B-2026-05-31-landing-G03-positive, and B-2026-05-31-landing-G03-negative pass."
behavior_ids = ["B-2026-05-31-landing-G01-positive", "B-2026-05-31-landing-G02-negative", "B-2026-05-31-landing-G03-positive", "B-2026-05-31-landing-G03-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/schema.rs"
object = "CliCommand verify-landing opcode"
required_change = "Add canonical verify-landing command enum value."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/runtime.rs"
object = "CLI dispatcher and usage"
required_change = "Route verify-landing and remove direct status completion from the user-facing completion path."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/activation.rs"
object = "task status transition entrypoint"
required_change = "Reject direct completed status while preserving internal transition validation for landing completion."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/landing.rs"
object = "verify-landing command implementation"
required_change = "Bind changed files to active task targets, reject registry-only or unbound changes, run verifiers, and write completion status."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/model.rs"
object = "RegistryTask completion evidence fields"
required_change = "Add optional landing-owned completion evidence fields."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/receipts.rs"
object = "default receipt command list"
required_change = "Record verify-landing command receipts by default."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/main.rs"
object = "landing module registration"
required_change = "Register the landing module."

[[tasks]]
task_id = "TASK-2026-05-31-ci-owned-landing-002"
status = "active"
title = "Cover landing behavior with focused tests"
kind = "test"
reason = "The landing authority change needs positive and negative tests for completion, direct status denial, and off-plan drift."
acceptance_proof = "Behaviors B-2026-05-31-landing-G01-positive, B-2026-05-31-landing-G02-negative, B-2026-05-31-landing-G03-positive, and B-2026-05-31-landing-G03-negative pass."
behavior_ids = ["B-2026-05-31-landing-G01-positive", "B-2026-05-31-landing-G02-negative", "B-2026-05-31-landing-G03-positive", "B-2026-05-31-landing-G03-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/landing_tests.rs"
object = "landing positive and negative tests"
required_change = "Add focused tests for verify-landing success, direct completed status rejection, and unbound or registry-only changed files."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/mod.rs"
object = "test module registry and completion expectations"
required_change = "Register landing tests and update direct completion expectations."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/activation_terminal_tests.rs"
object = "terminal task setup"
required_change = "Use landing-owned completion setup for terminal activation tests."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/state_transition_tests.rs"
object = "transition test notes"
required_change = "Keep internal transition tests aligned with landing-owned completion behavior."

[[tasks]]
task_id = "TASK-2026-05-31-ci-owned-landing-003"
status = "active"
title = "Update workflow docs and projections"
kind = "documentation"
reason = "Agent-facing docs must not keep teaching direct completed status after completion becomes landing-owned."
acceptance_proof = "Behaviors B-2026-05-31-landing-G04-positive and B-2026-05-31-landing-G04-negative pass."
behavior_ids = ["B-2026-05-31-landing-G04-positive", "B-2026-05-31-landing-G04-negative"]

[[tasks.targets]]
file = "README.md"
object = "daily workflow completion guidance"
required_change = "Replace direct completion status command with verify-landing workflow."

[[tasks.targets]]
file = "AGENTS.md"
object = "gap closure completion instruction"
required_change = "State that verify-landing owns completed status after typed behavior verification."

[[tasks.targets]]
file = "docs/runtime-schemas.md"
object = "runtime completion contract"
required_change = "Document landing-owned completion and registry-only drift rejection."

[[tasks.targets]]
file = ".codex/agent-governance.toml"
object = "task registry command metadata"
required_change = "Add verify_landing_command metadata."

[[tasks.targets]]
file = "templates/.codex/agent-governance.toml.template"
object = "projected task registry metadata"
required_change = "Add verify_landing_command metadata to generated installs."

[[tasks.targets]]
file = "scripts/render-from-config.sh"
object = "project config renderer"
required_change = "Preserve canonical verify-landing metadata in generated docs/config."

[[tasks.targets]]
file = "templates/AGENTS.md.template"
object = "Codex projected completion workflow"
required_change = "State that verify-landing owns completion."

[[tasks.targets]]
file = "templates/CLAUDE.md.template"
object = "Claude projected completion workflow"
required_change = "State that verify-landing owns completion."

[[tasks.targets]]
file = "templates/GEMINI.md.template"
object = "Antigravity projected completion workflow"
required_change = "State that verify-landing owns completion."

[[tasks.targets]]
file = ".agents/skills/task-registry-flow/SKILL.md"
object = "Codex skill completion workflow"
required_change = "Add TASK_VERIFY_LANDING and remove direct completed status guidance."

[[tasks.targets]]
file = ".agents/skills/task-registry-flow.md"
object = "flat Codex skill completion workflow"
required_change = "Add TASK_VERIFY_LANDING and remove direct completed status guidance."

[[tasks.targets]]
file = ".cursor/skills/task-registry-flow/SKILL.md"
object = "Cursor skill completion workflow"
required_change = "Add TASK_VERIFY_LANDING and remove direct completed status guidance."

[[tasks.targets]]
file = ".claude/skills/task-registry-flow/SKILL.md"
object = "Claude skill completion workflow"
required_change = "Add TASK_VERIFY_LANDING and remove direct completed status guidance."

[[tasks.targets]]
file = ".agents/skills/task-registry-flow/PROJECT.md"
object = "Codex skill project opcode table"
required_change = "Add verify-landing opcode row."

[[tasks.targets]]
file = ".cursor/skills/task-registry-flow/PROJECT.md"
object = "Cursor skill project opcode table"
required_change = "Add verify-landing opcode row."

[[tasks.targets]]
file = ".claude/skills/task-registry-flow/PROJECT.md"
object = "Claude skill project opcode table"
required_change = "Add verify-landing opcode row."

[[tasks]]
task_id = "TASK-2026-05-31-ci-owned-landing-004"
status = "active"
title = "Run final governance validation"
kind = "validation"
reason = "The new completion authority touches shared CLI, registry, docs, and release surfaces."
acceptance_proof = "Behavior B-2026-05-31-landing-G05-validation passes."
behavior_ids = ["B-2026-05-31-landing-G05-validation"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/Cargo.toml"
object = "Cargo validation target"
required_change = "Run focused and full cargo validation for the task-registry CLI."

[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release source declaration"
required_change = "Declare new landing Rust sources so release readiness cannot ship undeclared source."

[[tasks.targets]]
file = "docs/plans/ci-owned-landing-2026-05-31.md"
object = "validation evidence contract"
required_change = "Keep this closure contract as the task-bound validation evidence surface."
```
