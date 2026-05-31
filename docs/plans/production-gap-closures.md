# Production Gap Closures

## Approved Scope

Close the three review-blocking production gaps in the agent-governance plugin:

- The mutation hook must not deadlock plan drafting, plan repair, or plan activation when a valid Task Manifest exists in `docs/plans/` but has not been activated yet.
- The rendered `.codex/scripts/task-registry` wrapper must execute the Rust registry CLI from the repository root regardless of the caller's current directory.
- `--merge` install output must match `status.sh --strict`; stale legacy governance files must be removed during merge rather than preserved.

Out of scope:

- Restoring `--overlay`, preserving legacy hook locations, or adding compatibility shims.
- Changing task-registry schema shape beyond what the existing implementation already supports.
- Replacing the plugin-owned Rust registry CLI.

Primitive change gate: N/A. This changes governance tooling behavior, shell wrapper execution, and tests; it does not change runtime product primitives.

## Required Change Checklist

- [MODIFY] `rust/task-registry-flow-cli/src/main.rs` - move mutation-hook implementation out of the 1600-line main file and keep CLI dispatch stable. Acceptance proof: `cargo test --manifest-path rust/task-registry-flow-cli/Cargo.toml` and source-limit check pass.
- [NEW] `rust/task-registry-flow-cli/src/mutation_hook.rs` - inspect hook payload paths before full validation, allow deterministic governance repair/activation writes, and keep implementation writes target-bound. Acceptance proof: positive and negative hook tests pass.
- [MODIFY] `rust/task-registry-flow-cli/src/tests.rs` - add behavior tests for governance-write allow paths and forbidden implementation-write paths. Acceptance proof: hook test filter and full Rust tests pass.
- [MODIFY] `templates/.codex/scripts/task-registry.template` - `cd` to repo root before invoking `cargo run`. Acceptance proof: installed wrapper validates from a nested directory without creating nested registry files.
- [MODIFY] `scripts/render-from-config.sh` - remove stale legacy files during `--merge` and `--force`; keep dry-run as projection-only. Acceptance proof: install-mode smoke test passes.
- [MODIFY] `scripts/test-install-modes.sh` - assert merge removes stale files, dry-run only projects removals, and nested wrapper execution uses the repo root. Acceptance proof: script passes.
- [MODIFY] `README.md` - document merge stale removal and wrapper behavior. Acceptance proof: docs match test behavior.
- [MODIFY] `examples/spectrum-arcana-overlay.md` - replace stale-preservation language with hard-cutover language. Acceptance proof: docs match test behavior.

## Per-Gap Success Criteria

### Gap 1: Mutation hook deadlock

Good behavior:

- Empty or non-write hook payloads are allowed.
- Deterministic governance writes under `.agents/`, `.codex/`, `.cursor/`, `docs/plans/`, `docs/task-registry/`, `tools/agent-governance/`, plus `AGENTS.md`, `GEMINI.md`, and `project.config.toml`, are allowed before full validation.
- `.codex/scripts/task-registry activate docs/plans/example.md` is allowed even when `validate_all` would fail because the same plan is not activated yet.
- Implementation writes are allowed only when bound to a planned or active registry task target.
- Uncertain writes, malformed JSON, outside-repo paths, and implementation writes without active/planned task targets are denied.

### Gap 2: Wrapper cwd

Good behavior:

- `.codex/scripts/task-registry validate` succeeds from repo root and nested directories.
- Registry files and event receipts are read/written under the repo root only.
- No `docs/task-registry.toml` or `docs/task-registry/events.jsonl` is created under the nested caller directory.

### Gap 3: Merge/status mismatch

Good behavior:

- `--merge` removes `.codex/settings.toml`, `.codex/hooks/user-plan-approval.toml`, `.gemini/settings.json`, root `hooks.json`, and `tools/antigravity/pre-tool-use-gap-closure.sh`.
- `--merge` preserves valid `docs/task-registry.toml` and `docs/task-registry/events.jsonl`.
- `--dry-run` reports `would-remove-stale` and does not mutate tracked workspace state.
- `status.sh --strict` remains strict: recreated stale files are failures.

## Validation Plan

Focused tests:

- `cargo test --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_`
- `bash scripts/test-install-modes.sh`

Full gates:

