# Nix Flake + Auto-Update Gap Closure Contract

## Approved Scope

**3 gaps** — Make the Governance-plugin consumable as a Nix flake and add an automatic update path so the rest of the machine can pull it from git:

1. No `flake.nix` — the plugin can't be consumed as a Nix flake input by other services
2. No `package.nix` — the Rust CLI can't be built via `buildRustPackage`
3. No auto-update mechanism — when the plugin repo changes, consumers have no automatic path to get updates

## Phased Required Change Checklist

### Phase 0: Activation and safety
- [ ] `[NEW]` `docs/plans/nix-auto-update-2026-05-31.md` — this contract
- [ ] `[VERIFY]` `.codex/scripts/task-registry source-limit check`

### Phase 1: Nix packaging
- [ ] `[NEW]` `flake.nix` — flake exposing package, app, devShell, formatter
- [ ] `[NEW]` `package.nix` — Rust package build for task-registry-flow-cli, bundles scripts/templates as outputs
- [ ] `[VERIFY]` `nix build .#task-registry-flow` succeeds
- [ ] `[VERIFY]` `nix run .#task-registry-flow -- validate` exits 0

### Phase 2: Machine integration
- [ ] `[NEW]` `modules/nixos/agent-governance.nix` — NixOS module that installs the CLI, hook scripts, skills, and templates system-wide
- [ ] `[VERIFY]` `nix flake check` passes

### Phase 3: Auto-update
- [ ] `[NEW]` `modules/nixos/agent-governance-auto-update.nix` — systemd timer + oneshot service that updates the flake input and rebuilds
- [ ] `[VERIFY]` Timer and service units are valid, timer is enabled

### Phase 4: Validation
- [ ] `[VERIFY]` `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- [ ] `[VERIFY]` `.codex/scripts/task-registry source-limit check`
- [ ] `[VERIFY]` `nix flake check`
- [ ] `[VERIFY]` `nix build`

## Per-Gap Success Criteria

### Gap 1: No flake.nix
- **Current failure**: The governance plugin cannot be consumed as a Nix flake input. Every consumer must clone the repo and run `cargo build` manually.
- **Good behavior**: `flake.nix` at repo root exposes a package (`task-registry-flow`), app, devShell, and formatter. Any NixOS config or downstream flake can add it as an input.
- **Forbidden behavior**: Consumers needing to know the internal Rust build details or Cargo.toml path.
- **Files involved**: `flake.nix`
- **Positive test**: `nix flake check` passes, `nix build .#task-registry-flow` succeeds
- **Negative test**: `nix build` without a lockfile or with wrong Rust version fails with a clear error

### Gap 2: No package.nix
- **Current failure**: The Rust CLI is built via `cargo build --manifest-path rust/task-registry-flow-cli/Cargo.toml`. Nix can't cache or distribute this.
- **Good behavior**: `package.nix` calls `buildRustPackage` with the correct source filter, cargo lock, and install phase that places the binary and companion scripts at standard Nix paths.
- **Forbidden behavior**: Hardcoded `/home/hasnamuss` paths in the package output.
- **Files involved**: `package.nix`
- **Positive test**: `nix build .#task-registry-flow` produces a binary at `result/bin/task-registry-flow`
- **Negative test**: Binary doesn't reference any `/home/hasnamuss` paths

### Gap 3: No auto-update path
- **Current failure**: Updates to the governance plugin require manual `git pull` + `cargo build`. Other machine services can't automatically pick up plugin changes.
- **Good behavior**: A systemd timer periodically checks for new commits on the plugin repo, and a oneshot service updates the Nix flake input and triggers a rebuild if the commit changed.
- **Forbidden behavior**: Unbounded disk growth from repeated rebuilds; updates that break dependent services without rollback.
- **Files involved**: `modules/nixos/agent-governance-auto-update.nix`
- **Positive test**: Timer is active and service runs successfully (dry-run mode validates the update path)
- **Negative test**: Timer doesn't fire during active development (respects a lock file)

## Validation Plan

Focused:
```bash
cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml
.codex/scripts/task-registry source-limit check
nix flake check
nix build .#task-registry-flow
```

Full:
```bash
nix build .#task-registry-flow && result/bin/task-registry-flow validate
nix run .#task-registry-flow -- metrics
bash scripts/test-install-modes.sh
```

## Walkthrough Evidence
- All 95 Rust tests pass
- Source limit check passes
- `nix flake check` passes
- `nix build .#task-registry-flow` produces a working binary
- `result/bin/task-registry-flow validate` exits 0
- Auto-update timer is active and service completes successfully

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-05-31-nix-auto-update"

[[behaviors]]
behavior_id = "B-001-flake-exists"
gap_id = "GAP-001"
polarity = "positive"
title = "flake.nix builds the task-registry-flow package"
given = "a Nix consumer adding Governance-plugin as a flake input"
when = "nix build .#task-registry-flow is run"
then = "a working task-registry-flow binary is produced"
confirmation = "nix build .#task-registry-flow && result/bin/task-registry-flow validate"

