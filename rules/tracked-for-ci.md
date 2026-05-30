# Tracked for CI

After `--merge` or `--force` install, commit every path in [REQUIREMENTS.toml](REQUIREMENTS.toml) (`[tracked_for_ci].required`).

`status.sh --strict` and `.codex/scripts/task-registry validate` fail on fresh clones when any path is missing or untracked.

Validation includes the hard 1600-line source/governance file limit.

Install prints the `git add` checklist automatically.

Plugin-source v2 release readiness is separate from consumer install posture:
run `scripts/status.sh --release-source`, `scripts/release-version-check.sh`, and
`scripts/release-audit.sh` from this plugin checkout before tagging or publishing.
