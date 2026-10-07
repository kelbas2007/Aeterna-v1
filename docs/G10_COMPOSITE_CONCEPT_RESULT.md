# G10 — autonomous composite concept construction

Date: 2026-10-07

## Verdict

**MECHANISM PASS.**

Workflow: `37569651485`  
Scored source: `74b736986870419ea42989c387026a57e498d900`  
Numeric implementation freeze: `docs/G10_COMPOSITE_CONCEPT_IMPLEMENTATION_SPEC.md` committed before implementation.

## What was tested

A raw 12x12 scene contains two spatially separated local pair motifs. Generic local FHRR relation traces recruit lower-level atom units.

The evaluator-only task is XOR-like:

- one pair family maps to one opaque action class;
- the complementary pair family maps to the other;
- every individual acquired atom occurs equally under both classes.

Therefore no child atom is sufficient to predict the motor.

Both opaque motor permutations were tested.

GENUINE may promote a new composite that references two acquired atom carrier IDs.
NO_CONSTRUCTION gets the same raw scenes, actions, factual Need and acquired atom statistics but cannot promote a composite.
NO_READOUT is a post-tuition clone of GENUINE with composite readout suppressed; its child atoms remain available.

## Result

Observed:

- GENUINE: **8/8** held-out pair decisions;
- NO_CONSTRUCTION atom-only: **4/8**;
- NO_READOUT atom-only: **4/8**;
- normal motor permutation: **(4,2,2)** for FULL / NO_CONSTRUCTION / NO_READOUT;
- swapped motor permutation: **(4,2,2)**;
- four lower-level atom units were acquired identically in matched arms;
- four useful composite concepts were promoted only in GENUINE;
- every composite stores acquired child atom IDs, not evaluator relation labels;
- individual child action evidence stayed within the frozen absolute **0.20** ceiling;
- promoted composite winning-action evidence met the frozen **0.60** threshold;
- learning was frozen before held-out readout;
- full optimized regressions PASS;
- Release build PASS.

## Interpretation

This is a bounded causal witness that Aeterna-v1 can construct a new reusable internal predicate from acquired lower-level carrier units when the parts are individually insufficient.

The result is stronger than memorizing one whole raw scene: the promoted state references reusable acquired atom IDs and transfers to absolute motif bindings absent from tuition.

It is still a mechanism result. It does not establish unrestricted concept invention, arbitrary arity, semantic naming, natural language, AGI or consciousness.

The next evidence step is a one-use fresh G10 pack with newly sampled relation atoms, pair assignments, opaque motor mapping and held-out bindings.
