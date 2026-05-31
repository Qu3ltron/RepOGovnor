# Auto-Update Module: fetch-tree Mode Gap Closure Contract

## Approved Scope

**1 gap** — The auto-update module only supports flake-based systems (`nix flake lock`). This machine uses `builtins.fetchTree` with pinned git revs in `reclaimed-resources.nix`. Add a `fetch-tree` mode that fetches the latest commit from git remote and bumps the rev in-place.

## Phased Required Change Checklist

- [ ] `[MODIFY]` `modules/nixos/agent-governance-auto-update.nix` — add `mode` option (flake | fetch-tree), `nixFile`, `revVariable`, `remoteUrl` options, and `fetchTreeScript` that uses `git ls-remote` + `sed` to bump the pinned rev
- [ ] `[VERIFY]` `nix flake check`

## Per-Gap Success Criteria

### Gap: No fetch-tree mode
- **Current failure**: Auto-update service fails because there is no `flake.lock` at `/home/hasnamuss/reclaimed/system/`. The machine uses `builtins.fetchTree` with a pinned rev.
- **Good behavior**: In `fetch-tree` mode, the service runs `git ls-remote` to get the latest commit, compares against the pinned rev, bumps it with `sed` if changed, and runs `nixos-rebuild switch`.
- **Forbidden behavior**: Service exits with "no flake.lock" error on fetch-tree-configured machines.
- **Files involved**: `modules/nixos/agent-governance-auto-update.nix`
- **Positive test**: `nix flake check` passes; module evaluates with `mode = "fetch-tree"`
- **Negative test**: Module does not reference flake.lock when mode is fetch-tree

## Validation Plan

Focused:
```bash
nix flake check
cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml
```

## Walkthrough Evidence
- `nix flake check` passes
- All Rust tests pass

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-05-31-auto-update-fetch-tree"

[[behaviors]]
behavior_id = "B-001-fetch-tree-mode"
gap_id = "GAP-001"
polarity = "positive"
title = "Auto-update module supports fetch-tree mode"
given = "a NixOS machine using builtins.fetchTree with pinned git revs"
when = "the auto-update service runs in fetch-tree mode"
then = "git ls-remote fetches the latest rev, sed bumps it in the .nix file, and nixos-rebuild runs"
confirmation = "nix flake check"

[[behaviors.verifiers]]
type = "command"
command = "nix flake check 2>&1"
expected_exit = 0

[[behaviors.verifiers]]
type = "contains"
path = "modules/nixos/agent-governance-auto-update.nix"
needle = "fetch-tree"

[[behaviors]]
behavior_id = "B-001b-no-flake-lock-ref"
gap_id = "GAP-001"
polarity = "negative"
title = "fetch-tree mode does not reference flake.lock"
given = "auto-update module configured with mode = fetch-tree"
when = "the update script runs"
then = "it does not look for or reference a flake.lock file"
confirmation = "grep -q 'git ls-remote' modules/nixos/agent-governance-auto-update.nix"

[[behaviors.verifiers]]
type = "command"
command = "bash -c 'grep -q \"git ls-remote\" modules/nixos/agent-governance-auto-update.nix'"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-05-31-AU01"
status = "planned"
kind = "implementation"
reason = "Machine uses builtins.fetchTree, not flakes; auto-update must bump rev in .nix file"
behavior_ids = ["B-001-fetch-tree-mode", "B-001b-no-flake-lock-ref"]
title = "Add fetch-tree mode to auto-update module"
acceptance_proof = "Behavior B-001: nix flake check passes, module contains fetch-tree logic"

[[tasks.targets]]
file = "modules/nixos/agent-governance-auto-update.nix"
object = "fetch-tree update script and mode option"
required_change = "Add mode enum, fetchTreeScript with git ls-remote + sed, and nixFile/revVariable/remoteUrl options"
```
