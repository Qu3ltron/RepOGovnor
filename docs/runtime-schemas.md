# Runtime Schemas

Agent Governance treats runtime policy as typed data. Human output is only a
rendering layer; production gates should be debuggable from structured facts.

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

Completed legacy `schema_version = 1` manifests may remain as historical
evidence when the registry plan is completed or cancelled. New activations must
use schema version 2.

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

## Release Contract

`REQUIREMENTS.toml` owns release-source policy:

- `release_source.required`
- `release_source.stale_absent`
- `release_source.check_ids`
- `release_source.version_files`
- `tracked_for_ci.required`

`scripts/status.sh --release-source`, `scripts/release-version-check.sh`, and
`scripts/release-audit.sh` delegate release/version validation to the Rust
schema checks instead of owning separate file lists.

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
historical prose is not silently rewritten. Any archive rewrite needs an
approved registry-aware migration plan.
