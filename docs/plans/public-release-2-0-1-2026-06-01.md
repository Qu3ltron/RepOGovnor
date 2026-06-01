# Public v2.0.1 Release Gap Closure Contract

## Approved Scope

Prepare and publish the first public release of Governance-plugin as `2.0.1`.

In scope:

- Bump release/version surfaces from `2.0.0` to `2.0.1`.
- Move current unreleased changelog items into `2.0.1`.
- Add public contributor and vulnerability-reporting docs.
- Update GitHub Actions checkout from Node 20-backed `actions/checkout@v4` to
  Node 24-backed `actions/checkout@v6.0.2`.
- Update release-source requirements for new public docs.
- Validate locally, push, tag `v2.0.1`, retarget `v2`, create the GitHub
  Release, make the repository public, and smoke-test public install.

Out of scope:

- Changing runtime governance behavior or task-registry schemas.
- Rewriting historical `2.0.0` changelog entries.
- Adding compatibility shims for old hook or settings paths.

## Phased Required Change Checklist

### Phase 0: Activation

- [ ] `[NEW]` `docs/plans/public-release-2-0-1-2026-06-01.md` - `closure contract`: define public release scope, verifiers, and manifest; acceptance proof is `PLAN_ACTIVATE docs/plans/public-release-2-0-1-2026-06-01.md`.

### Phase 1: Release metadata

- [ ] `[MODIFY]` `VERSION` - `release version`: set public release version to `2.0.1`; acceptance proof is behavior `B-2026-06-01-public-release-G01-positive`.
- [ ] `[MODIFY]` `plugin.json` - `plugin version`: set version to `2.0.1`; acceptance proof is behavior `B-2026-06-01-public-release-G01-positive`.
- [ ] `[MODIFY]` `.codex-plugin/plugin.json` - `Codex plugin version`: set version to `2.0.1`; acceptance proof is behavior `B-2026-06-01-public-release-G01-positive`.
- [ ] `[MODIFY]` `MANIFEST.toml` - `plugin_version`: set version to `2.0.1`; acceptance proof is behavior `B-2026-06-01-public-release-G01-positive`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/Cargo.toml` - `package.version`: set version to `2.0.1`; acceptance proof is behavior `B-2026-06-01-public-release-G01-positive`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/Cargo.lock` - `task-registry-flow-cli package.version`: set lock package version to `2.0.1`; acceptance proof is behavior `B-2026-06-01-public-release-G01-positive`.
- [ ] `[MODIFY]` `package.nix` - `version`: set version to `2.0.1`; acceptance proof is behavior `B-2026-06-01-public-release-G01-positive`.
- [ ] `[MODIFY]` `README.md` - `current release`: set current release to `2.0.1`; acceptance proof is behavior `B-2026-06-01-public-release-G01-positive`.
- [ ] `[MODIFY]` `docs/releases/v2.md` - `release version`: set release version to `2.0.1`; acceptance proof is behavior `B-2026-06-01-public-release-G01-positive`.
- [ ] `[MODIFY]` `scripts/test-release-readiness.sh` - `version drift fixture`: keep the negative drift fixture different from current release version; acceptance proof is behavior `B-2026-06-01-public-release-G05-validation`.

### Phase 2: Public release docs

- [ ] `[MODIFY]` `CHANGELOG.md` - `2.0.1 release notes`: move current unreleased public-release items into a dated `2.0.1` section; acceptance proof is behavior `B-2026-06-01-public-release-G02-positive`.
- [ ] `[NEW]` `CONTRIBUTING.md` - `public contribution guidance`: add concise issue/PR/check guidance; acceptance proof is behavior `B-2026-06-01-public-release-G02-positive`.
- [ ] `[NEW]` `SECURITY.md` - `vulnerability reporting`: route vulnerability reports through private GitHub vulnerability reporting/advisories; acceptance proof is behavior `B-2026-06-01-public-release-G02-positive`.
- [ ] `[MODIFY]` `REQUIREMENTS.toml` - `release_source`: set release version to `2.0.1` and require `CONTRIBUTING.md` plus `SECURITY.md`; acceptance proof is behavior `B-2026-06-01-public-release-G03-positive`.

### Phase 3: Node 24 workflow update

