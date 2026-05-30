# Agent Environment Matrix

The plugin renders native repo-local projections for Codex, Antigravity CLI, and Cursor. The Rust registry CLI and CI are authoritative; hooks are guardrails.

| Environment | Native files | Verification |
|-------------|--------------|--------------|
| Codex | `AGENTS.md`, `.codex/config.toml`, `.codex/hooks.json`, `.agents/skills/<skill>/SKILL.md` | `plugins/agent-governance/scripts/status.sh --env codex`; Codex hooks require a trusted project |
| Antigravity CLI | `GEMINI.md`, `.agents/hooks.json`, `.agents/skills/*.md`, `.agents/plugins/agent-governance` | `agy --version` must be 1.0.3 or newer; `agy plugin validate plugins/agent-governance` must process hooks |
| Cursor | `.cursor/rules/agent-governance.mdc`, `.cursor/skills/<skill>/SKILL.md`, `.cursor/hooks.json` | `plugins/agent-governance/scripts/status.sh --env cursor`; `cursor-agent --plugin-dir plugins/agent-governance` can load local plugin code |

Do not add compatibility shims for old workspace `.gemini/settings.json`, stale `.codex/settings.toml`, or `.codex/hooks/user-plan-approval.toml`. Current install removes those generated paths.

The 1600-line source-file limit is design-time policy for all three agents and a CI rule through `.codex/scripts/task-registry source-limit check`.
