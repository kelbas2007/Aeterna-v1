# Aeterna-v1 — EvoPhase Genesis

A fresh no-LLM intelligence research line built around **full EvoPhase cognitive ownership**.

This repository is intentionally *not* a continuation of the old AETERNA implementation architecture. Earlier results, failures, controls and invariants are treated as empirical lessons. EvoPhase is the one mandatory architectural constraint.

## Non-negotiable boundary

EvoPhase is the cognitive substrate, not a planner plugin or execution backend.

Rust implements substrate physics and the trusted shell. It may perform I/O, persistence, serialization, bounded resource accounting, provenance checks, safety gates and exact verification. It may **not** carry hidden task answers or choose representations, hypotheses, programs, experiments, plans or actions on behalf of the organism.

The following adaptive state must be EvoPhase-owned:

- sensory distinctions and learned representations;
- concepts and relational structure;
- predictive hypotheses and rival models;
- reusable skills/programs and their composition;
- experiment selection;
- imagined rollouts and planning state;
- action policy;
- revision after factual counterexamples;
- structural growth, retirement and reuse.

No LLM is used by the runtime.

## Acceptance rule

A capability is accepted only through a causal chain:

```text
experience
  -> EvoPhase state change
  -> new prediction / concept / program / experiment / plan
  -> changed action
  -> factual external result
  -> revision of the same EvoPhase-owned structure
```

Matched controls must preserve the same observations, primitive substrate, resource limits and factual outcomes while removing only the formation/readout being tested.

## Current status

**GENESIS / carrier-kernel implementation. Not AGI.**

The first executable target is deliberately lower than ARC: prove that a phase-coded, locally plastic, structurally growing carrier can own prediction and recruit new internal structure from raw sensorimotor events without a symbolic solver taking over cognition.

See:

- `docs/ARCHITECTURE_RU.md`
- `docs/RESEARCH_2026.md`
- `docs/FIRST_EXPERIMENT.md`
