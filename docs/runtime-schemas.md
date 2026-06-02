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
  "summary": "Task registry metrics: plans=1",
  "receipt_recorded": false
}
```

Human text is a rendering layer over the command result.

Command-specific JSON is also available where the command owns a richer
diagnostic payload:

```bash
.codex/scripts/task-registry metrics --format json
.codex/scripts/task-registry install plan --format json
.codex/scripts/task-registry status-check --format json
.codex/scripts/task-registry version-check validate --format json
.codex/scripts/task-registry backlog-check --format json
.codex/scripts/task-registry model-attribution-check --format json
.codex/scripts/task-registry cost-evidence-check --format json
.codex/scripts/task-registry cost-coverage-check --format json
.codex/scripts/task-registry cost-report --format json
.codex/scripts/task-registry cost-ingest codex-transcript --transcript-path <path> --session-id <id> --since-line <n> --until-line <n> --pricing-snapshot <path> --service-tier <tier> --target-kind commit --target-id <sha|HEAD> --boundary-session-id <id> --boundary-turn-id <id> --boundary-tool-use-id <id> --format json
.codex/scripts/task-registry cost-record unmeasured --target-kind commit --target-id <sha|HEAD> --reason <why-unmeasured> --provider codex --model gpt-5-codex --boundary-session-id <id> --boundary-turn-id <id> --boundary-tool-use-id <id>
```

For command-specific diagnostic JSON, failures still emit the raw diagnostic
report and exit nonzero. The output is not wrapped in `task-registry-flow
error:`. For global `--format json` failures, the CLI emits a schema version 2
`CommandReport` with the parsed command, typed `failure_code`, and whether a
receipt was recorded.

Known command values are the `CliCommand` enum values in the CLI help. Unknown
command strings fail before execution. Known `failure_code` values are:
`usage`, `runtime`, `serialization`, `receipt-append`, and
`diagnostic-report`.

`version-check` emits diagnostic reports on the `version` surface. It validates
release surfaces, the version roadmap, completed-plan coverage, prerelease
metadata, and final tag state when `release-check` is requested.

`backlog-check` emits diagnostic reports on the `backlog` surface. It validates
the drainable gap pipeline, required gap fields, reactivation conditions, and
negative non-claims.

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
  "subject": {"kind": "command", "id": "activate", "path": "."},
  "summary": "PLAN_ACTIVATE docs/plans/x.md ok",
  "previous_event_hash_sha256": "0f...",
  "event_hash_sha256": "5a..."
}
```

Receipt events use typed command, outcome, subject-kind, diagnostic-surface,
verifier-type, and status enums. Known subject kinds are `command`,
`mutation-target`, and `verifier-target`. Unknown values fail deserialization
and are not accepted as runtime evidence.

Mutation receipts may include `agent_model_attribution` and
`mutation_attribution`. Codex is the first measured adapter: supported Codex
write-intent mutation hooks require `model`, `session_id`, `turn_id`, and
`tool_use_id` before an active-target mutation is allowed. `PreToolUse` receipts
record `allowed` or `denied`; `PostToolUse` receipts record `observed`.
Non-Codex adapters are reported as `unmeasured` until they expose equivalent
identity evidence. `model-attribution-check` reports measured and unmeasured
mutation attribution posture without guessing missing provider data.

Receipt events may also include `cost_evidence`. Cost evidence is provider
neutral and classified as `measured`, `estimated`, or `unmeasured`.
`cost-evidence-check` validates the classification. Measured evidence requires
provider, model, usage counts, pricing snapshot, measurement timestamp,
attribution target, evidence source, amount evidence, pricing rates, pricing
snapshot hash, service tier, selected token-event digest, session id, and usage
contribution evidence. Estimated evidence requires an explicit estimation
method. Unmeasured evidence requires a reason and must not carry a cost amount.
Cost per commit is measured only for commits with complete, non-overlapping
commit-linked measured usage receipts. `cost-report` preserves unmeasured
entries instead of reporting them as zero cost.
Measured evidence with reasoning tokens must name the pricing policy for those
tokens. The runtime does not infer reasoning-token billing from model names or
agent narration.

