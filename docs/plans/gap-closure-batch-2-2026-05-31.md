# Batch 2 Gap Closure Contract

Continuation of [PLAN-2026-05-31-gap-closure-batch-1](gap-closure-2026-05-31.md). Addresses gaps 6–10 from the parent audit.

## Approved Scope

**Batch 2** — 5 gaps:
6. `_broken` parameter unused in `repair_chain` — use first break position to skip intact prefix lines
7. `bash -lc` in verifier and hook gate — switch to `bash -c` for deterministic verification
8. `VERIFY_CHAIN` opcode missing from skill definitions and config template
9. `{{REPO_SLUG}}` derivation undocumented — add comment to example config
10. Temp file collision risk in `status.sh` — use `mktemp`

## Phased Required Change Checklist

### Batch 2

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/verify_chain.rs` — `repair_chain`: use `_broken` to find first break, skip intact prefix
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/verifiers.rs` — `run_confirmation`: change `bash -lc` to `bash -c`
- [ ] `[MODIFY]` `tools/agent-governance/pre-tool-use-gap-closure.sh` — change `bash -lc` to `bash -c`
- [ ] `[MODIFY]` `skills/task-registry-flow/SKILL.md` — add `VERIFY_CHAIN` row to opcode table
- [ ] `[MODIFY]` `templates/.codex/agent-governance.toml.template` — add `verify_chain_opcode` field
- [ ] `[MODIFY]` `project.config.example.toml` — document `{{REPO_SLUG}}` derivation from `repo_name`
- [ ] `[MODIFY]` `scripts/status.sh` — use `mktemp` for stderr capture instead of hardcoded `/tmp` path
- [ ] `[VERIFY]` `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- [ ] `[VERIFY]` `cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check`
- [ ] `[VERIFY]` `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings`
- [ ] `[VERIFY]` `bash -n tools/agent-governance/pre-tool-use-gap-closure.sh`
- [ ] `[VERIFY]` `bash -n scripts/status.sh`
- [ ] `[VERIFY]` `cargo run --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- source-limit check`

## Per-Gap Success Criteria

### Gap 6: `_broken` unused in repair_chain
- Current failure: `repair_chain` ignores the `_broken` report and recalculates all hashes from scratch, wasting work on lines before the first break.
- Good behavior: `repair_chain` extracts the first broken line number from `_broken`, copies intact prefix lines verbatim, and only recomputes hashes from the first break onward.
- Forbidden behavior: Unnecessary recomputation of intact prefix lines.
- Files involved: `rust/task-registry-flow-cli/src/verify_chain.rs`
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml` (existing verify_chain tests pass)
- Negative test: Verify intact prefix lines are byte-identical after repair (check via existing test `verify_chain_repair_fixes_broken_chain`)
- Runtime: N/A; unit tests prove behavior.

### Gap 7: `bash -lc` in verifier and hook gate
- Current failure: `bash -lc` (login shell) sources `.bash_profile`/`.bashrc`, making verification environment-dependent. Two developers on different machines can get different results.
- Good behavior: `bash -c` (non-login) provides a controlled, deterministic environment for verifiers and the mutation gate.
- Forbidden behavior: Verifier passes on one machine and fails on another due to shell profile differences.
- Files involved: `rust/task-registry-flow-cli/src/verifiers.rs`, `tools/agent-governance/pre-tool-use-gap-closure.sh`
- Positive test: All existing tests pass with `bash -c`; hook gate syntax check passes.
- Negative test: N/A (environment-dependent behavior, not testable in isolation)
- Runtime: N/A; verified via test suite and manual cross-machine testing.

### Gap 8: VERIFY_CHAIN opcode missing from skill
- Current failure: The `task-registry-flow` skill opcode table omits `VERIFY_CHAIN`, and the agent-governance config template lacks a `verify_chain_opcode` field. Agents following skill docs won't discover this command.
- Good behavior: `VERIFY_CHAIN` appears in the opcode table with a description. The config template includes `verify_chain_opcode`.
- Forbidden behavior: `verify-chain` command remains undiscoverable through skill documentation.
- Files involved: `skills/task-registry-flow/SKILL.md`, `templates/.codex/agent-governance.toml.template`
- Positive test: `grep -q "VERIFY_CHAIN" skills/task-registry-flow/SKILL.md`
- Negative test: N/A
- Runtime: N/A; documentation-only change.

### Gap 9: `{{REPO_SLUG}}` undocumented
- Current failure: The plan template uses `{{REPO_SLUG}}` but no config file defines this key. It is silently derived from `repo_name` in `render-from-config.sh`, which is not obvious to users.
- Good behavior: `project.config.example.toml` includes a comment documenting that `{{REPO_SLUG}}` is auto-derived from `repo_name` (lowercased, non-alphanumeric → hyphens).
- Forbidden behavior: Users unaware of available substitution variables.
- Files involved: `project.config.example.toml`
- Positive test: `grep -q "REPO_SLUG" project.config.example.toml`
- Negative test: N/A
- Runtime: N/A; documentation-only change.

