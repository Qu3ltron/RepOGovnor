# Patch Release 2.0.2 Gap Closure Contract

## Approved Scope
Publish a new patch release that includes post-2.0.1 public-release blockers already fixed on `main`.

In scope:
- Bump canonical release surfaces from `2.0.1` to `2.0.2`.
- Update the changelog with the post-2.0.1 hardening fixes.
- Validate release-source, version, CI, audit, and install gates.
- Tag and publish `v2.0.2` after green validation.

Out of scope: moving the existing public `v2.0.1` tag or changing runtime behavior beyond the already-merged fixes.

## Phased Required Change Checklist
### Phase 0: Activation and safety
- [ ] `[NEW]` `docs/plans/patch-release-2-0-2-2026-06-01.md` - `Task Manifest`: activate this contract before version edits.
- [ ] `[VERIFY]` `docs/task-registry.toml` - `registry`: activation, landing, and archive keep validation green.

### Phase 1: Version surfaces
- [ ] `[MODIFY]` `VERSION` - `release version`: set `2.0.2`.
- [ ] `[MODIFY]` `plugin.json` - `plugin version`: set `2.0.2`.
- [ ] `[MODIFY]` `.codex-plugin/plugin.json` - `Codex plugin version`: set `2.0.2`.
- [ ] `[MODIFY]` `MANIFEST.toml` - `plugin_version`: set `2.0.2`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/Cargo.toml` - `crate version`: set `2.0.2`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/Cargo.lock` - `crate lock version`: regenerate after the crate version bump.
- [ ] `[MODIFY]` `package.nix` - `Nix package version`: set `2.0.2`.
- [ ] `[MODIFY]` `REQUIREMENTS.toml` - `release source version`: set `2.0.2`.
- [ ] `[MODIFY]` `README.md` - `current release`: set `2.0.2`.

### Phase 2: Release notes and publication
- [ ] `[MODIFY]` `CHANGELOG.md` - `2.0.2 notes`: document typed runtime surfaces, archive budget, receipt lock release, security boundary hardening.
- [ ] `[VERIFY]` `scripts/release-version-check.sh` - `version consistency`: passes.
- [ ] `[VERIFY]` `scripts/test-release-readiness.sh` - `release readiness`: passes.
- [ ] `[VERIFY]` `.github/workflows/ci.yml` - `GitHub CI`: passes on pushed release commit.

## Per-Gap Success Criteria
### GAP-001: Public 2.0.1 tag is behind post-release blocker fixes
- Current failure: `v2.0.1` does not include release blocker fixes now on `main`.
- Good behavior: a new immutable `v2.0.2` tag and GitHub release point at the green release commit.
- Forbidden behavior: force-moving the existing public `v2.0.1` tag.
- Files involved: `VERSION`, `plugin.json`, `.codex-plugin/plugin.json`, `MANIFEST.toml`, `rust/task-registry-flow-cli/Cargo.toml`, `rust/task-registry-flow-cli/Cargo.lock`, `package.nix`, `REQUIREMENTS.toml`, `README.md`, `CHANGELOG.md`.
- Positive test: `bash scripts/release-version-check.sh`
- Negative test: `bash scripts/test-release-readiness.sh all`
- Domain/API/UI: public release metadata and install surfaces.
- Runtime: no behavior change beyond fixes already merged on `main`.

## Validation Plan
Focused:
- `bash scripts/release-version-check.sh`
- `.codex/scripts/task-registry source-limit check`
- `.codex/scripts/task-registry validate`

Full:
- `cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings`
- `bash scripts/test-release-readiness.sh all`
- `bash scripts/release-audit.sh`
- `nix flake check --no-build --all-systems`
- `nix run nixpkgs#gitleaks -- detect --no-git --source . --redact --verbose`

## Source File Limit
Expected impact is small release metadata edits below the 1600-line limit.

## Walkthrough Evidence
- Contract activation output.
- Release version check output.
- Release readiness and audit output.
- GitHub CI, Agent Governance, and CodeQL output.
- GitHub release URL for `v2.0.2`.

## Task Manifest
```toml
schema_version = 2
plan_id = "PLAN-2026-06-01-patch-release-2-0-2"

[[behaviors]]
behavior_id = "B-001-version-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Release metadata surfaces agree on 2.0.2"
given = "The patch release metadata is updated"
when = "the release version checker runs"
then = "all canonical version surfaces report 2.0.2"
confirmation = "bash scripts/release-version-check.sh"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/release-version-check.sh"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-readiness-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Release readiness rejects stale or incomplete release surfaces"
given = "The patch release is ready to publish"
when = "strict release readiness runs"
then = "no stale version, release-source, install, or audit issue remains"
confirmation = "bash scripts/test-release-readiness.sh all"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-release-readiness.sh all"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-06-01-patch-release-2-0-2-001"
behavior_ids = [
  "B-001-version-positive",
  "B-002-readiness-negative",
]
status = "planned"
title = "Prepare and validate patch release 2.0.2"
kind = "implementation"
reason = "The existing public 2.0.1 release predates blocker fixes now required for public consumption."
acceptance_proof = "Behaviors B-001 and B-002."

[[tasks.targets]]
file = "VERSION"
object = "release version"
required_change = "Set patch release version to 2.0.2."

[[tasks.targets]]
file = "plugin.json"
object = "plugin version"
required_change = "Set patch release version to 2.0.2."

[[tasks.targets]]
file = ".codex-plugin/plugin.json"
object = "Codex plugin version"
required_change = "Set patch release version to 2.0.2."

[[tasks.targets]]
file = "MANIFEST.toml"
object = "plugin_version"
required_change = "Set patch release version to 2.0.2."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/Cargo.toml"
object = "crate version"
required_change = "Set patch release version to 2.0.2."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/Cargo.lock"
object = "crate lock version"
required_change = "Regenerate after the crate version bump."

[[tasks.targets]]
file = "package.nix"
object = "Nix package version"
required_change = "Set patch release version to 2.0.2."

[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release source version"
required_change = "Set expected release source version to 2.0.2."

[[tasks.targets]]
file = "README.md"
object = "current release"
required_change = "Set current release to 2.0.2."

[[tasks.targets]]
file = "CHANGELOG.md"
object = "patch release notes"
required_change = "Document post-2.0.1 release blocker fixes."

[[tasks.targets]]
file = "docs/plans/patch-release-2-0-2-2026-06-01.md"
object = "closure contract"
required_change = "Track approved scope, behavior verifiers, and validation evidence."

[[tasks.targets]]
file = "docs/task-registry.toml"
object = "task registry activation and landing"
required_change = "Record this plan state through task-registry CLI only."

[[tasks.targets]]
file = "docs/task-registry/events.jsonl"
object = "task registry receipts"
required_change = "Append activation and landing receipts through task-registry CLI only."

[[tasks.targets]]
file = "docs/task-registry/archive/"
object = "completed task archives"
required_change = "Archive completed task rows after landing."
```