- [ ] `[MODIFY]` `.github/workflows/ci.yml` - `checkout action`: use `actions/checkout@v6.0.2`; acceptance proof is behavior `B-2026-06-01-public-release-G04-positive`.
- [ ] `[MODIFY]` `.github/workflows/agent-governance.yml` - `checkout action`: use `actions/checkout@v6.0.2`; acceptance proof is behavior `B-2026-06-01-public-release-G04-positive`.
- [ ] `[MODIFY]` `templates/.github/workflows/agent-governance.yml.template` - `checkout action`: use `actions/checkout@v6.0.2`; acceptance proof is behavior `B-2026-06-01-public-release-G04-positive`.

### Phase 4: Validation and public handoff

- [ ] `[VERIFY]` `docs/plans/public-release-2-0-1-2026-06-01.md` - `local validation`: run focused verifiers, release readiness, source-limit, registry validation, and receipt-chain verification; acceptance proof is behavior `B-2026-06-01-public-release-G05-validation`.
- [ ] `[VERIFY]` `docs/plans/public-release-2-0-1-2026-06-01.md` - `public GitHub handoff`: after commit and push, run final release-source status, confirm CI/Agent Governance/CodeQL success, tag `v2.0.1`, retarget `v2`, create the GitHub Release, make the repo public, and run public install smoke tests.

## Per-Gap Success Criteria

### GAP-001: Public release version still identifies as 2.0.0

- Current failure: public release metadata points at `2.0.0` even though later
  release fixes have landed.
- Good behavior: all canonical version files identify the public release as
  `2.0.1`.
- Forbidden behavior: canonical version files keep `2.0.0` as the current
  release version.
- Files involved: `VERSION`, `plugin.json`, `.codex-plugin/plugin.json`,
  `MANIFEST.toml`, `rust/task-registry-flow-cli/Cargo.toml`,
  `rust/task-registry-flow-cli/Cargo.lock`, `package.nix`, `README.md`,
  `docs/releases/v2.md`, `REQUIREMENTS.toml`,
  `scripts/test-release-readiness.sh`.
- Positive test: `python3 - <<'PY' ... version surface assertions ... PY`
- Negative test: `! rg -n "2\\.0\\.0" VERSION plugin.json .codex-plugin/plugin.json MANIFEST.toml rust/task-registry-flow-cli/Cargo.toml rust/task-registry-flow-cli/Cargo.lock package.nix README.md docs/releases/v2.md REQUIREMENTS.toml`
- Data/schema/provenance: release-source version remains schema version 1 with
  version value `2.0.1`.
- Runtime: N/A; metadata release only.

### GAP-002: Public release lacks contributor and security ceremony

- Current failure: the repo has install docs and MIT license, but no public
  contribution or vulnerability-reporting guidance.
- Good behavior: public readers can see how to contribute and how to report
  vulnerabilities without opening public security issues.
- Forbidden behavior: public docs contain placeholders or route
  vulnerabilities only to public issues.
- Files involved: `CHANGELOG.md`, `CONTRIBUTING.md`, `SECURITY.md`.
- Positive test: `test -f CONTRIBUTING.md && test -f SECURITY.md && rg -q "GitHub private vulnerability reporting|GitHub Security Advisory" SECURITY.md && rg -q "scripts/test-release-readiness.sh all" CONTRIBUTING.md && rg -q "## 2.0.1 - 2026-06-01" CHANGELOG.md`
- Negative test: `! rg -n "PLACEHOLDER|your-email|example.com" CONTRIBUTING.md SECURITY.md CHANGELOG.md`
- Data/schema/provenance: release-source requirements include public docs.
- Runtime: N/A; public repository ceremony only.

### GAP-003: New public docs are not release-source required

- Current failure: `CONTRIBUTING.md` and `SECURITY.md` do not exist and are not
  declared as release-source files.
- Good behavior: release-source status fails if either public doc is absent.
- Forbidden behavior: public docs exist outside release-source authority.
- Files involved: `REQUIREMENTS.toml`.
- Positive test: `python3 - <<'PY' ... REQUIREMENTS release-source assertions ... PY`
- Negative test: `! python3 - <<'PY' ... assert missing public docs ... PY`
- Data/schema/provenance: release-source required list includes native public
  docs.
- Runtime: `scripts/status.sh --release-source` validates both files.

### GAP-004: GitHub Actions still uses Node 20-backed checkout

- Current failure: CI emitted a Node 20 deprecation warning for
  `actions/checkout@v4`.
