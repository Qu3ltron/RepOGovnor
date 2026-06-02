# RepOGovnor

RepOGovnor targets engineering policy compliance for agent-assisted repos. It
helps maintainers turn declared engineering policy into local workflow evidence:
what was approved, which files were in scope, which checks ran, what passed or
failed, and what remains unproven.

The install namespace is still `agent-governance`, but the product direction is
broader than agent hooks. It installs a local policy and evidence layer for
Codex, Antigravity CLI, Cursor, and Claude Code:

- every meaningful change starts from an approved plan or declared policy
- plan tasks are activated into a local registry
- mutation hooks block unplanned implementation writes
- installs and upgrades are checked by repeatable scripts
- local receipts show what was validated, without network telemetry

The goal is simple: keep agent-assisted development fast while making
engineering-policy compliance inspectable enough for production repositories.

## Who this is for

Use this plugin if you are:

- a maintainer who lets agents edit a real repository
- a team lead who needs policy-first agent workflows
- a project owner who wants local evidence before merging agent changes
- a regulated or high-risk project that needs explicit engineering-policy
  provenance, not certification
- a multi-tool user moving between Codex, Antigravity CLI, Cursor, and Claude
  Code

This is probably not the right tool if you want:

- a hosted dashboard
- automatic product management
- regulatory certification or external attestation
- a zero-setup chat-only assistant
- compatibility shims for old governance layouts
- agents to freely edit any file without a declared task

## What you get today

Current release: `2.1.0`

License: MIT. The plugin is free to use, copy, modify, and distribute under
the terms in [LICENSE](LICENSE).

- A Rust task-registry CLI installed into consumer repos as
  `.codex/scripts/task-registry`.
- Plan activation, task status, deferral, reports, metrics, behavior checks, and
  source-limit checks.
- Codex, Antigravity CLI, Cursor, and Claude Code hook/templates surfaces.
- Native skills for plan closure and task-registry flow.
- Local mutation gates that allow governance repair paths but block unbound
  implementation writes.
- Fail-closed command inspection for shell redirections and inline write calls
  when the target path is not deterministic.
- Schema-backed diagnostics for release checks, source limits, behavior
  verifiers, installer dry-runs, and mutation denials.
- JSON command reports with `--format json`, and explicit receipt recording for
  read-only commands with `--record-receipt`.
- Schema version 2 receipt chains with locked appends and `verify-chain`
  integrity checks.
- Model attribution for supported Codex mutation hooks: Codex is the first
  measured adapter, and non-Codex mutation surfaces remain unmeasured until
  equivalent adapter evidence exists.
- Strict v2 plan activation: phased checklists, exact file targets, behavior
  `gap_id`, behavior `polarity`, typed verifiers, and required positive plus
  negative behavior coverage before implementation work.
- Terminal task protection: completed and cancelled tasks are immutable; changed
  follow-up work needs a new task id.
- Install modes for real upgrades: `--dry-run`, `--merge`, and `--force`.
- Release checks for version consistency, dependency audit, source limits,
  plugin layout, Nix package assets, native required files, and tracked release
  artifacts.

Future policy work will make the input and output more explicit: a typed
engineering policy as input, a compliance artifact as output, and token/cost
evidence where usage can be measured honestly. Current model attribution says
which measured adapter/model requested supported repo mutation; it is not token
usage evidence. Cost per commit is a product goal, but it requires structured
usage receipts and pricing evidence; hidden or unavailable usage must be
reported as unmeasured, not guessed.

Important limits:

- It is local-first. There is no hosted service or remote sync.
- Receipts are local files; there is no built-in analytics pipeline.
- The workflow is intentionally strict. Legacy hook paths and `--overlay` are
  removed in v2.
- It validates process, policy conformance, and provenance; it does not prove
  product correctness by itself.
- It is not a regulatory certification or external attestation tool.

## Proof boundaries

Engineering-policy compliance proof and product correctness proof are separate.

Policy compliance proof means the repository can show the declared policy or
approved plan, active task targets, typed behavior verifiers, mutation
boundaries, receipt chain, and release-source checks for a change. It answers
"did this repo/build/change comply with the engineering policy this repository
declared?"

