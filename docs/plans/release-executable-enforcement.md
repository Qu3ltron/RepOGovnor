# Release Executable Enforcement Gap Closure Contract

## Approved Scope

In scope: close the release-readiness gap where shell scripts with shebangs can be listed as release artifacts while lacking executable mode, causing direct invocation to fail after install or checkout.

Out of scope: changing installer behavior, changing hook authorization, changing release version policy, changing stale artifact policy, or preserving legacy compatibility. The v2 contract is canonical only.

Runtime/schema impact: `REQUIREMENTS.toml` becomes the typed source of truth for executable release artifacts. `task-registry-flow release-check` emits `release-file-executable` diagnostics. Human scripts render this typed API; they do not own independent prose checks.

## Phased Required Change Checklist

### Phase 0: Activation and safety

- [ ] `[NEW]` `docs/plans/release-executable-enforcement.md` - closure contract: declare exact scope, success criteria, executable release behavior, negative migration tests, and Task Manifest.
- [ ] `[VERIFY]` `docs/task-registry.toml` - active registry: run `.codex/scripts/task-registry activate docs/plans/release-executable-enforcement.md` before implementation writes.
- [ ] `[VERIFY]` source file budget - run `wc -l REQUIREMENTS.toml rust/task-registry-flow-cli/src/release_checks.rs rust/task-registry-flow-cli/src/schema.rs rust/task-registry-flow-cli/src/tests/mod.rs scripts/test-release-readiness.sh docs/runtime-schemas.md` and avoid adding to files that would exceed 1600 lines.

### Phase 1: Canonical schema and runtime API

- [ ] `[MODIFY]` `REQUIREMENTS.toml` - `[release_source].executable`: list release shell scripts that must be executable in a fresh checkout.
- [ ] `[MODIFY]` `REQUIREMENTS.toml` - `[release_source].check_ids`: add `release-file-executable` as a known release diagnostic.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/schema.rs` - `ReleaseCheckId`: add the canonical `release-file-executable` enum value.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/release_checks.rs` - `ReleaseSource`: parse `executable` as typed configuration with unknown-field rejection intact.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/release_checks.rs` - release-check runtime: validate every configured executable artifact as an existing file with executable mode, and emit structured pass/fail diagnostics.

### Phase 2: Behavioral and negative migration tests

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/mod.rs` - release check tests: seed executable requirements and assert non-executable files fail with `release-file-executable`.
- [ ] `[MODIFY]` `scripts/test-release-readiness.sh` - executable mode suite: prove the real repo passes, and a copied release tree with `scripts/test-install-modes.sh` chmodded to `0644` fails with machine-readable `release-file-executable`.
- [ ] `[MODIFY]` `scripts/test-install-modes.sh` - filesystem mode: make the script directly executable in git.
- [ ] `[VERIFY]` `scripts/test-install-modes.sh` - direct invocation: run `scripts/test-install-modes.sh`, not only `bash scripts/test-install-modes.sh`.

### Phase 3: Documentation and handoff

- [ ] `[MODIFY]` `docs/runtime-schemas.md` - `Release Contract`: document `release_source.executable` and `release-file-executable` as the runtime API.
- [ ] `[VERIFY]` release gates - run focused and full validation commands listed below.
- [ ] `[VERIFY]` task registry - complete all tasks only after behavior verifiers pass, then run `.codex/scripts/task-registry report PLAN-2026-05-31-release-executable-enforcement` and `.codex/scripts/task-registry metrics`.

## Per-Gap Success Criteria

### GAP-001: Release scripts can be non-executable while still passing release artifact checks

- Current failure: `scripts/test-install-modes.sh` has a shell shebang but mode `0644`; direct invocation fails with permission denied, and release checks do not report the defect.
- Good behavior: Given a v2 checkout, when `.codex/scripts/task-registry release-check all --format json` runs, then every path in `release_source.executable` emits a `release-file-executable` pass diagnostic only if it is an executable file.
- Forbidden behavior: Given a release copy where a configured script is chmodded `0644`, when release readiness runs, then the gate exits nonzero and preserves a JSON diagnostic with `check_id = "release-file-executable"`, the script path, and `status = "fail"`.
- Files involved: `REQUIREMENTS.toml`, `rust/task-registry-flow-cli/src/schema.rs`, `rust/task-registry-flow-cli/src/release_checks.rs`, `rust/task-registry-flow-cli/src/tests/mod.rs`, `scripts/test-release-readiness.sh`, `scripts/test-install-modes.sh`, `docs/runtime-schemas.md`.
- Positive test: `.codex/scripts/task-registry release-check all --format json` exits 0 in the real repo and includes executable diagnostics.
- Negative test: `bash scripts/test-release-readiness.sh executable` constructs a release copy, chmods `scripts/test-install-modes.sh` to `0644`, and requires release-check failure with `release-file-executable`.
- Migration test: `scripts/test-install-modes.sh` runs directly from the repo root after the executable bit is recorded.
- Data/schema/provenance criteria: executable requirements live only in `REQUIREMENTS.toml`; runtime diagnostics use `ReleaseCheckId`; no loose prose list becomes authoritative.
- Runtime criteria: the Rust release-check API is the clean runtime owner; shell scripts only call or assert its structured output.

