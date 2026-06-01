# Antigravity — RepOGovnor Agent Instructions

Antigravity (`agy`) entry point. Shared repo rules: [AGENTS.md](AGENTS.md) (Codex uses that file as its primary entry).

Keep it terse.Do not update the user with non-actionable items. Wait to the end of your implimentation and provide updates then. No fluff, concise and focused.[CRITICAL]

## What Antigravity reads

| Asset | Path | Role |
|-------|------|------|
| Context | [GEMINI.md](GEMINI.md) | Always-loaded instructions (this file) |
| Context | [AGENTS.md](AGENTS.md) | Shared governance |
| Workspace hooks | [.agents/hooks.json](.agents/hooks.json) | PreToolUse hard gate on implementation writes (when executor supports it) |
| Mutation gate | `tools/agent-governance/pre-tool-use-gap-closure.sh` | Calls plugin-owned `verify-mutation-hook` command |
| Workspace skills | [.agents/skills/](.agents/skills/) | AGY markdown skills plus Codex-compatible skill folders |
| User CLI settings | `~/.gemini/antigravity-cli/settings.json` | Model, permissions, trusted workspaces |

Antigravity CLI does **not** read `.codex/config.toml`, `.codex/hooks.json`,
`.codex/agent-governance.toml`, or repo-local Gemini settings. Do not create a
repo-local Gemini settings file.

## Codebase policy

Read [AGENTS.md](AGENTS.md) before feature work.

- **Repo boundary.** Work only in `/home/hasnamuss/reclaimed/work/RepOGovnor`. Scratch: `/home/hasnamuss/reclaimed/work/tmp/repogovnor-gap-closure/<short-slug>/`.
- **Architecture and product rules** live in project docs — not in this file.

## Mandatory skills

Canonical skill content ships from `plugins/agent-governance/skills/`. Antigravity discovers markdown projections in `.agents/skills/*.md`; Codex discovers folder projections in `.agents/skills/<skill>/`; Cursor discovers `.cursor/skills/<skill>/`; Claude Code discovers `.claude/skills/<skill>/`.

| Skill | Antigravity path | When |
|-------|------------------|------|
| `gap-closure-contract` | [.agents/skills/gap-closure-contract.md](.agents/skills/gap-closure-contract.md) | After gap analysis + user approval, before plan edits or implementation |
| `task-registry-flow` | [.agents/skills/task-registry-flow.md](.agents/skills/task-registry-flow.md) | Plan activation, status, deferrals, handoff |

## Plan activation (before code edits)

After user-approved gap closure:

1. Write or refresh `docs/plans/<short-slug>.md` with Approved Scope, Phased Required Change Checklist, Per-Gap Success Criteria, Validation Plan, Walkthrough Evidence, and one fenced `## Task Manifest` TOML block.
2. Manifest: `schema_version = 2`, `[[behaviors]]` with `gap_id` and `polarity`, typed `[[behaviors.verifiers]]`, and `[[tasks]]` rows linked through `behavior_ids`.
3. `.codex/scripts/task-registry activate docs/plans/<short-slug>.md`
4. Confirm tasks in `docs/task-registry.toml` with matching `source_plan_hash_sha256`.
5. Only then edit implementation files listed in active task targets.

**Hard gate:** `.agents/hooks.json` calls `.codex/scripts/task-registry verify-mutation-hook` and blocks unbound implementation writes outside governance files or active/planned task targets.

Handoff: `.codex/scripts/task-registry report <plan_id>`; archive completed when supported.

## Governance reminders

- **Spec authority:** `README.md` → `docs/runtime-schemas.md` → `docs/releases/v2.md` → `VISION.md` → `ROADMAP.md`. On conflict, ask.
- **Registry:** use only `.codex/scripts/task-registry` opcodes; typed validation via `validate`; new activations require Task Manifest schema v2, exact targets, positive and negative gap behavior, and no placeholders.
- **Deferrals:** `TASK_DEFER` via registry CLI `defer`; requires `deferral_governance_basis` + `reactivation_condition`.
- **Source file limit:** 1600 lines is a hard design-time budget for source/governance files. Run `.codex/scripts/task-registry source-limit check`; use `.codex/scripts/task-registry source-limit plan --path <file>` before splitting existing violations.
- **Production hardening:** use exact active or planned task targets; ambiguous shell redirections, compact redirects, and inline write calls without a deterministic path fail closed.
- **Terminal task rule:** `completed` and `cancelled` are immutable. Changed follow-up work needs a new `task_id`; do not rewrite terminal provenance.
- **Receipt chain:** keep local receipts intact and run `.codex/scripts/task-registry verify-chain --format json` before production handoff.
- **Release source:** required files are native files, not symlinks, and final release validation forbids local waiver variables.
- **Version release:** automation may push prerelease branch state and `vX.Y.Z-rc.N` tags only; final tags, final tag pushes, GitHub Releases, and public release publication remain manual.
- **Zero backwards compatibility:** do not add legacy shims, old hook paths, or settings compatibility.

## Verify in Antigravity

Run `agy --version` and require 1.0.3 or newer. Run `agy` in this repo, then `/skills` — expect `gap-closure-contract` and `task-registry-flow`. Hooks load from `.agents/hooks.json`; plugin package validation must show hooks processed with `agy plugin validate plugins/agent-governance`.

<!-- agent-governance:begin -->
## Agent governance (portable plugin)

Plugin: [plugins/agent-governance/](plugins/agent-governance/). **Policy and workflow:** [AGENTS.md](AGENTS.md) and sections above — this block is install posture only.

- Posture: `plugins/agent-governance/scripts/status.sh`
- Skills: `.agents/skills/*.md` for AGY, `.agents/skills/<skill>/` for Codex, `.cursor/skills/<skill>/` for Cursor, `.claude/skills/<skill>/` for Claude Code
- Antigravity hook: `.agents/hooks.json` -> `tools/agent-governance/pre-tool-use-gap-closure.sh` via `.codex/scripts/task-registry verify-mutation-hook`
- Cursor hook: `.cursor/hooks.json` + `.cursor/hooks/gap-closure-gate.sh`
- Claude Code hook: `.claude/settings.json` PreToolUse -> `tools/agent-governance/pre-tool-use-gap-closure.sh`
- Source limit: 1600 lines; check `.codex/scripts/task-registry source-limit check`; split existing violations with `.codex/scripts/task-registry source-limit plan --path <file>`
<!-- agent-governance:end -->