Product correctness proof still belongs to the project. Maintainers still need
domain tests, code review, security review where relevant, and product
acceptance evidence. A green governance report is useful review evidence, not a
substitute for deciding whether the product behavior is correct.

See [VISION.md](VISION.md) for the product direction and
[ROADMAP.md](ROADMAP.md) for planned improvements.

## Install

Vendoring or submodule installation is recommended.

```bash
mkdir -p plugins
git submodule add https://github.com/Qu3ltron/RepOGovnor.git plugins/agent-governance

cp plugins/agent-governance/project.config.example.toml project.config.toml
# edit project.config.toml for your repo

plugins/agent-governance/scripts/install-to-workspace.sh \
  --config project.config.toml \
  --merge
```

Then validate the install:

```bash
plugins/agent-governance/scripts/status.sh --strict
.codex/scripts/task-registry source-limit check
.codex/scripts/task-registry validate
```

`AGENTS.md` and `GEMINI.md` must each carry exactly one
`agent-governance:begin` / `agent-governance:end` marker block. Markerless
files are unaligned posture, even when the surrounding prose looks correct.
`status.sh --strict` renders the Rust `status-check` diagnostics for this
policy.

Release waivers are local-only and must carry a reason. Set
`AGENT_GOVERNANCE_FINAL_RELEASE=1` for final release validation; waiver flags
are rejected in that mode.

Version governance is local and explicit. `version-check next <plan_id>` shows
the governed release metadata. `version-check prerelease <plan_id> --rc <n>`
may be used by automation to push the branch and `vX.Y.Z-rc.N` tag. Final
`vX.Y.Z` tagging, final tag push, GitHub Release creation, and public release
publication remain manual.

Fresh installs or intentional rebaselines can use `--force`. Preview the full
write set first:

```bash
plugins/agent-governance/scripts/install-to-workspace.sh \
  --config project.config.toml \
  --dry-run
```

For machine-readable installer output:

```bash
DRY_RUN_FORMAT=json MODE=force DRY_RUN=1 \
  plugins/agent-governance/scripts/render-from-config.sh project.config.toml .
```

## Nix Consumption

The flake exposes `packages.<system>.task-registry-flow`, an app, and two NixOS
modules:

- `nixosModules.agent-governance` installs the CLI and exposes
  `AGENT_GOVERNANCE_ASSET_ROOT`.
- `nixosModules.auto-update` updates the configured flake input, validates it,
  rebuilds, runs an optional health command, and restores the previous lock on
  failure.

The package installs runtime assets under
`share/agent-governance`: `templates/`, `skills/`, `hooks/`, `modules/`,
`MANIFEST.toml`, `REQUIREMENTS.toml`, `project.config.example.toml`, and runtime
docs. Consumers should use this packaged asset root instead of reading from a
mutable checkout.

Validate Nix-facing release changes with:

```bash
nix flake check --no-build --all-systems
```

## Daily workflow

1. Draft a plan in `docs/plans/<name>.md`.
2. Use the installed plan template. New activations must include Approved
   Scope, Phased Required Change Checklist, Per-Gap Success Criteria,
   Validation Plan, Walkthrough Evidence, and a `Task Manifest`.
3. In the manifest, use `schema_version = 2`; every behavior needs `gap_id`,
   `polarity`, and typed `[[behaviors.verifiers]]`. Each implementation gap
   needs positive and negative behavior coverage.
4. Activate it:

```bash
.codex/scripts/task-registry activate docs/plans/<name>.md
```

5. Work only inside the activated task targets.
6. Run focused validation from the plan.
7. Land completion through changed-file verification:

```bash
.codex/scripts/task-registry verify-landing --plan-id PLAN-YYYY-MM-DD-example --changed-files src/example.rs
.codex/scripts/task-registry report PLAN-YYYY-MM-DD-example
.codex/scripts/task-registry reviewer-report
.codex/scripts/task-registry reviewer-report --format markdown
```

In this plugin source repo, release work also runs `version-check validate` and
`backlog-check`. Those checks validate RepOGovnor release governance, not every
consumer install.

