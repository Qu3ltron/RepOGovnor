# Post-Rollback Governance Hardening Gap Closure Contract

## Approved Scope

Close the surfaced post-rollback gaps in this repo only:

1. Mutation hooks allow write-intent shell payloads when no path is extracted.
2. Receipt append computes previous hash before locking and can fork under concurrency.
3. Required receipt append failures are ignored.
4. Hook template still emits weak JSON escaping.
5. Release manifest coverage omits Nix and Claude release surfaces.
6. Nix package omits runtime templates, skills, hooks, and metadata.
7. Auto-update module rebuild path is not root-safe or rollback-safe.
8. Verify-chain treats unchained v2 receipts as warnings and release gates miss them.
9. Re-activation can rewrite terminal task metadata.

Out of scope: new user-facing plugin features, legacy compatibility shims, accepting schema v1 receipts at runtime, and changes outside `/home/hasnamuss/reclaimed/work/Governance-plugin`.

## Phased Required Change Checklist

### Phase 0: Activation and safety
- [ ] `[NEW]` `docs/plans/post-rollback-hardening-2026-05-31.md` - `closure_contract`: activate this contract before implementation; acceptance proof: `.codex/scripts/task-registry activate docs/plans/post-rollback-hardening-2026-05-31.md`.
- [ ] `[VERIFY]` `.codex/agent-governance.toml` - `workspace_boundary`: confirm repo root and mutation root match; acceptance proof: `pwd && git rev-parse --show-toplevel`.
- [ ] `[VERIFY]` `rust/task-registry-flow-cli/src/tests/mod.rs` - `line_budget`: add new test modules instead of growing the 1546-line file; acceptance proof: `.codex/scripts/task-registry source-limit check`.

### Phase 1: Mutation hook fail-closed behavior
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/mutation_hook.rs` - `shell_write_detection`: deny write-intent commands when no deterministic path is extracted; acceptance proof: behavior `B-H01-negative`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/hook_io.rs` - `payload_contract`: preserve canonical format validation while supporting stricter shell denial; acceptance proof: behavior `B-H01-positive`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/tests/hook_command_tests.rs` - `hook_command_negative_suite`: cover inline interpreter, heredoc, nested shell, and read-only command cases; acceptance proof: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_command_`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/mod.rs` - `test_module_registration`: register new hook command test module only; acceptance proof: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_command_`.
- [ ] `[MODIFY]` `templates/tools/agent-governance/pre-tool-use-gap-closure.sh.template` - `json_escape`: use canonical JSON serialization for denial reasons; acceptance proof: behavior `B-H04-negative`.
- [ ] `[MODIFY]` `tools/agent-governance/pre-tool-use-gap-closure.sh` - `json_escape_parity`: keep live hook and template aligned; acceptance proof: `bash -n tools/agent-governance/pre-tool-use-gap-closure.sh`.
- [ ] `[MODIFY]` `scripts/test-install-modes.sh` - `rendered_hook_json_negative`: prove rendered denial JSON parses with control characters; acceptance proof: `bash scripts/test-install-modes.sh`.

