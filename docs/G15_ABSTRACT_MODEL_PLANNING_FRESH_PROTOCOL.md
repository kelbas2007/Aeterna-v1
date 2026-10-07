# FRESH-G15 — ABSTRACT PHASE-NATIVE MODEL-BASED PLANNING QUALIFICATION

Status: **PRE-REGISTERED BEFORE FRESH-G15 EVALUATOR/RUN**

Date: 2026-10-07

## Claim

Can acquired physical abstractions act as reusable states of a learned phase-native future model and support delayed-reward action choice across fresh abstract hierarchies, opaque motors, raw bindings and route lengths?

This qualifies the bounded G15 abstract model-based planning mechanism. It does not qualify unrestricted search or AGI.

## Fresh authority

First valid GitHub Actions attempt only:

- authority seed = `github.run_id`;
- source SHA = `github.sha`;
- spec SHA = blob SHA of this file;
- run attempt = 1;
- exactly 10 deterministic sub-seeds.

Before transition tuition/scoring, every sub-seed must print and hash:

- selected/permuted primitive relations;
- L1 hierarchy;
- six acquired L2 abstract-state identities;
- six opaque motor-role assignment;
- delayed route length;
- immediate distractor value;
- transition tuition order;
- raw tuition layouts;
- eight held-out start-state layouts.

The first observed pack is permanently burned.

## Frozen physical substrate

Per sub-seed:

- raw 20x20 raster;
- 6 opaque motor cells;
- HDC dimension 192;
- local relation radius 4;
- phase-native planning horizon = 6;
- discount = 0.95;
- one generic physical abstraction engine.

No legacy graph planner is enabled.

## Fresh abstract state construction

Authority permutes the 16 G13 relation offsets and selects the first 8.

Those 8 raw relation atoms are paired into 4 physical L1 concepts.

The generic higher-abstraction engine then acquires all 6 unordered pair combinations of the four L1 concepts as 6 reusable physical L2 abstract states.

Individual L1 motor evidence is balanced with single-L1 factual experience, so the six L2 states are acquired through the same supported-weak abstraction rule rather than fixed state IDs.

Required before transition tuition:

- exactly 8 acquired primitive atom units;
- exactly 4 promoted physical L1 concepts;
- exactly 6 promoted generic L2 nodes;
- every L2 node references two acquired L1 physical cells;
- dedicated table composite state remains unused.

Evaluator names S0..S5 are only aliases for scoring. Production receives raw sensory input and acquired physical cells only.

## Fresh delayed world

Authority selects:

- two of the six motor IDs as planning actions via a full six-role permutation;
- delayed chain length `L in 2..=4`, meaning L transitions occur **after** the first delayed action;
- immediate distractor factual value from `[0.50, 0.70]`.

The delayed chain uses distinct acquired abstract states:

`S0 --delayed--> S1 -> ... -> S(L+1)`

The last chain transition has factual value 1.0. Earlier chain transitions have value 0.

The immediate action at S0 transitions to an absorbing abstract state with the sampled immediate value.

At every abstract state both planning actions receive factual transition observations. Off-route actions receive deterministic zero-valued transitions, so transition presence/absence cannot identify the delayed solution.

With discount 0.95 and L<=4, the delayed branch discounted value remains above 0.70 while depth-1 still sees immediate > 0 versus delayed immediate 0.

## Transition tuition

Transition facts are presented using tuition raw bindings only.

Authority permutes factual transition order.

Each transition is learned through `observe_phase_native_abstract_transition`.

After tuition:

- freeze transition learning;
- freeze abstraction learning;
- transition circuits must begin/end on acquired L2 physical cells;
- legacy planning transition table count must remain 0.

## Held-out scoring

Per sub-seed:

- 8 fresh raw start-state bindings not used in transition tuition;
- no transition relearning;
- FULL plans from raw sensory using `plan_phase_native_abstract`.

Total FULL N = **80**.

For every successful FULL decision:

- chosen motor must be the delayed first action;
- selected physical depth must be at least `L+1`;
- REAL factual frame must remain unchanged;
- phase-native learned fingerprint must remain unchanged.

## Matched controls

