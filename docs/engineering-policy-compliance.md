# Engineering Policy Compliance

RepOGovnor targets engineering policy compliance for agent-assisted repos.
Compliance here means conformance to declared engineering policy: repo workflow,
build posture, release gates, mutation scope, evidence retention, and review
handoff rules.

It does not mean regulatory certification, external attestation, or proof that
product behavior is correct.

## Product Model

The intended product loop is:

1. A repo declares engineering policy.
2. RepOGovnor evaluates repo, build, release, and agent-workflow evidence against
   that policy.
3. The system emits a local compliance artifact showing passed, failed, warned,
   skipped, waived, and unproven controls.
4. Maintainers use that artifact for review, release, and audit-style handoff.

The current release is the evidence substrate for that loop. It already provides
plan activation, task-bound mutation targets, typed behavior verifiers,
source-limit checks, status diagnostics, release-source checks, version and
backlog governance checks, reviewer reports, and schema version 2 receipt chains.

## Policy Input

Policy input should be typed and explicit. A human-readable policy document can
explain intent, but enforcement should come from machine-readable controls.

Good controls answer:

- what must be true
- where it is checked
- which command or file proves it
- what failure means
- when a waiver is allowed
- what evidence is retained

Policy should not depend on unstated project expectations or agent narration.

## Compliance Artifact

A future compliance artifact should include:

- policy id, version, and content hash
- repo commit or build id
- RepOGovnor version
- evaluated controls and control groups
- status for each control: pass, fail, warn, skip, waived, or unproven
- evidence command, file, receipt, or diagnostic surface
- waiver reason, scope, approver source, and expiry when relevant
- unmeasured or unavailable evidence gaps
- final local posture summary

The artifact should be machine-readable first and human-readable second.

## Token And Cost Evidence

Token spend should become a first-class policy evidence domain. The end goal is
cost per commit plus related sub-metrics for plans, tasks, verifier runs,
landing attempts, retries, and release cycles.

This must be honest. Cost evidence is now represented as typed receipt data and
validated by `cost-evidence-check`. A cost value should be classified as:

- `measured` when structured usage receipts and pricing evidence exist
- `estimated` when clearly labeled assumptions are used
- `unmeasured` when usage is hidden, unavailable, or not attributable

Measured cost requires provider, model, usage counts, pricing snapshot or
version, timestamp, attribution target, evidence source, and amount evidence.
Estimated cost requires an explicit estimation method. Unmeasured cost requires
a reason and must not carry a cost amount. The system must not guess spend from
elapsed time, commit size, file count, or agent narration.

Current shipped evidence starts with model responsibility for supported Codex
repo mutation hooks. Codex is the first measured adapter because its hook
payload exposes model/session/turn/tool-use identity. Non-Codex mutation
surfaces remain unmeasured unless an adapter exposes equivalent evidence.
Model attribution is necessary for cost evidence, but it is not token usage or
cost per commit.

Cost per commit should only be reported when commit-linked measured usage
receipts exist. Otherwise the artifact should say the metric is unmeasured and
explain which usage evidence is missing.

## Current Non-Claims

- No regulatory certification.
- No external attestation.
- No hosted compliance dashboard.
- No remote telemetry.
- No proof that product behavior is correct.
- No reliable cost per commit in the current release.
- No guessed or silently inferred token spend.