### Phase 2: Receipt durability and chain migration
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/runtime.rs` - `append_event_lock_order`: lock before reading previous hash, hashing, appending, and syncing; acceptance proof: behavior `B-H02-positive`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/cli.rs` - `required_receipt_errors`: fail commands when required receipt append fails; acceptance proof: behavior `B-H03-negative`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/receipts.rs` - `receipt_policy`: return append errors to callers and preserve explicit read-only receipt policy; acceptance proof: behavior `B-H03-positive`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/metrics.rs` - `unchained_failure_accounting`: count unchained v2 receipts as failed receipt state; acceptance proof: behavior `B-H08-negative`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/verify_chain.rs` - `strict_chain_and_repair`: fail on unchained v2 receipts and repair parseable v2 chain fields; acceptance proof: behavior `B-H08-positive`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/tests/receipt_chain_tests.rs` - `receipt_chain_suite`: add concurrency, lock-failure, and CLI receipt-failure tests; acceptance proof: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml receipt_chain_`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/verify_chain_tests.rs` - `strict_unchained_suite`: add strict unchained and repair negative tests; acceptance proof: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml verify_chain_`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/mod.rs` - `receipt_chain_module_registration`: register new receipt-chain test module only; acceptance proof: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml receipt_chain_`.
- [ ] `[MODIFY]` `docs/task-registry/events.jsonl` - `receipt_ledger_migration`: migrate local receipts to fully chained schema v2; acceptance proof: `.codex/scripts/task-registry verify-chain --format json`.
- [ ] `[MODIFY]` `docs/runtime-schemas.md` - `receipt_chain_contract`: document strict v2-only chained receipt state; acceptance proof: `rg -n "unchained|verify-chain" docs/runtime-schemas.md`.
- [ ] `[MODIFY]` `scripts/status.sh` - `chain_gate`: include verify-chain in strict and release-source posture; acceptance proof: `scripts/status.sh --strict && scripts/status.sh --release-source`.

### Phase 3: Release manifest coverage
- [ ] `[MODIFY]` `REQUIREMENTS.toml` - `release_source_required`: add Nix, Claude, template, hook, and new test files; acceptance proof: behavior `B-H05-positive`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/release_checks.rs` - `governed_source_discovery`: discover governed release surfaces beyond Rust files; acceptance proof: behavior `B-H05-negative`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/schema.rs` - `release_check_id`: add canonical governed-source release check id; acceptance proof: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_source_`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/tests/release_source_tests.rs` - `release_source_negative_suite`: cover omitted Nix, Claude, hook, and template files; acceptance proof: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_source_`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/mod.rs` - `release_source_module_registration`: register new release-source test module only; acceptance proof: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_source_`.
- [ ] `[MODIFY]` `scripts/test-release-readiness.sh` - `governed_source_negative_fixtures`: prove release readiness fails when governed release surfaces are omitted; acceptance proof: `bash scripts/test-release-readiness.sh all`.
- [ ] `[MODIFY]` `scripts/status.sh` - `release_source_gate`: surface governed-source failures through release-source status; acceptance proof: `scripts/status.sh --release-source`.

### Phase 4: Nix package and module hardening
- [ ] `[MODIFY]` `package.nix` - `runtime_asset_output`: install binary, scripts, templates, skills, hooks, metadata, and docs under stable output paths; acceptance proof: behavior `B-H06-positive`.
- [ ] `[MODIFY]` `flake.nix` - `module_exports`: export package app plus `agent-governance` and `auto-update` modules; acceptance proof: `nix flake check --no-build --all-systems`.
- [ ] `[NEW]` `modules/nixos/agent-governance.nix` - `runtime_install_module`: expose packaged CLI and asset root through NixOS; acceptance proof: behavior `B-H06-positive`.
- [ ] `[MODIFY]` `modules/nixos/agent-governance-auto-update.nix` - `root_safe_rollback`: remove hardcoded user default, require root-safe rebuild, backup lock, validate, and rollback on failure; acceptance proof: behavior `B-H07-positive`.
- [ ] `[MODIFY]` `scripts/test-release-readiness.sh` - `nix_asset_negative_fixtures`: prove package assets and auto-update rollback guards exist; acceptance proof: `bash scripts/test-release-readiness.sh all`.
- [ ] `[MODIFY]` `README.md` - `nix_consumption_contract`: document package asset layout and module usage; acceptance proof: `rg -n "agent-governance.nix|share/agent-governance" README.md`.

### Phase 5: Terminal task immutability
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/activation.rs` - `terminal_reactivation_guard`: reject terminal task rewrites unless activation is idempotent; acceptance proof: behavior `B-H09-negative`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/validation.rs` - `terminal_hash_policy`: remove terminal hash exceptions that permit metadata drift; acceptance proof: behavior `B-H09-positive`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/tests/activation_terminal_tests.rs` - `terminal_task_suite`: cover completed/cancelled rewrite denial and idempotent activation; acceptance proof: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_terminal_`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/mod.rs` - `activation_terminal_module_registration`: register new terminal activation test module only; acceptance proof: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_terminal_`.

