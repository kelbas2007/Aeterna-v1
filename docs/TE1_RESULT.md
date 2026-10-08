# TE1 — physical temporal evidence pilot, sealed scoring status

Date: 2026-10-08
Research branch: `research/beyond-intel4`
Protocol: `docs/TE1_TEMPORAL_EVIDENCE_PROTOCOL.md`

## Verdict separation

1. **MECHANISM PASS** — structural birth of physical cue->hub synapses, persistent accumulation across multiple raw cue observations, physical necessity (synapse lesion and π-phase shift), exact restore and checkpoint, and episode reset were verified. Source model: `src/phase_temporal_evidence.rs`. This is an opt-in physical substrate, **not yet a unified runtime active-sensing policy**.
2. **PREDECLARED NOISY SCORE FAIL** — independently generated balanced 80 episodes missed both the required ≥70 correct and ≥10 margin vs last cue. The thresholds are unchanged after seeing the result.

Exact test workflow: [37834819483](https://github.com/kelbas2007/Aeterna-v1/actions/runs/37834819483). Initial workflow `37834678394` failed **before any evaluation** due a visibility mistake in a private test PRNG helper; fixed without changing the random algorithm, seed, raw target scoring, cognitive code or thresholds. Corrected run's scored output:

```
TE1_NOISY full=63/80 last_only=56/80 majority=63/80 abstain=11/80 zero_accumulator=0
```

The new phase-synapse accumulation returned the same correct score as an 8-reading majority with abstention on 4:4 ties: **63/80**. It improved upon a last-only policy by **7/80** but never claimed better-than-majority reasoning. All 11 abstentions were tied evidence after 8 observations; no extra sample action was requested because TE1 intentionally tests memory without an action policy.

## Causal meaning

Evidence is represented by actual phase-sensitive synapses in EvoPhase's shared arrays, not a host answer-table map. A necessary lesion or π-phase offset changes the physical winner; exact restore reverses it without new training. Checkpoint preserves the carrier state. Reset zeros transient evidence but preserves learned source addresses.

This is a bounded two-source phase-native observation accumulator with inherited hard-coded comparison/margin rules. It is not yet a learned generative sensor likelihood model, Bayesian posterior, multi-source open-world inference, or an end-to-end active-sensing intelligent agent.

## Next precise problem

A future TE2 mechanism must learn the **observation-producing affordance** of an opaque action from factual PRE/action/POST alone and grant it competitive information value when the carrier's evidence margin is inadequate. The project should select another observation when necessary, yet stop and commit when evidence supports a decision. Controls must distinguish genuine sensory sampling from no-op motors and terminal motors. Do not use the burned FRONTIER-1 seed/world as training or scoring.

Older frozen INTEL-4 remains PASS in the bounded deterministic five-world family; FRONTIER-1 remains the earlier independent stochastic FAIL.
