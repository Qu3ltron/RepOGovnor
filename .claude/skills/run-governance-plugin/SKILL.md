---
name: run-governance-plugin
description: Build, test, and smoke-test the Agent Governance plugin (Rust CLI + bash scripts). Use when asked to run, build, test, or verify the governance plugin, task-registry CLI, or install scripts.
---

# Run: Agent Governance Plugin

Plugin that installs plan-first agent governance (task registry CLI, mutation hooks,
skills, templates) into consumer repos. The main deliverable is a Rust CLI
(`task-registry-flow`) + supporting bash scripts.

**All paths relative to repo root.**

## Prerequisites

```bash
sudo apt-get install -y build-essential cargo python3
```

`python3` + `tomllib` (stdlib ≥3.11) needed by `render-from-config.sh`.

## Build

```bash
cargo build --locked --release --manifest-path rust/task-registry-flow-cli/Cargo.toml
```

Binary lands at `/tmp/agent-governance-cargo-target/release/task-registry-flow`
(or `$AGENT_GOVERNANCE_CARGO_TARGET_DIR` if set).

Debug build (faster compile, no optimisation):
```bash
cargo build --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml
```

## Run (agent path) — smoke test driver

The driver exercises every read-only subcommand and script in one pass:

```bash
bash .claude/skills/run-governance-plugin/smoke.sh
```

Add `--quick` for a debug build.

What it checks:
- Release (or debug) build succeeds
- `cargo test` passes without hardcoded count assumptions
- Read-only CLI subcommands exit 0 with expected output:
  `validate`, `metrics`, `source-limit check`, `source-limit plan`,
  `status-check`, `verify-mutation-hook` (×3 formats), `install plan`,
  `verify-chain`, `verify-chain --format json`, `release-check all`, and
  `release-check all --format json`
- `scripts/status.sh --strict` exits 0
- `scripts/test-install-modes.sh` passes
- `MODE=merge DRY_RUN=1 scripts/render-from-config.sh` succeeds
- Wrapper `.codex/scripts/task-registry validate` works
- The local receipt chain stays valid through `verify-chain`

Exit code 0 = all clear.

## Direct invocation

For PRs touching a single Rust module, skip the full smoke test:

```bash
# Build + run one test module
cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- <module>

# Run one CLI subcommand
BIN=/tmp/agent-governance-cargo-target/release/task-registry-flow
$BIN validate
$BIN source-limit check
$BIN status-check --format json
```

## Run (human path)

The `.codex/scripts/task-registry` wrapper runs `cargo run` under the hood:

```bash
.codex/scripts/task-registry validate
.codex/scripts/task-registry metrics
.codex/scripts/task-registry source-limit check
```

First run compiles from source (~35 s); subsequent runs are faster.

## Test

```bash
cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml
```

All tests should pass without relying on a fixed test count. In other words,
cargo test passes without hardcoded count assumptions.

## Gotchas

- **`set -e` + `((var++))`**: `((pass++))` returns 1 when `pass=0`, killing the
  script under `set -e`. Use `pass=$((pass + 1))` instead.
- **`status.sh --strict` exits 1**: treat this as a real posture failure. Read
  the failed diagnostic and fix the missing, stale, or noncanonical governance
  artifact.
- **`render-from-config.sh`** reads `MODE` and `DRY_RUN` from the environment,
  not positional args. Use `env MODE=merge DRY_RUN=1 bash ...`.
- **`--help` is not a flag**: the CLI shows usage on any unknown argument, not
  via `--help`. Omit it.
- **The wrapper script at `.codex/scripts/task-registry`** resolves the repo
  root and delegates to the plugin-owned Rust CLI. Run it from a tracked repo
  with the plugin checkout present.

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| `manifest path ... does not exist` from wrapper | `cd` to repo root first; wrapper uses `git rev-parse --show-toplevel` |
| `cargo build` fails with missing `toml` crate | Run `cargo fetch` first, or check network |
| `render-from-config.sh` says `missing or invalid MODE` | Set `MODE=merge` or `MODE=force` in environment |
| Smoke test reports `FAIL` on first run | Check `$CARGO_TARGET_DIR` matches what `smoke.sh` expects (default `/tmp/agent-governance-cargo-target`) |
