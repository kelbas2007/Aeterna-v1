# AETERNA v1

A clean no-LLM intelligence research line targeting **full EvoPhase cognitive ownership**.

## Current architectural qualification

**Full physical phase-native execution is NOT yet established.** The recorded task results in `STATUS.md` remain evidence, but they must not be confused with proof that the phase-cell/synapse network itself executes every cognitive operation.

The 2026-10-06 [phase-execution audit](docs/PHASE_EXECUTION_AUDIT_RESULT.md) reproduced a specific gap: the current G8 planning path gave identical decisions in 18/18 deterministic matched cases with phase plasticity and structural recruitment disabled and no dormant cells. Its learned transition table remained present; removing that table removed the plan. A standalone version of the same production planner also worked without constructing an EvoPhase instance.

This is a negative architectural witness, not an all-learning ablation and not an invalidation of every earlier result. The strong ownership requirement below remains unchanged. G10 is preregistered; this audit does not implement or qualify it.

## Non-negotiable rule

EvoPhase is the cognitive substrate, not a planner plugin.

Rust may implement substrate physics and the trusted shell (I/O, persistence, serialization, provenance, safety and exact verification), but Rust must not contain hidden task answers or perform task-level cognition on behalf of the organism.

The following adaptive state must be EvoPhase-owned:

- learned sensory distinctions and representations;
- concepts and relations;
- predictive hypotheses and rival models;
- reusable skills / programs and their composition;
- experiment selection;
- imagined rollouts and planning state;
- action policy;
- revision after factual counterexamples;
- structural growth, retirement and reuse.

No LLM is used by the runtime.

## Acceptance chain

A capability is accepted only when the same ordinary organism demonstrates:

```text
raw experience
  -> EvoPhase-owned state change
  -> new prediction / program / experiment / plan
  -> changed physical action
  -> factual external result
  -> revision of the same EvoPhase-owned structure
```

Matched controls must preserve raw observations, primitive substrate, factual outcomes and resource limits while disabling only the mechanism under test.

## Current branch

Development is on **`main`**, following the owner's merge of `genesis/full-evophase`.

Historical capability measurements and failed packs are retained in [STATUS.md](STATUS.md) and [EXPERIMENT_LEDGER.md](EXPERIMENT_LEDGER.md). The [phase-execution audit result](docs/PHASE_EXECUTION_AUDIT_RESULT.md) qualifies their architectural interpretation; it does not erase them.

Next architectural requirement: a learned physical execution path with selective phase-path suppression and restoration, rather than counting a nested graph-search data structure as proof of phase-native imagination. The existing graph planner remains a useful reference implementation.

Status: **research implementation; no AGI claim or production promotion**. CI is manual-only between explicit execution checkpoints.
