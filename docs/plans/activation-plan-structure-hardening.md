# Activation Plan Structure Hardening

## Approved Scope

Close the activation-plan quality gap surfaced by review of
`/home/hasnamuss/Documents/certification-gpt/docs/plans`: this repo should
accept only production-ready, phased, file-specific closure plans after the
v2 hardening work lands.

In scope:

- Add strict v2 behavior metadata for gap id and polarity.
- Enforce required activation-plan Markdown sections.
- Reject placeholders, broad targets, wildcard paths, missing negative
  coverage, and implementation tasks backed only by validation behaviors.
- Refresh templates, installed projections, skills, and user docs to describe
  the stricter activation contract.

Out of scope:

- Rewriting historical archived v1 plans.
- Supporting compatibility shims for weak future plans.
- Changing runtime mutation authorization semantics beyond plan activation
  validation.

Affected surfaces:

- Rust registry CLI schema and activation validation.
- Plan authoring templates and installed skill projections.
- Runtime schema/user documentation.

Primitive change gate: N/A. This changes governance validation and plan
activation policy, not application runtime primitives, services, persistence,
or external providers.

Bootstrap exception: this plan is first activated using the current v2 manifest
shape because `gap_id` and `polarity` do not exist before implementation. The
implementation must then update this same plan manifest with those fields and
reactivate it before final validation.

## Phased Required Change Checklist

### Phase 0: Activation and bootstrap

- [ ] `[NEW]` `docs/plans/activation-plan-structure-hardening.md` - register
  the approved closure contract; acceptance proof:
  `.codex/scripts/task-registry activate docs/plans/activation-plan-structure-hardening.md`.

### Phase 1: Canonical runtime schema

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/schema.rs` - add typed
  behavior polarity values `positive`, `negative`, and `validation`;
  acceptance proof: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/model.rs` - add v2 behavior
  metadata fields for `gap_id` and `polarity`; acceptance proof:
  `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/plan_contract.rs` - validate
  phased Markdown structure, placeholders, gap coverage, and task-to-behavior
  closure semantics; acceptance proof:
  `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/main.rs` - call the
  plan-contract validator during activation and registry validation while
  keeping the file below 1600 lines; acceptance proof:
  `.codex/scripts/task-registry source-limit check`.

### Phase 2: Behavioral test coverage

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests.rs` - add positive
  activation tests for comprehensive phased v2 plans; acceptance proof:
  `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_accepts_comprehensive_phased_v2_contract`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests.rs` - add negative
  activation tests for missing sections, placeholders, missing metadata,
  missing negative behavior, validation-only implementation closure, and
  wildcard targets; acceptance proof:
  `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_rejects_`.

### Phase 3: Authoring surfaces

- [ ] `[MODIFY]` `templates/.codex/templates/task-registry-plan-template.md.template` -
  replace the shallow template with the comprehensive phased activation
  skeleton; acceptance proof: `bash scripts/test-install-modes.sh`.
- [ ] `[MODIFY]` `.codex/templates/task-registry-plan-template.md` - refresh
  the installed plan template from the canonical template; acceptance proof:
  `bash scripts/status.sh --strict`.
- [ ] `[MODIFY]` `skills/gap-closure-contract/SKILL.md` - require phased
  checklists, exact targets, positive and negative typed verifier behaviors,
  and no placeholders; acceptance proof: `bash scripts/status.sh --strict`.
- [ ] `[MODIFY]` `skills/task-registry-flow/SKILL.md` - require strict v2
  activation metadata and behavior polarity discipline; acceptance proof:
  `bash scripts/status.sh --strict`.
- [ ] `[GENERATE]` `.agents/skills/gap-closure-contract/SKILL.md` - refresh
  installed Codex/AGY skill projection; acceptance proof:
  `bash scripts/status.sh --strict`.
- [ ] `[GENERATE]` `.agents/skills/task-registry-flow/SKILL.md` - refresh
  installed Codex/AGY skill projection; acceptance proof:
  `bash scripts/status.sh --strict`.
