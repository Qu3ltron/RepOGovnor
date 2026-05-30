# Agent governance (portable)

## Before implementation writes

1. Load **gap-closure-contract** after gap analysis and explicit user approval.
2. Write or refresh `docs/plans/<short-slug>.md` with `## Task Manifest` (typed TOML, `[[behaviors]]`, `[[tasks]]`).
3. Run **task-registry-flow** and activate the plan through `.codex/scripts/task-registry`.
4. Only then mutate paths listed in active task targets.
5. Keep source/governance files at or below 1600 lines. Run `.codex/scripts/task-registry source-limit check`; split with `.codex/scripts/task-registry source-limit plan --path <file>` before adding to violating files.

## Hard gate (when enabled)

PreToolUse hooks call `.codex/scripts/task-registry verify-mutation-hook`. Unbound writes to implementation paths are denied unless the path is a governance artifact or an active/planned task target.

## Authority

Resolve conflicts using the project's authority order in `.codex/agent-governance.toml` → `[authority]`. Do not guess across conflicts; ask.

## Registry

`docs/task-registry.toml` is authoritative after activation. Use `.codex/scripts/task-registry` opcodes; do not hand-edit when the CLI can perform the operation. Local efficacy receipts live in `docs/task-registry/events.jsonl`.
