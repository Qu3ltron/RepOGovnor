# Governance-plugin — Agent Instructions

Portable governance entry installed by the **agent-governance** plugin. Project-specific architecture, CI, and product rules belong in your constitution, vision, and AGENTS extensions — not in the plugin.

## Governance

Read before feature work:

| Document | Role |
|----------|------|
| [README.md](README.md) | Non-negotiable product and engineering rules |
| [VISION.md](VISION.md) | Why the project exists; direction |
| [docs/](docs/) | Exploratory drafts — not spec authority unless your project says otherwise |
| [.agents/skills/gap-closure-contract/](.agents/skills/gap-closure-contract/) | Codex skill for mandatory gap closure contracts before approved implementation |
| [.agents/skills/task-registry-flow/](.agents/skills/task-registry-flow/) | Codex skill for plugin-owned plan activation and `docs/task-registry.toml` workflow |
| [.cursor/rules/agent-governance.mdc](.cursor/rules/agent-governance.mdc) | Cursor-native design-time rule |
| [GEMINI.md](GEMINI.md) | Antigravity (`agy`) entry — same governance, Antigravity-native paths |

**Active agents:** Codex (this file + [.codex/config.toml](.codex/config.toml) + [.codex/hooks.json](.codex/hooks.json) + [.agents/skills/](.agents/skills/)), Antigravity CLI 1.0.3+ ([GEMINI.md](GEMINI.md) + [.agents/skills/](.agents/skills/) + [.agents/hooks.json](.agents/hooks.json)), and Cursor ([.cursor/rules/agent-governance.mdc](.cursor/rules/agent-governance.mdc) + [.cursor/skills/](.cursor/skills/) + [.cursor/hooks.json](.cursor/hooks.json)).

**Authority order** (highest first): `README.md` → `docs/runtime-schemas.md` → `docs/releases/v2.md` → `VISION.md` → `ROADMAP.md`. Do not guess across conflicts; ask.

## Gap closure and registry (mandatory)

After user-approved gap closure:

1. Load `$gap-closure-contract` and write `docs/plans/<slug>.md` with the full phased structure: Approved Scope, Phased Required Change Checklist, Per-Gap Success Criteria, Validation Plan, Walkthrough Evidence, and `schema_version = 2` Task Manifest.
2. `$task-registry-flow` → `.codex/scripts/task-registry activate docs/plans/<file>.md` before code edits.
3. Mark tasks `completed` only after linked typed behavior verifiers pass.
4. Final handoff: `.codex/scripts/task-registry report <plan_id>` and `.codex/scripts/task-registry metrics`; archive completed history with `.codex/scripts/task-registry archive-completed`.

Registry work uses the plugin-owned Rust CLI at `.codex/scripts/task-registry`. Typed TOML validation, behavior verification, mutation checks, and local metrics are canonical there. New activations require Task Manifest schema v2, behavior `gap_id`, behavior `polarity`, typed verifiers, and positive plus negative behavior coverage for each implementation gap. `TASK_DEFER` requires `deferral_governance_basis` and `reactivation_condition`.



## Architecture

Define stack boundaries, layer ownership, and forbidden patterns in project docs ([README.md](README.md), README, or a dedicated architecture doc). This plugin does not prescribe language, framework, or CI layout.

## Source file limit

1600 lines is a hard limit for source, scripts, configs, docs, templates, and governance files. Treat this as a design-time constraint: split before adding code or prose that would exceed the limit. Check with `.codex/scripts/task-registry source-limit check`; for existing violations, get a deterministic split path with `.codex/scripts/task-registry source-limit plan --path <file>`.

## Validation

Each closure contract carries its own **Validation Plan** with runnable commands. Suggested project gates (from install config):

**Focused:** `.codex/scripts/task-registry source-limit check`, `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`

**Full:** `.codex/scripts/task-registry validate`, `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`, `bash scripts/test-release-readiness.sh all`

Run the smallest honest subset that proves the approved scope; broaden when shared infrastructure or contracts change.

## Workspace boundary

- **Repo root:** `/home/hasnamuss/reclaimed/work/Governance-plugin`
- **Scratch:** `/home/hasnamuss/reclaimed/work/tmp/governance-plugin-gap-closure/<short-slug>/`
- Confirm `git rev-parse --show-toplevel` matches repo root before mutating files.

<!-- agent-governance:begin -->
## Agent governance (portable plugin)

Maintained by [plugins/agent-governance/](plugins/agent-governance/). **Workflow authority:** the main sections above in this file — not this block.

| Item | Location |
|------|----------|
| Reference config | `plugins/agent-governance/examples/spectrum-arcana.project.config.toml` |
| Posture check | `plugins/agent-governance/scripts/status.sh` |
| Refresh overlay | `plugins/agent-governance/scripts/install-to-workspace.sh --config plugins/agent-governance/examples/spectrum-arcana.project.config.toml --merge` |
| Registry CLI | `.codex/scripts/task-registry` |
| Mutation verify | `.codex/scripts/task-registry verify-mutation-hook` |
| Hook script | `tools/agent-governance/pre-tool-use-gap-closure.sh` |
| Codex hooks | `.codex/hooks.json` -> canonical gate (`PreToolUse`) |
| Antigravity hooks | `.agents/hooks.json` -> canonical gate (`run_command` / file edit tools) |
| Cursor hooks | `.cursor/hooks.json` + `.cursor/hooks/gap-closure-gate.sh` |
| Skills | `.agents/skills/*`, `.agents/skills/*.md`, `.cursor/skills/*` |
| Source limit | 1600 lines; check `.codex/scripts/task-registry source-limit check`; split plan `.codex/scripts/task-registry source-limit plan --path <file>` |
<!-- agent-governance:end -->