1. **FULL_ABSTRACT_PLANNING**
   - same learned abstract model;
   - horizon 6.

2. **DEPTH1**
   - exact same physical model;
   - planning depth fixed to 1.

3. **NO_ABSTRACT_MODEL**
   - exact same acquired abstraction;
   - transition tuition absent.

4. **BROKEN_ABSTRACT_STATE**
   - lesion one necessary lower physical L1->L2 synapse beneath S0.

5. **BROKEN_TRANSITION**
   - lesion the physical successor synapse on S0's delayed transition.

6. **PI_PHASE_TRANSITION**
   - shift that same successor synapse by pi.

7. **RESTORE**
   - restore exact saved transition synapse without relearning.

8. **IRRELEVANT_LESION**
   - lesion a learned transition not on the selected delayed route.

## Geometry-only layout authority

All tuition/held-out layouts are generated before scoring.

A candidate layout is valid only if:

- all motif pixels are in bounds;
- motif pixels do not overlap;
- unintended cross-motif local relations do not match the selected primitive relation prototypes;
- required motifs remain recognizable.

Layout search may inspect only authority seed, relation geometry and raster bounds.

It may not inspect Need, action success, learned weights, predictions or scores.

Report pre-seal rejection count.

## Reporting

Report:

- source/spec/authority/pack digest;
- every sealed sub-seed world description;
- FULL /80 and Wilson95;
- per-seed FULL;
- selected-depth witness by route length;
- DEPTH1 delayed score;
- NO_ABSTRACT_MODEL delayed score;
- BROKEN_ABSTRACT_STATE targeted score;
- BROKEN_TRANSITION targeted score;
- PI_PHASE targeted score;
- RESTORE targeted score;
- IRRELEVANT_LESION target score;
- structure/reference violations;
- physical transition endpoint violations;
- REAL/fingerprint mutation violations;
- legacy graph-table violations;
- motor-role mask;
- layout rejection count.

## Fresh causal interventions

Per sub-seed, use the first two held-out start bindings:

- 20 BROKEN_ABSTRACT_STATE decisions;
- 20 BROKEN_TRANSITION decisions;
- 20 PI_PHASE decisions;
- 20 RESTORE decisions.

Use the first held-out start binding for one IRRELEVANT_LESION decision per seed:

- 10 unrelated-lesion decisions.

## Frozen PASS thresholds

FRESH-G15 PASS requires all:

1. Exactly 10 sub-seeds and FULL N=80.
2. FULL delayed branch >= **76/80**.
3. Wilson 95% lower bound >= **0.87**.
4. Every sub-seed FULL >= **6/8**.
5. Selected-depth violations = **0**; successful FULL depth >= L+1.
6. DEPTH1 delayed branch <= **4/80**.
7. NO_ABSTRACT_MODEL delayed branch = **0/80**.
8. Abstract structure/reference violations = **0**.
9. Physical transition endpoint violations = **0**.
10. BROKEN_ABSTRACT_STATE succeeds in <= **4/20** targeted decisions.
11. BROKEN_TRANSITION succeeds in <= **4/20**.
12. PI_PHASE_TRANSITION succeeds in <= **4/20**.
13. RESTORE succeeds in >= **19/20** without retraining.
14. IRRELEVANT_LESION preserves >= **9/10** target decisions.
15. REAL/fingerprint mutation violations during planning = **0**.
16. Legacy graph-table violations = **0**.
17. Both planning-action positions and all six opaque motor roles are exercised across the sealed pack.
18. Source guard PASS.
19. G0-G14 / P1-P5 regressions PASS.
20. Release build PASS.

A workflow/compile failure before `FRESH_G15_SEAL` is technical. A completed authority-scored run missing any frozen cognitive threshold is a burned scientific FAIL.

## Interpretation boundary

PASS establishes statistical bounded abstract model-based planning: learned physical concepts become state variables of a multi-step learned future model and support delayed-reward choice across fresh route lengths and raw bindings.

PASS does not establish:

- unrestricted stochastic planning;
- arbitrary program search;
- autonomous goal invention;
- natural-language reasoning;
- AGI or consciousness.