### Phase 6: Final validation and handoff
- [ ] `[VERIFY]` `docs/task-registry.toml` - `registry_validation`: `.codex/scripts/task-registry validate`.
- [ ] `[VERIFY]` `docs/task-registry/events.jsonl` - `receipt_chain_validation`: `.codex/scripts/task-registry verify-chain --format json`.
- [ ] `[VERIFY]` `rust/task-registry-flow-cli` - `rust_quality`: `cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check && cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml && cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings`.
- [ ] `[VERIFY]` `scripts/test-install-modes.sh` - `installer_runtime_validation`: `bash scripts/test-install-modes.sh`.
- [ ] `[VERIFY]` `scripts/test-release-readiness.sh` - `release_readiness_validation`: `bash scripts/test-release-readiness.sh all`.
- [ ] `[VERIFY]` `flake.nix` - `nix_validation`: `nix build .#task-registry-flow --no-link && nix flake check --no-build --all-systems`.
- [ ] `[VERIFY]` `scripts/status.sh` - `posture_validation`: `scripts/status.sh --strict && scripts/status.sh --release-source`.

## Per-Gap Success Criteria

### GAP-H01: Mutation hook shell payloads fail closed
- Current failure: Write-intent Bash payloads can pass when path extraction misses inline interpreter writes.
- Good behavior: Given a Bash payload with write intent and no deterministic target path, when `verify-mutation-hook` runs, then it denies with a structured reason.
- Forbidden behavior: Inline Python, Node, heredoc, nested shell, or patch write intent passes with no active target.
- Files involved: `rust/task-registry-flow-cli/src/mutation_hook.rs`, `rust/task-registry-flow-cli/src/hook_io.rs`, `rust/task-registry-flow-cli/src/tests/hook_command_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_command_allows_read_only_without_path`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_command_denies_`.
- Data/schema/provenance: No schema change; denial receipts keep v2 format.
- Runtime: Hook denies uncertain write payloads before registry target matching.

### GAP-H02: Receipt append is atomic under concurrency
- Current failure: Previous hash is computed before the event file lock.
- Good behavior: Given concurrent receipt writers, when both append, then the chain remains linear and valid.
- Forbidden behavior: Two events share the same previous hash after concurrent append.
- Files involved: `rust/task-registry-flow-cli/src/runtime.rs`, `rust/task-registry-flow-cli/src/tests/receipt_chain_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml receipt_chain_concurrent_writers_preserve_chain`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml receipt_chain_locked_file_fails_append`.
- Data/schema/provenance: v2 receipt hash semantics remain canonical.
- Runtime: Lock covers read previous hash, hash computation, write, and sync.

