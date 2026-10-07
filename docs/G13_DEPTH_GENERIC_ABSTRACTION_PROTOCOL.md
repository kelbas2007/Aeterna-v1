# G13 — DEPTH-GENERIC PHASE-NATIVE ABSTRACTION

Status: **PRE-REGISTERED BEFORE G13 IMPLEMENTATION**

Date: 2026-10-07

## Why G13 exists

G12 statistically qualified self-triggered physical escalation from L1 to L2, but the implementation still has a dedicated L1/L2 recursive layer.

G13 asks whether the architecture can replace that special case with **one generic higher-abstraction mechanism** whose learned nodes may themselves become children of still higher learned nodes.

The development witness requires depth 3:

```text
raw raster
  -> acquired physical relation atoms
  -> acquired physical L1 concepts
  -> generic physical L2 nodes
  -> generic physical L3 nodes
  -> opaque motor
```

This is not a claim of arbitrary unbounded recursion. It is a mechanism gate for one level-generic rule reused at two successive higher levels.

## Non-negotiable rule

Production code must not contain separate task algorithms for L2 and L3.

One generic node structure and one generic factual learning/readout algorithm must handle every higher level.

A higher node stores:

- stable node ID;
- integer abstraction level;
- two child references;
- child physical cell addresses;
- one recruited physical concept cell;
- child->concept physical synapses;
- concept->opaque-motor physical synapses;
- factual support;
- per-motor factual support;
- promoted state.

A child reference contains its own level + stable ID + physical cell. For level 2, children are acquired L1 physical concepts. For level >=3, children are already promoted generic higher nodes.

No higher node may flatten its children back to primitive atom IDs.

## Raw substrate

Development witness uses:

- raw **20x20** raster;
- 6 opaque motor cells;
- HDC dimension 192;
- local relation radius 4;
- sufficient dormant physical cells.

Six motor roles are used only by the evaluator:

- two for L1 foundation acquisition;
- two for the L1->L2 abstraction task;
- two for the L2->L3 abstraction task.

Cognition receives only opaque motor IDs and factual Need.

## Primitive relation pool

Use 16 generic two-pixel offsets:

```text
(1,0),(0,1),(1,1),(2,0),
(0,2),(2,1),(1,2),(2,2),
(3,0),(0,3),(3,1),(1,3),
(3,2),(2,3),(3,3),(4,0)
```

The evaluator uses them only to generate raw scenes.

## Stage 1 — eight physical L1 concepts

Pair the 16 relation atoms into 8 target L1 pairs.

Each target pair:

- is shown at 4 distinct absolute bindings;
- factual foundation action A succeeds;
- factual foundation action B fails.

Use four non-target perfect matchings of the 16 atoms as balancing experience:

- each atom appears once in each balancing matching;
- balancing action B succeeds and A fails;
- each balancing pair is shown once.

Order-invariant physical promotion from G11/G12 remains active.

Required after Stage 1:

- exactly 16 acquired relation atoms;
- exactly 8 promoted physical L1 concepts;
- dedicated table composite path unused.

## Generic higher-abstraction API

Enable one generic higher-abstraction engine with `max_level = 3`.

The ordinary factual API receives only:

- raw raster;
- opaque executed action;
- factual Need.

For each factual scene the engine:

1. recognizes active acquired atoms;
2. activates promoted L1 physical concepts;
3. repeatedly activates promoted generic higher nodes bottom-up;
4. selects the **highest currently active promoted level** as the current explanatory children;
5. updates factual motor evidence for those children;
6. measures supported child sufficiency before/after the factual update;
7. if >=2 same-level children are supported but weak, may recruit a candidate at `child_level + 1`, subject to `max_level`;
8. an already legally recruited candidate continues learning on later co-activations;
9. re-evaluates supported candidates for promotion after factual updates.

There is no evaluator-supplied current level.

## Stage 2 — learn four L2 nodes with the generic rule

Use the 8 acquired L1 concepts.

Before pair formation, make each L1 child individually supported-but-weak on Stage-2 motor roles by **single-L1 raw scenes**:

- 4 factual repetitions;
- both Stage-2 opaque actions;
- balanced outcomes.

This must allocate zero higher nodes because only one L1 is active at a time.

Choose four **disjoint** pairs of L1 concepts as L2 targets.

For 4 cycles:

1. present every L2 target pair as one raw scene;
2. Stage-2 action A succeeds, B fails;
3. then present each constituent L1 alone with opposite balancing evidence so individual child evidence returns/remains weak.

Required:

