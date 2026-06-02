# Cost-Ingest Interface Exposure Gap Closure Contract

## Approved Scope

Close the public-consumer discoverability gap for canonical token spend
ingestion. The canonical interface remains `cost-ingest codex-transcript` with
explicit transcript, session, line range, pricing, and target evidence. This
scope does not add `--latest`, `--commit`, automatic transcript discovery, or a
Rust library API.

## Phased Required Change Checklist

### Phase 0: Activation and safety

- [ ] `[NEW]` `docs/plans/cost-ingest-interface-exposure-2026-06-02.md` - `closure_contract`: create this approved closure contract with schema v2 behaviors and task targets.
- [ ] `[VERIFY]` `docs/plans/cost-ingest-interface-exposure-2026-06-02.md` - `PLAN_ACTIVATE`: activate this plan before implementation.

### Phase 1: CLI discoverability

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/cost_ingest.rs` - `help_surface`: add command-specific help for `cost-ingest --help` and `cost-ingest codex-transcript --help` without changing canonical arguments.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs` - `cost_ingest_help_tests`: prove help exposes required flags and does not advertise legacy selectors.
- [ ] `[MODIFY]` `plugins/agent-governance/rust/task-registry-flow-cli/src/cost_ingest.rs` - `help_surface`: mirror command-specific help for the repo-local wrapper runtime.
- [ ] `[MODIFY]` `plugins/agent-governance/rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs` - `cost_ingest_help_tests`: mirror focused help tests for the wrapper runtime.

### Phase 2: Public and agent docs

- [ ] `[MODIFY]` `README.md` - `cost_ingest_public_usage`: document installed binary, repo-local wrapper, packaged pricing snapshot, target kinds, and fail-closed selector posture.
- [ ] `[MODIFY]` `docs/runtime-schemas.md` - `cost_ingest_runtime_schema`: document command help, installed/package usage, pricing snapshot path, and canonical target kinds.
- [ ] `[MODIFY]` `docs/provider-usage-adapter-contract.md` - `consumer_contract`: state that measured adapter ingestion must expose discoverable help and package-visible pricing evidence.
- [ ] `[MODIFY]` `skills/task-registry-flow/SKILL.md` - `cost_commands`: add cost-ingest, cost-evidence-check, and cost-report to the canonical task-registry skill.
- [ ] `[MODIFY]` `.agents/skills/task-registry-flow/SKILL.md` - `cost_commands`: mirror canonical task-registry skill guidance.
- [ ] `[MODIFY]` `.agents/skills/task-registry-flow.md` - `cost_commands`: mirror canonical task-registry skill guidance.
- [ ] `[MODIFY]` `.cursor/skills/task-registry-flow/SKILL.md` - `cost_commands`: mirror canonical task-registry skill guidance.
- [ ] `[MODIFY]` `.claude/skills/task-registry-flow/SKILL.md` - `cost_commands`: mirror canonical task-registry skill guidance.
- [ ] `[MODIFY]` `plugins/agent-governance/skills/task-registry-flow/SKILL.md` - `cost_commands`: mirror canonical task-registry skill guidance in plugin source.

### Phase 3: Package/readiness proof

- [ ] `[MODIFY]` `scripts/test-release-readiness.sh` - `nix_asset_cost_ingest_checks`: prove packaged binary help, provider contract, runtime docs, and pricing snapshot are installed and discoverable.

## Per-Gap Success Criteria

### GAP-001: Canonical cost-ingest is not discoverable enough

- Current failure: The command is implemented, but public consumers must infer
  the exact interface from source or docs fragments; package readiness does not
  prove command help and installed cost assets.
- Good behavior: Given a built or repo-local CLI, when a user runs
  `cost-ingest --help` or `cost-ingest codex-transcript --help`, then the output
  shows the canonical command and required evidence flags.
- Forbidden behavior: Help must not advertise `--latest`, `--commit`, or any
  automatic transcript selector as supported.