### GAP-H03: Required receipt write failures fail commands
- Current failure: CLI ignores receipt append errors.
- Good behavior: Given a mutating command whose required receipt cannot be written, when the command runs, then it exits nonzero.
- Forbidden behavior: Command succeeds while required receipt append failed or JSON reports a receipt that was not written.
- Files involved: `rust/task-registry-flow-cli/src/cli.rs`, `rust/task-registry-flow-cli/src/receipts.rs`, `rust/task-registry-flow-cli/src/tests/receipt_chain_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml receipt_chain_explicit_read_only_policy`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml receipt_chain_required_failure_fails_command`.
- Data/schema/provenance: Required receipt results reflect actual persisted state.
- Runtime: Mutating command success requires receipt append success.

### GAP-H04: Rendered hook template uses canonical JSON escaping
- Current failure: Live hook is hardened but the template still uses manual shell escaping.
- Good behavior: Given rendered hook denial reasons with control characters, when parsed as JSON, then every supported agent format is valid.
- Forbidden behavior: Rendered hook emits invalid JSON for quotes, slashes, tabs, carriage returns, or newlines.
- Files involved: `templates/tools/agent-governance/pre-tool-use-gap-closure.sh.template`, `tools/agent-governance/pre-tool-use-gap-closure.sh`, `scripts/test-install-modes.sh`.
- Positive test: `bash scripts/test-install-modes.sh`.
- Negative test: `bash scripts/test-install-modes.sh` with generated control-character denial fixtures.
- Data/schema/provenance: Hook output remains canonical per agent format.
- Runtime: Installed workspaces inherit the hardened hook from templates.

### GAP-H05: Release manifest covers all governed release surfaces
- Current failure: Nix and Claude release surfaces can be omitted from `REQUIREMENTS.toml` while release-source status passes.
- Good behavior: Given any governed release surface, when it is absent from `release_source.required`, then `release-check all` fails.
- Forbidden behavior: New Nix, Claude, hook, template, or test files are tracked but ungoverned.
- Files involved: `REQUIREMENTS.toml`, `rust/task-registry-flow-cli/src/release_checks.rs`, `rust/task-registry-flow-cli/src/schema.rs`, `rust/task-registry-flow-cli/src/tests/release_source_tests.rs`, `scripts/test-release-readiness.sh`.
- Positive test: `.codex/scripts/task-registry release-check all --format json`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_source_rejects_undeclared_`.
- Data/schema/provenance: Add a canonical release check id; no ad hoc shell-only policy.
- Runtime: `scripts/status.sh --release-source` fails on governed-source omissions.

### GAP-H06: Nix package includes runtime assets
- Current failure: Package installs only binaries and excludes templates, skills, hooks, docs, and metadata.
- Good behavior: Given `nix build .#task-registry-flow`, when output is inspected, then binaries and runtime assets exist under stable store paths.
- Forbidden behavior: Package consumers need the mutable repo checkout for templates, skills, hooks, or metadata.
- Files involved: `package.nix`, `flake.nix`, `modules/nixos/agent-governance.nix`, `scripts/test-release-readiness.sh`, `README.md`.
- Positive test: `nix build .#task-registry-flow --no-link` plus output asset assertions.
- Negative test: `bash scripts/test-release-readiness.sh all` fails fixture when templates or skills are missing from package checks.
- Data/schema/provenance: Package output includes release metadata matching `REQUIREMENTS.toml`.
- Runtime: Nix consumers can install and locate the canonical asset root.

### GAP-H07: Auto-update is root-safe and rollback-safe
- Current failure: Auto-update runs as hardcoded user and calls `nixos-rebuild switch` without rollback guard.
- Good behavior: Given the module is enabled, when an update changes the lock, then it validates, rebuilds as root, and restores the old lock on failure.
- Forbidden behavior: Hardcoded `/home/hasnamuss`, default unprivileged rebuild, or no lock backup/restore path.
- Files involved: `modules/nixos/agent-governance-auto-update.nix`, `scripts/test-release-readiness.sh`.
- Positive test: `nix flake check --no-build --all-systems`.
- Negative test: `bash scripts/test-release-readiness.sh all` fixture asserts backup/restore and root-safe defaults.
- Data/schema/provenance: Nix module options document rebuild command and health command.
- Runtime: Dev lock still suppresses timer.