## Validation Plan

Focused:

- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_schema_reports_executable_failures`
- `bash scripts/test-release-readiness.sh executable`
- `scripts/test-install-modes.sh`
- `.codex/scripts/task-registry release-check all --format json`
- `.codex/scripts/task-registry source-limit check`

Full:

- `cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check`
- `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `bash scripts/test-release-readiness.sh all`
- `scripts/status.sh --strict`
- `.codex/scripts/task-registry validate`
- `.codex/scripts/task-registry verify-behaviors PLAN-2026-05-31-release-executable-enforcement`

Source file limit:

- Expected impact stays below 1600 lines for all touched source/config/script/doc files.
- Run `.codex/scripts/task-registry source-limit check` before marking tasks complete.
- Do not archive completed tasks if doing so would push `docs/task-registry/archive/completed-001.toml` over the line budget.

## Walkthrough Evidence

- Capture JSON release-check output showing `release-file-executable` pass diagnostics.
- Capture negative executable test output proving chmod `0644` fails closed.
- Capture direct `scripts/test-install-modes.sh` invocation success.
- Capture full validation command outcomes and `TASK_REPORT`.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-05-31-release-executable-enforcement"

[[behaviors]]
behavior_id = "B-001-executable-release-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Executable release artifacts pass through typed runtime diagnostics"
given = "The repo is a v2 release source with scripts listed in release_source.executable"
when = "release-check all runs in JSON mode"
then = "configured executable artifacts emit release-file-executable pass diagnostics and the command exits zero"
confirmation = ".codex/scripts/task-registry release-check all --format json"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry release-check all --format json >/tmp/release-executable-positive.json"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-executable-release-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Non-executable release scripts fail closed"
given = "A copied release source has a configured executable script chmodded to 0644"
when = "the executable readiness suite runs"
then = "release-check exits nonzero with a release-file-executable failure diagnostic"
confirmation = "bash scripts/test-release-readiness.sh executable"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-release-readiness.sh executable"
expected_exit = 0

[[behaviors]]
behavior_id = "B-003-direct-script-validation"
gap_id = "GAP-001"
polarity = "validation"
title = "Direct install-mode script invocation works"
given = "scripts/test-install-modes.sh is tracked as an executable release artifact"
when = "the script is invoked directly"
then = "the operating system can execute it without a bash shim"
confirmation = "scripts/test-install-modes.sh"

[[behaviors.verifiers]]
type = "command"
command = "scripts/test-install-modes.sh"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-05-31-release-executable-001"
status = "planned"
title = "Add executable release schema and runtime checks"
kind = "implementation"
reason = "Release artifact presence is insufficient when scripts are not executable."
acceptance_proof = "Behaviors B-001-executable-release-positive and B-002-executable-release-negative pass."
behavior_ids = ["B-001-executable-release-positive", "B-002-executable-release-negative"]

[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release_source.executable and check_ids"
required_change = "Declare executable release artifacts and the release-file-executable diagnostic id."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/schema.rs"
object = "ReleaseCheckId"
required_change = "Add release-file-executable as a typed release check id."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/release_checks.rs"
object = "release_check runtime"
required_change = "Parse executable requirements and emit pass/fail executable diagnostics."

[[tasks]]
task_id = "TASK-2026-05-31-release-executable-002"
status = "planned"
title = "Add executable migration tests"
kind = "validation"
reason = "The release gate must prove both good executable state and non-executable migration failure."
acceptance_proof = "Behaviors B-002-executable-release-negative and B-003-direct-script-validation pass."
behavior_ids = ["B-002-executable-release-negative", "B-003-direct-script-validation"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/mod.rs"
object = "release executable tests"
required_change = "Seed executable requirements and assert chmod 0644 failures produce release-file-executable diagnostics."

[[tasks.targets]]
file = "scripts/test-release-readiness.sh"
object = "check_executable"
required_change = "Add positive and negative executable-mode release readiness checks."

[[tasks.targets]]
file = "scripts/test-install-modes.sh"
object = "filesystem mode"
required_change = "Track executable mode so direct invocation works."

[[tasks]]
task_id = "TASK-2026-05-31-release-executable-003"
status = "planned"
title = "Document executable release runtime contract"
kind = "documentation"
reason = "Release executable policy must be discoverable as schema, not loose prose."
acceptance_proof = "Behavior B-001-executable-release-positive passes and docs/runtime-schemas.md names release_source.executable."
behavior_ids = ["B-001-executable-release-positive"]

[[tasks.targets]]
file = "docs/runtime-schemas.md"
object = "Release Contract"
required_change = "Document release_source.executable and release-file-executable diagnostics."
```