- Good behavior: all checkout steps use `actions/checkout@v6.0.2`, whose action
  metadata declares `using: node24`.
- Forbidden behavior: any workflow or workflow template keeps
  `actions/checkout@v4`.
- Files involved: `.github/workflows/ci.yml`,
  `.github/workflows/agent-governance.yml`,
  `templates/.github/workflows/agent-governance.yml.template`.
- Positive test: `rg -n "actions/checkout@v6.0.2" .github/workflows templates/.github && gh api repos/actions/checkout/contents/action.yml?ref=v6.0.2 --jq '.content' | base64 -d | rg -q "using: node24"`
- Negative test: `! rg -n "actions/checkout@v4" .github/workflows templates/.github`
- Data/schema/provenance: no schema change.
- Runtime: GitHub Actions no longer runs checkout on Node 20.

## Validation Plan

Focused:

- `python3 - <<'PY' ... version surface assertions ... PY`
- `test -f CONTRIBUTING.md && test -f SECURITY.md && rg -q "GitHub private vulnerability reporting|GitHub Security Advisory" SECURITY.md && rg -q "scripts/test-release-readiness.sh all" CONTRIBUTING.md && rg -q "## 2.0.1 - 2026-06-01" CHANGELOG.md`
- `python3 - <<'PY' ... REQUIREMENTS release-source assertions ... PY`
- `rg -n "actions/checkout@v6.0.2" .github/workflows templates/.github && gh api repos/actions/checkout/contents/action.yml?ref=v6.0.2 --jq '.content' | base64 -d | rg -q "using: node24"`
- `! rg -n "actions/checkout@v4" .github/workflows templates/.github`
- `.codex/scripts/task-registry source-limit check`

Full:

- `scripts/release-version-check.sh`
- `bash scripts/test-release-readiness.sh all`
- `.codex/scripts/task-registry validate`
- `.codex/scripts/task-registry verify-chain --format json`
- `.codex/scripts/task-registry report PLAN-2026-06-01-public-release-2-0-1`
- `.codex/scripts/task-registry metrics`
- After commit: `AGENT_GOVERNANCE_FINAL_RELEASE=1 scripts/status.sh --release-source`
- After push: `gh run list --repo Qu3ltron/Governance-plugin --branch main --limit 5`
- After public release: public unauthenticated `git ls-remote`, fresh install smoke test, and Nix build from public tag.

## Documentation and Release Sync

- README current release and public install instructions must name `2.0.1`.
- `docs/releases/v2.md` must name release version `2.0.1`.
- `CHANGELOG.md` must have a dated `2.0.1` section and leave an empty
  `Unreleased` section.
- `REQUIREMENTS.toml` must own new public docs and release version `2.0.1`.
- Workflow template must match live workflow checkout version.

## Source File Limit

The added docs are small and all touched files are expected to remain below
1600 lines. Run `.codex/scripts/task-registry source-limit check` before landing.

## Walkthrough Evidence

- Local focused verifiers exit 0.
- `scripts/release-version-check.sh` exits 0.
- `bash scripts/test-release-readiness.sh all` exits 0.
- `.codex/scripts/task-registry verify-chain --format json` reports intact chain.
- `AGENT_GOVERNANCE_FINAL_RELEASE=1 scripts/status.sh --release-source` exits 0
  after commit.
- GitHub CI, Agent Governance, and CodeQL succeed on pushed `main`.
- Tags `v2.0.1` and `v2` point at the release commit.
- Repository visibility is public.
- Public install and Nix smoke tests pass.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-06-01-public-release-2-0-1"

[[behaviors]]
behavior_id = "B-2026-06-01-public-release-G01-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Release metadata identifies 2.0.1"
given = "canonical release version surfaces"
when = "the version verifier runs"
then = "all current release surfaces identify 2.0.1"
confirmation = "python3 - <<'PY'\nimport json, tomllib\nfrom pathlib import Path\nassert Path('VERSION').read_text().strip() == '2.0.1'\nassert json.loads(Path('plugin.json').read_text())['version'] == '2.0.1'\nassert json.loads(Path('.codex-plugin/plugin.json').read_text())['version'] == '2.0.1'\nassert tomllib.loads(Path('MANIFEST.toml').read_text())['plugin_version'] == '2.0.1'\nassert tomllib.loads(Path('rust/task-registry-flow-cli/Cargo.toml').read_text())['package']['version'] == '2.0.1'\nassert 'name = \"task-registry-flow-cli\"\\nversion = \"2.0.1\"' in Path('rust/task-registry-flow-cli/Cargo.lock').read_text()\nassert 'version = \"2.0.1\"' in Path('package.nix').read_text()\nassert 'Current release: `2.0.1`' in Path('README.md').read_text()\nassert 'Release version: `2.0.1`' in Path('docs/releases/v2.md').read_text()\nassert tomllib.loads(Path('REQUIREMENTS.toml').read_text())['release_source']['version'] == '2.0.1'\nPY"