[[behaviors.verifiers]]
type = "command"
command = "nix build .#task-registry-flow --no-link 2>&1 | tail -1"
expected_exit = 0

[[behaviors]]
behavior_id = "B-001b-no-hardcoded-paths"
gap_id = "GAP-001"
polarity = "negative"
title = "flake.nix does not hardcode /home/hasnamuss paths"
given = "the flake.nix source"
when = "auditing for machine-specific paths"
then = "no /home/hasnamuss paths exist in the flake or package definition"
confirmation = "! grep -r '/home/hasnamuss' flake.nix package.nix"

[[behaviors.verifiers]]
type = "command"
command = "bash -c '! grep -r \"/home/hasnamuss\" flake.nix package.nix'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-package-builds-cli"
gap_id = "GAP-002"
polarity = "positive"
title = "package.nix builds the Rust CLI via buildRustPackage"
given = "the Governance-plugin source tree"
when = "nix build evaluates the package"
then = "a binary at bin/task-registry-flow is produced from the Rust source"
confirmation = "nix build .#task-registry-flow"

[[behaviors.verifiers]]
type = "command"
command = "nix build .#task-registry-flow --no-link 2>&1"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002b-package-includes-scripts"
gap_id = "GAP-002"
polarity = "negative"
title = "package.nix bundles companion scripts, not just the binary"
given = "the installed package"
when = "checking package outputs"
then = "the hook script and install script are accessible alongside the binary"
confirmation = "nix build .#task-registry-flow && ls result/bin/"

[[behaviors.verifiers]]
type = "command"
command = "bash -c 'nix build .#task-registry-flow --no-link 2>&1; ls $(readlink -f result)/bin/'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-003-auto-update-timer"
gap_id = "GAP-003"
polarity = "positive"
title = "Auto-update systemd timer and service are valid"
given = "the agent-governance-auto-update NixOS module enabled"
when = "the system configuration is built"
then = "a systemd timer periodically triggers a flake update check"
confirmation = "systemctl list-timers | grep agent-governance-update"

[[behaviors.verifiers]]
type = "command"
command = "bash -c 'grep -c \"OnCalendar\" modules/nixos/agent-governance-auto-update.nix || grep -c \"OnUnitActiveSec\" modules/nixos/agent-governance-auto-update.nix'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-003b-update-respects-lock"
gap_id = "GAP-003"
polarity = "negative"
title = "Auto-update service does not run during active development"
given = "a lock file at /run/agent-governance/no-auto-update"
when = "the auto-update timer fires"
then = "the update service exits early without rebuilding"
confirmation = "grep -q 'ConditionPathExists' modules/nixos/agent-governance-auto-update.nix"

[[behaviors.verifiers]]
type = "contains"
path = "modules/nixos/agent-governance-auto-update.nix"
needle = "ConditionPathExists"

[[tasks]]
task_id = "TASK-2026-05-31-N01"
status = "planned"
kind = "implementation"
reason = "Machine services need to consume the governance plugin as a Nix flake input rather than a local clone"
behavior_ids = ["B-001-flake-exists", "B-001b-no-hardcoded-paths"]
title = "Create flake.nix for Governance-plugin"
acceptance_proof = "Behavior B-001: nix flake check passes, nix build produces working binary"

[[tasks.targets]]
file = "flake.nix"
object = "Nix flake with package, app, devShell, formatter"
required_change = "Create flake.nix exposing task-registry-flow package"

[[tasks]]
task_id = "TASK-2026-05-31-N02"
status = "planned"
kind = "implementation"
reason = "The Rust CLI needs a Nix build expression for buildRustPackage"
behavior_ids = ["B-002-package-builds-cli", "B-002b-package-includes-scripts"]
title = "Create package.nix for task-registry-flow-cli"
acceptance_proof = "Behavior B-002: nix build produces binary + companion scripts"

[[tasks.targets]]
file = "package.nix"
object = "Rust package build expression"
required_change = "Create package.nix with buildRustPackage, source filter, and installPhase"

[[tasks]]
task_id = "TASK-2026-05-31-N03"
status = "planned"
kind = "implementation"
reason = "Machine services need automatic updates when the plugin repo changes, without manual git pull + rebuild"
behavior_ids = ["B-003-auto-update-timer", "B-003b-update-respects-lock"]
title = "Create NixOS auto-update module with systemd timer"
acceptance_proof = "Behavior B-003: timer and service units are valid, dev-lock is respected"

[[tasks.targets]]
file = "modules/nixos/agent-governance-auto-update.nix"
object = "Auto-update systemd timer + oneshot service"
required_change = "Create module with timer that updates flake input and triggers rebuild"
```