### GAP-H08: Receipt-chain strictness rejects unchained events
- Current failure: Unchained v2 receipts are warnings and `--repair` does nothing when only warnings exist.
- Good behavior: Given an unchained v2 receipt, when `verify-chain` runs, then it fails; when `--repair` runs, then it repairs parseable v2 lines.
- Forbidden behavior: Strict or release status passes with unchained v2 receipts.
- Files involved: `rust/task-registry-flow-cli/src/verify_chain.rs`, `rust/task-registry-flow-cli/src/metrics.rs`, `docs/task-registry/events.jsonl`, `scripts/status.sh`.
- Positive test: `.codex/scripts/task-registry verify-chain --format json`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml verify_chain_fails_unchained_v2_receipt`.
- Data/schema/provenance: Legacy schema v1 remains invalid; repair is only for parseable v2 chain fields.
- Runtime: Release gates require fully chained receipts.

### GAP-H09: Terminal task metadata is immutable
- Current failure: Re-activation can rewrite completed or cancelled task metadata while keeping terminal status.
- Good behavior: Given a terminal task, when its source plan changes metadata or targets, then activation fails; unchanged reactivation is allowed.
- Forbidden behavior: Completed or cancelled tasks receive new source hash, targets, behavior ids, title, or acceptance proof.
- Files involved: `rust/task-registry-flow-cli/src/activation.rs`, `rust/task-registry-flow-cli/src/validation.rs`, `rust/task-registry-flow-cli/src/tests/activation_terminal_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_terminal_allows_idempotent_reactivation`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_terminal_rejects_`.
- Data/schema/provenance: Terminal task provenance remains immutable after closure.
- Runtime: Active and planned task refresh behavior remains available.

## Validation Plan

Focused:
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_command_`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml receipt_chain_`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_source_`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_terminal_`
- `.codex/scripts/task-registry verify-chain --format json`
- `.codex/scripts/task-registry source-limit check`

Full:
- `.codex/scripts/task-registry validate`
- `.codex/scripts/task-registry metrics --format json`
- `cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings`
- `bash scripts/test-install-modes.sh`
- `bash scripts/test-release-readiness.sh all`
- `nix build .#task-registry-flow --no-link`
- `nix flake check --no-build --all-systems`
- `scripts/status.sh --strict`
- `scripts/status.sh --release-source`

## Walkthrough Evidence

Capture after implementation:
- `TASK_REPORT PLAN-2026-05-31-post-rollback-hardening`: all tasks completed, no deferred or blocked tasks.
- `TASK_METRICS`: `malformed_events=0`, `receipt_chain_breaks=0`, and no active registry tasks after archival.
- `VERIFY_CHAIN`: JSON summary has zero failures and zero unchained receipt diagnostics.
- Full validation commands in `Validation Plan` exit zero.
- `git status --short --branch` shows only intentional staged changes before commit.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-05-31-post-rollback-hardening"

[[behaviors]]
behavior_id = "B-H01-positive"
gap_id = "GAP-H01"
polarity = "positive"
title = "Read-only shell hooks without paths remain allowed"
given = "A canonical hook payload containing a read-only shell command without target paths"
when = "verify-mutation-hook evaluates the payload"
then = "The hook allows the payload without requiring an active registry target"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_command_allows_read_only_without_path"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_command_allows_read_only_without_path"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H01-negative"
gap_id = "GAP-H01"
polarity = "negative"
title = "Ambiguous write-intent shell hooks are denied"
given = "A canonical hook payload containing inline interpreter write intent with no deterministic target path"
when = "verify-mutation-hook evaluates the payload"
then = "The hook denies the payload with a deterministic-target error"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_command_denies_"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_command_denies_"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H02-positive"
gap_id = "GAP-H02"
polarity = "positive"
title = "Concurrent receipt writers preserve one linear chain"
given = "A receipt ledger receiving concurrent appends"
when = "events are appended through append_event"
then = "Every event points to the immediately previous event hash"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml receipt_chain_concurrent_writers_preserve_chain"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml receipt_chain_concurrent_writers_preserve_chain"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H02-negative"
gap_id = "GAP-H02"
polarity = "negative"
title = "Locked receipt file fails append instead of racing"
given = "A receipt ledger already held by another writer"
when = "append_event attempts to append"
then = "The append fails with a locked-file error"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml receipt_chain_locked_file_fails_append"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml receipt_chain_locked_file_fails_append"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H03-positive"
gap_id = "GAP-H03"
polarity = "positive"
title = "Explicit read-only receipt policy remains opt-in"
given = "A read-only command"
when = "it runs without explicit receipt recording"
then = "No receipt is required or reported"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml receipt_chain_explicit_read_only_policy"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml receipt_chain_explicit_read_only_policy"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H03-negative"
gap_id = "GAP-H03"
polarity = "negative"
title = "Required receipt failure fails command"
given = "A mutating command with a required receipt"
when = "receipt append fails"
then = "The command exits nonzero and does not claim a persisted receipt"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml receipt_chain_required_failure_fails_command"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml receipt_chain_required_failure_fails_command"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H04-positive"
gap_id = "GAP-H04"
polarity = "positive"
title = "Live hook remains valid shell"
given = "The canonical live mutation hook"
when = "bash syntax validation runs"
then = "The hook parses successfully"
confirmation = "bash -n tools/agent-governance/pre-tool-use-gap-closure.sh"