Direct completed-status writes are rejected; `verify-landing` owns completion
after it binds changed files to active task targets and runs typed behavior
verifiers.

`reviewer-report` is a pasteable local handoff summary. It shows active plans,
landed tasks, changed targets, receipt state, blocked or deferred work, and the
proof boundary between governance evidence and product correctness evidence.
Use `--format markdown` for manual pull request handoff text. It remains local
stdout only; it does not post to GitHub or send telemetry.

The hook layer is designed to keep this honest. It permits plan bootstrap work,
such as writing or activating plans, and denies implementation or runtime
governance edits that are not tied to an active or planned task target.
Command inspection fails closed for ambiguous shell redirections, compact
redirect syntax, and inline write calls whose target cannot be proven.

Completed and cancelled tasks are terminal. Reactivating an unchanged plan is
idempotent, but changing title, kind, source hash, acceptance proof, behavior
ids, targets, blockers, or projected steps requires a new task id.

Runtime failures are schema-backed. A failed check should name the check id,
path, expected state, actual state, and remediation. See
[docs/runtime-schemas.md](docs/runtime-schemas.md).

Read-only inspection commands do not append receipts by default. Use
`--record-receipt` before the command when a validation receipt is intentional:

```bash
.codex/scripts/task-registry --record-receipt validate
.codex/scripts/task-registry metrics --format json
.codex/scripts/task-registry install plan --format json
.codex/scripts/task-registry status-check --format json
```

Diagnostic JSON failures stay machine-readable and exit nonzero. Global JSON
failures return a command envelope that identifies the parsed command and
whether a receipt was recorded.

## Install modes

`--dry-run` prints what would change and does not write files.

`--merge` is for existing repositories. It refreshes managed governance files,
preserves valid project registry state, and removes stale v1/v0.x paths that
v2 no longer supports.

`--force` is for a fresh install or intentional rebaseline. It applies the
complete managed projection.

`--overlay` is removed. Use `--merge`.

## Requirements

- Git repository
- Bash
- Python 3.11 or newer
- Rust toolchain for the registry CLI
- `agy` 1.0.3 or newer if you use Antigravity

For release verification of this plugin itself:

- `cargo-audit`
- `cargo-deny`

## Files users should know

- `project.config.example.toml` - starting config for a consumer repo
- `REQUIREMENTS.toml` - files a consumer repo must commit after install
- `docs/migration-v2.md` - existing workspace migration guide
- `docs/example-workflow.md` - minimal plan-to-report workflow
- `docs/multi-repo.md` - manual boundary for teams using several repos
- `docs/agent-environment-matrix.md` - supported agent surfaces and checks
- `docs/engineering-policy-compliance.md` - product direction for policy input,
  compliance artifacts, and cost evidence
- `docs/releases/v2.md` - v2 release and migration notes
- `docs/runtime-schemas.md` - structured runtime report and verifier schemas
- `VISION.md` - why this exists and where it is going
- `ROADMAP.md` - planned improvements and known gaps
- `CHANGELOG.md` - release history

## Current release checks

Production readiness is clean-tree and manifest-backed. `REQUIREMENTS.toml`
owns required release-source paths, executable release scripts, stale path
denials, version files, and CI-tracked governance artifacts. Required release
files must be native files, not symlinks.

For this plugin source repo:

```bash
scripts/status.sh --release-source
scripts/release-version-check.sh
scripts/release-audit.sh
bash scripts/test-release-readiness.sh all
nix flake check --no-build --all-systems
```

`scripts/status.sh --release-source` must run from a clean worktree for
production release. Local waiver variables are for development diagnostics only;
`AGENT_GOVERNANCE_FINAL_RELEASE=1` rejects them.

For a consumer repo after install:

```bash
plugins/agent-governance/scripts/status.sh --strict
.codex/scripts/task-registry validate
.codex/scripts/task-registry source-limit check
```

## Privacy

Agent Governance does not emit network telemetry. Task events and validation
receipts are local repo files. If a project publishes those files, that is a
project decision, not plugin behavior.

## License

MIT. See [LICENSE](LICENSE).
