# AETERNA v1

A clean no-LLM intelligence research line with **full EvoPhase cognitive ownership**.

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

Development starts on `genesis/full-evophase`.

First target: **EVO-OWNERSHIP-0** — prove that residual-driven structural growth inside EvoPhase changes held-out prediction and action, and that a factual counterexample revises the same carrier-owned structure.

Status: **GENESIS / research implementation, not AGI claim**.