- [ ] `[GENERATE]` `.cursor/skills/gap-closure-contract/SKILL.md` - refresh
  installed Cursor skill projection; acceptance proof:
  `bash scripts/status.sh --strict`.
- [ ] `[GENERATE]` `.cursor/skills/task-registry-flow/SKILL.md` - refresh
  installed Cursor skill projection; acceptance proof:
  `bash scripts/status.sh --strict`.
- [ ] `[MODIFY]` `templates/AGENTS.md.template` - describe the stricter
  activation contract; acceptance proof: `bash scripts/test-install-modes.sh`.
- [ ] `[MODIFY]` `templates/GEMINI.md.template` - describe the stricter
  activation contract for Antigravity; acceptance proof:
  `bash scripts/test-install-modes.sh`.
- [ ] `[MODIFY]` `templates/.codex/agent-governance.toml.template` - expose
  required manifest fields and plan-contract validation rules; acceptance
  proof: `bash scripts/test-install-modes.sh`.
- [ ] `[GENERATE]` `AGENTS.md` - refresh installed repo instructions;
  acceptance proof: `bash scripts/status.sh --strict`.
- [ ] `[GENERATE]` `GEMINI.md` - refresh installed Antigravity instructions;
  acceptance proof: `bash scripts/status.sh --strict`.
- [ ] `[GENERATE]` `.codex/agent-governance.toml` - refresh installed
  governance metadata; acceptance proof: `bash scripts/status.sh --strict`.

### Phase 4: Documentation and release gates

- [ ] `[MODIFY]` `docs/runtime-schemas.md` - document behavior `gap_id`,
  `polarity`, required sections, and activation rejection rules; acceptance
  proof: `rg -n "gap_id|polarity|Phased Required Change Checklist" docs/runtime-schemas.md`.
- [ ] `[MODIFY]` `README.md` - describe user-facing plan activation quality
  gates; acceptance proof:
  `rg -n "negative behavior|typed verifier|activation" README.md`.
- [ ] `[MODIFY]` `CHANGELOG.md` - record the activation-plan hardening;
  acceptance proof: `rg -n "activation-plan|polarity" CHANGELOG.md`.

### Phase 5: Validation and handoff

- [ ] `[VERIFY]` `rust/task-registry-flow-cli` - format, lint, and unit-test
  the registry CLI; acceptance proof: Rust gates pass.
- [ ] `[VERIFY]` `scripts/test-install-modes.sh` - prove generated projections
  still install and align; acceptance proof: install-mode test passes.
- [ ] `[VERIFY]` `scripts/test-release-readiness.sh` - prove release readiness
  remains green; acceptance proof: release readiness all passes.
- [ ] `[VERIFY]` `docs/task-registry/events.jsonl` - local metrics contain no
  malformed or failed receipt events; acceptance proof:
  `.codex/scripts/task-registry metrics`.

## Per-Gap Success Criteria

### GAP-001: Weak plan structure can still activate

- Behavioral: Given a v2 plan without the required phased Markdown sections,
  when activation runs, then activation fails before registry mutation.
- Data/schema/provenance: Registry rows are created only for plans that include
  the required sections and a normalized hash of the exact approved body.
- Runtime: N/A; CLI tests and activation attempts prove behavior.
- Good behavior: A plan with Approved Scope, Phased Required Change Checklist,
  Per-Gap Success Criteria, Validation Plan, Walkthrough Evidence, and Task
  Manifest activates.
- Forbidden behavior: Missing sections, duplicate `## Task Manifest`, or
  malformed fenced TOML activates.
- Positive test: `activation_accepts_comprehensive_phased_v2_contract`.
- Negative test: `activation_rejects_plan_missing_required_sections`.

### GAP-002: Plans can rely on unresolved placeholders

- Behavioral: Given a v2 plan containing unresolved placeholder tokens, when
  activation runs, then activation fails with a specific placeholder error.