[[behaviors.verifiers]]
type = "command"
command = "python3 - <<'PY'\nimport json, tomllib\nfrom pathlib import Path\nassert Path('VERSION').read_text().strip() == '2.0.1'\nassert json.loads(Path('plugin.json').read_text())['version'] == '2.0.1'\nassert json.loads(Path('.codex-plugin/plugin.json').read_text())['version'] == '2.0.1'\nassert tomllib.loads(Path('MANIFEST.toml').read_text())['plugin_version'] == '2.0.1'\nassert tomllib.loads(Path('rust/task-registry-flow-cli/Cargo.toml').read_text())['package']['version'] == '2.0.1'\nassert 'name = \"task-registry-flow-cli\"\\nversion = \"2.0.1\"' in Path('rust/task-registry-flow-cli/Cargo.lock').read_text()\nassert 'version = \"2.0.1\"' in Path('package.nix').read_text()\nassert 'Current release: `2.0.1`' in Path('README.md').read_text()\nassert 'Release version: `2.0.1`' in Path('docs/releases/v2.md').read_text()\nassert tomllib.loads(Path('REQUIREMENTS.toml').read_text())['release_source']['version'] == '2.0.1'\nPY"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-06-01-public-release-G01-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Current release surfaces do not keep 2.0.0"
given = "canonical current release version surfaces"
when = "the stale-version scanner runs"
then = "2.0.0 is absent from current release surfaces"
confirmation = "! rg -n '2\\.0\\.0' VERSION plugin.json .codex-plugin/plugin.json MANIFEST.toml rust/task-registry-flow-cli/Cargo.toml rust/task-registry-flow-cli/Cargo.lock package.nix README.md docs/releases/v2.md REQUIREMENTS.toml"

[[behaviors.verifiers]]
type = "command"
command = "! rg -n '2\\.0\\.0' VERSION plugin.json .codex-plugin/plugin.json MANIFEST.toml rust/task-registry-flow-cli/Cargo.toml rust/task-registry-flow-cli/Cargo.lock package.nix README.md docs/releases/v2.md REQUIREMENTS.toml"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-06-01-public-release-G02-positive"
gap_id = "GAP-002"
polarity = "positive"
title = "Public contributor and security docs exist"
given = "the public release docs"
when = "the doc verifier runs"
then = "contribution and vulnerability reporting guidance exists and the changelog has 2.0.1"
confirmation = "test -f CONTRIBUTING.md && test -f SECURITY.md && rg -q 'GitHub private vulnerability reporting|GitHub Security Advisory' SECURITY.md && rg -q 'scripts/test-release-readiness.sh all' CONTRIBUTING.md && rg -q '## 2.0.1 - 2026-06-01' CHANGELOG.md"

[[behaviors.verifiers]]
type = "command"
command = "test -f CONTRIBUTING.md && test -f SECURITY.md && rg -q 'GitHub private vulnerability reporting|GitHub Security Advisory' SECURITY.md && rg -q 'scripts/test-release-readiness.sh all' CONTRIBUTING.md && rg -q '## 2.0.1 - 2026-06-01' CHANGELOG.md"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-06-01-public-release-G02-negative"
gap_id = "GAP-002"
polarity = "negative"
title = "Public docs contain no placeholders"
given = "the public release docs"
when = "the placeholder scanner runs"
then = "placeholder text is absent"
confirmation = "! rg -n 'PLACEHOLDER|your-email|example.com' CONTRIBUTING.md SECURITY.md CHANGELOG.md"

