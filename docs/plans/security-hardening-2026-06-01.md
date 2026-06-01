# Security Hardening Gap Closure Contract

## Approved Scope
Close security gaps found in the public plugin audit on 2026-06-01:

- Mutation hook command execution from `.codex/governance-cli.env`.
- Installer writes outside the target repo through configurable `hook_script_path`.
- `status.sh` execution of a target repo-controlled `.codex/scripts/task-registry` wrapper.
- Mutable GitHub Actions references and missing explicit workflow permissions.
- Nix auto-update shell interpolation of `flakeInput`.
- Repository rename drift after the local and remote repository moved to `RepOGovnor`.

Out of scope: changing the governance product model, adding network telemetry, changing public API names, or adding compatibility shims.

## Phased Required Change Checklist
### Phase 0: Activation and safety
- [ ] `[NEW]` `docs/plans/security-hardening-2026-06-01.md` - `Task Manifest`: activate this contract before implementation.
- [ ] `[VERIFY]` `docs/task-registry.toml` - `registry`: receipt chain remains valid after activation and landing.

### Phase 1: Hook and installer hardening
- [ ] `[MODIFY]` `tools/agent-governance/pre-tool-use-gap-closure.sh` - `env loading`: parse `.codex/governance-cli.env` as data, require the canonical verifier command, and execute verifier argv without `source` or `bash -lc`.
- [ ] `[MODIFY]` `templates/tools/agent-governance/pre-tool-use-gap-closure.sh.template` - `env loading`: keep the installed hook template identical to the canonical hardened behavior.
- [ ] `[MODIFY]` `scripts/render-from-config.sh` - `config validation`: reject absolute paths, `..`, shell metacharacters, and unsafe hook script locations before rendering.
- [ ] `[MODIFY]` `templates/.codex/scripts/task-registry.template` - `wrapper`: remove install-host fallback and require a repo-local plugin checkout.
- [ ] `[MODIFY]` `.codex/scripts/task-registry` - `wrapper`: align this repo with the hardened template.

### Phase 2: Runtime and supply-chain hardening
- [ ] `[MODIFY]` `scripts/status.sh` - `task_registry`: use the plugin-owned Rust manifest under `PLUGIN_ROOT` instead of executing a target repo-controlled wrapper.
- [ ] `[MODIFY]` `modules/nixos/agent-governance-auto-update.nix` - `flakeInput`: validate the input name and pass it to jq through `--arg` instead of shell interpolation.
- [ ] `[MODIFY]` `.github/workflows/ci.yml` - `workflow`: add explicit read-only permissions and pin third-party actions by commit SHA.
- [ ] `[MODIFY]` `.github/workflows/agent-governance.yml` - `workflow`: add explicit read-only permissions and pin third-party actions by commit SHA.
- [ ] `[MODIFY]` `templates/.github/workflows/agent-governance.yml.template` - `workflow`: keep installed governance workflow pinned and read-only.
- [ ] `[MODIFY]` `package.nix` - `source filter`: include workflow fixtures needed by package-time security tests.

### Phase 3: Tests, docs, release metadata
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/tests/security_tests.rs` - `security regressions`: add tests for safe hook env parsing, installer path rejection, wrapper behavior, workflow pinning, and Nix input validation.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/mod.rs` - `test module list`: include `security_tests`.
- [ ] `[MODIFY]` `REQUIREMENTS.toml` - `release_source.required`: declare the new Rust security test file.
- [ ] `[MODIFY]` `SECURITY.md` - `hardening notes`: document local trust boundaries and public reporting posture.
- [ ] `[MODIFY]` `.codex/agent-governance.toml` - `workspace_boundary`: align governed repo name, root, mutation root, and scratch root with `RepOGovnor`.
- [ ] `[MODIFY]` `project.config.toml` - `project`: align local project metadata with `RepOGovnor`.
- [ ] `[MODIFY]` `README.md` - `Install`: point public install instructions at `Qu3ltron/RepOGovnor`.
- [ ] `[MODIFY]` `AGENTS.md` - `Workspace boundary`: align agent instructions with the renamed local repo path.
- [ ] `[MODIFY]` `GEMINI.md` - `Repo boundary`: align Antigravity instructions with the renamed local repo path.
- [ ] `[MODIFY]` `CLAUDE.md` - `Workspace boundary`: align Claude instructions with the renamed local repo path.
- [ ] `[MODIFY]` `CONTRIBUTING.md` - `intro`: use the public repo name.
- [ ] `[MODIFY]` `plugin.json` - `repository metadata`: point public metadata at `Qu3ltron/RepOGovnor`.
- [ ] `[MODIFY]` `.codex-plugin/plugin.json` - `repository metadata`: point plugin metadata at `Qu3ltron/RepOGovnor`.
- [ ] `[MODIFY]` `flake.nix` - `dev shell banner`: use the public repo name.
- [ ] `[MODIFY]` `modules/nixos/agent-governance.nix` - `package option defaultText`: use the renamed repository placeholder.
- [ ] `[MODIFY]` `modules/nixos/agent-governance-auto-update.nix` - `flake input docs`: describe the renamed repository.

