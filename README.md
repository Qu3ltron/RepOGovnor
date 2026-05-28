# Agent Governance Plugin

Portable **Codex + Cursor + Antigravity** workflow: gap-closure contract, task-registry flow, approval hooks, and mutation gates.

Repository: https://github.com/Qu3ltron/Governance-plugin

## Add to a project

Vendoring (recommended path in consumer repos):

```bash
# from your project root
mkdir -p plugins
git submodule add https://github.com/Qu3ltron/Governance-plugin.git plugins/agent-governance
# or: git clone https://github.com/Qu3ltron/Governance-plugin.git plugins/agent-governance

cp plugins/agent-governance/project.config.example.toml project.config.toml
# edit project.config.toml

plugins/agent-governance/scripts/install-to-workspace.sh --config project.config.toml --overlay

mkdir -p .agents/plugins
ln -sfn ../../plugins/agent-governance .agents/plugins/agent-governance

plugins/agent-governance/scripts/status.sh --strict
```

Antigravity discovers the plugin via `.agents/plugins/agent-governance` (relative symlink to `plugins/agent-governance`).

## Layout (this repo)

```
.
├── plugin.json
├── REQUIREMENTS.toml      # CI tracked-artifact requirements (authoritative)
├── hooks.json
├── project.config.example.toml
├── skills/
├── templates/
├── scripts/
│   ├── install-to-workspace.sh
│   ├── render-from-config.sh
│   ├── status.sh
│   └── pre-tool-use-gap-closure.sh
├── rules/
└── examples/
    └── spectrum-arcana-overlay.md
```

## CI tracked artifacts (required)

After install, commit every path in [`REQUIREMENTS.toml`](REQUIREMENTS.toml) → `[tracked_for_ci].required`:

- `.codex/governance-cli.env`
- `.agents/plugins/agent-governance` (relative symlink → `../../plugins/agent-governance`)
- `.cursor/hooks.json` and `.cursor/hooks/gap-closure-gate.sh` (Cursor `preToolUse` Write gate)
- `AGENTS.md` and `GEMINI.md` (with `<!-- agent-governance:begin/end -->` overlay markers)
- `.cursor/skills/gap-closure-contract/PROJECT.md`
- `.cursor/skills/task-registry-flow/PROJECT.md`

Fresh CI clones only include tracked files. **`scripts/status.sh --strict`** fails if any path is missing or untracked.

Install prints a `git add` checklist automatically.

## Install modes

```bash
# Greenfield
./scripts/install-to-workspace.sh --config project.config.toml

# Existing repo (merge overlay markers; skip existing infra unless --force)
./scripts/install-to-workspace.sh --config project.config.toml --overlay

# Posture check
./scripts/status.sh --strict
```

## Consumer registry integration

Projects using Rust `task_registry` can enforce the same tracked-artifact list via `validate_posture` / `validate_tracked_for_ci` (see Spectrum Arcana `src/agent_governance.rs` for reference).

Mutation hook default: `cargo run --quiet --bin task_registry -- verify-mutation-hook` (override in `project.config.toml`).

## Examples

- [`examples/spectrum-arcana-overlay.md`](examples/spectrum-arcana-overlay.md) — Tarot-specific overlay
- [`examples/spectrum-arcana.project.config.toml`](examples/spectrum-arcana.project.config.toml) — reference config

## What is portable vs project-specific

| Portable (this plugin) | Project-specific (your repo) |
|-----------------------|------------------------------|
| Gap closure contract structure | Constitution, vision, product rules |
| Task manifest schema | Registry CLI implementation |
| Codex hook metadata | CI verify commands |
| Antigravity skills + hooks | Architecture, implementation path globs |
| Cursor preToolUse Write gate | `.cursor/hooks.json` + adapter script |