[[behaviors.verifiers]]
type = "command"
command = "! rg -n 'PLACEHOLDER|your-email|example.com' CONTRIBUTING.md SECURITY.md CHANGELOG.md"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-06-01-public-release-G03-positive"
gap_id = "GAP-003"
polarity = "positive"
title = "Release-source owns public docs"
given = "REQUIREMENTS.toml"
when = "release-source requirements are parsed"
then = "CONTRIBUTING.md and SECURITY.md are required release-source files"
confirmation = "python3 - <<'PY'\nimport tomllib\nfrom pathlib import Path\nrelease = tomllib.loads(Path('REQUIREMENTS.toml').read_text())['release_source']\nassert release['version'] == '2.0.1'\nrequired = set(release['required'])\nassert 'CONTRIBUTING.md' in required\nassert 'SECURITY.md' in required\nPY"

[[behaviors.verifiers]]
type = "command"
command = "python3 - <<'PY'\nimport tomllib\nfrom pathlib import Path\nrelease = tomllib.loads(Path('REQUIREMENTS.toml').read_text())['release_source']\nassert release['version'] == '2.0.1'\nrequired = set(release['required'])\nassert 'CONTRIBUTING.md' in required\nassert 'SECURITY.md' in required\nPY"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-06-01-public-release-G03-negative"
gap_id = "GAP-003"
polarity = "negative"
title = "Public docs are not omitted from release-source"
given = "REQUIREMENTS.toml"
when = "the omission verifier runs"
then = "it cannot prove either public doc is absent from release requirements"
confirmation = "! python3 - <<'PY'\nimport tomllib\nfrom pathlib import Path\nrequired = set(tomllib.loads(Path('REQUIREMENTS.toml').read_text())['release_source']['required'])\nassert 'CONTRIBUTING.md' not in required or 'SECURITY.md' not in required\nPY"

[[behaviors.verifiers]]
type = "command"
command = "! python3 - <<'PY'\nimport tomllib\nfrom pathlib import Path\nrequired = set(tomllib.loads(Path('REQUIREMENTS.toml').read_text())['release_source']['required'])\nassert 'CONTRIBUTING.md' not in required or 'SECURITY.md' not in required\nPY"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-06-01-public-release-G04-positive"
gap_id = "GAP-004"
polarity = "positive"
title = "Checkout action uses Node 24"
given = "GitHub workflow checkout steps"
when = "the workflow and action metadata verifiers run"
then = "checkout uses v6.0.2 and v6.0.2 declares node24"
confirmation = "rg -n 'actions/checkout@v6.0.2' .github/workflows templates/.github && gh api repos/actions/checkout/contents/action.yml?ref=v6.0.2 --jq '.content' | base64 -d | rg -q 'using: node24'"

[[behaviors.verifiers]]
type = "command"
command = "rg -n 'actions/checkout@v6.0.2' .github/workflows templates/.github && gh api repos/actions/checkout/contents/action.yml?ref=v6.0.2 --jq '.content' | base64 -d | rg -q 'using: node24'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-06-01-public-release-G04-negative"
gap_id = "GAP-004"
polarity = "negative"
title = "Node 20-backed checkout v4 is absent"
given = "GitHub workflow checkout steps"
when = "the stale checkout scanner runs"
then = "actions/checkout@v4 is absent"
confirmation = "! rg -n 'actions/checkout@v4' .github/workflows templates/.github"

[[behaviors.verifiers]]
type = "command"
command = "! rg -n 'actions/checkout@v4' .github/workflows templates/.github"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-06-01-public-release-G05-validation"
gap_id = "GAP-001"
polarity = "validation"
title = "Local release validation passes"
given = "public release prep changes"
when = "release and registry gates run"
then = "version, release readiness, source-limit, registry validation, and receipt chain pass"
confirmation = "scripts/release-version-check.sh && bash scripts/test-release-readiness.sh all && .codex/scripts/task-registry source-limit check && .codex/scripts/task-registry validate && .codex/scripts/task-registry verify-chain --format json"

[[behaviors.verifiers]]
type = "command"
command = "scripts/release-version-check.sh && bash scripts/test-release-readiness.sh all && .codex/scripts/task-registry source-limit check && .codex/scripts/task-registry validate && .codex/scripts/task-registry verify-chain --format json"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-06-01-public-release-001"
status = "active"
title = "Update public release metadata to 2.0.1"
kind = "release"
reason = "Public release metadata must identify the actual post-fix release."
acceptance_proof = "Behaviors B-2026-06-01-public-release-G01-positive and B-2026-06-01-public-release-G01-negative pass."
behavior_ids = ["B-2026-06-01-public-release-G01-positive", "B-2026-06-01-public-release-G01-negative"]

