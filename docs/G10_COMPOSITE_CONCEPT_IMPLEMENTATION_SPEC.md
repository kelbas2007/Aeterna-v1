# G10 — COMPOSITE CONCEPT IMPLEMENTATION FREEZE

Status: **FROZEN BEFORE G10 IMPLEMENTATION**

Date: 2026-10-07

This file fixes the numeric substrate choices left symbolic in `docs/G10_COMPOSITE_CONCEPT_PROTOCOL.md`.

## Generic atom substrate

- input: raw 12x12 raster;
- local pair radius: Chebyshev distance <= 2;
- generic retinotopic FHRR position roles;
- atom trace: relative position role `to.unbind(from)`;
- atom match threshold: **0.97**;
- maximum atom units: **64**.

No evaluator relation-family label enters the atom learner.

## Factual evidence

For each factual scene/action/Need observation:

1. extract all local raw-pixel pair relations;
2. match/recruit atom units;
3. update action-conditioned factual evidence for each active atom;
4. enumerate all unordered pairs of distinct active atom IDs;
5. update action-conditioned factual evidence for each active atom pair.

Signed evidence for an action is:

`(success - failure) / (success + failure)`

after at least **4** factual observations for that item/action. Before that it is 0.

## Composite promotion

A pair candidate may promote only when:

- total pair support >= **8** factual observations;
- strongest absolute pair action evidence >= **0.60**;
- for the promoted action, absolute evidence of **each child atom <= 0.20**;
- composite construction is enabled.

The composite stores:
- a new stable composite carrier ID;
- the two acquired child atom IDs;
- copied accumulated factual action evidence at promotion;
- continuing support / utility / revision state.

No raw coordinates or evaluator labels are stored in the composite.

## Readout

- composite readout is disabled during tuition;
- after tuition learning is frozen;
- when enabled, the active composite with the strongest positive action evidence supplies motor evidence;
- ties are resolved only by stable opaque action index;
- if no composite matches, no composite action is returned.

## Development tuition

Use all four evaluator-only pair families from the preregistered XOR-like construction.

- both opaque actions are physically/factually tried for every tuition scene;
- each pair combination appears at least 4 times at distinct absolute bindings;
- GENUINE and NO_CONSTRUCTION receive identical raw scenes/actions/Need;
- atom formation is enabled in both;
- only composite formation differs.

## Held-out mechanism gate

Held-out scenes use absolute motif bindings absent from tuition.

Run both opaque motor permutations.

PASS requires the original G10 protocol plus:

- GENUINE = **8/8** held-out pair decisions across the two motor permutations;
- NO_CONSTRUCTION <= **4/8**;
- NO_READOUT <= **4/8**;
- all four acquired child atom action evidences stay <=0.20 absolute;
- every used composite has >=0.60 absolute winning-action evidence;
- atom inventories in GENUINE and NO_CONSTRUCTION are identical by ID/prototype count/support;
- optimized regressions and Release build PASS.

These values are frozen before any G10 production implementation or test execution.