- Data/schema/provenance: Approved plan text contains no unresolved placeholder
  tokens at activation time.
- Runtime: N/A; CLI tests prove behavior.
- Good behavior: Complete plan text with no placeholders activates.
- Forbidden behavior: Any unresolved placeholder activates.
- Positive test: `activation_accepts_comprehensive_phased_v2_contract`.
- Negative test: `activation_rejects_plan_with_tbd_or_placeholders`.

### GAP-003: Gap closure lacks explicit positive and negative behavior mapping

- Behavioral: Given a v2 manifest behavior, when validation runs, then the
  behavior must include a non-empty `gap_id`, a typed `polarity`, and at least
  one typed verifier.
- Data/schema/provenance: Each `gap_id` has at least one `positive` and one
  `negative` behavior in the activated manifest.
- Runtime: N/A; manifest validation tests prove behavior.
- Good behavior: Every implementation gap maps to positive and negative typed
  verifiers.
- Forbidden behavior: A gap with only positive, only negative, only validation,
  or missing metadata activates.
- Positive test: `activation_accepts_comprehensive_phased_v2_contract`.
- Negative tests: `activation_rejects_v2_behavior_missing_gap_id`,
  `activation_rejects_v2_behavior_missing_polarity`, and
  `activation_rejects_gap_without_negative_behavior`.

### GAP-004: Implementation closure can be backed only by broad validation

- Behavioral: Given an implementation, migration, authorization, schema,
  release, or governance task links only validation behavior, when activation
  runs, then activation fails.
- Data/schema/provenance: Closure tasks cite behavior ids tied to actual gap
  polarity, not just full-repo gates.
- Runtime: N/A; manifest validation tests prove behavior.
- Good behavior: Validation behavior is allowed as supplementary proof but not
  as the only implementation closure proof.
- Forbidden behavior: A task that changes runtime/schema/governance surfaces
  links only a `validation` behavior and activates.
- Positive test: `activation_accepts_comprehensive_phased_v2_contract`.
- Negative test: `activation_rejects_implementation_task_with_only_validation_behavior`.

### GAP-005: Targets can remain broad or wildcarded

- Behavioral: Given a task target such as `src/*`, `docs/**`, `tests/*`, or a
  broad object like `backend`, when manifest validation runs, then validation
  fails.
- Data/schema/provenance: Every task target is an exact file/object/change
  tuple.
- Runtime: N/A; manifest validation tests prove behavior.
- Good behavior: Exact file paths and specific object names activate.
- Forbidden behavior: Wildcard paths, broad directories, or broad object names
  activate.
- Positive test: `activation_accepts_comprehensive_phased_v2_contract`.
- Negative test: `activation_rejects_broad_or_wildcard_targets`.

## Validation Plan

Focused:

- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_accepts_comprehensive_phased_v2_contract`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_rejects_`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml verifier_`
- `.codex/scripts/task-registry verify-behaviors PLAN-2026-05-30-activation-plan-structure-hardening`
- `.codex/scripts/task-registry source-limit check`

Full:

- `cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check`
- `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `bash scripts/test-install-modes.sh`
- `bash scripts/test-release-readiness.sh all`
- `bash scripts/status.sh --strict`
- `.codex/scripts/task-registry validate`
- `.codex/scripts/task-registry metrics`
- `git diff --check`

Failure evidence:

- Any weak plan that activates means the plan-structure gap remains open.
- Any missing negative coverage that activates means the behavioral closure gap
  remains open.
- Any source-limit violation means the implementation must be split before
  handoff.

## Walkthrough Evidence

Capture:

- Plan activation output before implementation.
- Plan reactivation output after adding `gap_id` and `polarity`.
- Focused Rust test output for positive and negative activation tests.
- `verify-behaviors` output for this plan.
- Strict status, release readiness, source-limit, metrics, and final
  `TASK_REPORT PLAN-2026-05-30-activation-plan-structure-hardening`.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-05-30-activation-plan-structure-hardening"

