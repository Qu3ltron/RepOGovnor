# Provider Usage Adapter Contract

Provider token spend is measured only when the adapter can replay the evidence
locally. Codex is the first measured adapter because local transcripts expose
session identity, model context, turn ids, and `token_count` usage events.

A measured adapter must provide:

- Stable provider and model identity.
- Stable session or run identity.
- Bounded usage source selection.
- Token usage semantics for input, cached input, output, and reasoning tokens
  when available.
- A source digest for the selected token-event slice.
- Pricing source, version, service tier, currency, rates, reasoning-token policy
  when reasoning tokens are present, and snapshot digest.
- A target kind and target id for commit, plan, task, verifier run, landing
  attempt, retry, release cycle, or session attribution.
- Mutation boundary ids when the evidence is intended to cover repo mutation:
  session id, turn id, and tool-use id where the adapter exposes them.
- Tool-bound mutation coverage requires measured or explicitly unmeasured cost
  evidence to carry the same exposed tool-use id; session/turn/model alone is
  not sufficient when the mutation receipt has a tool id.
- A discoverable CLI help surface that names the required evidence fields.
- Package-visible pricing evidence so installed consumers are not dependent on
  a mutable source checkout.

Unsupported providers are explicit `unmeasured` evidence. The runtime must not
infer pricing from model names, elapsed time, file count, commit size, or agent
narration. DeepSeek, Gemma, and other non-Codex models remain unmeasured until
they have an adapter that satisfies this contract and a governed pricing
snapshot.

Cost reports must preserve measured and unmeasured states separately. Missing
evidence is not zero cost; it is unknown cost with a recorded reason.

Public release artifacts must not publish private provider transcript paths.
When local measured evidence cannot be published, record `cost-record
unmeasured` with the governed target, provider/model if known, boundary ids if
known, and a clear reason. `cost-coverage-check` is the governance gate that
connects mutation attribution to measured or explicitly unmeasured cost
evidence. This is the public-safe path for unknown spend. Codex is the first
adapter for this path, not the contract boundary.
