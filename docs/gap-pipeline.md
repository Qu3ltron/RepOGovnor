# Gap Pipeline

This document is the drainable gap queue for the next version. It is not a
release promise. A gap closes only through a governed plan, activation, typed
behavior proof, landing verification, and registry report.

## Current Evidence

Verified on 2026-06-01:

- Release metadata is currently `2.0.2` across canonical machine surfaces.
- Local registry validation passes with no active, blocked, or deferred tasks.
- Source/governance files are under the 1600-line limit.
- Release-source status passes on the local checkout.
- Runtime receipts are schema version 2 hash-chain events, with v1 receipts
  treated as malformed historical evidence rather than current proof.

These checks prove release posture and workflow provenance. They do not prove
that the product is easy for new users, that every migration path is pleasant,
or that every future integration exists.

## Remaining Gaps

### GP-001: First-run adoption path is still dense
- Claim pressure: README and ROADMAP position the plugin as public and usable.
- Current evidence: install commands and validation gates exist, but the setup
  still assumes comfort with Git, shell commands, and local config. The release
  now includes `docs/migration-v2.md` and `docs/example-workflow.md` as packaged
  docs for migration and the plan -> activate -> edit -> land -> report loop.
  Installer output now prints dry-run continuation, applied-install first-run
  next steps, canonical posture checks, and direct workflow/migration doc
  pointers.
- User impact: first-time public users get a direct local next action from the
  command they just ran. A richer sample repository may still be useful if
  external smoke users stall after install.
- Next closure: add a richer fixture or sample repository only if external
  smoke evidence shows the current installer guidance is still insufficient.
- Reactivation condition: when a fresh external install smoke test shows
  confusion after reading the first-run output.

### GP-002: Reviewer handoff is not yet a compact product surface
- Claim pressure: the workflow produces reports, receipts, and metrics.
- Current evidence: `reviewer-report` now summarizes active plans, landed
  tasks, changed targets, receipt state, blocked or deferred work, and proof
  boundaries in one local handoff surface.
- User impact: maintainers have pasteable local reviewer evidence, but there is
  no GitHub or pull request integration yet.
- Next closure: add optional PR-oriented formatting or integration without
  adding remote telemetry.
- Reactivation condition: before publicizing a hosted or GitHub-native PR
  workflow.

### GP-003: Migration safety needs more old-layout fixtures
- Claim pressure: v2 says stale legacy paths are removed and compatibility
  shims are not supported.
- Current evidence: release checks reject known stale paths; merge/force install
  tests seed and remove stale settings, hook, Antigravity, and skill symlink
  layouts; `status-check --format json` reports stale legacy paths and nonnative
  `.agents` skills with remediation.
- User impact: common old-layout failures now have a local preflight diagnostic.
  Unusual external layouts may still need new fixtures after real migration
  reports.
- Next closure: add fixtures for any newly observed external migration layout
  without restoring compatibility shims.
- Reactivation condition: after the first external migration failure report or
  before documenting a larger migration campaign.

### GP-004: Multi-repo governance remains manual
- Claim pressure: the plugin is portable and usable across agents and repos.
- Current evidence: one repo can install and validate local governance; there
  is no first-class multi-repo policy or status aggregation.
- User impact: teams with several repos must coordinate versions, evidence, and
  policy posture by convention.
- Next closure: define a multi-repo consumption model or explicitly document
  the manual boundary.
- Reactivation condition: when a second production consumer repo needs shared
  release pinning or cross-repo status.

### GP-005: Product correctness remains out of scope
- Claim pressure: governance checks can look stronger than they are.
- Current evidence: registry, receipts, hooks, and release checks prove process
  invariants; they do not replace domain tests or code review. README now
  includes a `Proof boundaries` section that separates Governance proof from
  Product correctness proof, and `reviewer-report` repeats that boundary in
  handoff output.
- User impact: public docs and local reviewer output both state the proof
  boundary.
- Next closure: keep the same wording in any future PR integration or dashboard.
- Reactivation condition: before publishing remote reviewer integrations.

## Drain protocol

1. Pick one gap by id and write a new `docs/plans/<slug>.md` closure contract.
2. Include positive and negative typed behavior verifiers for the gap.
3. Activate through `.codex/scripts/task-registry activate`.
4. Implement only declared task targets.
5. Land with `.codex/scripts/task-registry verify-landing`.
6. Run `report`, `metrics`, `source-limit check`, `validate`, and
   `verify-chain --format json` before handoff.
7. Update this pipeline only with evidence from the current worktree or a live
   public release check.