[[tasks.targets]]
file = "VERSION"
object = "release version"
required_change = "Set to 2.0.1."

[[tasks.targets]]
file = "plugin.json"
object = "version"
required_change = "Set to 2.0.1."

[[tasks.targets]]
file = ".codex-plugin/plugin.json"
object = "version"
required_change = "Set to 2.0.1."

[[tasks.targets]]
file = "MANIFEST.toml"
object = "plugin_version"
required_change = "Set to 2.0.1."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/Cargo.toml"
object = "package.version"
required_change = "Set to 2.0.1."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/Cargo.lock"
object = "task-registry-flow-cli package.version"
required_change = "Set to 2.0.1."

[[tasks.targets]]
file = "package.nix"
object = "version"
required_change = "Set to 2.0.1."

[[tasks.targets]]
file = "README.md"
object = "current release"
required_change = "Set to 2.0.1."

[[tasks.targets]]
file = "docs/releases/v2.md"
object = "release version"
required_change = "Set to 2.0.1."

[[tasks.targets]]
file = "scripts/test-release-readiness.sh"
object = "version drift fixture"
required_change = "Set the negative drift fixture to a version different from 2.0.1."

[[tasks]]
task_id = "TASK-2026-06-01-public-release-002"
status = "active"
title = "Add public contribution and security docs"
kind = "documentation"
reason = "Public users need clear contribution and vulnerability-reporting guidance."
acceptance_proof = "Behaviors B-2026-06-01-public-release-G02-positive and B-2026-06-01-public-release-G02-negative pass."
behavior_ids = ["B-2026-06-01-public-release-G02-positive", "B-2026-06-01-public-release-G02-negative"]

[[tasks.targets]]
file = "CHANGELOG.md"
object = "2.0.1 release notes"
required_change = "Add dated 2.0.1 section and leave Unreleased empty."

[[tasks.targets]]
file = "CONTRIBUTING.md"
object = "public contribution guidance"
required_change = "Add concise issue, PR, and validation guidance."

[[tasks.targets]]
file = "SECURITY.md"
object = "vulnerability reporting"
required_change = "Route vulnerability reports through GitHub private reporting/advisories."

[[tasks]]
task_id = "TASK-2026-06-01-public-release-003"
status = "active"
title = "Declare public docs in release-source requirements"
kind = "release"
reason = "New public release docs must be native release-source files."
acceptance_proof = "Behaviors B-2026-06-01-public-release-G03-positive and B-2026-06-01-public-release-G03-negative pass."
behavior_ids = ["B-2026-06-01-public-release-G03-positive", "B-2026-06-01-public-release-G03-negative"]

[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release_source"
required_change = "Set release version to 2.0.1 and require CONTRIBUTING.md plus SECURITY.md."

[[tasks]]
task_id = "TASK-2026-06-01-public-release-004"
status = "active"
title = "Update checkout actions to Node 24-backed version"
kind = "implementation"
reason = "GitHub warns actions/checkout@v4 uses deprecated Node 20 runtime."
acceptance_proof = "Behaviors B-2026-06-01-public-release-G04-positive and B-2026-06-01-public-release-G04-negative pass."
behavior_ids = ["B-2026-06-01-public-release-G04-positive", "B-2026-06-01-public-release-G04-negative"]

[[tasks.targets]]
file = ".github/workflows/ci.yml"
object = "checkout action"
required_change = "Use actions/checkout@v6.0.2."

[[tasks.targets]]
file = ".github/workflows/agent-governance.yml"
object = "checkout action"
required_change = "Use actions/checkout@v6.0.2."

[[tasks.targets]]
file = "templates/.github/workflows/agent-governance.yml.template"
object = "checkout action"
required_change = "Use actions/checkout@v6.0.2."

[[tasks]]
task_id = "TASK-2026-06-01-public-release-005"
status = "active"
title = "Validate and publish public release"
kind = "validation"
reason = "Public release requires local validation and GitHub handoff evidence."
acceptance_proof = "Behavior B-2026-06-01-public-release-G05-validation passes; final release-source, CI, tags, release, visibility, and public smoke tests pass after commit."
behavior_ids = ["B-2026-06-01-public-release-G05-validation"]

[[tasks.targets]]
file = "docs/plans/public-release-2-0-1-2026-06-01.md"
object = "walkthrough evidence"
required_change = "Capture local validation and public release handoff evidence."
```
