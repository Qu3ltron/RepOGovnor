# Spectrum Arcana — project overlay (reference)

The live Tarot repo extends the portable **agent-governance** plugin with project-specific rules. Do not copy these into the plugin; keep them in the repo.

## What Tarot adds beyond the plugin

| Area | Tarot-specific |
|------|----------------|
| Product | Deck/booklet specs, veil/orientation constitution, no prediction UX |
| Architecture | Rust core, WASM facade, Tauri shell, API contract at `api/v1/` |
| Registry executor | `.codex/scripts/task-registry` with plugin-owned Rust mutation gate |
| CI gates | `enforce-architecture`, `enforce-plan-commits`, `verify-artifacts`, plan-ID commit footer |
| Source limit | 1600-line source/governance hard gate with plugin split planner |
| Feature registry | `docs/feature-registry.toml` synced by CI after behavior-confirmed landing |
| Validation | Visual regression, mobile Maestro flows, nix/flake dev shell |
| Paths | `/home/hasnamuss/Tarot`, scratch under `reclaimed/work/tmp/spectrum-arcana-gap-closure` |

## Reference config (Spectrum Arcana)

Checked-in, docs-only config:

`examples/spectrum-arcana.project.config.toml`

Use `--merge` to refresh plugin markers without replacing project-specific AGENTS/GEMINI prose:

```bash
plugins/agent-governance/scripts/install-to-workspace.sh \
  --config plugins/agent-governance/examples/spectrum-arcana.project.config.toml \
  --merge

plugins/agent-governance/scripts/status.sh --strict
```

`.codex/scripts/task-registry validate` enforces the portable registry posture.
`.codex/scripts/task-registry source-limit check` enforces the 1600-line source/governance limit.

## Live canonical files

Tarot's working tree already has tailored `AGENTS.md`, `GEMINI.md`, `.codex/agent-governance.toml`, skills, and hooks.

| Mode | AGENTS/GEMINI | Infrastructure (.codex, hooks, skills) |
|------|---------------|----------------------------------------|
| `--dry-run` | Project full replacement | Project create/update/remove/chmod only |
| `--merge` | Merge `<!-- agent-governance:begin/end -->` block | Refresh managed plugin files; preserve registry/events and remove stale legacy paths |
| `--force` | Replace from templates | Apply the dry-run projection, including stale path removal |

Use `--force` only when intentionally re-baselining a repo to the plugin projection.