## Per-Gap Success Criteria
### GAP-001: Hook env command execution
- Current failure: the hook sources `.codex/governance-cli.env` and runs the resulting string with `bash -lc`.
- Good behavior: a canonical `GOVERNANCE_VERIFY_HOOK_CMD=".codex/scripts/task-registry verify-mutation-hook"` value is accepted and executed as fixed argv with `--format`.
- Forbidden behavior: arbitrary shell statements, extra assignments, command substitutions, or noncanonical verifier commands are rejected before execution.
- Files involved: `tools/agent-governance/pre-tool-use-gap-closure.sh`, `templates/tools/agent-governance/pre-tool-use-gap-closure.sh.template`, `rust/task-registry-flow-cli/src/tests/security_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml security_hook_accepts_canonical_env -- --nocapture`
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml security_hook_rejects_malicious_env -- --nocapture`
- Domain/API/UI: hook JSON output remains the existing allow/deny contract.
- Runtime: mutation verification still delegates to `.codex/scripts/task-registry verify-mutation-hook`.

### GAP-002: Installer path traversal
- Current failure: `hook_script_path` can be absolute or contain `..`, causing writes outside the target repo.
- Good behavior: only safe repo-relative hook script paths under `tools/agent-governance/` are accepted.
- Forbidden behavior: absolute paths, parent traversal, shell metacharacters, and non-hook destinations fail before writes.
- Files involved: `scripts/render-from-config.sh`, `rust/task-registry-flow-cli/src/tests/security_tests.rs`.
- Positive test: `bash scripts/test-install-modes.sh`
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml security_installer_rejects_unsafe_hook_paths -- --nocapture`
- Domain/API/UI: project config remains TOML-based; unsupported unsafe variants fail closed.
- Runtime: installer never writes outside `TARGET_ROOT`.

### GAP-003: Status script target-wrapper execution
- Current failure: `status.sh` prefers the target repo `.codex/scripts/task-registry` wrapper, which can be repo-controlled code.
- Good behavior: status checks invoke the Rust CLI from the plugin checkout running the status script.
- Forbidden behavior: a malicious target wrapper is executed during status checks.
- Files involved: `scripts/status.sh`, `rust/task-registry-flow-cli/src/tests/security_tests.rs`.
- Positive test: `plugins/agent-governance/scripts/status.sh --strict` in a fresh smoke install.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml security_status_uses_plugin_manifest_not_target_wrapper -- --nocapture`
- Domain/API/UI: status output remains unchanged.
- Runtime: status still runs against the target repo as working directory.

### GAP-004: Mutable CI action references
- Current failure: workflows rely on mutable action refs and do not declare explicit token permissions.
- Good behavior: workflows and templates pin external actions by SHA and set `permissions: contents: read`.
- Forbidden behavior: tag or branch action refs in release workflows.
- Files involved: `.github/workflows/ci.yml`, `.github/workflows/agent-governance.yml`, `templates/.github/workflows/agent-governance.yml.template`, `rust/task-registry-flow-cli/src/tests/security_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml security_workflows_pin_actions_and_permissions -- --nocapture`
- Negative test: same test fails when a workflow contains `uses: owner/repo@tag` or lacks read-only permissions.
- Domain/API/UI: GitHub Actions behavior remains equivalent.
- Runtime: repository Actions token stays read-only.

### GAP-005: Nix auto-update input interpolation
- Current failure: `flakeInput` is interpolated into shell/jq strings in a root-running update service.
- Good behavior: unsafe flake input names fail evaluation and jq receives the input through `--arg`.
- Forbidden behavior: shell metacharacters from `flakeInput` appear unquoted in the generated service script.
- Files involved: `modules/nixos/agent-governance-auto-update.nix`, `rust/task-registry-flow-cli/src/tests/security_tests.rs`.
- Positive test: `nix flake check --no-build --all-systems`
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml security_nix_auto_update_uses_safe_flake_input -- --nocapture`
- Domain/API/UI: NixOS module options remain the public surface.
- Runtime: update service still updates only the configured plugin flake input.