[[behaviors]]
behavior_id = "B-2026-05-30-plan-structure-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Comprehensive phased activation plans pass"
given = "A v2 plan with all required phased sections, exact file targets, and typed verifiers"
when = "The registry CLI activates the plan"
then = "Activation succeeds and registry rows are created"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_accepts_comprehensive_phased_v2_contract"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_accepts_comprehensive_phased_v2_contract"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-30-plan-structure-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Weak activation plans fail closed"
given = "A v2 plan missing sections, containing placeholders, lacking negative behavior, or using broad targets"
when = "The registry CLI validates or activates the plan"
then = "Activation fails before registry mutation"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_rejects_"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_rejects_"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-30-placeholder-positive"
gap_id = "GAP-002"
polarity = "positive"
title = "Complete activation plans have no placeholders"
given = "A v2 plan with complete file names, test commands, and acceptance proof"
when = "The registry CLI activates the plan"
then = "Activation succeeds without unresolved placeholder tokens"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_accepts_comprehensive_phased_v2_contract"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_accepts_comprehensive_phased_v2_contract"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-30-placeholder-negative"
gap_id = "GAP-002"
polarity = "negative"
title = "Placeholder activation plans fail"
given = "A v2 plan containing unresolved placeholder tokens"
when = "The registry CLI activates the plan"
then = "Activation fails before registry mutation"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_rejects_plan_with_tbd_or_placeholders"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_rejects_plan_with_tbd_or_placeholders"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-30-behavior-metadata-positive"
gap_id = "GAP-003"
polarity = "positive"
title = "Gap behavior metadata is complete"
given = "A v2 manifest with gap ids, behavior polarity, and typed verifiers"
when = "The registry CLI activates the plan"
then = "Every gap has positive and negative executable behavior proof"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_accepts_comprehensive_phased_v2_contract"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_accepts_comprehensive_phased_v2_contract"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-30-behavior-metadata-negative"
gap_id = "GAP-003"
polarity = "negative"
title = "Missing gap behavior metadata fails"
given = "A v2 manifest missing gap id, polarity, or negative behavior coverage"
when = "The registry CLI validates the manifest"
then = "Activation fails before registry mutation"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_rejects_v2_behavior_missing_gap_id && cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_rejects_v2_behavior_missing_polarity && cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_rejects_gap_without_negative_behavior"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_rejects_v2_behavior_missing_gap_id && cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_rejects_v2_behavior_missing_polarity && cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_rejects_gap_without_negative_behavior"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-30-validation-only-positive"
gap_id = "GAP-004"
polarity = "positive"
title = "Closure tasks cite gap behavior proof"
given = "A schema or implementation task links positive and negative gap behavior proof"
when = "The registry CLI activates the plan"
then = "Activation succeeds because implementation closure is not backed only by validation"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_accepts_comprehensive_phased_v2_contract"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_accepts_comprehensive_phased_v2_contract"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-30-validation-only-negative"
gap_id = "GAP-004"
polarity = "negative"
title = "Validation-only implementation closure fails"
given = "An implementation task links only a full-validation behavior"
when = "The registry CLI activates the plan"
then = "Activation fails before registry mutation"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_rejects_implementation_task_with_only_validation_behavior"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_rejects_implementation_task_with_only_validation_behavior"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-30-target-positive"
gap_id = "GAP-005"
polarity = "positive"
title = "Exact task targets activate"
given = "A v2 manifest with exact file, object, and required change targets"
when = "The registry CLI activates the plan"
then = "Activation succeeds with diffable target provenance"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_accepts_comprehensive_phased_v2_contract"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_accepts_comprehensive_phased_v2_contract"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-30-target-negative"
gap_id = "GAP-005"
polarity = "negative"
title = "Broad or wildcard task targets fail"
given = "A v2 manifest with wildcard paths or broad task target objects"
when = "The registry CLI validates the manifest"
then = "Activation fails before registry mutation"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_rejects_broad_or_wildcard_targets"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml activation_rejects_broad_or_wildcard_targets"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-30-template-projection"
gap_id = "GAP-006"
polarity = "validation"
title = "Installed authoring surfaces use strict v2 contract"
given = "The installer renders templates and skill projections"
when = "Install mode and strict status gates run"
then = "Generated authoring surfaces align with canonical strict v2 plan structure"
confirmation = "bash scripts/test-install-modes.sh && bash scripts/status.sh --strict"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-install-modes.sh && bash scripts/status.sh --strict"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-30-full-validation"
gap_id = "GAP-VALIDATION"
polarity = "validation"
title = "Full activation-plan hardening validation passes"
given = "The strict activation-plan implementation is complete"
when = "Full repository gates run"
then = "Rust, installer, release, status, source-limit, and metrics gates pass"
confirmation = "cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check && cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings && cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml && bash scripts/test-install-modes.sh && bash scripts/test-release-readiness.sh all && bash scripts/status.sh --strict && .codex/scripts/task-registry source-limit check && .codex/scripts/task-registry metrics"

