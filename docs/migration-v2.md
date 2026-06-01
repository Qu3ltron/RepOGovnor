# V2 Migration Guide

This guide is for existing repositories moving to Agent Governance v2.

V2 is a hard cutover. Removed v0.x and v1 paths are not compatibility surfaces.
Use `--merge` for existing repositories and `--force` only when intentionally
rebaselining all managed governance files.

## Before You Start

1. Commit or stash unrelated local work.
2. Make sure the plugin is vendored or submoduled under
   `plugins/agent-governance`.
3. Copy `project.config.example.toml` to `project.config.toml`.
4. Edit repo name, repo root, scratch root, hook script path, and agent paths in
   `project.config.toml`.

## Existing Workspace Migration

Run a dry-run first:

```bash
plugins/agent-governance/scripts/install-to-workspace.sh \
  --config project.config.toml \
  --dry-run
```

Review the planned writes, then merge the v2 projection:

```bash
plugins/agent-governance/scripts/install-to-workspace.sh \
  --config project.config.toml \
  --merge
```

`--merge` refreshes managed governance files, preserves a valid task registry,
and removes stale v1/v0.x hook and settings paths that v2 rejects.

## Fresh Rebaseline

Use `--force` only when the repository is intentionally accepting a complete
managed projection:

```bash
plugins/agent-governance/scripts/install-to-workspace.sh \
  --config project.config.toml \
  --force
```

## Required Validation

After install, run:

```bash
.codex/scripts/task-registry validate
.codex/scripts/task-registry source-limit check
.codex/scripts/task-registry verify-chain --format json
plugins/agent-governance/scripts/status.sh --strict
```

Commit every path listed by `REQUIREMENTS.toml` under
`[tracked_for_ci].required`. Fresh CI clones only see tracked files.

## What Changed In V2

- Consumer repositories use `.codex/scripts/task-registry` as the canonical
  registry command.
- New plans use Task Manifest `schema_version = 2`.
- Every implementation gap needs positive and negative behavior coverage.
- Runtime governance writes are task-bound, with plan drafting as the narrow
  bootstrap exception.
- Direct completed-status writes are rejected; `verify-landing` owns task
  completion after changed files bind to active task targets.
- Local receipts use schema version 2 hash chains.
- Required release-source files must be native files, not symlinks.

## If Validation Fails

Treat failures as state evidence, not as installer noise.

- Missing CI artifact: commit the required path or rerun install.
- Marker failure: inspect `AGENTS.md` and `GEMINI.md` marker blocks.
- Stale path: remove the legacy file through an approved governance plan, or
  rerun `--merge` if it is a managed stale path.
- Receipt-chain failure: inspect `docs/task-registry/events.jsonl` and run
  `verify-chain --format json` for structured diagnostics.
- Source-limit failure: run
  `.codex/scripts/task-registry source-limit plan --path <file>` and split the
  file through a governed closure.