### GAP-006: Repository rename drift
- Current failure: public install metadata, agent workspace boundaries, local governance config, and release validation examples still reference `Governance-plugin`.
- Good behavior: current public install URLs, plugin metadata, local governance boundaries, and CI examples reference `RepOGovnor`.
- Forbidden behavior: current release-facing files continue pointing to the retired GitHub repository or old local path.
- Files involved: `.codex/agent-governance.toml`, `project.config.toml`, `README.md`, `AGENTS.md`, `GEMINI.md`, `CLAUDE.md`, `CONTRIBUTING.md`, `plugin.json`, `.codex-plugin/plugin.json`, `flake.nix`, `modules/nixos/agent-governance.nix`, `modules/nixos/agent-governance-auto-update.nix`, `docs/plans/security-hardening-2026-06-01.md`.
- Positive test: `git remote get-url origin | grep -qx 'git@github.com:Qu3ltron/RepOGovnor.git'`
- Negative test: `! rg --hidden --no-ignore -n 'Qu3ltron/Governance-plugin|/home/hasnamuss/reclaimed/work/Governance-plugin|repo_name = "Governance-plugin"|Governance-plugin>/package.nix|Governance-plugin dev shell' README.md AGENTS.md GEMINI.md CLAUDE.md CONTRIBUTING.md plugin.json .codex-plugin/plugin.json .codex/agent-governance.toml project.config.toml flake.nix modules/nixos/agent-governance.nix modules/nixos/agent-governance-auto-update.nix`
- Domain/API/UI: public repository name changes; package name and plugin id remain `agent-governance`.
- Runtime: local governance mutation boundary points at `/home/hasnamuss/reclaimed/work/RepOGovnor`.

## Validation Plan
Focused:
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml security_ -- --nocapture`
- `.codex/scripts/task-registry source-limit check`
- `nix run nixpkgs#shellcheck -- scripts/*.sh tools/agent-governance/*.sh templates/.codex/scripts/task-registry.template templates/.cursor/hooks/gap-closure-gate.sh.template templates/tools/agent-governance/pre-tool-use-gap-closure.sh.template`

Full:
- `.codex/scripts/task-registry validate`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings`
- `bash scripts/test-install-modes.sh`
- `bash scripts/test-release-readiness.sh all`
- `bash scripts/release-audit.sh`
- `nix flake check --no-build --all-systems`
- `nix run nixpkgs#gitleaks -- detect --no-git --source . --redact --verbose`
- `gh run list --repo Qu3ltron/RepOGovnor --branch main --limit 8`

## Source File Limit
Expected impact: one new focused test file plus small changes to existing scripts, templates, workflows, and Nix module. `rust/task-registry-flow-cli/src/tests/mod.rs` is near the line limit, so this plan only adds one module declaration there and places tests in `security_tests.rs`. Run `.codex/scripts/task-registry source-limit check` before landing.

## Walkthrough Evidence
- Contract activation output.
- Focused security test output.
- Full local validation output.
- `verify-landing` completion output for this plan.
- `verify-chain --format json` output.
- GitHub Actions green result after push.

