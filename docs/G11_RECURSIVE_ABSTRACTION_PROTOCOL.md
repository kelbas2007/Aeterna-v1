# G11 — RECURSIVE PHASE-NATIVE ABSTRACTION

Status: **PRE-REGISTERED BEFORE G11 IMPLEMENTATION**

Date: 2026-10-07

## Question

Can Aeterna-v1 use previously acquired **physical composite concepts as children** of a new physical composite, rather than returning to primitive atoms or using a host-side hierarchy table?

G11 is the first bounded recursive-abstraction gate.

## Non-negotiable architectural requirement

The causal chain must be:

```text
raw raster
  -> acquired atom cells
  -> acquired level-1 physical composite cells
  -> newly acquired level-2 physical composite cell
  -> opaque motor cell
```

The level-2 concept must reference acquired level-1 concept IDs/cells.

Not allowed:
- evaluator concept/class labels in production state;
- flattening the level-2 concept into primitive atom IDs;
- dedicated Rust action lookup from level-1 concept pair to motor;
- legacy graph/search;
- copying a correct motor from evaluator metadata.

Generic raw relation recognition remains inherited substrate.

## Physical substrate

Use one EvoPhase with:

- raw 12x12 raster;
- 4 opaque motor cells;
- phase-native mode enabled;
- generic local relation atom recognizer;
- physical concept layer enabled;
- sufficient dormant cells for structural growth.

All task-level adaptive pair evidence, promotion and action influence for both abstraction levels must live in actual EvoPhase cells/synapses.

## Stage 1 — acquire reusable level-1 concepts

Eight evaluator-only raw relation offsets are used:

`[(1,0),(0,1),(1,1),(2,0),(0,2),(2,1),(1,2),(2,2)]`.

They are treated only as raw geometry by the evaluator. No A/B/... labels enter cognition.

Choose the following four disjoint target pairs:

- L0 = {r0,r1}
- L1 = {r2,r7}
- L2 = {r3,r6}
- L3 = {r4,r5}

Foundation opaque actions are motor IDs 0 and 1 in the development witness.

Each target pair is shown at four unseen-to-each-other absolute bindings. For every scene both foundation actions are factually tried.

To make every primitive child individually non-predictive, add four one-shot negative perfect matchings over the eight atoms:

- M0 = {(0,7),(1,6),(2,5),(3,4)}
- M1 = {(0,6),(7,5),(1,4),(2,3)}
- M2 = {(0,5),(6,4),(7,3),(1,2)}
- M3 = {(0,4),(5,3),(6,2),(7,1)}

Each negative pair is shown once and both foundation actions are tried.

The four target pairs therefore reach the frozen physical promotion support; negative pair candidates do not.

After Stage 1 require:

- exactly 8 acquired raw relation atoms;
- exactly 4 promoted level-1 physical concepts corresponding to the four target pairs;
- no dedicated table composite state is used by the physical path.

## Stage 2 — concept-of-concepts task

The four acquired level-1 concepts are now the reusable children.

Top-level evaluator classes are balanced:

- class 0: {L0,L1} and {L2,L3}
- class 1: {L0,L2} and {L1,L3}

Therefore every individual L0..L3 appears once in class 0 and once in class 1.

Top-level opaque actions use motor IDs 2 and 3 in the development witness. Their semantic meaning is not supplied to cognition.

Each top-level pair is shown at four absolute raw-motif layouts. A top-level scene contains exactly the four primitive motifs needed to activate its two previously acquired level-1 concepts.

For every tuition scene both top-level actions are factually tried.

During Stage 2:

- acquired level-1 physical concept cells accumulate individual evidence for top-level actions;
- individual level-1 evidence must remain below the predictiveness ceiling;
- co-active level-1 concept pairs recruit physical level-2 candidate cells;
- pair factual evidence is stored in physical synapses;
- promotion to level 2 uses joint evidence + weak-child criterion.

No primitive atom pair may be stored as the children of a level-2 concept.

## Held-out

Freeze all learning.

Use absolute raw-motif layouts absent from both Stage 1 and Stage 2 tuition.

Evaluate all four top-level concept pairs under two opaque top-level motor permutations.

Total deterministic mechanism score: 8 decisions.

## Required controls

1. **FULL_RECURSIVE**
   - level-1 concepts present;
   - level-2 physical construction/readout present.

2. **NO_RECURSION**
   - same Stage-1 concepts and same Stage-2 factual experience;
   - level-2 promotion/readout disabled.

3. **LEVEL1_ONLY**
   - may aggregate individual level-1 top-action evidence;
   - cannot use pair evidence.

4. **ZERO_PHASE**
   - phase learning disabled for level-2 physical acquisition.

5. **ZERO_WEIGHT**
   - weight learning disabled for level-2 physical acquisition.

6. **NO_GROWTH**
   - no structural capacity for level-2 physical recruitment.

## Causal interventions

On a frozen FULL organism:

- lesion one necessary level1->level2 physical synapse;
- shift only that synapse phase by pi;
- restore the exact saved synapse without retraining;
- lesion a synapse belonging to a different level-2 concept;
- additionally lesion one lower atom->level1 synapse and require the dependent level-2 decision to fail.

## Mechanism PASS thresholds

PASS requires all:

1. Stage 1 acquires exactly 8 atom units.
2. Stage 1 promotes exactly 4 intended level-1 physical concepts.
3. Stage 2 promotes exactly 4 level-2 physical concepts.
4. Every level-2 concept stores references to acquired level-1 concept IDs/cells, not primitive atom IDs.
5. FULL_RECURSIVE = **8/8**.
6. NO_RECURSION <= **4/8**.
7. LEVEL1_ONLY <= **4/8**.
8. Max absolute individual level-1 top-action evidence <= **0.20**.
9. Min winning level-2 joint evidence >= **0.60**.
10. Necessary level1->level2 lesion loses >=2/8 decisions across the two motor permutations.
11. Pi phase shift loses >=2/8.
12. Exact restoration returns **8/8** without retraining.
13. Unrelated level-2 lesion preserves the targeted decision in both permutations.
14. Lower-level atom->level1 lesion removes the dependent top-level decision in both permutations.
15. ZERO_PHASE <= **4/8**.
16. ZERO_WEIGHT <= **4/8**.
17. NO_GROWTH <= **4/8**.
18. Source guard finds no table/search/evaluator fallback.
19. G0-G10-PHYS regressions PASS.
20. Release build PASS.

## Fresh qualification after mechanism PASS

A new one-use authority pack must use:

- >=10 sub-seeds;
- >=80 held-out top-level decisions;
- randomized 8-of-larger relation pool or randomized relation permutation;
- randomized four disjoint level-1 pair identities;
- randomized balanced level-2 pair assignment;
- randomized opaque motor IDs;
- held-out absolute bindings;
- Wilson 95% interval;
- fresh causal lesion / phase-shift / restore controls.

## Interpretation boundary

PASS would establish bounded **recursive physical abstraction**: an acquired concept can become the child of another acquired concept and causally influence action.

It would not establish:
- arbitrary recursion depth;
- autonomous invention of the abstraction objective;
- arbitrary arity;
- broad semantic concept learning;
- language;
- AGI or consciousness.