`cost-ingest codex-transcript` reads actual local Codex transcript
`token_count` events. Codex hook docs expose `transcript_path`, but the
transcript format is not a stable hook interface, so ingestion fails closed when
required token-count fields are missing. The command requires explicit
`--transcript-path`, `--session-id`, `--since-line`, `--until-line`,
`--pricing-snapshot`, `--service-tier`, `--target-kind`, and `--target-id`
values. Optional `--boundary-session-id`, `--boundary-turn-id`, and
`--boundary-tool-use-id` bind measured usage to mutation attribution. Boundary
turn ids must appear in the selected transcript range. `--latest` and legacy
`--commit` selection are rejected because modified-time transcript selection is
not attribution evidence. `--append-receipt` records the measured cost evidence
locally.

`cost-record unmeasured` appends explicit unmeasured evidence with a reason and
optional provider, model, session, turn, and tool boundary ids. It rejects
amounts. Measured receipt dedupe includes service tier, pricing snapshot path
and hash, pricing rates, amount, target, and selected event digest; different
tiers are distinct evidence. `cost-report` groups measured entries by service
tier so tier-specific spend is not merged.

Reasoning token pricing currently supports only
`reasoning_tokens_not_billed_separately`. When a selected transcript range has
reasoning tokens, any missing or unsupported `reasoning_token_policy` fails
closed during ingest and replay.

`cost-coverage-check` compares model-attributed mutation receipts to measured or
unmeasured cost receipts by model/session/turn/tool boundary and fails when a
repo mutation lacks cost coverage. If the mutation attribution includes a
`tool_use_id`, matching cost evidence must name the same tool through
`boundary_tool_use_id` or `usage_contributions[].tool_use_ids`; an empty tool
list does not cover a tool-bound mutation.

Command-specific help is available with `cost-ingest --help` and
`cost-ingest codex-transcript --help`. Installed consumers can run
`task-registry-flow cost-ingest ...`; governed workspaces can run the same
interface through `.codex/scripts/task-registry cost-ingest ...`. The packaged
OpenAI Codex pricing snapshot is installed at
`$AGENT_GOVERNANCE_ASSET_ROOT/docs/pricing/openai-codex-rate-card-2026-06-02.toml`.
Supported attribution target kinds are `commit`, `plan`, `task`,
`verifier-run`, `landing-attempt`, `retry`, `release-cycle`, and `session`.

```json
{
  "cost_evidence": {
    "status": "measured",
    "evidence_source": "codex-transcript-token-count",
    "attribution_target": {"kind": "commit", "id": "abc1234"},
    "provider": "openai",
    "model_slug": "gpt-5.5",
    "usage": {"input_tokens": 1200, "cached_input_tokens": 100, "output_tokens": 300},
    "pricing": {"source": "https://help.openai.com/en/articles/20001106-codex-rate-card", "version": "2026-06-02", "currency": "CREDITS", "service_tier": "codex-cloud", "snapshot_path": "docs/pricing/openai-codex-rate-card-2026-06-02.toml", "snapshot_sha256": "5b...", "reasoning_token_policy": "reasoning_tokens_not_billed_separately"},
    "pricing_rates": {"input_micros_per_million": 125000000, "cached_input_micros_per_million": 12500000, "output_micros_per_million": 750000000},
    "amount": {"currency": "CREDITS", "amount_micros": 42},
    "usage_contributions": [{"source_kind": "codex-transcript-token-count", "source_path": "<local-private-codex-transcript>", "source_sha256": "9f...", "start_line": 10, "end_line": 20, "event_count": 3, "model_slug": "gpt-5.5", "model_context_line": 9, "session_id": "session-id", "selected_event_digest_sha256": "9f...", "turn_ids": ["turn-id"], "tool_use_ids": ["tool-use-id"]}],
    "measurement_timestamp": "2026-06-02T00:00:00Z"
  }
}
```

Schema version 1 receipt lines are invalid for current runtime verification.
Metrics count them as malformed, `verify-chain` fails them, and `--repair`
refuses to rewrite them.

