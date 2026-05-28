# Spectrum Arcana — project overlay (reference)

The live Tarot repo extends the portable **agent-governance** plugin with project-specific rules. Do not copy these into the plugin; keep them in the repo.

## What Tarot adds beyond the plugin

| Area | Tarot-specific |
|------|----------------|
| Product | Deck/booklet specs, veil/orientation constitution, no prediction UX |
| Architecture | Rust core, WASM facade, Tauri shell, API contract at `api/v1/` |
| Registry executor | `cargo run --bin task_registry --` with mutation gate in Rust |
| CI gates | `enforce-architecture`, `enforce-plan-commits`, `verify-artifacts`, plan-ID commit footer |
| Feature registry | `docs/feature-registry.toml` synced by CI after behavior-confirmed landing |
| Validation | Visual regression, mobile Maestro flows, nix/flake dev shell |
| Paths | `/home/hasnamuss/Tarot`, scratch under `reclaimed/work/tmp/spectrum-arcana-gap-closure` |

## Reference config (Spectrum Arcana)

Checked-in, docs-only config:

`examples/spectrum-arcana.project.config.toml`

Use with overlay mode to refresh plugin markers without replacing project-specific AGENTS/GEMINI prose:

```bash
plugins/agent-governance/scripts/install-to-workspace.sh \
  --config plugins/agent-governance/examples/spectrum-arcana.project.config.toml \
  --overlay

plugins/agent-governance/scripts/status.sh --strict
```

`cargo run --bin task_registry -- validate` enforces the same posture in Rust (see `src/agent_governance.rs`).

## Live canonical files (not replaced by default install)

Tarot's working tree already has tailored `AGENTS.md`, `GEMINI.md`, `.codex/settings.toml`, skills, and hooks.

| Mode | AGENTS/GEMINI | Infrastructure (.codex, hooks, skills) |
|------|---------------|----------------------------------------|
| default | Replace from templates | Write (or skip with `--overlay` + existing) |
| `--overlay` | Merge `<!-- agent-governance:begin/end -->` block | Skip if exists |
| `--overlay --force` | Merge markers | Overwrite all |

Use `--dry-run` to preview. Use full install (no `--overlay`) only when intentionally re-baselining a greenfield repo.