[[behaviors.verifiers]]
type = "command"
command = "bash -n tools/agent-governance/pre-tool-use-gap-closure.sh"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H04-negative"
gap_id = "GAP-H04"
polarity = "negative"
title = "Rendered hook denial JSON survives control characters"
given = "A rendered hook receiving denial text with quotes, slashes, and control characters"
when = "installer mode tests parse hook denial output"
then = "Every supported agent JSON shape parses"
confirmation = "bash scripts/test-install-modes.sh"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-install-modes.sh"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H05-positive"
gap_id = "GAP-H05"
polarity = "positive"
title = "Current release-source manifest covers governed surfaces"
given = "The current repository release-source manifest"
when = "release-check all runs"
then = "All governed release surfaces are declared and present"
confirmation = ".codex/scripts/task-registry release-check all --format json"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry release-check all --format json"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H05-negative"
gap_id = "GAP-H05"
polarity = "negative"
title = "Release check rejects undeclared governed release surfaces"
given = "A release fixture omitting Nix, Claude, hook, or template files from REQUIREMENTS.toml"
when = "release-check all runs"
then = "The governed-source check fails"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_source_rejects_undeclared_"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_source_rejects_undeclared_"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H06-positive"
gap_id = "GAP-H06"
polarity = "positive"
title = "Nix package output includes runtime assets"
given = "The flake package is built"
when = "the output store path is inspected"
then = "Binaries, templates, skills, hooks, docs, and metadata are present"
confirmation = "bash scripts/test-release-readiness.sh all"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-release-readiness.sh all"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H06-negative"
gap_id = "GAP-H06"
polarity = "negative"
title = "Nix package asset omissions are caught"
given = "Package asset assertions in release readiness tests"
when = "templates or skills are missing from package output"
then = "The release readiness gate fails"
confirmation = "bash scripts/test-release-readiness.sh all"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-release-readiness.sh all"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H07-positive"
gap_id = "GAP-H07"
polarity = "positive"
title = "Auto-update module evaluates with rollback guards"
given = "The flake modules are evaluated"
when = "nix flake check runs without building"
then = "The agent-governance and auto-update modules evaluate"
confirmation = "nix flake check --no-build --all-systems"

