# INTEL-2R1 — corrected frozen unified-cognition system verdict

Status: **PRE-REGISTERED BEFORE R1 AUTHORITY RUN**
Date: 2026-10-08
Branch: `intel2-r1-frozen`
Frozen cognitive source: `07b44fb8e00837568bc9655e760eb031d1944dff`

## Preserved history

Original INTEL-2 run `37687243350` exposed and burned its pack.

Post-verdict exact-core diagnosis established that World C reached its raw goal at:

```text
k=7 state=16 target=16 trials=2
```

The evaluator helper then mapped `StepOutcome::GoalReached` to
`Err(NoSupportedAction)` because it attempted another cognitive action while
already sitting in the terminal goal state.

The original pack remains burned and is classified:

**INVALID_AFTER_SEAL_EVALUATOR_TERMINAL_LIFECYCLE**

It is neither PASS nor evidence of a cognitive W3 failure.

## R1 cognitive freeze

No cognitive change is allowed.

Frozen paths must remain byte-identical to:
`07b44fb8e00837568bc9655e760eb031d1944dff`

- `src/**`
- `Cargo.toml`
- `Cargo.lock` if tracked

R1 may change only evaluator/docs/workflow.

## Sole prospective evaluator correction

World C is episodic.

A trial remains:

```text
reset -> predecessor -> aliased junction -> terminal goal/dead
```

After the junction decision reaches either terminal:

- the evaluator starts the next episode by supplying a fresh reset observation;
- no host-selected motor is fabricated;
- no terminal->reset transition is learned;
- the organism is NOT reset;
- U1/U2 state, model, representations and lifetime memory persist;
- the raw goal remains the same.

This removes only the invalid request for a cognitive action while already at
`GoalReached`.

All World C history laws, opaque actions, context promotion criteria and held-out
thresholds remain unchanged.

## Authority

First valid R1 run only:

- authority seed = `github.run_id`;
- run attempt = 1;
- authority seed MUST differ from `37687243350`;
- all target state roles, motors, D operator/bins and E law are sealed before interaction;
- compile/workflow failure before `INTEL2_R1_SEAL` consumes no pack;
- after `INTEL2_R1_SEAL`, pack is permanently burned.

## Pre-target gate

Before seal, exact frozen cognitive core must pass:

1. frozen-source comparison;
2. unified runtime assembly;
3. U3;
4. U2;
5. U1;
6. Human Protection;
7. Release build;
8. R1 evaluator source guard.

## Runtime / lifetime

Unchanged from INTEL-2:

- one organism for A-E;
- `ScientificRuntime::step_unified` only;
- no per-world module switching;
- no world/task IDs in cognition;
- no route/search answer tables;
- no correct-action hints;
- no change flags;
- target U2 ecology starts empty;
- context/perceptual/compositional refinement enabled once before target lifetime;
- protected cognitive restart after C;
- Human Protection intervention before D.

## Frozen target criteria

All original thresholds remain unchanged.

### A
- unknown branching transport <=48 actions;
- translated frozen reuse <=6.

### B
- first causal tool/lock goal <=64;
- switched goal <=12 additional actions.

### C
- target-lifetime context promotion;
- held-out >=28/32;
- each predecessor side >=13/16;
- memoryless comparator <=20/32.

### D
- target-lifetime composition promotion;
- FULL >=28/32;
- single-feature comparator <=24/32 for AND or <=20/32 for XOR.

### E
- learn original law;
- after >=4 successful uses, changed successor with no flag;
- factual contradiction/revision;
- alternative recovery <=16 actions after changed consequence;
- unchanged branch retained.

### Final retention
- A revisit <=6;
- B first-goal revisit <=8.

### System invariants
- Human Protection PASS;
- U1 meta weights unchanged;
- no legacy fixed-priority runtime;
- no legacy graph/table fallback;
- frozen cognitive source unchanged;
- Release build PASS.

## Verdict

Any missed frozen criterion after `INTEL2_R1_SEAL` -> **INTEL-2R1 FAIL**.

If all pass, allowed bounded claim:

**PASS_AUTONOMOUS_DEVELOPING_INTELLIGENCE_UNIFIED — in the tested deterministic world family.**

This is not human-level AGI, unrestricted open-world intelligence, consciousness,
language competence or real-world safety qualification.