New schema version 2 receipts are hash-chained locally. The event hash is
computed from canonical event JSON with `event_hash_sha256` omitted; the
previous hash links to the immediately preceding non-empty event line. Metrics
report chained events, unchained v2 events, malformed events, and chain breaks.
They also count cost evidence receipts as `cost_measured_events`,
`cost_estimated_events`, `cost_unmeasured_events`, and
`cost_measured_amount_micros`; counts and measured totals are local evidence
posture, not billing authority.
Unchained v2 receipts are failures because they are not integrity evidence.
`verify-chain --repair` may repair parseable schema version 2 receipts only.

Receipt append is part of mutating command success. If the runtime cannot append
and sync the receipt, the mutating command fails instead of reporting success.
Receipt appends take an exclusive file lock and compute the next hash while that
lock is held.

## Production Runtime Invariants

Runtime checks fail closed when provenance cannot be proven:

- Mutating commands must append and sync a schema version 2 receipt before the
  command reports success.
- Receipt chain verification treats malformed, unchained, or tampered events as
  failures; `verify-chain --repair` is limited to parseable schema version 2
  receipts.
- Mutation hooks must extract deterministic write targets. Ambiguous compact
  shell redirections, variable redirects, and inline write calls without a
  provable path are denied before task target matching.
- Completed and cancelled tasks are terminal task states. Unchanged
  reactivation is idempotent; changing terminal task provenance requires a new
  task id.
- `verify-landing` owns new completed-status writes. It rejects registry-only
  changed-file sets and changed files that do not bind to active task targets.
- Release-source gates are manifest-backed through `REQUIREMENTS.toml`.
  Required files must be native files, not symlinks.
- Final release mode forbids local waiver variables.
- Prerelease automation may push only branch state and `vX.Y.Z-rc.N` tags.
  Final `vX.Y.Z` release tagging, tag push, GitHub Release creation, and public
  publication remain manual.

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

Known surfaces: `cli`, `manifest`, `migration`, `release-source`,
`tracked-for-ci`, `source-limit`, `source-limit-plan`, `status`, `version`,
`backlog`, `receipt-chain`, `receipt-chain-fix`, and `model-attribution`.
Unknown surfaces fail deserialization.

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

## Reviewer Report

`reviewer-report` is a read-only local handoff surface:

```bash
.codex/scripts/task-registry reviewer-report
.codex/scripts/task-registry reviewer-report --format markdown
```

It summarizes active plans, landed tasks, changed targets, receipt metrics, and
blocked or deferred work. It also states the proof boundary: governance proof is
not product correctness proof. The report is intended for pull request or
review handoff text; domain tests, code review, security review, and product
acceptance evidence remain project-owned. The default format is compact text.
`--format markdown` emits local Markdown suitable for manual pull request paste;
it does not post to GitHub or call a remote service.

## Future Policy And Cost Artifacts

RepOGovnor's product direction is engineering policy compliance for
agent-assisted repos. Future runtime surfaces should evaluate declared
engineering policy and emit local compliance artifacts. Those artifacts should
identify policy version, repo commit or build id, evaluated controls, evidence,
waivers, and unproven controls.

Token and cost evidence should be modeled as measured, estimated, or unmeasured.
Measured cost requires structured usage evidence, provider and model identity,
pricing snapshot or version, timestamp, attribution target, evidence source,
pricing rates, and contribution evidence. The current release can calculate
Codex credit spend from actual local transcript token-count events for priced
Codex models. Cost per commit is reliable only for explicit commit-linked
measured receipts; unavailable, hidden, or unpriced usage must be reported as
unmeasured rather than guessed.

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

## Landing Completion Contract

`verify-landing --plan-id <plan_id> --changed-files <paths>` is the only runtime
path that writes `completed` status for new task rows. The command validates the
registry and active manifests, maps each non-registry changed file to exactly one
active task target, runs the selected task behavior verifiers, then writes
completion evidence onto the task row. Direct `status TASK-ID completed` fails
closed.

Registry-generated files such as `docs/task-registry.toml`,
`docs/task-registry/events.jsonl`, and completed-task archives never satisfy
landing by themselves. They may accompany a landing, but at least one changed
file must bind to a task target.

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