- Files involved: `rust/task-registry-flow-cli/src/cost_ingest.rs`,
  `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs`,
  `plugins/agent-governance/rust/task-registry-flow-cli/src/cost_ingest.rs`,
  `plugins/agent-governance/rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs`,
  `scripts/test-release-readiness.sh`, `README.md`, `docs/runtime-schemas.md`,
  `docs/provider-usage-adapter-contract.md`, `skills/task-registry-flow/SKILL.md`,
  `.agents/skills/task-registry-flow/SKILL.md`,
  `.agents/skills/task-registry-flow.md`,
  `.cursor/skills/task-registry-flow/SKILL.md`,
  `.claude/skills/task-registry-flow/SKILL.md`,
  `plugins/agent-governance/skills/task-registry-flow/SKILL.md`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest`
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_rejects`
- Data/schema/provenance: No schema change; the existing canonical cost evidence
  receipt remains authoritative.
- Runtime: Repo-local wrapper and installed binary both expose the same
  canonical command shape.

### GAP-002: Package readiness does not prove cost-ingest assets

- Current failure: Nix package checks prove general docs/assets, but not the
  provider adapter contract, pricing snapshot, or cost-ingest docs/help.
- Good behavior: Given `nix build .#task-registry-flow`, when release readiness
  inspects the output, then it finds cost docs, provider contract, pricing
  snapshot, and usable command-specific help.
- Forbidden behavior: Packaged docs or binary help must not rely on local
  `/home/hasnamuss` paths.
- Files involved: `scripts/test-release-readiness.sh`, `package.nix`,
  `README.md`, `docs/runtime-schemas.md`,
  `docs/provider-usage-adapter-contract.md`.
- Positive test: `bash scripts/test-release-readiness.sh all`
- Negative test: The same readiness script must fail if the package omits the
  pricing snapshot, provider contract, runtime schema, or command help.
- Data/schema/provenance: Packaged pricing snapshot remains
  `docs/pricing/openai-codex-rate-card-2026-06-02.toml`.
- Runtime: Installed `task-registry-flow` can print cost-ingest help without
  needing a mutable checkout.

## Validation Plan

Focused:

- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_rejects`
- `.codex/scripts/task-registry source-limit check`

Full:

- `.codex/scripts/task-registry validate`
- `.codex/scripts/task-registry verify-chain --format json`
- `.codex/scripts/task-registry release-check all --format json`
- `bash scripts/test-release-readiness.sh all`

## Source File Limit

Expected additions are small and keep all changed files below 1600 lines. Run
`.codex/scripts/task-registry source-limit check` before completion.

## Walkthrough Evidence

- `cost-ingest --help` output includes canonical command and required flags.
- Focused positive and negative Rust tests pass.
- Release readiness proves packaged cost-ingest docs/assets/help.
- `TASK_REPORT` and `TASK_METRICS` after `verify-landing`.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-06-02-cost-ingest-interface-exposure"

[[behaviors]]
behavior_id = "B-001-cost-ingest-help-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Cost-ingest help exposes canonical interface"
given = "The task-registry-flow CLI is available"
when = "cost-ingest help is requested"
then = "The help output shows the codex-transcript command and required evidence flags"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-cost-ingest-help-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Cost-ingest help rejects legacy selectors"
given = "The task-registry-flow CLI is available"
when = "legacy selectors or help text are inspected"
then = "Unsupported --latest and --commit remain rejected and are not advertised as supported"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_rejects"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_rejects"
expected_exit = 0

[[behaviors]]
behavior_id = "B-003-package-cost-assets-positive"
gap_id = "GAP-002"
polarity = "positive"
title = "Release package exposes cost-ingest assets"
given = "The Nix package is built during release readiness"
when = "release readiness inspects package outputs"
then = "The binary help, provider contract, runtime schema, and pricing snapshot are present"
confirmation = "bash scripts/test-release-readiness.sh all"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-release-readiness.sh all"
expected_exit = 0

[[behaviors]]
behavior_id = "B-004-package-cost-assets-negative"
gap_id = "GAP-002"
polarity = "negative"
title = "Release package fails closed without cost-ingest assets"
given = "Release readiness checks package outputs"
when = "cost-ingest docs, help, provider contract, or pricing snapshot are absent"
then = "The readiness script fails instead of passing an incomplete public package"
confirmation = "bash scripts/test-release-readiness.sh all"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-release-readiness.sh all"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-06-02-cost-ingest-interface-exposure-001"
status = "active"
title = "Expose command-specific cost-ingest help"
kind = "implementation"
reason = "Public users need a discoverable canonical ingestion interface without source inspection."
acceptance_proof = "Behaviors B-001 and B-002 pass."
behavior_ids = ["B-001-cost-ingest-help-positive", "B-002-cost-ingest-help-negative"]

[[tasks.targets]]
file = "docs/plans/cost-ingest-interface-exposure-2026-06-02.md"
object = "closure_contract"
required_change = "Create and maintain the activated closure contract for this scope."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cost_ingest.rs"
object = "help_surface"
required_change = "Add command-specific help for cost-ingest and codex-transcript."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs"
object = "cost_ingest_help_tests"
required_change = "Add positive and negative help discoverability tests."

[[tasks.targets]]
file = "plugins/agent-governance/rust/task-registry-flow-cli/src/cost_ingest.rs"
object = "help_surface"
required_change = "Mirror command-specific help for the repo-local wrapper runtime."

[[tasks.targets]]
file = "plugins/agent-governance/rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs"
object = "cost_ingest_help_tests"
required_change = "Mirror positive and negative help discoverability tests for the wrapper runtime."

[[tasks]]
task_id = "TASK-2026-06-02-cost-ingest-interface-exposure-002"
status = "active"
title = "Document public and agent cost-ingest usage"
kind = "documentation"
reason = "Humans and agents need the installed and repo-local command shape documented."
acceptance_proof = "Behaviors B-001 and B-002 pass and source-limit remains valid."
behavior_ids = ["B-001-cost-ingest-help-positive", "B-002-cost-ingest-help-negative"]

[[tasks.targets]]
file = "README.md"
object = "cost_ingest_public_usage"
required_change = "Document installed binary, wrapper, target kinds, pricing snapshot, and fail-closed selectors."

[[tasks.targets]]
file = "docs/runtime-schemas.md"
object = "cost_ingest_runtime_schema"
required_change = "Document command help, installed/package usage, target kinds, and pricing snapshot path."

[[tasks.targets]]
file = "docs/provider-usage-adapter-contract.md"
object = "consumer_contract"
required_change = "Add discoverable help and package-visible pricing evidence requirements."

[[tasks.targets]]
file = "skills/task-registry-flow/SKILL.md"
object = "cost_commands"
required_change = "Add canonical cost command guidance."

[[tasks.targets]]
file = ".agents/skills/task-registry-flow/SKILL.md"
object = "cost_commands"
required_change = "Mirror canonical cost command guidance."

[[tasks.targets]]
file = ".agents/skills/task-registry-flow.md"
object = "cost_commands"
required_change = "Mirror canonical cost command guidance."

[[tasks.targets]]
file = ".cursor/skills/task-registry-flow/SKILL.md"
object = "cost_commands"
required_change = "Mirror canonical cost command guidance."

[[tasks.targets]]
file = ".claude/skills/task-registry-flow/SKILL.md"
object = "cost_commands"
required_change = "Mirror canonical cost command guidance."

[[tasks.targets]]
file = "plugins/agent-governance/skills/task-registry-flow/SKILL.md"
object = "cost_commands"
required_change = "Mirror canonical cost command guidance in plugin source."

[[tasks]]
task_id = "TASK-2026-06-02-cost-ingest-interface-exposure-003"
status = "active"
title = "Prove packaged cost-ingest interface and assets"
kind = "release"
reason = "Public release readiness must fail if cost-ingest is not installable and discoverable."
acceptance_proof = "Behaviors B-003 and B-004 pass."
behavior_ids = ["B-003-package-cost-assets-positive", "B-004-package-cost-assets-negative"]

[[tasks.targets]]
file = "scripts/test-release-readiness.sh"
object = "nix_asset_cost_ingest_checks"
required_change = "Assert packaged binary help, provider contract, runtime schema, and pricing snapshot."
```