[[behaviors.verifiers]]
type = "command"
command = "cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check && cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings && cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml && bash scripts/test-install-modes.sh && bash scripts/test-release-readiness.sh all && bash scripts/status.sh --strict && .codex/scripts/task-registry source-limit check && .codex/scripts/task-registry metrics"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-05-30-activation-plan-001"
title = "Enforce strict v2 plan contract"
status = "planned"
kind = "schema"
reason = "Production closures need schema-backed plan quality gates before activation"
acceptance_proof = "Behaviors B-2026-05-30-plan-structure-positive and B-2026-05-30-plan-structure-negative pass"
behavior_ids = [
  "B-2026-05-30-plan-structure-positive",
  "B-2026-05-30-plan-structure-negative",
  "B-2026-05-30-placeholder-positive",
  "B-2026-05-30-placeholder-negative",
  "B-2026-05-30-behavior-metadata-positive",
  "B-2026-05-30-behavior-metadata-negative",
  "B-2026-05-30-validation-only-positive",
  "B-2026-05-30-validation-only-negative",
  "B-2026-05-30-target-positive",
  "B-2026-05-30-target-negative",
]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/schema.rs"
object = "behavior_polarity_schema"
required_change = "Add typed behavior polarity vocabulary."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/model.rs"
object = "behavior_metadata"
required_change = "Add v2 gap_id and polarity behavior metadata fields."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/plan_contract.rs"
object = "activation_plan_contract_validator"
required_change = "Validate required phased sections, placeholders, gap coverage, and task behavior closure semantics."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/main.rs"
object = "activation_validation_flow"
required_change = "Call plan-contract validation during activation and registry validation."

[[tasks]]
task_id = "TASK-2026-05-30-activation-plan-002"
title = "Add positive and negative activation tests"
status = "planned"
kind = "test"
reason = "Activation hardening must prove good plans pass and weak plans fail"
acceptance_proof = "Behaviors B-2026-05-30-plan-structure-positive and B-2026-05-30-plan-structure-negative pass"
behavior_ids = [
  "B-2026-05-30-plan-structure-positive",
  "B-2026-05-30-plan-structure-negative",
  "B-2026-05-30-placeholder-positive",
  "B-2026-05-30-placeholder-negative",
  "B-2026-05-30-behavior-metadata-positive",
  "B-2026-05-30-behavior-metadata-negative",
  "B-2026-05-30-validation-only-positive",
  "B-2026-05-30-validation-only-negative",
  "B-2026-05-30-target-positive",
  "B-2026-05-30-target-negative",
]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests.rs"
object = "activation_plan_contract_tests"
required_change = "Add positive and negative tests for strict plan activation behavior."

