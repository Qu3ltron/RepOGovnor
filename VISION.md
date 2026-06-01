# Vision

## Purpose

RepOGovnor exists to make engineering policy compliance legible for
agent-assisted repositories.

Coding agents are useful because they can move quickly across a codebase. That
same speed creates risk: edits can outrun planning, tests can become detached
from intent, and reviewers can lose the thread of why a change exists.

This plugin turns agent work and repo policy into a local, inspectable workflow:

- declared engineering policy before release claims
- plans before implementation changes
- tasks before mutation
- behavior checks before completion
- receipts before release claims
- schemas before runtime policy

The product direction is not "more automation at any cost." The direction is
controlled autonomy: agents can move fast inside explicit boundaries, and the
repository keeps enough evidence for humans to inspect whether the work complied
with declared engineering policy.

## Primary users

### Solo maintainers

Maintainers who use agents on important repos need lightweight structure:
what was approved, which files were in scope, which tests proved closure, and
what remains blocked.

### Engineering teams

Teams need a common workflow across tools. Codex, Antigravity CLI, Cursor, and
Claude Code should not each require a separate governance model or incompatible
hook layout.

### High-risk projects

Projects with compliance, safety, security, or operational risk need stronger
evidence than "the agent said it passed." They need local records, repeatable
checks, explicit failure states, and a clear boundary between engineering-policy
compliance and regulatory certification.

### Tool builders and workflow owners

People building internal agent workflows need portable primitives they can
install into many repositories without designing a governance system from
scratch.

## Principles

- Local first: the repository owns the workflow and receipts.
- Policy first: compliance claims should evaluate declared engineering policy,
  not unstated expectations.
- Explicit scope: implementation writes should map to approved task targets.
- Honest state: blocked, deferred, planned, and completed mean different
  things and should not be blurred.
- Hard cutovers: stale compatibility paths should not survive as hidden
  production risk.
- Tool portability: governance should work across supported agent surfaces.
- Evidence over trust: completion claims need behavior checks.
- Clean runtime API: checks, scopes, verifier types, and receipts should be
  typed values, not loose prose hidden in scripts.

## Current state

Version `2.1.0` provides the core local workflow:

- task-registry CLI
- plan activation and task lifecycle
- mutation hook verification
- Codex, Antigravity CLI, Cursor, and Claude Code surfaces
- install and release readiness checks
- source-file budget enforcement
- local validation receipts
- schema-backed diagnostics for mutation, release, source-limit, and installer
  dry-run behavior
- schema version 2 local receipts with explicit recording for read-only
  inspection commands
- task-bound runtime governance writes with a narrow plan-bootstrap exception
- schema version 2 manifests with typed behavior verifiers for every runtime
  plan, including migrated historical evidence
- version and backlog checks for RepOGovnor release governance

This is usable today for repositories that accept a strict plan-first workflow.

It is not yet a full governance platform. There is no hosted dashboard, no
remote policy service, no built-in token-spend analytics, no reliable cost per
commit calculation, and no automatic semantic proof that a product change is
correct.

## Desired future

The long-term goal is a small, durable governance layer that can travel with a
repository:

- typed engineering policy input
- local compliance artifacts that show passed, failed, waived, skipped, and
  unproven controls
- waiver and exception lifecycle for engineering policy controls
- token and cost evidence that distinguishes measured, estimated, and
  unmeasured spend
- cost per commit, plan, task, and release cycle when structured usage receipts
  and pricing evidence exist
- easy install for first-time users
- clear migration diagnostics for existing repos
- continued migration of installer/status wrappers toward thinner render-only
  entrypoints
- stronger policy presets for different engineering risk levels
- richer local reports for reviewers
- better examples showing real agent workflows
- optional integrations that do not compromise local-first behavior

The plugin should stay boring in the right places: plain files, explicit
commands, predictable failure modes, and no hidden service dependency.
