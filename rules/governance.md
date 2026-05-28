# Agent governance (portable)

## Before implementation writes

1. Load **gap-closure-contract** after gap analysis and explicit user approval.
2. Write or refresh `docs/plans/<short-slug>.md` with `## Task Manifest` (typed TOML, `[[behaviors]]`, `[[tasks]]`).
3. Run **task-registry-flow** and activate the plan through the configured registry CLI.
4. Only then mutate paths listed in active task targets.

## Hard gate (when enabled)

PreToolUse hooks call the project's configured `verify-mutation-hook` command. Unbound writes to implementation paths should be denied by the project executor.

## Authority

Resolve conflicts using the project's authority order in `.codex/settings.toml` → `[authority]`. Do not guess across conflicts; ask.

## Registry

`docs/task-registry.toml` is authoritative after activation. Use registry CLI opcodes; do not hand-edit when the CLI can perform the operation.
