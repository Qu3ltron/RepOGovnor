# Runtime Schemas

Agent Governance treats runtime policy as typed data. Human output is only a
rendering layer; production gates should be debuggable from structured facts.

## Command Report

Core commands support a JSON command envelope with `--format json` before the
command:

```json
{
  "schema_version": 2,
  "command": "metrics",
  "status": "pass",
  "summary": "Task registry metrics: plans=1"
}
```

Human text is a rendering layer over the command result.

Command-specific JSON is also available where the command owns a richer
diagnostic payload:

```bash
.codex/scripts/task-registry metrics --format json
.codex/scripts/task-registry install plan --format json
.codex/scripts/task-registry status-check --format json
```

For command-specific diagnostic JSON, failures still emit the raw diagnostic
report and exit nonzero. The output is not wrapped in `task-registry-flow
error:`. For global `--format json` failures, the CLI emits a schema version 2
`CommandReport` with the parsed command and whether a receipt was recorded.

## Receipt Event

Receipts use schema version 2. Read-only commands do not append receipts unless
`--record-receipt` is passed before the command. Mutating commands record local
receipts by default.

```json
{
  "schema_version": 2,
  "timestamp": "2026-05-31T00:00:00-04:00",
  "command": "activate",
  "outcome": "ok",
  "duration_ms": 7,
  "subject": {"kind": "plan", "id": "PLAN-...", "path": "docs/plans/x.md"},
  "summary": "PLAN_ACTIVATE docs/plans/x.md ok",
  "previous_event_hash_sha256": "0f...",
  "event_hash_sha256": "5a..."
}
```

Schema version 1 receipt lines are invalid for current runtime verification.
Metrics count them as malformed, `verify-chain` fails them, and `--repair`
refuses to rewrite them.

New schema version 2 receipts are hash-chained locally. The event hash is
computed from canonical event JSON with `event_hash_sha256` omitted; the
previous hash links to the immediately preceding non-empty event line. Metrics
report chained events, unchained v2 events, malformed events, and chain breaks.
Unchained v2 receipts are failures because they are not integrity evidence.
`verify-chain --repair` may repair parseable schema version 2 receipts only.

Receipt append is part of mutating command success. If the runtime cannot append
and sync the receipt, the mutating command fails instead of reporting success.
Receipt appends take an exclusive file lock and compute the next hash while that
lock is held.

## Diagnostic Report

JSON reports use this shape:

```json
{
  "schema_version": 1,
  "surface": "release-source",
  "summary": {"pass": 1, "warn": 0, "fail": 0, "skip": 0},
  "checks": [
    {
      "check_id": "release-file-present",
      "surface": "release-source",
      "path": "README.md",
      "severity": "info",
      "status": "pass",
      "expected": "file present",
      "actual": "file present",
      "remediation": "none"
    }
  ]
}
```

Required fields: `check_id`, `surface`, `path`, `severity`, `status`,
`expected`, `actual`, and `remediation`.

Known statuses: `pass`, `warn`, `fail`, `skip`.

Known severities: `info`, `warning`, `error`.

## Mutation Scopes

Task targets are converted to explicit mutation scopes.

- `exact_file`: one declared file only.
- `directory_tree`: an explicitly declared child tree.
- `generated_artifact`: a declared generated output.
- `governance_repair`: plan, registry, hook, and governance repair surfaces.

Prefix collisions are denied. `src/lib.rs` does not authorize
`src/lib.rs.bak`. Broad implementation targets such as `.`, `src/`, `docs/`,
`.codex/`, `.agents/`, and `.cursor/` are invalid task targets.

Runtime and governance files are task-bound. New plan drafting under
`docs/plans/*.md` is the bootstrap exception; direct edits to hook/config
surfaces such as `.codex/config.toml`, `.agents/hooks.json`, and
`.cursor/hooks.json` require an active task target or installer context.

## Behavior Verifiers

New Task Manifests use `schema_version = 2` and require behavior metadata plus
typed verifiers:

```toml
[[behaviors]]
behavior_id = "B-001-license-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Live docs state the license"
given = "The README exists"
when = "The license text is checked"
then = "The README states the MIT license"
confirmation = "rg -n \"MIT\" README.md"

[[behaviors.verifiers]]
type = "contains"
path = "README.md"
needle = "License: MIT"
```

