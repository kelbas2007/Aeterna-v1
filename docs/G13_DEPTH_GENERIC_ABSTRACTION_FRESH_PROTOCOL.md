# FRESH-G13 — DEPTH-GENERIC PHASE-NATIVE ABSTRACTION QUALIFICATION

Status: **PRE-REGISTERED BEFORE FRESH-G13 EVALUATOR/RUN**

Date: 2026-10-07

## Claim

Can one generic physical higher-abstraction engine, without separate L2/L3 task algorithms, repeatedly build and execute a depth-3 hierarchy on fresh authority-selected primitive relations, hierarchy structure, opaque motor roles and spatial bindings?

This qualifies the bounded G13 depth-generic mechanism through level 3. It does not qualify unbounded recursion.

## Frozen substrate

The fresh evaluator uses the same substrate contract as the original G13 protocol:

- raw **20x20** raster;
- 6 opaque motor cells;
- HDC dimension 192;
- local relation radius 4;
- 16 two-pixel relation offsets:
  `[(1,0),(0,1),(1,1),(2,0),(0,2),(2,1),(1,2),(2,2),(3,0),(0,3),(3,1),(1,3),(3,2),(2,3),(3,3),(4,0)]`;
- one generic higher-abstraction engine with max_level=3.

## Fresh authority

First valid GitHub Actions attempt only:

- authority seed = `github.run_id`;
- source SHA = `github.sha`;
- spec SHA = blob SHA of this file;
- run attempt = 1;
- exactly 10 deterministic sub-seeds.

For every sub-seed, before any tuition/scoring:

1. permute the 16 primitive relation indices;
2. choose the 8 L1 target pairs from that permutation;
3. permute the 8 acquired L1 positions and form 4 disjoint L2 targets;
4. permute the 4 L2 positions and form 2 disjoint L3 targets;
5. permute all six opaque motor IDs and assign two roles to Stage1, two to Stage2, two to Stage3;
6. derive factual order permutations for Stage2 and Stage3 cycles;
7. deterministically derive tuition/held-out 20x20 bindings.

The complete hierarchy/motor/layout description and pack digest must be printed before tuition/scoring. The first observed pack is permanently burned.

## Geometry-only layout authority

Because eight raw motifs must coexist on a 20x20 raster, the evaluator may search for a valid binding layout before sealing.

The frozen validity rule is purely representational:

- every motif pixel is in bounds;
- motif pixels do not overlap;
- cross-motif local pixel relations must not match any of the 16 primitive relation prototypes;
- required motifs themselves remain recognizable.

The search may depend only on:
- authority seed/sub-seed;
- relation geometry;
- spatial bounds.

It may not inspect Need, motor success, learned weights, predictions or scoring outcomes.

If a candidate layout fails the representation-only rule, it is rejected **before seal** and the next deterministic candidate is tried. Report rejection count.

## Stage 1 — fresh L1 foundation

Per sub-seed:

- acquire 16 primitive atom units;
- acquire exactly 8 promoted physical L1 concepts;
- dedicated table composite path remains unused.

Use four target bindings per L1 pair and four non-target perfect matching rounds for balancing evidence, as in the frozen G13 mechanism.

## Stage 2 — same generic engine creates L2

- establish supported-but-weak evidence for every L1 child on the two authority-selected Stage2 motors;
- require 0 generic higher candidates after single-child evidence;
- use the authority-selected 4 disjoint L2 target pairs;
- factual pair/balancing order is authority-permuted;
- end with exactly 4 promoted generic level-2 nodes;
- require 0 level-3 candidates before Stage3.

## Stage 3 — same generic engine creates L3

- establish supported-but-weak evidence for every promoted L2 child on authority-selected Stage3 motors;
- require 0 level-3 candidates after single-child evidence;
- use authority-selected 2 disjoint L3 target pairs;
- factual pair/balancing order is authority-permuted;
- end with exactly 2 promoted generic level-3 nodes.

