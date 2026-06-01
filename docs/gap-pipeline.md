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
- User impact: first-time public users have a direct guide, but installer output
  still needs clearer first-run messaging.
- Next closure: improve first-run installer output and add a richer fixture or
  sample repository if external smoke users still stall.
- Reactivation condition: before the next public minor release or when a fresh
  external install smoke test shows confusion.

### GP-002: Reviewer handoff is not yet a compact product surface
- Claim pressure: the workflow produces reports, receipts, and metrics.
- Current evidence: `report`, `metrics`, and `verify-chain` exist, but there is
  no single reviewer summary designed for pull requests.
- User impact: maintainers still need to assemble proof manually from several
  commands.
- Next closure: add a compact reviewer report with active plans, landed tasks,
  changed targets, receipts, and remaining blocked or deferred work.
- Reactivation condition: before publicizing a recommended PR workflow.

### GP-003: Migration safety needs more old-layout fixtures
- Claim pressure: v2 says stale legacy paths are removed and compatibility
  shims are not supported.
- Current evidence: release checks reject known stale paths, and merge/force
  install tests exist.
- User impact: repositories with unusual v0.x or v1 governance layouts may hit
  hard failures without enough preflight explanation.
- Next closure: add targeted fixtures for common stale symlink, hook, and skill
  layouts, with direct remediation diagnostics.
- Reactivation condition: before a broader public announcement or after the
  first external migration failure report.

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
  Product correctness proof.
- User impact: users have a visible public boundary, but future reviewer output
  should keep the same distinction when a compact report command exists.
- Next closure: add reviewer-output language that labels governance proof
  separately from product correctness proof.
- Reactivation condition: before publishing reviewer-report features.

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