Known behavior polarities:

- `positive` - expected good behavior is proven.
- `negative` - forbidden behavior is proven absent or rejected.
- `validation` - full-repo or release validation; supplementary proof only.

Each implementation gap in an active v2 plan must have at least one `positive`
and one `negative` behavior. Implementation, schema, authorization, migration,
release, and governance tasks cannot be backed only by `validation` behavior.

Known verifier types:

- `command`
- `file_exists`
- `file_absent`
- `contains`
- `not_contains`
- `json_valid`
- `json_schema`

`command` is the only verifier type that executes a shell command. File,
content, and JSON checks use typed runtime assertions. `json_valid` parses JSON.
`json_schema` requires both `path` and `schema_path` and validates the JSON
instance against that schema.

All runtime Task Manifests must use `schema_version = 2`. Completed historical
plans are migrated to v2 rather than accepted through a compatibility path.

## Activation Plan Contract

New activations must be comprehensive phased plans with these exact Markdown
sections:

- `Approved Scope`
- `Phased Required Change Checklist`
- `Per-Gap Success Criteria`
- `Validation Plan`
- `Walkthrough Evidence`
- `Task Manifest`

Activation fails before registry mutation when a plan contains unresolved
placeholder tokens, omits positive or negative gap behavior, uses wildcard task
target paths, uses broad target objects such as `backend` or `tests`, or links
implementation closure only to validation behavior.

Completed and cancelled tasks are terminal. Reactivating an unchanged plan is
idempotent, but activation cannot rewrite terminal task provenance, including
title, kind, source hash, acceptance proof, behavior IDs, targets, blockers, or
projected steps. Changed follow-up work requires a new `task_id`.

## Release Contract

`REQUIREMENTS.toml` owns release-source policy:

- `release_source.required`
- `release_source.executable`
- `release_source.stale_absent`
- `release_source.check_ids`
- `release_source.version_files`
- `tracked_for_ci.required`

`scripts/status.sh --release-source`, `scripts/release-version-check.sh`, and
`scripts/release-audit.sh` delegate release/version validation to the Rust
schema checks instead of owning separate file lists.

Executable release artifacts emit `release-file-executable` diagnostics.
Required release-source paths must be native files, not symlinks. A path listed
in `release_source.executable` must be a file with executable mode; a present
but non-executable script is a release failure, not a warning.

Required release scripts under `scripts/` must also be declared in
`release_source.executable`; omissions emit
`release-script-executable-undeclared`. Unix release checks enforce executable
mode bits. Non-Unix checks emit `release-executable-platform` diagnostics so
callers can see when mode enforcement is not available.

Rust task-registry source files under `rust/task-registry-flow-cli/src/**/*.rs`
must be declared in `release_source.required`; omissions emit
`release-rust-source-undeclared`. This keeps new modules and tests inside the
release manifest instead of relying on reviewers to notice new source files.

Governed runtime assets under Nix files, NixOS modules, hooks, templates,
Claude skills, and `tools/agent-governance` must also be declared in
`release_source.required`; omissions emit
`release-governed-source-undeclared`.

Local release waivers require a reason variable:

- `AGENT_GOVERNANCE_DIRTY_RELEASE_CHECK_REASON`
- `AGENT_GOVERNANCE_ACTIVE_RELEASE_TASKS_REASON`
- `AGENT_GOVERNANCE_AUDIT_TOOL_WAIVER_REASON`

When `AGENT_GOVERNANCE_FINAL_RELEASE=1`, release waivers are forbidden.

## Installer Actions

`MANIFEST.toml [install_policy]` owns installer action vocabulary and stale
path policy. Dry-run JSON uses:

```json
{
  "schema_version": 1,
  "surface": "installer-dry-run",
  "mode": "force",
  "target_root": "/repo",
  "actions": [
    {"path": ".codex/config.toml", "action": "would-create"}
  ]
}
```

Unknown action values fail rendering.

Project config keys that affect runtime paths are canonical in v2. Unsupported
or noncanonical values fail during rendering instead of being silently ignored.

## Historical Evidence

Historical task plans and archives are evidence, not live user docs. Stale
historical prose is not silently rewritten, but runtime manifests and registry
hashes are migrated when schema policy changes. Any archive rewrite needs an
approved registry-aware migration plan.