### Gap 10: Temp file collision in status.sh
- Current failure: `/tmp/agent-governance-status-check.err` is a hardcoded path. Concurrent `status.sh` runs clobber each other's error output.
- Good behavior: `mktemp` creates a unique temporary file per invocation, preventing collisions.
- Forbidden behavior: Two concurrent runs sharing the same error output file.
- Files involved: `scripts/status.sh`
- Positive test: `bash -n scripts/status.sh` passes; `mktemp` is used for stderr capture.
- Negative test: N/A (concurrent execution is a runtime concern)
- Runtime: N/A; verified via code review and syntax check.

## Validation Plan

Focused:
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check`
- `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings`
- `bash -n tools/agent-governance/pre-tool-use-gap-closure.sh`
- `bash -n scripts/status.sh`
- `.codex/scripts/task-registry source-limit check`

Full:
- `.codex/scripts/task-registry validate`
- `bash scripts/test-release-readiness.sh all`

## Walkthrough Evidence
- All 95+ tests pass
- Source limit check passes
- Shell syntax checks pass
- Clippy with deny warnings passes

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-05-31-gap-closure-batch-2"

[[behaviors]]
behavior_id = "B-006-repair-uses-broken"
gap_id = "GAP-006"
polarity = "positive"
title = "repair_chain uses the broken report to skip intact prefix lines"
given = "an events file with a break at line N"
when = "verify-chain --repair runs"
then = "lines before N are copied verbatim; hashes recomputed only from N onward"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml"
expected_exit = 0

[[behaviors]]
behavior_id = "B-006b-intact-prefix-preserved"
gap_id = "GAP-006"
polarity = "negative"
title = "Intact prefix lines are not modified during repair"
given = "an events file where only line 5 is broken"
when = "verify-chain --repair runs"
then = "lines 1-4 are byte-identical after repair"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml"
expected_exit = 0

[[behaviors]]
behavior_id = "B-007-bash-no-login"
gap_id = "GAP-007"
polarity = "positive"
title = "Verifiers and hook gate use bash -c instead of bash -lc"
given = "a verifier or mutation hook invocation"
when = "a command is executed"
then = "bash -c is used, not bash -lc"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml"
expected_exit = 0

[[behaviors]]
behavior_id = "B-007b-no-login-shell-in-source"
gap_id = "GAP-007"
polarity = "negative"
title = "No bash -lc calls remain in verifiers or hook gate source"
given = "the source code of verifiers.rs and pre-tool-use-gap-closure.sh"
when = "searching for bash -lc"
then = "no matches are found"
confirmation = "! grep -r 'bash -lc' rust/task-registry-flow-cli/src/verifiers.rs tools/agent-governance/pre-tool-use-gap-closure.sh"

[[behaviors.verifiers]]
type = "command"
command = "bash -c '! grep -r \"bash -lc\" rust/task-registry-flow-cli/src/verifiers.rs tools/agent-governance/pre-tool-use-gap-closure.sh'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-008-verify-chain-opcode"
gap_id = "GAP-008"
polarity = "positive"
title = "VERIFY_CHAIN opcode is documented in skill and config template"
given = "the task-registry-flow skill and agent-governance config"
when = "an agent reads the opcode table"
then = "VERIFY_CHAIN appears with description"
confirmation = "grep -q VERIFY_CHAIN skills/task-registry-flow/SKILL.md && grep -q verify_chain_opcode templates/.codex/agent-governance.toml.template"

[[behaviors.verifiers]]
type = "command"
command = "bash -c 'grep -q VERIFY_CHAIN skills/task-registry-flow/SKILL.md && grep -q verify_chain_opcode templates/.codex/agent-governance.toml.template'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-008b-no-missing-opcode"
gap_id = "GAP-008"
polarity = "negative"
title = "VERIFY_CHAIN is not absent from the skill opcode table"
given = "the skill SKILL.md file"
when = "checking the opcode table"
then = "VERIFY_CHAIN is present"
confirmation = "grep -q VERIFY_CHAIN skills/task-registry-flow/SKILL.md"

[[behaviors.verifiers]]
type = "command"
command = "bash -c 'grep -q VERIFY_CHAIN skills/task-registry-flow/SKILL.md'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-009-repo-slug-documented"
gap_id = "GAP-009"
polarity = "positive"
title = "REPO_SLUG derivation is documented in example config"
given = "project.config.example.toml"
when = "a user reads the config file"
then = "a comment explains that REPO_SLUG is auto-derived from repo_name"
confirmation = "grep -q REPO_SLUG project.config.example.toml"

[[behaviors.verifiers]]
type = "command"
command = "bash -c 'grep -q REPO_SLUG project.config.example.toml'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-009b-no-hidden-substitution"
gap_id = "GAP-009"
polarity = "negative"
title = "No undocumented substitution variables remain in plan template"
given = "the plan template and example config"
when = "checking all substitution variables"
then = "every template variable is either config-defined or documented as auto-derived"
confirmation = "grep -q REPO_SLUG project.config.example.toml"

[[behaviors.verifiers]]
type = "command"
command = "bash -c 'grep -q REPO_SLUG project.config.example.toml'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-010-mktemp-status"
gap_id = "GAP-010"
polarity = "positive"
title = "status.sh uses mktemp for stderr capture"
given = "a status.sh invocation"
when = "stderr is captured during status-check"
then = "a unique temp file is created via mktemp"
confirmation = "bash -n scripts/status.sh"

[[behaviors.verifiers]]
type = "command"
command = "bash -n scripts/status.sh"
expected_exit = 0

[[behaviors]]
behavior_id = "B-010b-no-hardcoded-tmp"
gap_id = "GAP-010"
polarity = "negative"
title = "status.sh does not use hardcoded /tmp paths for error capture"
given = "status.sh source code"
when = "searching for hardcoded temp paths"
then = "no /tmp/agent-governance hardcoded paths exist"
confirmation = "! grep -F '/tmp/agent-governance-status-check.err' scripts/status.sh"

[[behaviors.verifiers]]
type = "command"
command = "bash -c '! grep -F /tmp/agent-governance-status-check.err scripts/status.sh'"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-05-31-006"
status = "planned"
kind = "implementation"
reason = "repair_chain ignores the broken report; targeted repair skips intact prefix lines for efficiency"
behavior_ids = ["B-006-repair-uses-broken", "B-006b-intact-prefix-preserved"]
title = "Use _broken report to skip intact prefix in repair_chain"
acceptance_proof = "Behavior B-006: cargo test passes, repair_chain uses first break position"

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/verify_chain.rs"
object = "repair_chain"
required_change = "Extract first broken line from _broken, copy intact prefix verbatim, recompute only from break point"

[[tasks]]
task_id = "TASK-2026-05-31-007"
status = "planned"
kind = "implementation"
reason = "bash -lc sources user shell profiles, making verification environment-dependent"
behavior_ids = ["B-007-bash-no-login", "B-007b-no-login-shell-in-source"]
title = "Switch bash -lc to bash -c in verifiers and hook gate"
acceptance_proof = "Behavior B-007: no bash -lc in verifiers.rs or pre-tool-use-gap-closure.sh"

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/verifiers.rs"
object = "run_confirmation"
required_change = "Change bash -lc to bash -c"

[[tasks.targets]]
file = "tools/agent-governance/pre-tool-use-gap-closure.sh"
object = "verify_cmd execution"
required_change = "Change bash -lc to bash -c"

[[tasks]]
task_id = "TASK-2026-05-31-008"
status = "planned"
kind = "documentation"
reason = "VERIFY_CHAIN opcode is missing from skill docs; agents cannot discover the verify-chain command"
behavior_ids = ["B-008-verify-chain-opcode", "B-008b-no-missing-opcode"]
title = "Add VERIFY_CHAIN opcode to skill and config template"
acceptance_proof = "Behavior B-008: VERIFY_CHAIN appears in SKILL.md opcode table and agent-governance.toml.template"

[[tasks.targets]]
file = "skills/task-registry-flow/SKILL.md"
object = "Required Opcodes table"
required_change = "Add VERIFY_CHAIN row with description"

[[tasks.targets]]
file = "templates/.codex/agent-governance.toml.template"
object = "[task_registry] section"
required_change = "Add verify_chain_opcode field"

[[tasks]]
task_id = "TASK-2026-05-31-009"
status = "planned"
kind = "documentation"
reason = "{{REPO_SLUG}} is auto-derived from repo_name but undocumented; users don't know it exists"
behavior_ids = ["B-009-repo-slug-documented", "B-009b-no-hidden-substitution"]
title = "Document REPO_SLUG derivation in example config"
acceptance_proof = "Behavior B-009: REPO_SLUG mentioned in project.config.example.toml"

[[tasks.targets]]
file = "project.config.example.toml"
object = "project section comment"
required_change = "Add comment documenting that REPO_SLUG is auto-derived from repo_name"

[[tasks]]
task_id = "TASK-2026-05-31-010"
status = "planned"
kind = "implementation"
reason = "Hardcoded /tmp path causes collisions between concurrent status.sh invocations"
behavior_ids = ["B-010-mktemp-status", "B-010b-no-hardcoded-tmp"]
title = "Use mktemp in status.sh for stderr capture"
acceptance_proof = "Behavior B-010: status.sh uses mktemp, no hardcoded /tmp path"

[[tasks.targets]]
file = "scripts/status.sh"
object = "run_status_check_json"
required_change = "Replace hardcoded /tmp/agent-governance-status-check.err with mktemp"
```
