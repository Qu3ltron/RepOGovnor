# Agent Governance Plugin

Portable **Codex + Cursor + Antigravity CLI** workflow: gap-closure contract, plugin-owned task-registry CLI, native hooks, mutation gates, and local efficacy receipts.

Hard source/governance file limit: **1600 lines**. The plugin checks this during validation and provides deterministic split guidance.

Repository: https://github.com/Qu3ltron/Governance-plugin

Release: [`VERSION`](VERSION), [`CHANGELOG.md`](CHANGELOG.md), [`LICENSE`](LICENSE), and [`docs/releases/v2.md`](docs/releases/v2.md).

## Add to a project

Vendoring (recommended path in consumer repos):

```bash
# from your project root
mkdir -p plugins
git submodule add https://github.com/Qu3ltron/Governance-plugin.git plugins/agent-governance
# or vendor a copy under plugins/agent-governance without a nested .git directory

cp plugins/agent-governance/project.config.example.toml project.config.toml
# edit project.config.toml

plugins/agent-governance/scripts/install-to-workspace.sh --config project.config.toml --merge

.codex/scripts/task-registry validate
.codex/scripts/task-registry source-limit check
.codex/scripts/task-registry metrics
plugins/agent-governance/scripts/status.sh --strict
```

Antigravity CLI targets `agy` 1.0.3 or newer. It discovers workspace hooks and skills from `.agents/`, and plugin packages must pass `agy plugin validate`.

## Layout (this repo)

```
.
├── plugin.json
├── .codex-plugin/plugin.json
├── REQUIREMENTS.toml      # CI tracked-artifact requirements (authoritative)
├── hooks/
├── project.config.example.toml
├── skills/
├── rust/task-registry-flow-cli/
├── templates/
├── scripts/
│   ├── install-to-workspace.sh
│   ├── render-from-config.sh
│   ├── status.sh
│   ├── release-audit.sh
│   ├── release-version-check.sh
│   └── pre-tool-use-gap-closure.sh
├── rules/
└── examples/
    └── spectrum-arcana-overlay.md
```

## CI tracked artifacts (required)

After install, commit every path in [`REQUIREMENTS.toml`](REQUIREMENTS.toml) → `[tracked_for_ci].required`:

- `plugins/agent-governance` (repo-local submodule or vendored checkout)
- `.codex/config.toml`, `.codex/hooks.json`, `.codex/agent-governance.toml`
- `.codex/governance-cli.env`, `.codex/scripts/task-registry`
- `.codex/templates/task-registry-plan-template.md`
- `.github/workflows/agent-governance.yml`
- `.agents/plugins/agent-governance` (relative symlink → `../../plugins/agent-governance`)
- `.agents/hooks.json`, `.agents/skills/*`, and `.agents/skills/*.md`
- `.cursor/rules/agent-governance.mdc`
- `.cursor/hooks.json` and `.cursor/hooks/gap-closure-gate.sh`
- `AGENTS.md` and `GEMINI.md` (with `<!-- agent-governance:begin/end -->` overlay markers)
- `.cursor/skills/gap-closure-contract/PROJECT.md`
- `.cursor/skills/task-registry-flow/PROJECT.md`
- `docs/task-registry.toml`
- `docs/task-registry/events.jsonl`

Fresh CI clones only include tracked files. **`scripts/status.sh --strict`** fails if any path is missing or untracked.

Install prints a `git add` checklist automatically.

Plugin-source release readiness is separate from consumer install posture:

```bash
scripts/status.sh --release-source
scripts/release-version-check.sh
scripts/release-audit.sh
```

`--release-source` checks package metadata, v2 release artifacts, plugin hooks/skills, registry state, and source package hygiene. `--strict` remains the consumer-workspace check after install.

## Install modes

```bash
# Project full install/rebaseline projection; no files changed
./scripts/install-to-workspace.sh --config project.config.toml --dry-run

# Existing repo; merge AGENTS/GEMINI markers and refresh managed plugin files
./scripts/install-to-workspace.sh --config project.config.toml --merge

# New repo or intentional rebaseline; applies the dry-run projection
./scripts/install-to-workspace.sh --config project.config.toml --force

# Posture check
./scripts/status.sh --strict
```

## Consumer registry integration

The plugin owns the task-registry executor. Consumer repos use:

```bash
.codex/scripts/task-registry validate
.codex/scripts/task-registry activate docs/plans/example.md
.codex/scripts/task-registry status TASK-YYYY-MM-DD-example-001 completed
.codex/scripts/task-registry defer TASK-YYYY-MM-DD-example-002 "governed basis" "reactivation evidence"
.codex/scripts/task-registry report PLAN-YYYY-MM-DD-example
.codex/scripts/task-registry metrics
.codex/scripts/task-registry source-limit check
.codex/scripts/task-registry source-limit plan --path src/large.rs
```

Project-native `task_registry` binaries are noncanonical. Existing valid `docs/task-registry.toml` files are preserved, but install hard-cuts command wiring to `.codex/scripts/task-registry`. Incompatible registries fail install instead of being overwritten.

`--merge` is a hard cutover for legacy governance files: stale `.codex/settings.toml`, root `hooks.json`, `.gemini/settings.json`, and old Antigravity hook paths are removed so `status.sh --strict` matches the installed state. The rendered registry wrapper changes to the repo root before running, so it works from subdirectories without creating nested registry artifacts.

Mutation hook default: `.codex/scripts/task-registry verify-mutation-hook --format codex|antigravity|cursor`.

Local efficacy measurement lives in `docs/task-registry/events.jsonl`; no network telemetry is emitted.

The source limit covers source, scripts, configs, docs, templates, and governance files. Generated locks, vendored dependencies, build outputs, task-registry receipts, and archive shards are excluded.

## Examples

- [`examples/spectrum-arcana-overlay.md`](examples/spectrum-arcana-overlay.md) — Tarot-specific overlay
- [`examples/spectrum-arcana.project.config.toml`](examples/spectrum-arcana.project.config.toml) — reference config

## What is portable vs project-specific

| Portable (this plugin) | Project-specific (your repo) |
|-----------------------|------------------------------|
| Gap closure contract structure | Constitution, vision, product rules |
| Rust registry CLI, task manifest schema | Project-specific behavior confirmation commands |
| Codex config, skills, and hooks | CI verify commands |
| Antigravity CLI skills + hooks | Architecture, implementation path globs |
| Cursor rules, skills, and hooks | `.cursor/hooks.json` + adapter script |
| 1600-line source/governance limit + split planner | Actual module boundaries chosen during implementation |
