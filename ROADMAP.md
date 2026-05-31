# Roadmap

This roadmap is intentionally transparent. Items here are planned direction, not
release promises.

## Now: v2 hard cutover

Status: shipped in `2.0.0`.

- Remove `--overlay`.
- Use `.codex/scripts/task-registry` as the canonical registry command.
- Install Codex, Cursor, and Antigravity governance surfaces.
- Preserve valid task registry state while removing stale legacy hook paths.
- Enforce source/governance file limits.
- Add release-source checks, dependency audit, and version consistency checks.
- Add schema-backed diagnostics for runtime receipts, mutation scopes, release
  checks, behavior verifiers, and installer dry-runs.
- Require typed behavior verifiers for new task manifests while preserving
  completed legacy evidence.
- Bind runtime governance writes to task targets except for plan bootstrap.

## Next: adoption quality

Target: v2.x.

- Improve first-run install messages so users understand what changed and why.
- Add a concise migration guide for v0.x/v1 workspaces.
- Add a small example repo or fixture showing the full plan -> activate -> edit
  -> validate -> complete workflow.
- Add config validation with direct messages for missing or stale settings.
- Improve `status.sh --strict` output so remediation steps are easier to follow.
- Move the remaining status rendering surfaces directly onto structured
  diagnostics.

## Next: reviewer experience

Target: v2.x.

- Add a compact reviewer report that summarizes active plans, completed tasks,
  blocked work, validation receipts, and changed targets.
- Make task reports easier to paste into pull requests.
- Add negative-test guidance to the plan template.
- Add clearer docs for when to defer work versus keep it active.
- Expand typed behavior-verifier examples for migration and authorization
  plans.

## Next: migration safety

Target: v2.x.

- Add more upgrade fixtures for older workspace layouts.
- Detect common stale symlink and hook layouts before install writes files.
- Continue moving installer rendering from shell/Python into the Rust runtime
  API.
- Add an uninstall or rebaseline guide for teams that need to reset governance
  surfaces deliberately.

## Later: policy profiles

Target: post-v2.x.

- Provide preset policy profiles for personal projects, teams, and high-risk
  repos.
- Allow projects to opt into stricter behavior confirmation requirements.
- Add better local policy documentation generated from the active config.
- Explore signing or hashing local receipts for stronger provenance.

## Later: integrations

Target: exploratory.

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

## Non-goals

- No hidden remote service dependency.
- No network telemetry.
- No compatibility shim for removed v2 paths.
- No claim that passing governance checks replaces code review.