## Task Manifest
```toml
schema_version = 2
plan_id = "PLAN-2026-06-01-security-hardening"

[[behaviors]]
behavior_id = "B-001-hook-canonical-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Hook accepts canonical verifier only"
given = "A governance-cli.env with the canonical verifier command"
when = "the mutation hook runs"
then = "it invokes the fixed verifier argv with the requested format"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml security_hook_accepts_canonical_env -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml security_hook_accepts_canonical_env -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-hook-env-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Hook rejects executable env content"
given = "A governance-cli.env containing shell execution or a noncanonical verifier"
when = "the mutation hook runs"
then = "it denies without executing the malicious content"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml security_hook_rejects_malicious_env -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml security_hook_rejects_malicious_env -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-003-installer-path-negative"
gap_id = "GAP-002"
polarity = "negative"
title = "Installer rejects unsafe hook paths"
given = "A project config with absolute, traversal, or shell-like hook_script_path"
when = "render-from-config is invoked"
then = "the installer exits nonzero before writing outside the target root"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml security_installer_rejects_unsafe_hook_paths -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml security_installer_rejects_unsafe_hook_paths -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-004-installer-positive"
gap_id = "GAP-002"
polarity = "positive"
title = "Installer keeps canonical merge behavior"
given = "The project example config and a normal target repo"
when = "install mode behavior tests run"
then = "merge, force, dry-run, and posture checks still pass"
confirmation = "bash scripts/test-install-modes.sh"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-install-modes.sh"
expected_exit = 0

[[behaviors]]
behavior_id = "B-005-status-wrapper-negative"
gap_id = "GAP-003"
polarity = "negative"
title = "Status does not execute target wrapper"
given = "A target repo with a malicious .codex/scripts/task-registry wrapper"
when = "status posture checks run"
then = "the wrapper side effect is absent"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml security_status_uses_plugin_manifest_not_target_wrapper -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml security_status_uses_plugin_manifest_not_target_wrapper -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-006-status-positive"
gap_id = "GAP-003"
polarity = "positive"
title = "Status remains functional"
given = "The plugin source checkout"
when = "strict status runs"
then = "the posture report still passes"
confirmation = "scripts/status.sh --strict"

[[behaviors.verifiers]]
type = "command"
command = "scripts/status.sh --strict"
expected_exit = 0

[[behaviors]]
behavior_id = "B-007-workflow-pinning"
gap_id = "GAP-004"
polarity = "positive"
title = "Workflows are pinned and read-only"
given = "Repository workflows and installed workflow template"
when = "security workflow tests inspect them"
then = "all external actions use SHA refs and permissions are read-only"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml security_workflows_pin_actions_and_permissions -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml security_workflows_pin_actions_and_permissions -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-008-workflow-negative"
gap_id = "GAP-004"
polarity = "negative"
title = "Mutable action refs are detected"
given = "A workflow fixture with a tag action ref or missing permissions"
when = "security workflow tests inspect it"
then = "the verifier rejects the fixture"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml security_workflows_pin_actions_and_permissions -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml security_workflows_pin_actions_and_permissions -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-009-nix-flake-input"
gap_id = "GAP-005"
polarity = "positive"
title = "Nix auto-update uses safe flake input handling"
given = "The NixOS auto-update module"
when = "security tests inspect the generated script source"
then = "flakeInput is asserted safe and jq receives it through --arg"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml security_nix_auto_update_uses_safe_flake_input -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml security_nix_auto_update_uses_safe_flake_input -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-010-full-security-validation"
gap_id = "GAP-VALIDATION"
polarity = "validation"
title = "Full security validation passes"
given = "All hardening changes are in place"
when = "the full local release and security gates run"
then = "dependency audit, secret scan, source limit, tests, and release checks pass"
confirmation = "bash scripts/release-audit.sh && nix run nixpkgs#gitleaks -- detect --no-git --source . --redact --verbose"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/release-audit.sh && nix run nixpkgs#gitleaks -- detect --no-git --source . --redact --verbose"
expected_exit = 0

[[behaviors]]
behavior_id = "B-011-nix-flake-input-negative"
gap_id = "GAP-005"
polarity = "negative"
title = "Unsafe Nix flake input names are rejected"
given = "The NixOS auto-update module is evaluated with a shell-like flake input name"
when = "the module assertion is checked"
then = "evaluation fails before generating a root service script"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml security_nix_auto_update_uses_safe_flake_input -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml security_nix_auto_update_uses_safe_flake_input -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-012-repogovnor-remote-positive"
gap_id = "GAP-006"
polarity = "positive"
title = "Origin remote points at RepOGovnor"
given = "The local checkout after repository rename"
when = "the origin remote is inspected"
then = "it points at Qu3ltron/RepOGovnor"
confirmation = "git remote get-url origin | grep -qx 'git@github.com:Qu3ltron/RepOGovnor.git'"

[[behaviors.verifiers]]
type = "command"
command = "git remote get-url origin | grep -qx 'git@github.com:Qu3ltron/RepOGovnor.git'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-013-repogovnor-stale-negative"
gap_id = "GAP-006"
polarity = "negative"
title = "Current release metadata has no old repo pointers"
given = "Current release-facing metadata and agent boundary files"
when = "stale Governance-plugin repository pointers are searched"
then = "no retired remote URL, retired local path, or retired repo name remains in current files"
confirmation = "! rg --hidden --no-ignore -n 'Qu3ltron/Governance-plugin|/home/hasnamuss/reclaimed/work/Governance-plugin|repo_name = .*Governance-plugin|Governance-plugin>/package.nix|Governance-plugin dev shell' README.md AGENTS.md GEMINI.md CLAUDE.md CONTRIBUTING.md plugin.json .codex-plugin/plugin.json .codex/agent-governance.toml project.config.toml flake.nix modules/nixos/agent-governance.nix modules/nixos/agent-governance-auto-update.nix"

[[behaviors.verifiers]]
type = "command"
command = "! rg --hidden --no-ignore -n 'Qu3ltron/Governance-plugin|/home/hasnamuss/reclaimed/work/Governance-plugin|repo_name = .*Governance-plugin|Governance-plugin>/package.nix|Governance-plugin dev shell' README.md AGENTS.md GEMINI.md CLAUDE.md CONTRIBUTING.md plugin.json .codex-plugin/plugin.json .codex/agent-governance.toml project.config.toml flake.nix modules/nixos/agent-governance.nix modules/nixos/agent-governance-auto-update.nix"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-06-01-001"
behavior_ids = ["B-001-hook-canonical-positive", "B-002-hook-env-negative"]
status = "planned"
title = "Harden mutation hook env handling"
kind = "implementation"
reason = "Eliminate sourced env and shell-string execution from mutation hook."
acceptance_proof = "Behaviors B-001-hook-canonical-positive and B-002-hook-env-negative."

[[tasks.targets]]
file = "tools/agent-governance/pre-tool-use-gap-closure.sh"
object = "mutation hook env parser and verifier execution"
required_change = "Parse governance-cli.env as data, require canonical verifier, execute fixed argv."

[[tasks.targets]]
file = "templates/tools/agent-governance/pre-tool-use-gap-closure.sh.template"
object = "mutation hook template env parser and verifier execution"
required_change = "Mirror canonical hardened hook behavior."

[[tasks]]
task_id = "TASK-2026-06-01-002"
behavior_ids = ["B-003-installer-path-negative", "B-004-installer-positive"]
status = "planned"
title = "Harden installer path handling"
kind = "implementation"
reason = "Prevent config-controlled writes outside target repo while preserving install behavior."
acceptance_proof = "Behaviors B-003-installer-path-negative and B-004-installer-positive."

[[tasks.targets]]
file = "scripts/render-from-config.sh"
object = "hook_script_path validation"
required_change = "Reject unsafe hook script paths before writes."

[[tasks.targets]]
file = "scripts/test-install-modes.sh"
object = "install mode regression coverage"
required_change = "Assert canonical install modes still pass and unsafe hook paths fail."

[[tasks.targets]]
file = "templates/.codex/scripts/task-registry.template"
object = "task registry wrapper"
required_change = "Require repo-local plugin checkout; remove absolute fallback."

[[tasks.targets]]
file = ".codex/scripts/task-registry"
object = "repo task registry wrapper"
required_change = "Align local wrapper with hardened template."

[[tasks]]
task_id = "TASK-2026-06-01-003"
behavior_ids = ["B-005-status-wrapper-negative", "B-006-status-positive"]
status = "planned"
title = "Harden status script execution boundary"
kind = "implementation"
reason = "Avoid executing target-controlled wrappers during posture checks."
acceptance_proof = "Behaviors B-005-status-wrapper-negative and B-006-status-positive."

[[tasks.targets]]
file = "scripts/status.sh"
object = "task_registry function"
required_change = "Invoke plugin-owned Rust manifest instead of target wrapper."

[[tasks]]
task_id = "TASK-2026-06-01-004"
behavior_ids = ["B-007-workflow-pinning", "B-008-workflow-negative"]
status = "planned"
title = "Pin workflows and read-only permissions"
kind = "implementation"
reason = "Reduce GitHub Actions supply-chain and token-permission exposure."
acceptance_proof = "Behaviors B-007-workflow-pinning and B-008-workflow-negative."

[[tasks.targets]]
file = ".github/workflows/ci.yml"
object = "CI action refs and permissions"
required_change = "Set read-only permissions and pin external actions by SHA."

[[tasks.targets]]
file = ".github/workflows/agent-governance.yml"
object = "governance action refs and permissions"
required_change = "Set read-only permissions and pin external actions by SHA."

[[tasks.targets]]
file = "templates/.github/workflows/agent-governance.yml.template"
object = "installed governance workflow action refs and permissions"
required_change = "Set read-only permissions and pin external actions by SHA."

[[tasks.targets]]
file = "package.nix"
object = "Nix source filter"
required_change = "Include workflow fixtures needed by security tests during package checks."

[[tasks]]
task_id = "TASK-2026-06-01-005"
behavior_ids = ["B-009-nix-flake-input", "B-011-nix-flake-input-negative"]
status = "planned"
title = "Harden Nix auto-update flake input handling"
kind = "implementation"
reason = "Avoid root service shell interpolation of unvalidated flake input values."
acceptance_proof = "Behaviors B-009-nix-flake-input and B-011-nix-flake-input-negative."

[[tasks.targets]]
file = "modules/nixos/agent-governance-auto-update.nix"
object = "flakeInput validation and jq invocation"
required_change = "Validate flake input names and use jq --arg input."

[[tasks]]
task_id = "TASK-2026-06-01-006"
behavior_ids = ["B-010-full-security-validation"]
status = "planned"
title = "Synchronize release metadata and security docs"
kind = "documentation"
reason = "Keep public release source and security reporting posture aligned with hardening changes."
acceptance_proof = "Behavior B-010-full-security-validation."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/mod.rs"
object = "test module list"
required_change = "Include security_tests without expanding existing near-limit test file."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/security_tests.rs"
object = "security regression tests"
required_change = "Cover hook env parsing, installer paths, status boundary, workflow pinning, and Nix flake input handling."

[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release_source.required"
required_change = "Declare new Rust security test source."

[[tasks.targets]]
file = "SECURITY.md"
object = "Security posture notes"
required_change = "Document local trust boundaries and reporting posture."

[[tasks]]
task_id = "TASK-2026-06-01-007"
behavior_ids = ["B-012-repogovnor-remote-positive", "B-013-repogovnor-stale-negative"]
status = "planned"
title = "Align repository rename metadata"
kind = "governance"
reason = "Public release metadata and local governance boundaries must point at RepOGovnor after the repo rename."
acceptance_proof = "Behaviors B-012-repogovnor-remote-positive and B-013-repogovnor-stale-negative."

[[tasks.targets]]
file = ".codex/agent-governance.toml"
object = "workspace boundary"
required_change = "Use RepOGovnor repo name, root, mutation root, and scratch root."

[[tasks.targets]]
file = "project.config.toml"
object = "project metadata"
required_change = "Use RepOGovnor repo name and scratch root."

[[tasks.targets]]
file = "README.md"
object = "install URL"
required_change = "Point public submodule instructions at Qu3ltron/RepOGovnor."

[[tasks.targets]]
file = "AGENTS.md"
object = "workspace boundary"
required_change = "Use the RepOGovnor local repo path and scratch root."

[[tasks.targets]]
file = "GEMINI.md"
object = "repo boundary"
required_change = "Use the RepOGovnor local repo path and scratch root."

[[tasks.targets]]
file = "CLAUDE.md"
object = "workspace boundary"
required_change = "Use the RepOGovnor local repo path and scratch root."

[[tasks.targets]]
file = "CONTRIBUTING.md"
object = "intro repository name"
required_change = "Use RepOGovnor as the public repository name."

[[tasks.targets]]
file = "plugin.json"
object = "repository metadata"
required_change = "Point repository and homepage at Qu3ltron/RepOGovnor."

[[tasks.targets]]
file = ".codex-plugin/plugin.json"
object = "repository metadata"
required_change = "Point repository and homepage at Qu3ltron/RepOGovnor."

[[tasks.targets]]
file = "flake.nix"
object = "dev shell banner"
required_change = "Use RepOGovnor as the displayed repo name."

[[tasks.targets]]
file = "modules/nixos/agent-governance.nix"
object = "package defaultText"
required_change = "Use the renamed repository placeholder."

[[tasks.targets]]
file = "docs/plans/security-hardening-2026-06-01.md"
object = "validation references"
required_change = "Use Qu3ltron/RepOGovnor for release validation examples."
```