[[behaviors.verifiers]]
type = "command"
command = "nix flake check --no-build --all-systems"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H07-negative"
gap_id = "GAP-H07"
polarity = "negative"
title = "Auto-update module forbids unsafe defaults"
given = "The release readiness Nix module assertions"
when = "auto-update lacks backup, restore, root-safe rebuild, or dev lock guards"
then = "The release readiness gate fails"
confirmation = "bash scripts/test-release-readiness.sh all"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-release-readiness.sh all"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H08-positive"
gap_id = "GAP-H08"
polarity = "positive"
title = "Current receipt chain is strict and valid"
given = "The repository receipt ledger"
when = "verify-chain runs in JSON mode"
then = "The report exits zero with no failures"
confirmation = ".codex/scripts/task-registry verify-chain --format json"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry verify-chain --format json"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H08-negative"
gap_id = "GAP-H08"
polarity = "negative"
title = "Unchained v2 receipt fails strict verification"
given = "A receipt fixture with an unchained schema v2 line"
when = "verify-chain runs"
then = "The report fails instead of warning only"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml verify_chain_fails_unchained_v2_receipt"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml verify_chain_fails_unchained_v2_receipt"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H09-positive"
gap_id = "GAP-H09"
polarity = "positive"
title = "Idempotent terminal plan reactivation is allowed"
given = "A completed or cancelled task whose manifest data is unchanged"
when = "its plan is reactivated"
then = "Activation succeeds without changing terminal provenance"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_terminal_allows_idempotent_reactivation"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_terminal_allows_idempotent_reactivation"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H09-negative"
gap_id = "GAP-H09"
polarity = "negative"
title = "Terminal task rewrites are rejected"
given = "A completed or cancelled task whose manifest metadata changes"
when = "its plan is reactivated"
then = "Activation fails and terminal provenance remains unchanged"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_terminal_rejects_"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_terminal_rejects_"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-05-31-H01"
status = "planned"
kind = "authorization"
reason = "Mutation hooks must fail closed for ambiguous write-intent payloads before implementation writes can bypass registry targets."
behavior_ids = ["B-H01-positive", "B-H01-negative", "B-H04-positive", "B-H04-negative"]
title = "Harden mutation hook command parsing and template JSON"
acceptance_proof = "Behaviors B-H01-positive, B-H01-negative, B-H04-positive, and B-H04-negative pass their verifier commands."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/mutation_hook.rs"
object = "shell_write_detection"
required_change = "Deny write-intent shell commands without deterministic target extraction."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/hook_io.rs"
object = "payload_contract"
required_change = "Preserve canonical payload shape validation for stricter hook denial."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/hook_command_tests.rs"
object = "hook_command_negative_suite"
required_change = "Add shell write-intent positive and negative tests."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/mod.rs"
object = "hook_command_module_registration"
required_change = "Register hook command tests without adding behavior to the monolithic test file."

[[tasks.targets]]
file = "templates/tools/agent-governance/pre-tool-use-gap-closure.sh.template"
object = "json_escape"
required_change = "Use canonical JSON serialization for denial reasons."

[[tasks.targets]]
file = "tools/agent-governance/pre-tool-use-gap-closure.sh"
object = "json_escape_parity"
required_change = "Keep live hook aligned with the template."

[[tasks.targets]]
file = "scripts/test-install-modes.sh"
object = "rendered_hook_json_negative"
required_change = "Verify rendered hook denial JSON handles control characters."

[[tasks]]
task_id = "TASK-2026-05-31-H02"
status = "planned"
kind = "implementation"
reason = "Receipt writes must be durable, required receipt failures must fail, and strict chain verification must reject unchained receipts."
behavior_ids = ["B-H02-positive", "B-H02-negative", "B-H03-positive", "B-H03-negative", "B-H08-positive", "B-H08-negative"]
title = "Make receipt writes atomic and receipt chain strict"
acceptance_proof = "Behaviors B-H02-positive, B-H02-negative, B-H03-positive, B-H03-negative, B-H08-positive, and B-H08-negative pass."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/runtime.rs"
object = "append_event_lock_order"
required_change = "Lock receipt file before reading previous hash and appending."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cli.rs"
object = "required_receipt_errors"
required_change = "Propagate required receipt append failures."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/receipts.rs"
object = "receipt_policy"
required_change = "Represent required receipt append results accurately."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/metrics.rs"
object = "unchained_failure_accounting"
required_change = "Count unchained v2 receipts as failed receipt state."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/verify_chain.rs"
object = "strict_chain_and_repair"
required_change = "Fail on unchained v2 receipts and repair parseable v2 chain fields."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/receipt_chain_tests.rs"
object = "receipt_chain_suite"
required_change = "Add concurrency, lock, and receipt failure tests."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/verify_chain_tests.rs"
object = "strict_unchained_suite"
required_change = "Add unchained strict failure and repair tests."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/mod.rs"
object = "receipt_chain_module_registration"
required_change = "Register receipt chain tests."

