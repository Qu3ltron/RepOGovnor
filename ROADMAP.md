# Roadmap

This roadmap is intentionally transparent. Items here are planned direction, not
release promises.

## Now: v2.1 evidence substrate

Status: shipped in `2.1.0`.

- Remove `--overlay`.
- Use `.codex/scripts/task-registry` as the canonical registry command.
- Install Codex, Antigravity CLI, Cursor, and Claude Code governance surfaces.
- Preserve valid task registry state while removing stale legacy hook paths.
- Enforce source/governance file limits.
- Add release-source checks, dependency audit, and version consistency checks.
- Add schema-backed diagnostics for runtime receipts, mutation scopes, release
  checks, behavior verifiers, and installer dry-runs.
- Require typed behavior verifiers for all runtime task manifests; completed
  historical manifests are migrated instead of accepted through v1 compatibility.
- Bind runtime governance writes to task targets except for plan bootstrap.
- Make read-only receipt recording explicit and keep schema version 1 receipt
  lines out of the current runtime ledger.
- Add version and backlog governance checks for this plugin release train.
- Add local reviewer reports for manual pull-request handoff.

## Next: engineering policy compliance

Target: v2.x.

- Define typed engineering policy input for repo/build/governance controls.
- Emit a local compliance artifact that records pass, fail, warn, skip, waived,
  and unproven control states.
- Map existing registry, verifier, release-source, source-limit, status, version,
  backlog, and receipt-chain checks into policy controls.
- Add a waiver and exception lifecycle with reason, scope, evidence, and expiry.
- Keep regulatory framework mappings optional and explicit; do not claim
  certification or external attestation.

## Next: token and cost evidence

Target: v2.x.

- Treat current Codex mutation model attribution as the first measured adapter
  step, not as token spend or universal attribution.
- Current shipped cost evidence is a typed receipt model and
  `cost-evidence-check`, not automatic collection or spend calculation.
- Classify every cost value as measured, estimated, or unmeasured.
- Require provider, model, usage counts, pricing snapshot, timestamp, attribution
  target, and evidence source before reporting measured cost.
- Attribute measured spend to commits, plans, tasks, verifier runs, landing
  attempts, retries, and release cycles.
- Produce cost per commit only when commit-linked usage receipts exist; otherwise
  report the metric as unmeasured.

## Next: adoption quality

Target: v2.x.

- Improve first-run install messages so users understand what changed and why.
- Add a concise migration guide for v0.x/v1 workspaces.
- Add a small example repo or fixture showing the full plan -> activate -> edit
  -> validate -> complete workflow.
- Add config validation with direct messages for missing or stale settings.
- Improve `status.sh --strict` output so remediation steps are easier to follow.
- Keep thinning status and installer wrappers so user entrypoints render typed
  runtime diagnostics without owning policy.

## Next: reviewer and compliance artifact experience

Target: v2.x.

- Make task reports easier to paste into pull requests.
- Add a human-readable compliance summary for policy artifacts.
- Add negative-test guidance to the plan template.
- Add clearer docs for when to defer work versus keep it active.
- Expand typed behavior-verifier examples for migration and authorization
  plans.

## Next: migration safety

Target: v2.x.

- Add more upgrade fixtures for older workspace layouts.
- Detect common stale symlink and hook layouts before install writes files.
- Continue reducing shell wrapper policy to Rust-owned schema APIs.
- Add an uninstall or rebaseline guide for teams that need to reset governance
  surfaces deliberately.

## Later: policy profiles and integrations

Target: post-v2.x.

- Provide preset policy profiles for personal projects, teams, and high-risk
  repos.
- Allow projects to opt into stricter behavior confirmation requirements.
- Add better local policy documentation generated from the active config.
- Explore signing or hashing local receipts for stronger provenance.
- GitHub Action packaging for easier CI adoption.
- Optional pull request summary generation.
- Optional local dashboard or static HTML report.
- Export formats for teams that want to ingest receipts into their own systems.

## Known gaps

- The plugin assumes users are comfortable with Git, shell commands, and local
  repo configuration.
- The setup path is still too dense for first-time users.
- There is no hosted UI.
- There is no automatic merge-request policy bot.
- The registry proves workflow state, not product correctness.
- Multi-repo governance is possible manually but not yet first-class.
- There is no shipped typed policy engine command yet.
- There is no reliable cost per commit until commit-linked measured usage
  receipts and pricing evidence exist.

## Non-goals

- No hidden remote service dependency.
- No network telemetry.
- No compatibility shim for removed v2 paths.
- No claim that passing governance checks replaces code review.
- No regulatory certification or external attestation claim.
- No guessed token spend.