Every L2 node must reference level-1 children.
Every L3 node must reference level-2 children.
No L3 node may directly reference primitive atoms or level-1 children.

## Held-out scoring

For each sub-seed:

- 2 L3 target concepts;
- 4 fresh held-out valid 20x20 layouts per L3 target;
- 8 decisions per sub-seed.

Total FULL N = **80**.

All learning is frozen before held-out scoring.

## Controls

On matched authority-selected worlds:

1. **FULL_DEPTH3**
   - max_level=3.

2. **MAX_LEVEL2**
   - same Stage1/Stage2/Stage3 facts;
   - max_level=2.

3. **NO_HIGHER_ENGINE**
   - same L1 foundation;
   - generic higher engine absent.

4. **ZERO_PHASE_L3**
   - normal L1/L2;
   - phase learning = 0 before Stage3.

5. **ZERO_WEIGHT_L3**
   - normal L1/L2;
   - weight learning = 0 before Stage3.

6. **NO_GROWTH_L3**
   - normal L1/L2;
   - structural growth disabled before Stage3.

## Fresh causal interventions

Per sub-seed, choose the first L3 target and its first two held-out layouts:

- intact target decisions;
- lesion one necessary actual L2->L3 child synapse;
- independently shift that same synapse by pi;
- restore the exact saved synapse without retraining;
- lesion a physical child synapse from the other L3 target;
- lesion one necessary L1->L2 synapse underneath the target hierarchy.

Totals across 10 sub-seeds:

- 20 necessary-lesion decisions;
- 20 phase-shift decisions;
- 20 restoration decisions;
- 10 unrelated-lesion target decisions;
- 20 lower-hierarchy-lesion decisions.

## Source guard

The generic higher engine must contain no:

- special L2 task function;
- special L3 task function;
- branch on exact task level 2 or 3;
- evaluator hierarchy constants;
- evaluator correct-action mapping;
- graph search / legacy planner fallback.

One generic node structure and one factual/readout rule must serve both L2 and L3.

## Frozen PASS thresholds

FRESH-G13 PASS requires all:

1. Exactly 10 sub-seeds and FULL N=80.
2. FULL_DEPTH3 >= **76/80**.
3. Wilson 95% lower bound >= **0.87**.
4. Every sub-seed FULL >= **6/8**.
5. Stage1 structure violations = 0: 16 atoms / 8 promoted physical L1 concepts.
6. Stage2 single-child premature-candidate violations = 0.
7. End-Stage2 structure violations = 0: exactly 4 promoted L2 and 0 L3 candidates.
8. Stage3 single-child premature-candidate violations = 0.
9. End-Stage3 structure violations = 0: exactly 2 promoted L3.
10. Child-level/reference violations = 0.
11. MAX_LEVEL2 <= **40/80**.
12. NO_HIGHER_ENGINE <= **40/80**.
13. ZERO_PHASE_L3 <= **40/80**.
14. ZERO_WEIGHT_L3 <= **40/80**.
15. NO_GROWTH_L3 <= **40/80**.
16. Necessary L2->L3 lesion succeeds in <= **4/20** targeted decisions.
17. Pi phase shift succeeds in <= **4/20**.
18. Exact restoration succeeds in >= **19/20** without retraining.
19. Unrelated L3 lesion preserves >= **9/10** target decisions.
20. Lower L1->L2 lesion leaves dependent L3 decision correct in <= **1/20**.
21. All six opaque motor IDs are exercised across the sealed pack.
22. Source guard PASS.
23. G0-G12 / P1-P5 regressions PASS.
24. Release build PASS.

A compile/workflow failure before `FRESH_G13_SEAL` is technical. A completed authority-scored run missing any frozen cognitive threshold is a burned scientific FAIL.

## Interpretation boundary

PASS establishes statistical bounded depth-generic physical abstraction through level 3 with the same higher-level mechanism reused twice.

PASS does not establish:

- unbounded recursion;
- self-chosen unlimited depth;
- autonomous primitive feature invention;
- general semantic intelligence;
- language;
- AGI or consciousness.