- exactly 4 promoted generic **level-2** nodes;
- no level-3 candidate yet.

## Stage 3 — reuse L2 nodes as children and learn L3

The four promoted L2 nodes now become ordinary children of the same generic engine.

First establish supported-but-weak Stage-3 evidence for each L2 node using raw scenes that activate one L2 node at a time:

- 4 factual repetitions;
- both Stage-3 opaque actions;
- balanced outcomes.

This must allocate zero level-3 nodes because only one L2 child is active.

Pair the 4 L2 nodes into two disjoint L3 targets.

For 4 cycles:

1. present each L3 target pair as a raw scene that activates exactly its two L2 children;
2. Stage-3 action A succeeds, B fails;
3. rebalance each constituent L2 alone with opposite factual evidence.

Required:

- exactly 2 promoted generic **level-3** nodes.

## Held-out

Freeze all learning.

Use new absolute raw bindings absent from all tuition.

For each L3 target:

- score two held-out raw scenes.

Repeat with the Stage-3 two-motor mapping swapped.

Total mechanism FULL score = 8 decisions.

The readout must propagate raw atom activation -> L1 cells -> L2 cells -> L3 cells -> motor through actual physical conductance/coherence.

## Matched controls

1. **FULL_DEPTH3**
   - generic engine max_level=3.

2. **MAX_LEVEL2**
   - identical Stage1/Stage2/Stage3 factual experience;
   - generic engine max_level=2;
   - may keep revising L2 evidence but cannot allocate L3.

3. **NO_HIGHER_ENGINE**
   - same physical L1 foundation;
   - no generic higher abstraction state.

4. **ZERO_PHASE_L3**
   - acquire L1 and L2 normally;
   - set phase learning to zero before Stage3.

5. **ZERO_WEIGHT_L3**
   - acquire L1 and L2 normally;
   - set weight learning to zero before Stage3.

6. **NO_GROWTH_L3**
   - acquire L1 and L2 normally;
   - disable structural growth before Stage3.

## Causal interventions

On frozen FULL:

- lesion one necessary L2->L3 physical synapse;
- shift only that learned synapse phase by pi;
- restore the exact saved synapse without retraining;
- lesion an unrelated L3 synapse;
- lesion one necessary L1->L2 synapse under the target hierarchy.

## Mechanism PASS thresholds

PASS requires all:

1. Stage1: exactly 16 atom units and 8 promoted physical L1 concepts.
2. Stage2 single-child balancing allocates 0 generic higher candidates.
3. End Stage2: exactly 4 promoted generic level-2 nodes.
4. Before Stage3 pair tuition: 0 level-3 candidates.
5. End Stage3: exactly 2 promoted generic level-3 nodes.
6. Every L2 node stores child refs at level 1.
7. Every L3 node stores child refs at level 2.
8. No L3 node stores primitive atom IDs or L1 refs directly.
9. FULL_DEPTH3 = **8/8**.
10. MAX_LEVEL2 <= **4/8**.
11. NO_HIGHER_ENGINE <= **4/8**.
12. ZERO_PHASE_L3 <= **4/8**.
13. ZERO_WEIGHT_L3 <= **4/8**.
14. NO_GROWTH_L3 <= **4/8**.
15. Necessary L2->L3 lesion loses >=2/8.
16. Pi phase shift loses >=2/8.
17. Exact restoration returns 8/8 without retraining.
18. Unrelated L3 lesion preserves target in both motor permutations.
19. Lower L1->L2 lesion removes the dependent L3 target in both motor permutations.
20. Production source guard finds no level-specific task branch, hidden class/pair mapping, graph search or legacy planner fallback in the generic higher engine.
21. G0-G12 and P1-P5 regressions PASS.
22. Release build PASS.

## Fresh qualification after mechanism PASS

A separate one-use authority pack must randomize:

- the 16 relation-offset permutation;
- the 8 L1 target matching;
- the 4 L2 target pairing;
- the 2 L3 target pairing;
- all six opaque motor IDs;
- tuition and held-out spatial layouts;
- factual order within Stage2/Stage3 cycles.

Fresh qualification must include >=80 held-out L3 decisions across >=10 sub-seeds, Wilson95, matched max-depth controls, and causal lesion/phase/restore checks.

## Interpretation boundary

PASS would establish a bounded **depth-generic physical abstraction mechanism through level 3**.

It would not establish:

- unbounded recursion;
- autonomous choice of an unlimited depth;
- autonomous primitive feature invention;
- general semantic intelligence;
- language;
- AGI or consciousness.