[[tasks]]
task_id = "TASK-2026-05-30-activation-plan-003"
title = "Refresh plan authoring surfaces"
status = "planned"
kind = "documentation"
reason = "Design-time plan authors need an opinionated activation-ready template and matching skills"
acceptance_proof = "Behavior B-2026-05-30-template-projection passes"
behavior_ids = ["B-2026-05-30-template-projection"]

[[tasks.targets]]
file = "templates/.codex/templates/task-registry-plan-template.md.template"
object = "strict_plan_template"
required_change = "Provide a comprehensive phased activation-ready plan skeleton."

[[tasks.targets]]
file = ".codex/templates/task-registry-plan-template.md"
object = "installed_strict_plan_template"
required_change = "Refresh installed plan template from the canonical template."

[[tasks.targets]]
file = "skills/gap-closure-contract/SKILL.md"
object = "gap_closure_contract_authoring_rules"
required_change = "Require phased checklist, exact targets, and positive/negative typed verifier coverage."

[[tasks.targets]]
file = "skills/task-registry-flow/SKILL.md"
object = "task_registry_activation_rules"
required_change = "Require strict v2 activation metadata and behavior polarity discipline."

[[tasks.targets]]
file = ".agents/skills/gap-closure-contract/SKILL.md"
object = "installed_gap_closure_contract_skill"
required_change = "Refresh installed skill projection from canonical skill."

[[tasks.targets]]
file = ".agents/skills/task-registry-flow/SKILL.md"
object = "installed_task_registry_flow_skill"
required_change = "Refresh installed skill projection from canonical skill."

[[tasks.targets]]
file = ".cursor/skills/gap-closure-contract/SKILL.md"
object = "installed_cursor_gap_closure_contract_skill"
required_change = "Refresh installed Cursor skill projection from canonical skill."

[[tasks.targets]]
file = ".cursor/skills/task-registry-flow/SKILL.md"
object = "installed_cursor_task_registry_flow_skill"
required_change = "Refresh installed Cursor skill projection from canonical skill."

[[tasks.targets]]
file = "templates/AGENTS.md.template"
object = "codex_instruction_template"
required_change = "Describe strict v2 activation contract."

[[tasks.targets]]
file = "templates/GEMINI.md.template"
object = "antigravity_instruction_template"
required_change = "Describe strict v2 activation contract."

[[tasks.targets]]
file = "templates/.codex/agent-governance.toml.template"
object = "governance_metadata_template"
required_change = "Expose strict activation-plan manifest requirements."

[[tasks.targets]]
file = "AGENTS.md"
object = "installed_codex_instructions"
required_change = "Refresh installed repo instructions from template."

[[tasks.targets]]
file = "GEMINI.md"
object = "installed_antigravity_instructions"
required_change = "Refresh installed Antigravity instructions from template."

[[tasks.targets]]
file = ".codex/agent-governance.toml"
object = "installed_governance_metadata"
required_change = "Refresh installed governance metadata from template."

[[tasks]]
task_id = "TASK-2026-05-30-activation-plan-004"
title = "Document and validate activation-plan hardening"
status = "planned"
kind = "validation"
reason = "Users and release gates need proof that strict plan activation works"
acceptance_proof = "Behavior B-2026-05-30-full-validation passes"
behavior_ids = ["B-2026-05-30-full-validation"]

[[tasks.targets]]
file = "docs/runtime-schemas.md"
object = "plan_manifest_schema_docs"
required_change = "Document gap_id, polarity, required sections, and activation rejection rules."

[[tasks.targets]]
file = "README.md"
object = "activation_quality_gate_docs"
required_change = "Document user-facing plan activation quality gates."

[[tasks.targets]]
file = "CHANGELOG.md"
object = "unreleased_activation_plan_hardening_entry"
required_change = "Record activation-plan structure hardening."

[[tasks.targets]]
file = "docs/task-registry/events.jsonl"
object = "local_metrics_receipts"
required_change = "Keep local receipt metrics schema-valid with zero malformed events."
```
