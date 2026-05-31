# Runtime Schema Production Closures

## Approved Scope

Close the eight schema and runtime-cleanliness gaps approved for production hardening:

- Replace loose internal command/action/status/outcome strings with typed runtime schema values.
- Replace path-prefix mutation authorization with typed mutation scopes.
- Make behavior verification machine-readable instead of prose-only shell strings.
- Make installer actions and manifest handling schema-backed.
- Consolidate release/status policy around `REQUIREMENTS.toml`.
- Emit structured diagnostics for production checks.
- Align user docs with the runtime schema contract.
- Preserve historical archive integrity unless a separate registry-aware migration is approved.

Out of scope:

- Backward compatibility shims, legacy v0.3 behavior, or overlay restoration.
- Rewriting historical archived plans solely to change past prose.
- Publishing, tagging, or committing the release.

Primitive change gate: N/A. This changes governance tooling, CLI schemas, installer/report contracts, and release gates; it does not change application runtime primitives, persistence, queues, providers, or external services.

## Required Change Checklist

- [NEW] `rust/task-registry-flow-cli/src/schema.rs` - add typed schema primitives for commands, hook formats, event outcomes, diagnostics, check reports, verifier specs, mutation scopes, and release contracts. Acceptance proof: Rust schema tests pass.
- [MODIFY] `rust/task-registry-flow-cli/src/model.rs` - use typed event records and extend plan behavior/verifier manifest support without accepting unknown fields. Acceptance proof: manifest validation tests pass.
- [MODIFY] `rust/task-registry-flow-cli/src/main.rs` - parse CLI commands into typed commands, write schema-versioned receipt events, run typed behavior verifiers, and expose structured validation/release reports. Acceptance proof: Rust CLI tests pass.
- [MODIFY] `rust/task-registry-flow-cli/src/mutation_hook.rs` - enforce typed mutation scopes instead of accidental prefix matching and return structured denial details. Acceptance proof: hook positive and negative tests pass.
- [NEW] `rust/task-registry-flow-cli/src/release_checks.rs` - load `REQUIREMENTS.toml`, validate release-source/tracked-for-CI/version contracts, and emit structured check reports. Acceptance proof: release-check tests pass.
- [MODIFY] `rust/task-registry-flow-cli/src/source_limit.rs` - support structured check reports and reuse diagnostic schema. Acceptance proof: source-limit tests pass.
- [MODIFY] `rust/task-registry-flow-cli/src/tests.rs` - add comprehensive positive and negative tests for schemas, mutation authorization, behavior verifiers, diagnostics, and release contract validation. Acceptance proof: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`.
- [MODIFY] `scripts/render-from-config.sh` - validate installer manifest/config shape, emit typed dry-run action JSON, and ensure installer actions are enumerated. Acceptance proof: release readiness/install tests pass.
- [MODIFY] `scripts/status.sh` - consume release/tracked-for-CI schema through the Rust CLI and keep shell output as rendering only. Acceptance proof: strict and release-source status gates pass.
- [MODIFY] `scripts/release-audit.sh` - call schema-backed release validation from repo root and reject unknown args. Acceptance proof: release audit tests pass.
- [MODIFY] `scripts/release-version-check.sh` - consume version-bearing paths from `REQUIREMENTS.toml` instead of hard-coded file mapping. Acceptance proof: release readiness version tests pass.
- [MODIFY] `scripts/test-release-readiness.sh` - add positive and negative schema migration tests. Acceptance proof: `bash scripts/test-release-readiness.sh all`.
- [MODIFY] `REQUIREMENTS.toml` - add structured release check IDs and version-bearing file schema. Acceptance proof: release-check schema validation passes.
- [MODIFY] `MANIFEST.toml` - add installer stale-path/action schema where needed. Acceptance proof: installer schema validation passes.
- [MODIFY] `README.md` - describe the user-facing schema-backed failure model without developer-only prose. Acceptance proof: docs match CLI output.
- [MODIFY] `VISION.md` - align vision with clean runtime API and schema-first governance. Acceptance proof: docs remain truthful.
- [MODIFY] `ROADMAP.md` - mark schema-backed runtime work accurately. Acceptance proof: roadmap does not promise unimplemented behavior.
- [MODIFY] `docs/releases/v2.md` - document schema-backed release/status checks. Acceptance proof: release artifact tests pass.
- [NEW] `docs/runtime-schemas.md` - document diagnostic, mutation-scope, verifier, installer-action, and release-check schemas. Acceptance proof: docs examples match tests.

## Per-Gap Success Criteria

### Gap 1: Runtime command API is stringly typed

Good behavior:

- CLI command names, hook formats, task status values, event outcomes, diagnostics, verifier types, mutation scopes, and installer actions parse through enums.
- Unknown enum values fail with explicit usage or validation errors.
- Receipt events include schema version and typed command/outcome fields.

Negative behavior that must fail:

- Unknown command, hook format, task status, event outcome, verifier type, or diagnostic status is accepted.
- Receipt events are emitted without schema version or typed outcome.

### Gap 2: Authorization is path-prefix based

Good behavior:

- Exact file targets authorize only the same file.
- Directory-tree targets authorize only explicit child paths.
- Governance repair paths are separate scoped authority.
- Denials include path, operation, matched scope, and reason.

Negative behavior that must fail:

- `src/lib.rs` authorizes `src/lib.rs.bak`.
- Repo root, `.`, `docs`, `.codex`, `.agents`, or `.cursor` are accepted as broad implementation targets.
- Parent traversal or undeclared runtime paths are authorized.

### Gap 3: Plan behavior remains prose-heavy

Good behavior:

- Behavior confirmations can declare typed verifiers: `command`, `file_exists`, `file_absent`, `contains`, `not_contains`, and `json_schema`.
- Migration/authorization tasks must include negative behavior coverage.
- `verify-behaviors` reports structured verifier failures.

Negative behavior that must fail:

- Unknown verifier type is accepted.
- Missing negative coverage is accepted for migration or authorization tasks.
- Failing command/file/content verifier reports success.

### Gap 4: Installer is under-schematized

Good behavior:

- Manifest-driven install actions are enumerated.
- Dry-run can emit JSON action reports.
- Symlink replacement, stale cleanup, generated files, and plugin links report typed actions.
- Unknown runtime-affecting config keys fail validation.

Negative behavior that must fail:

- Unknown installer action or manifest section is accepted.
- Generated destination escapes the repo.
- Legacy skill symlink is reported as aligned.

### Gap 5: Release/status checks duplicate source-of-truth

Good behavior:

- Release-source files, tracked-for-CI files, and version-bearing files come from `REQUIREMENTS.toml`.
- `status.sh`, `release-audit.sh`, and `release-version-check.sh` agree on the same schema.
- Nested release audit scans repo root.

Negative behavior that must fail:

- Missing required release file passes.
- Unknown release check id passes.
- Version mismatch passes.
- Extra ignored release-audit arg passes.

### Gap 6: Diagnostics are prose-only

Good behavior:

- Validation, release, source-limit, and installer dry-run can emit structured reports with `check_id`, `surface`, `path`, `severity`, `status`, `expected`, `actual`, and `remediation`.
- Human output is rendered from structured facts.

Negative behavior that must fail:

- Unknown diagnostic severity/status is accepted.
- A failing check lacks path, expected, actual, or remediation.
- Mixed pass/fail report exits zero.

### Gap 7: Behavior confirmations execute shell commands without a clean verifier API

Good behavior:

- Raw command execution is an explicit verifier type.
- Non-command verifiers avoid shell execution.
- Verifier failure details are structured.

Negative behavior that must fail:

- Implicit prose confirmation runs as shell without verifier declaration after schema migration.
- Nonzero command is reported as pass.

### Gap 8: Historical docs contain stale license language

Good behavior:

- Live user docs and release docs reflect MIT/free-use terms.
- Historical archived plans are not silently rewritten.
- Any future archive rewrite requires an explicit registry-aware migration plan.

Negative behavior that must fail:

- Live docs reintroduce all-rights-reserved language.
- Historical evidence files are edited as an unregistered side effect.

## Validation Plan

Focused gates:

```bash
cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml
bash scripts/test-release-readiness.sh all
bash scripts/release-audit.sh
```

Full gates:

```bash
cargo run --locked --quiet --manifest-path rust/task-registry-flow-cli/Cargo.toml -- validate
cargo run --locked --quiet --manifest-path rust/task-registry-flow-cli/Cargo.toml -- source-limit check
AGENT_GOVERNANCE_ALLOW_DIRTY_RELEASE_CHECK=1 bash scripts/status.sh --release-source
AGENT_GOVERNANCE_ALLOW_DIRTY_RELEASE_CHECK=1 bash scripts/status.sh --strict
agy plugin validate .
git diff --check
```

Failure evidence:

- Any accepted unknown enum value means the runtime API gap remains open.
- Any broad or prefix-collision mutation authorization means the authorization gap remains open.
- Any production gate with prose-only, unstructured failure evidence means the diagnostic gap remains open.
- Any duplicated release-source/version-bearing list outside the release schema means the release/status gap remains open.

## Walkthrough Evidence

Capture:

- Plan activation output.
- Rust test result summary.
- Release readiness and release audit result summary.
- Status strict/release-source result summary.
- Final `TASK_REPORT PLAN-2026-05-30-runtime-schema-production-closures`.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-05-30-runtime-schema-production-closures"

[[behaviors]]
behavior_id = "B-2026-05-30-runtime-schema-core"
gap_id = "GAP-2026-05-30-runtime-schema-core"
polarity = "positive"
title = "Runtime uses typed schema values"
given = "The task-registry CLI receives valid and invalid runtime API inputs"
when = "Commands, hook formats, events, diagnostics, and verifiers are parsed"
then = "Known values execute and unknown values fail with structured validation errors"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml schema_"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml schema_"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-30-mutation-scope-authorization"
gap_id = "GAP-2026-05-30-mutation-scope-authorization"
polarity = "positive"
title = "Mutation authorization uses typed scopes"
given = "A registry with exact file, directory tree, generated artifact, and governance repair scopes"
when = "The mutation hook evaluates allowed and forbidden write paths"
then = "Only declared scopes pass and broad, traversal, prefix-collision, or undeclared paths fail"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml mutation_scope_"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml mutation_scope_"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-30-behavior-verifier-schema"
gap_id = "GAP-2026-05-30-behavior-verifier-schema"
polarity = "positive"
title = "Behavior verification is schema-backed"
given = "Plan behaviors with typed command, file, content, and JSON verifiers"
when = "verify-behaviors runs"
then = "Valid verifier outcomes pass and invalid verifier declarations or failed assertions return structured errors"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml verifier_"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml verifier_"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-30-installer-action-schema"
gap_id = "GAP-2026-05-30-installer-action-schema"
polarity = "positive"
title = "Installer actions are enumerated and reportable"
given = "Fresh, merge, force, dry-run, and legacy-symlink install scenarios"
when = "The renderer validates config and computes install actions"
then = "Only known action values are emitted and forbidden symlink/alignment states fail tests"
confirmation = "bash scripts/test-release-readiness.sh all"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-release-readiness.sh all"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-30-release-status-schema"
gap_id = "GAP-2026-05-30-release-status-schema"
polarity = "positive"
title = "Release and status checks consume one release schema"
given = "The plugin release source contract in REQUIREMENTS.toml"
when = "status, version check, and release audit run from root and nested directories"
then = "All gates use the same required file and version-bearing path schema"
confirmation = "bash scripts/test-release-readiness.sh all"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-release-readiness.sh all"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-30-structured-diagnostics"
gap_id = "GAP-2026-05-30-structured-diagnostics"
polarity = "positive"
title = "Production gates emit structured diagnostics"
given = "Passing and failing validation, release, source-limit, and installer dry-run scenarios"
when = "JSON report mode is requested"
then = "Reports include check id, surface, path, severity, status, expected, actual, remediation, and summary counts"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml diagnostics_"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml diagnostics_"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-30-clean-verifier-api"
gap_id = "GAP-2026-05-30-clean-verifier-api"
polarity = "positive"
title = "Shell execution is explicit verifier behavior"
given = "A behavior manifest with command and non-command verifier declarations"
when = "The verifier runner evaluates the behavior"
then = "Only command verifiers execute shell and non-command verifiers evaluate typed assertions"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml verifier_"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml verifier_"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-30-live-license-docs"
gap_id = "GAP-2026-05-30-live-license-docs"
polarity = "positive"
title = "Live docs describe MIT/free-use terms without rewriting historical evidence"
given = "The live README, release docs, vision, roadmap, and historical task archive"
when = "Release readiness checks scan live artifacts"
then = "Live docs avoid stale all-rights-reserved terms and historical archives are left untouched"
confirmation = "bash scripts/test-release-readiness.sh artifacts"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-release-readiness.sh artifacts"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-05-30-runtime-schema-001"
title = "Add canonical runtime schema core"
status = "planned"
kind = "schema"
reason = "Runtime command, action, status, outcome, verifier, and diagnostic values are passed as loose strings."
acceptance_proof = "Behavior B-2026-05-30-runtime-schema-core passes."
behavior_ids = ["B-2026-05-30-runtime-schema-core"]
[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/schema.rs"
object = "runtime_schema_types"
required_change = "Define typed enums and structured report/event/verifier/mutation/release schema types."
[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/model.rs"
object = "event_and_manifest_schema"
required_change = "Use typed receipt events and behavior verifier manifest fields."
[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/main.rs"
object = "typed_command_dispatch"
required_change = "Parse public CLI strings into typed runtime commands and outcomes before dispatch."

[[tasks]]
task_id = "TASK-2026-05-30-runtime-schema-002"
title = "Replace mutation prefix matching with typed scopes"
status = "planned"
kind = "authorization"
reason = "Raw prefix matching can authorize accidental broad or prefix-collision paths."
acceptance_proof = "Behavior B-2026-05-30-mutation-scope-authorization passes."
behavior_ids = ["B-2026-05-30-mutation-scope-authorization"]
[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/mutation_hook.rs"
object = "mutation_scope_authorizer"
required_change = "Evaluate exact file, directory tree, generated artifact, and governance repair scopes."
[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests.rs"
object = "mutation_scope_negative_tests"
required_change = "Assert broad targets, traversal, prefix collisions, and undeclared paths fail."

[[tasks]]
task_id = "TASK-2026-05-30-runtime-schema-003"
title = "Schema-back behavior verification"
status = "planned"
kind = "validation"
reason = "Behavior confirmation currently depends on prose shell commands."
acceptance_proof = "Behavior B-2026-05-30-behavior-verifier-schema and B-2026-05-30-clean-verifier-api pass."
behavior_ids = ["B-2026-05-30-behavior-verifier-schema", "B-2026-05-30-clean-verifier-api"]
[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/model.rs"
object = "behavior_verifier_manifest"
required_change = "Add typed verifier declarations to behavior manifests."
[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/main.rs"
object = "verify_behaviors_runner"
required_change = "Run typed verifiers and return structured verifier failure diagnostics."
[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests.rs"
object = "verifier_positive_negative_tests"
required_change = "Add command, file, contains, not_contains, JSON, unknown type, and failing assertion tests."

[[tasks]]
task_id = "TASK-2026-05-30-runtime-schema-004"
title = "Schema-back installer actions"
status = "planned"
kind = "migration"
reason = "Installer behavior mixes config, hard-coded file lists, and loose action strings."
acceptance_proof = "Behavior B-2026-05-30-installer-action-schema passes."
behavior_ids = ["B-2026-05-30-installer-action-schema"]
[[tasks.targets]]
file = "scripts/render-from-config.sh"
object = "installer_action_schema"
required_change = "Validate install manifest/config and emit enumerated dry-run actions."
[[tasks.targets]]
file = "MANIFEST.toml"
object = "installer_manifest_contract"
required_change = "Declare installer stale paths and action policy needed by the renderer."
[[tasks.targets]]
file = "scripts/test-release-readiness.sh"
object = "installer_schema_negative_tests"
required_change = "Add tests for unknown actions, symlink alignment rejection, and dry-run JSON action output."

[[tasks]]
task_id = "TASK-2026-05-30-runtime-schema-005"
title = "Consolidate release and status schema"
status = "planned"
kind = "release"
reason = "Release/status/version checks duplicate file lists and policy across shell scripts."
acceptance_proof = "Behavior B-2026-05-30-release-status-schema passes."
behavior_ids = ["B-2026-05-30-release-status-schema"]
[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/release_checks.rs"
object = "release_contract_validator"
required_change = "Load REQUIREMENTS.toml and validate release source, version-bearing files, and tracked-for-CI paths."
[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release_check_schema"
required_change = "Declare release check IDs and version-bearing file paths."
[[tasks.targets]]
file = "scripts/status.sh"
object = "schema_backed_status"
required_change = "Use schema-backed CLI checks instead of owning duplicated release-source file policy."
[[tasks.targets]]
file = "scripts/release-version-check.sh"
object = "schema_backed_version_check"
required_change = "Read version-bearing files from REQUIREMENTS.toml."
[[tasks.targets]]
file = "scripts/release-audit.sh"
object = "schema_backed_release_audit"
required_change = "Run schema-backed release validation from repo root and reject unknown args."

[[tasks]]
task_id = "TASK-2026-05-30-runtime-schema-006"
title = "Emit structured diagnostics for production gates"
status = "planned"
kind = "diagnostics"
reason = "Status, release, validation, source-limit, and installer dry-run failures are primarily prose."
acceptance_proof = "Behavior B-2026-05-30-structured-diagnostics passes."
behavior_ids = ["B-2026-05-30-structured-diagnostics"]
[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/schema.rs"
object = "diagnostic_report_schema"
required_change = "Define reusable diagnostic and check report structures."
[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/source_limit.rs"
object = "source_limit_json_report"
required_change = "Emit source-limit diagnostics in JSON report mode."
[[tasks.targets]]
file = "scripts/render-from-config.sh"
object = "installer_dry_run_report"
required_change = "Emit structured installer dry-run action reports."
[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests.rs"
object = "diagnostic_negative_tests"
required_change = "Assert malformed diagnostic values and missing failure fields are rejected."

[[tasks]]
task_id = "TASK-2026-05-30-runtime-schema-007"
title = "Align docs with runtime schema behavior"
status = "planned"
kind = "documentation"
reason = "User docs need to explain current clean runtime API and schema-backed failures."
acceptance_proof = "Behavior B-2026-05-30-live-license-docs passes."
behavior_ids = ["B-2026-05-30-live-license-docs"]
[[tasks.targets]]
file = "README.md"
object = "schema_failure_model"
required_change = "Document user-facing schema-backed failure behavior."
[[tasks.targets]]
file = "VISION.md"
object = "schema_first_vision"
required_change = "Align vision with clean runtime API."
[[tasks.targets]]
file = "ROADMAP.md"
object = "schema_runtime_roadmap"
required_change = "Mark schema-backed runtime work accurately."
[[tasks.targets]]
file = "docs/releases/v2.md"
object = "schema_release_notes"
required_change = "Document schema-backed release and status checks."
[[tasks.targets]]
file = "docs/runtime-schemas.md"
object = "runtime_schema_reference"
required_change = "Document diagnostic, mutation-scope, verifier, installer-action, and release-check schemas."

[[tasks]]
task_id = "TASK-2026-05-30-runtime-schema-008"
title = "Protect historical license evidence"
status = "planned"
kind = "governance"
reason = "Historical archive files can contain stale prose but should not be silently rewritten."
acceptance_proof = "Behavior B-2026-05-30-live-license-docs passes."
behavior_ids = ["B-2026-05-30-live-license-docs"]
[[tasks.targets]]
file = "scripts/test-release-readiness.sh"
object = "live_license_scan_scope"
required_change = "Ensure license scans cover live docs and exclude historical archived evidence unless explicitly migrated."
[[tasks.targets]]
file = "docs/runtime-schemas.md"
object = "historical_evidence_policy"
required_change = "Document that historical evidence changes require registry-aware migration."
```
