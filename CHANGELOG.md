# Changelog

## Unreleased

## 2.1.0 - 2026-06-01

### Added

- Added `version-check` for Governed SemVer validation, release roadmap
  coverage, prerelease command output, and manual final tag verification.
- Added `backlog-check` for executable validation of the drainable gap pipeline
  and explicit negative non-claims.

### Changed

- Documented prerelease-only automation: branch and `vX.Y.Z-rc.N` tag pushes may
  be automated, while final release publication remains manual.

## 2.0.2 - 2026-06-01

### Fixed

- Hardened public plugin boundaries after the repository rename to RepOGovnor.
- Made runtime failure, receipt, report, and schema surfaces use typed enums.
- Kept completed task archives below the governed source line limit.
- Released the receipt event-file lock explicitly after durable appends.

## 2.0.1 - 2026-06-01

### Changed

- Switched the project license to MIT.
- Reworked user-facing README content around value, users, limits, install,
  and daily workflow.
- Added vision and roadmap documents.
- Tightened runtime schema enforcement: new Task Manifests require
  `schema_version = 2` with typed behavior verifiers, runtime governance writes
  are task-bound, release/version wrappers delegate to Rust schema checks, and
  installer config rejects noncanonical runtime keys.
- Hard-cut runtime Task Manifests and local receipts to schema version 2;
  read-only commands no longer append receipts unless `--record-receipt` is
  passed before the command.
- Added JSON command envelopes for core CLI inspection with `--format json`
  before the command.
- Hardened activation plans: new v2 plans require phased checklists,
  behavior `gap_id`, behavior `polarity`, typed positive and negative
  verifier coverage, exact task targets, and placeholder-free plan text.
- Updated GitHub workflow checkout actions to `actions/checkout@v6.0.2` for
  Node 24 execution.
- Added public contribution and security reporting documents.

### Fixed

- `verify-landing` now selects only active task targets before running behavior
  verifiers, so planned tasks cannot fail after verifier execution.
- GitHub CI now installs Nix before strict release readiness runs.

## 2.0.0 - 2026-05-30

### Breaking changes

- Removed the `--overlay` installer mode. Use `--merge` for existing repositories and `--force` for rebaseline installs.
- Legacy workspace hook files are hard-cut during merge: root `hooks.json`, `.codex/settings.toml`, `.codex/hooks/user-plan-approval.toml`, `.gemini/settings.json`, and `tools/antigravity/pre-tool-use-gap-closure.sh`.
- Project-native task registry binaries are noncanonical. Consumer repos use `.codex/scripts/task-registry`.
- The plugin enforces a hard 1600-line source/governance file limit.

### Added

- Plugin-owned Rust task-registry CLI with activation, status, deferral, reports, metrics, archive support, mutation hook verification, and source-limit planning.
- Codex, Cursor, and Antigravity CLI hook templates.
- Native Antigravity skills and Codex-compatible skill folders.
- Local task-registry event receipts in `docs/task-registry/events.jsonl`.
- `--dry-run`, `--merge`, and `--force` install modes with deterministic behavior tests.
- Release-source readiness checks and v2 version consistency checks.

### Fixed

- Mutation hook no longer deadlocks approved plan writing or activation when an unactivated manifest exists.
- Rendered task-registry wrapper now changes to the repository root before running the Rust CLI.
- Merge install now removes stale files that strict status rejects.
- Merge and force installs replace legacy skill projection symlinks with native v2 skill directories.
- Release audit now anchors source-limit checks at the repository root even when launched from a subdirectory.

### Validation

- Rust tests, fmt, clippy, source-limit, shell syntax, install-mode behavior, JSON/TOML syntax, Antigravity plugin validation, task-registry validation, and metrics checks are part of the release gate.
- Security audit requires `cargo-audit` and `cargo-deny`, unless a governed waiver explicitly sets `AGENT_GOVERNANCE_ALLOW_AUDIT_TOOL_WAIVER=1`.
