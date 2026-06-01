# Gap Pipeline

This document is the drainable gap queue for the next version. It is not a
release promise. A gap closes only through a governed plan, activation, typed
behavior proof, landing verification, and registry report.

## Current Evidence

Verified on 2026-06-01:

- Release metadata is currently `2.1.0` across canonical machine surfaces.
- Local registry validation passes with no active, blocked, or deferred tasks.
- Source/governance files are under the 1600-line limit.
- Release-source status passes on the local checkout.
- Runtime receipts are schema version 2 hash-chain events, with v1 receipts
  treated as malformed historical evidence rather than current proof.
- Version and backlog checks validate RepOGovnor release governance.

These checks prove release posture and workflow provenance. They do not prove
that the product is easy for new users, that every migration path is pleasant,
or that every future integration exists. They also do not yet prove conformance
to a first-class typed engineering policy manifest.

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
  boundaries in one local handoff surface. `reviewer-report --format markdown`
  adds local pull-request-oriented Markdown without posting remotely.
- User impact: maintainers have pasteable text and Markdown reviewer evidence.
  There is still no GitHub integration, hosted reviewer service, or remote
  telemetry.
- Next closure: add remote integration only under a separate approval that
  defines authentication, privacy, and failure behavior.
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
- Current evidence: `docs/multi-repo.md` now documents the manual boundary:
  one install per repo, independent plugin pinning, per-repo posture checks,
  per-repo `reviewer-report`, and no shipped fleet aggregator. The doc is a
  required release file and packaged asset.
- User impact: teams with several repos have an honest consumption model and
  know which coordination remains manual.
- Next closure: design shared release pinning, cross-repo status aggregation, or
  organization drift detection only after a second production repo needs it.
- Reactivation condition: when a second production consumer repo needs shared
  release pinning, central review summaries, or cross-repo status.

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

### GP-006: Engineering policy compliance is the central direction, not yet the full runtime
- Claim pressure: public positioning now targets engineering policy compliance
  for agent-assisted repos.
- Current evidence: the runtime already has plan activation, mutation gates,
  typed behavior verifiers, receipt chains, source limits, status diagnostics,
  release-source checks, version checks, backlog checks, and reviewer reports.
- User impact: maintainers can use the system as a strong policy evidence
  substrate today, but there is not yet a first-class typed policy manifest or
  compliance artifact command.
- Next closure: design and implement typed policy input plus a local compliance
  artifact with pass, fail, warn, skip, waived, and unproven control states.
- Reactivation condition: before claiming a shipped policy engine or compliance
  artifact.

### GP-007: Token spend and cost per commit need structured usage evidence
- Claim pressure: cost per commit is an intended first-class metric.
- Current evidence: local workflow metrics exist, but structured token usage,
  provider/model attribution, pricing snapshots, and commit-linked cost receipts
  are not shipped.
- User impact: any current cost claim would be false precision. Hidden or
  unavailable usage must be reported as unmeasured.
- Next closure: add a cost evidence model that distinguishes measured,
  estimated, and unmeasured spend, then attribute measured usage to commits,
  plans, tasks, verifier runs, landing attempts, retries, and release cycles.
- Reactivation condition: before publishing cost per commit or token-spend
  metrics as a shipped feature.

## Negative Non-Claims

- No automatic final release publication is provided by RepOGovnor.
- No automatic final release tag push is provided by RepOGovnor.
- No product correctness proof is created by governance checks alone.
- No hosted fleet governance is shipped in this release.
- No remote receipt sync is shipped in this release.
- No typed policy engine or compliance artifact command is shipped in this
  release.
- No reliable cost per commit or guessed token spend is shipped in this release.

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
