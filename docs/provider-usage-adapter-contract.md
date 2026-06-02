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
- Pricing source, version, service tier, currency, rates, and snapshot digest.
- A target kind and target id for commit, plan, task, verifier run, landing
  attempt, retry, release cycle, or session attribution.

Unsupported providers are explicit `unmeasured` evidence. The runtime must not
infer pricing from model names, elapsed time, file count, commit size, or agent
narration. DeepSeek, Gemma, and other non-Codex models remain unmeasured until
they have an adapter that satisfies this contract and a governed pricing
snapshot.

Cost reports must preserve measured and unmeasured states separately. Missing
evidence is not zero cost; it is unknown cost with a recorded reason.