- `bash -n scripts/install-to-workspace.sh scripts/render-from-config.sh scripts/status.sh scripts/test-install-modes.sh templates/.codex/scripts/task-registry.template templates/.cursor/hooks/gap-closure-gate.sh.template templates/tools/agent-governance/pre-tool-use-gap-closure.sh.template`
- `cargo test --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `cargo clippy --manifest-path rust/task-registry-flow-cli/Cargo.toml --all-targets -- -D warnings`
- `cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check`
- `cargo run --quiet --manifest-path rust/task-registry-flow-cli/Cargo.toml -- source-limit check --root .`
- `agy plugin validate .`
- `git diff --check`

Failing evidence means the corresponding production gap remains open.

## Walkthrough Evidence

Capture:

- Plan activation output.
- Hook positive and negative test result summary.
- Install-mode smoke test result summary.
- Full validation command result summary.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-05-30-production-gap-closures"

[[behaviors]]
behavior_id = "B-2026-05-30-hook-governance-before-validation"
gap_id = "GAP-2026-05-30-hook-governance-before-validation"
polarity = "positive"
title = "Mutation hook allows governance repair while preserving implementation guards"
given = "A repo with an unactivated plan manifest that makes full validation fail"
when = "The mutation hook receives governance and implementation write payloads"
then = "Governance writes and activation commands are allowed, while unbound implementation writes and unsafe payloads are denied"
confirmation = "cargo test --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-30-wrapper-root-cwd"
gap_id = "GAP-2026-05-30-wrapper-root-cwd"
polarity = "positive"
title = "Task-registry wrapper runs from repo root"
given = "An installed workspace and a nested current directory"
when = "The rendered task-registry wrapper runs validate"
then = "The command uses repo-root registry files and does not create nested registry artifacts"
confirmation = "bash scripts/test-install-modes.sh"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-install-modes.sh"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-30-merge-status-hard-cutover"
gap_id = "GAP-2026-05-30-merge-status-hard-cutover"
polarity = "positive"
title = "Merge install and status agree on stale-file removal"
given = "A workspace containing stale legacy governance files"
when = "The installer runs in dry-run, merge, and force modes"
then = "Dry-run projects stale removal, merge and force remove stale files, and no preserve-stale output remains"
confirmation = "bash scripts/test-install-modes.sh"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-install-modes.sh"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-05-30-production-gap-closures-001"
title = "Repair mutation hook validation order"
status = "planned"
kind = "implementation"
reason = "The hook currently runs full validation before path inspection and can block plan activation."
acceptance_proof = "Behavior B-2026-05-30-hook-governance-before-validation passes."
behavior_ids = ["B-2026-05-30-hook-governance-before-validation"]
[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/main.rs"
object = "verify_mutation_hook_dispatch"
required_change = "Delegate hook verification to a module without running full validation before governance path inspection."
[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/mutation_hook.rs"
object = "mutation_hook_verifier"
required_change = "Implement governance-first hook verification with negative guards for unsafe writes."
[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests.rs"
object = "mutation_hook_behavior_tests"
required_change = "Add positive and negative tests for governance writes, activation commands, and denied implementation writes."

[[tasks]]
task_id = "TASK-2026-05-30-production-gap-closures-002"
title = "Run rendered registry wrapper from repo root"
status = "planned"
kind = "implementation"
reason = "The wrapper computes the repo root but currently leaves the Rust CLI running from the caller directory."
acceptance_proof = "Behavior B-2026-05-30-wrapper-root-cwd passes."
behavior_ids = ["B-2026-05-30-wrapper-root-cwd"]
[[tasks.targets]]
file = "templates/.codex/scripts/task-registry.template"
object = "repo_root_execution"
required_change = "Change to the repo root before invoking cargo run."
[[tasks.targets]]
file = "scripts/test-install-modes.sh"
object = "nested_wrapper_validation"
required_change = "Assert nested wrapper execution reads and writes only repo-root registry artifacts."

[[tasks]]
task_id = "TASK-2026-05-30-production-gap-closures-003"
title = "Align merge install with strict status"
status = "planned"
kind = "implementation"
reason = "Merge currently preserves stale files that strict status rejects."
acceptance_proof = "Behavior B-2026-05-30-merge-status-hard-cutover passes."
behavior_ids = ["B-2026-05-30-merge-status-hard-cutover"]
[[tasks.targets]]
file = "scripts/render-from-config.sh"
object = "merge_stale_file_removal"
required_change = "Remove stale legacy governance files during merge and force while keeping dry-run projection-only."
[[tasks.targets]]
file = "scripts/test-install-modes.sh"
object = "merge_status_alignment_tests"
required_change = "Assert merge removes stale files and never reports preserve-stale."
[[tasks.targets]]
file = "README.md"
object = "install_mode_documentation"
required_change = "Document merge stale removal and root-wrapper behavior."
[[tasks.targets]]
file = "examples/spectrum-arcana-overlay.md"
object = "reference_install_mode_documentation"
required_change = "Update reference docs to describe hard-cutover merge behavior."
```