[[tasks.targets]]
file = "docs/task-registry/events.jsonl"
object = "receipt_ledger_migration"
required_change = "Migrate local ledger to fully chained schema v2 receipts."

[[tasks.targets]]
file = "docs/runtime-schemas.md"
object = "receipt_chain_contract"
required_change = "Document strict chained v2 receipt state."

[[tasks.targets]]
file = "scripts/status.sh"
object = "chain_gate"
required_change = "Run verify-chain in strict and release-source status."

[[tasks]]
task_id = "TASK-2026-05-31-H03"
status = "planned"
kind = "release"
reason = "Release-source checks must govern every tracked Nix, Claude, hook, and template release surface."
behavior_ids = ["B-H05-positive", "B-H05-negative"]
title = "Expand release-source coverage and governed-source checks"
acceptance_proof = "Behaviors B-H05-positive and B-H05-negative pass."

[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release_source_required"
required_change = "Declare all governed release surfaces and new test modules."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/release_checks.rs"
object = "governed_source_discovery"
required_change = "Discover non-Rust governed release surfaces and fail omissions."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/schema.rs"
object = "release_check_id"
required_change = "Add canonical governed-source release check id."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/release_source_tests.rs"
object = "release_source_negative_suite"
required_change = "Add omitted Nix, Claude, hook, and template negative tests."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/mod.rs"
object = "release_source_module_registration"
required_change = "Register release source tests."

[[tasks.targets]]
file = "scripts/test-release-readiness.sh"
object = "governed_source_negative_fixtures"
required_change = "Assert governed-source omissions fail release readiness."

[[tasks.targets]]
file = "scripts/status.sh"
object = "release_source_gate"
required_change = "Surface governed-source checks in release-source status."

[[tasks]]
task_id = "TASK-2026-05-31-H04"
status = "planned"
kind = "implementation"
reason = "Nix consumers must receive a complete runtime package and safe update module without relying on the mutable checkout."
behavior_ids = ["B-H06-positive", "B-H06-negative", "B-H07-positive", "B-H07-negative"]
title = "Package runtime assets and harden Nix modules"
acceptance_proof = "Behaviors B-H06-positive, B-H06-negative, B-H07-positive, and B-H07-negative pass."

[[tasks.targets]]
file = "package.nix"
object = "runtime_asset_output"
required_change = "Install runtime assets under stable share paths."

[[tasks.targets]]
file = "flake.nix"
object = "module_exports"
required_change = "Export agent-governance and auto-update NixOS modules."

[[tasks.targets]]
file = "modules/nixos/agent-governance.nix"
object = "runtime_install_module"
required_change = "Add module for packaged CLI and asset root."

[[tasks.targets]]
file = "modules/nixos/agent-governance-auto-update.nix"
object = "root_safe_rollback"
required_change = "Add root-safe rebuild, validation, lock backup, restore, rollback, and dev-lock behavior."

[[tasks.targets]]
file = "scripts/test-release-readiness.sh"
object = "nix_asset_negative_fixtures"
required_change = "Assert package assets and auto-update guards."

[[tasks.targets]]
file = "README.md"
object = "nix_consumption_contract"
required_change = "Document Nix package asset layout and modules."

[[tasks]]
task_id = "TASK-2026-05-31-H05"
status = "planned"
kind = "governance"
reason = "Completed and cancelled registry tasks must remain immutable after closure."
behavior_ids = ["B-H09-positive", "B-H09-negative"]
title = "Enforce terminal task immutability on activation"
acceptance_proof = "Behaviors B-H09-positive and B-H09-negative pass."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/activation.rs"
object = "terminal_reactivation_guard"
required_change = "Reject terminal task rewrites unless reactivation is idempotent."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/validation.rs"
object = "terminal_hash_policy"
required_change = "Remove terminal hash exceptions that permit metadata drift."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/activation_terminal_tests.rs"
object = "terminal_task_suite"
required_change = "Add completed and cancelled task immutability tests."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/mod.rs"
object = "activation_terminal_module_registration"
required_change = "Register terminal activation tests."
```
