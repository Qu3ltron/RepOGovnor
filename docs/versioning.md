# Version Governance

Version authority is executable. `VERSION`, package metadata, public release
claims, `CHANGELOG.md`, and `docs/version-roadmap.toml` must agree.

## Governed SemVer

- MAJOR: breaking public CLI, manifest, receipt, install, or governance
  contract change.
- MINOR: new CLI capability, release/backlog governance behavior, schema
  behavior, or public workflow capability.
- PATCH: bug, docs, tests, or release-surface correction without new public
  behavior.

## Flow

1. Create and activate the governed closure plan.
2. Add or update the matching `docs/version-roadmap.toml` release entry.
3. Run `.codex/scripts/task-registry version-check validate`.
4. Run `.codex/scripts/task-registry version-check next <plan_id>`.
5. Run validation, landing, report, metrics, and receipt-chain checks.
6. Commit with the roadmap `commit_subject`.
7. For prerelease automation only, use `version-check prerelease <plan_id>
   --rc <n>` and push the branch plus `vX.Y.Z-rc.N` tag.
8. Final `vX.Y.Z` tagging, final tag push, GitHub Release creation, and public
   release publication are manual.

No final-release force-push, automatic final tag push, or automatic GitHub
release publication is allowed.
