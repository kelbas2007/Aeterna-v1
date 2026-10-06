# G1 — Raw raster autonomous distinction

Status: **IMPLEMENTED CANDIDATE / UNCOMPILED / UNQUALIFIED**

This gate tests one narrow claim: can an EvoPhase-owned carrier form and use a predictive distinction directly from a raw raster, rather than receiving a developer-defined symbolic feature?

## Input boundary

The carrier receives only:

- a 12×12 binary raster flattened to 144 sensory channels;
- four opaque motor tokens;
- the protected factual Need outcome after an action.

It does **not** receive:

- object labels;
- coordinates;
- map structure;
- context IDs;
- task names;
- the correct action;
- a developer-selected feature vector.

The evaluator knows the hidden context only for scoring.

## Acquisition protocol

Two latent evaluator contexts produce different Need consequences for the same four opaque actions.

All arms receive the same balanced factual experience:

- both contexts;
- all four motor tokens in each context;
- varying nuisance pixels;
- factual Need=true/false.

The balanced acquisition stream is intentionally not an exploration test. G1 isolates representation formation. Carrier-owned experiment selection is a later gate.

## EvoPhase formation mechanism

Each sensory cell maintains action-local factual contrast through carrier-owned plastic state:

- outcome when the cell was active;
- outcome when it was inactive;
- support for both cases.

Residual-driven structural recruitment first prefers active sensory cells whose local factual histories distinguish consequences for that action. If insufficient contrast exists, the developmental fallback is phase-role diversity.

This is a generic substrate rule. No evaluator context or pixel coordinate is supplied to the constructor.

## Controls

### NO_FORMATION

Same raw observations, actions, factual outcomes, primitive carrier and resource limits.

Only structural recruitment is disabled.

### NO_READOUT

Clone the fully acquired GENUINE carrier, preserving all recruited structures, then disable only acquired-structure readout during evaluation.

This isolates use from formation.

## Held-out evaluation

Four evaluation rasters use nuisance masks absent from acquisition.

Required:

1. GENUINE has recruited EvoPhase structure.
2. NO_FORMATION has not.
3. NO_READOUT retains the same recruited structure count as GENUINE.
4. GENUINE chooses the correct opaque motor on all held-out cases.
5. Both controls score lower than GENUINE.

The test is implemented in `tests/g1_raw_raster.rs`.

## What a PASS would mean

Only:

> acquired EvoPhase structure extracted a consequence-relevant distinction from raw raster channels and that structure was necessary for held-out action choice under surface nuisance.

It would **not** yet prove:

- object discovery;
- general visual intelligence;
- active experiment selection;
- reusable programs/macros;
- hierarchical reasoning;
- ARC competence;
- AGI.

## Research basis

The design is informed by, but does not inherit guarantees from:

- Stöckl, Yang & Maass (Nature Communications, 2024), local next-observation prediction producing useful cognitive-map geometry for planning: https://www.nature.com/articles/s41467-024-46586-0
- Russo et al. (Nature Communications, 2024), rate/phase assembly coding disambiguating repeated spatial states by context: https://www.nature.com/articles/s41467-024-52988-x
- Harvey et al. (ICML 2026), unsupervised hierarchical skill discovery from unlabelled trajectories: https://proceedings.mlr.press/v306/harvey26a.html
- Pilzak, Pennington & Thivierge (Nature Communications, 2026), stabilization of ongoing plasticity by internally generated top-down signals: https://www.nature.com/articles/s41467-026-70920-3

These sources motivate local prediction, contextual phase coding, reusable structure, and plasticity control. None demonstrates the Aeterna-v1 gate itself.
