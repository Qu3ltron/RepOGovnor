# Tracked for CI

After overlay install, commit every path in [REQUIREMENTS.toml](REQUIREMENTS.toml) (`[tracked_for_ci].required`).

`status.sh --strict` and `cargo run --bin task_registry -- validate` fail on fresh clones when any path is missing or untracked.

Install prints the `git add` checklist automatically.
