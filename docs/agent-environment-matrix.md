# Agent Environment Matrix

The plugin renders native repo-local projections for Codex, Antigravity CLI,
Cursor, and Claude Code. The Rust registry CLI and CI are authoritative; hooks
are guardrails.

| Environment | Native files | Verification |
|-------------|--------------|--------------|
| Codex | `AGENTS.md`, `.codex/config.toml`, `.codex/hooks.json`, `.agents/skills/<skill>/SKILL.md` | `plugins/agent-governance/scripts/status.sh --env codex`; Codex hooks require a trusted project |
| Antigravity CLI | `GEMINI.md`, `.agents/hooks.json`, `.agents/skills/*.md`, `.agents/plugins/agent-governance` | `agy --version` must be 1.0.3 or newer; `agy plugin validate plugins/agent-governance` must process hooks |
| Cursor | `.cursor/rules/agent-governance.mdc`, `.cursor/rules/hook-gate-doctrine.mdc` (always-on gate triage), `.cursor/skills/<skill>/SKILL.md`, `.cursor/hooks.json` | `plugins/agent-governance/scripts/status.sh --env cursor`; hooks are **operational directions** at mutation time (deny = missing governance step; invalid JSON = gate repair on active hook target, not a new plan); optional user-level governed subagents complement repo-local skills/hooks |
| Claude Code | `CLAUDE.md`, `.claude/settings.json`, `.claude/skills/<skill>/SKILL.md` | `plugins/agent-governance/scripts/status.sh --strict`; `.claude/settings.json` must delegate PreToolUse to the canonical mutation gate |

Do not add compatibility shims for old workspace `.gemini/settings.json`, stale `.codex/settings.toml`, or `.codex/hooks/user-plan-approval.toml`. Current install removes those generated paths.

The 1600-line source-file limit is design-time policy for all agents and a CI
rule through `.codex/scripts/task-registry source-limit check`.

Shared hardening rules for every agent:

- Work only through exact active or planned task targets; ambiguous write paths
  fail closed.
- A terminal task is immutable after `completed` or `cancelled`; changed
  follow-up work needs a new task id.
- Keep the local receipt chain intact and verify it before production handoff.
- Release-source required files are native files, not symlinks, and final
  release validation does not accept local waiver flags.
