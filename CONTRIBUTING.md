# Contributing

Governance-plugin is intentionally strict: changes that affect runtime
behavior, release posture, or governance workflow need a governed closure plan
before implementation.

## Before Opening a Pull Request

- Fork or branch from `main`.
- Keep changes scoped to one coherent behavior or release task.
- For implementation gaps, add or update a plan under `docs/plans/`, activate it
  with `.codex/scripts/task-registry activate`, and land it with
  `.codex/scripts/task-registry verify-landing`.
- Do not add compatibility shims for removed v2 hook paths or settings files.

## Validation

Run the smallest honest subset for your change. For release-facing changes, run:

```bash
.codex/scripts/task-registry source-limit check
.codex/scripts/task-registry validate
bash scripts/test-release-readiness.sh all
scripts/release-audit.sh
```

For Rust changes, also run:

```bash
cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check
cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml
cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings
```

## Pull Request Notes

- Include the plan id when a governed plan is involved.
- Include the validation commands you ran.
- Keep generated registry receipts and archives intact when the task registry
  changes.
