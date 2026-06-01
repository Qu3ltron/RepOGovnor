# Multi-Repo Boundary

RepOGovnor is repo-local governance. Use one install per repo, with each repo
pinning the plugin revision it trusts.

## Current Model

For several repositories:

1. Vendor or submodule RepOGovnor into each repo.
2. Pin the plugin version independently in each repo.
3. Run the install mode for that repo: `--dry-run`, then `--merge` or `--force`.
4. Run that repo's posture checks:

```bash
plugins/agent-governance/scripts/status.sh --strict
.codex/scripts/task-registry validate
.codex/scripts/task-registry source-limit check
.codex/scripts/task-registry verify-chain --format json
.codex/scripts/task-registry metrics
```

5. Use `.codex/scripts/task-registry reviewer-report` in each repo for review
   handoff.

No fleet aggregator is shipped in this release. There is no hosted status
service, no remote receipt sync, and no automatic cross-repo policy merge.

## Manual Coordination

Teams managing several repos should keep a short release note or checklist that
records, per repo:

- pinned RepOGovnor revision or release tag
- install mode used
- latest posture-check result
- latest `reviewer-report` summary
- blocked or deferred local governance work

This is deliberately manual. Receipts remain local evidence. A central service
would change the privacy and failure model, so it needs a separate design and
explicit approval before implementation.

## Reactivation Triggers

Create a new governed gap when a second production repo needs one of these:

- shared release pinning policy
- cross-repo status aggregation
- central review summary generation
- organization-wide policy drift detection

Until then, do not claim RepOGovnor provides fleet governance. It provides a
repeatable local governance contract that can be installed in many repos.
