# FRESH-G17 — GOAL-CONDITIONED ABSTRACT PLANNING QUALIFICATION

Status: **PRE-REGISTERED BEFORE FRESH-G17 EVALUATOR/RUN**

Date: 2026-10-07

## Claim

Can one learned goal-agnostic physical abstract transition model support different
actions for different raw goal observations across fresh abstraction hierarchies,
state-role assignments, opaque motor roles and held-out bindings?

This qualifies bounded explicit goal-conditioned abstract planning.

## Fresh authority

First valid GitHub Actions attempt only:

- authority seed = `github.run_id`;
- source SHA = `github.sha`;
- spec SHA = blob SHA of this file;
- run attempt = 1;
- exactly 10 deterministic sub-seeds.

Before transition tuition/scoring each sub-seed prints and hashes:

- 8 selected/permuted primitive relations;
- 4 physical L1 target pairs;
- 6 physical L2 state identities;
- permutation of logical current/intermediate/goal/distractor roles over the 6 L2 states;
- all six opaque motor roles;
- zero-value transition topology;
- transition tuition order;
- tuition and held-out raw layouts.

The first observed pack is permanently burned.

## Frozen substrate

Per sub-seed:

- raw 20x20 raster;
- 6 opaque motors;
- HDC dimension 192;
- local relation radius 4;
- 8 acquired primitive relation atoms;
- 4 promoted physical L1 concepts;
- all 6 unordered L1 pairs promoted as L2 abstract states;
- planning horizon 6;
- discount 0.95.

A full six-motor permutation assigns:

- two Stage-1 foundation roles;
- two Stage-2 abstraction roles;
- two goal-planning roles.

Thus all six opaque motor IDs are exercised.

## Fresh goal topology

Authority permutes the six acquired L2 states into logical roles:

- START;
- MID_A;
- MID_B;
- GOAL_A;
- GOAL_B;
- DISTRACTOR.

Let the two authority-selected goal-planning motors be A and B.

Learn the following **zero-valued** physical transition structure:

- START --A--> MID_A;
- START --B--> MID_B;
- MID_A --A--> GOAL_A;
- MID_A --B--> DISTRACTOR;
- MID_B --A--> DISTRACTOR;
- MID_B --B--> GOAL_B;
- both actions at GOAL_A, GOAL_B and DISTRACTOR are self-transitions.

Authority may swap which opaque motor occupies A/B.

Every factual transition value is exactly 0.

The transition model therefore contains no factual reward that identifies either goal.

## Tuition

Transition tuition uses one authority-selected raw binding per state.

Facts are presented in authority-permuted order.

After tuition:

- freeze transition learning;
- freeze abstraction learning;
- assert every physical transition endpoint is an acquired L2 cell;
- assert legacy graph transition count = 0.

## Held-out goal scoring

For each sub-seed:

- 2 goals: GOAL_A and GOAL_B;
- 4 held-out current/goal raw-binding variants per goal;
- same START state/model for both goals.

Total FULL N = **80**.

For every decision:

- current raw observation resolves to START physical cell;
- goal raw observation resolves to requested goal physical cell;
- goal planning must choose A for GOAL_A and B for GOAL_B;
- no transition relearning occurs;
- REAL factual frame unchanged;
- learned fingerprint unchanged.

## Controls

1. **FULL_GOAL_PLANNING**
   - current + requested raw goal;
   - horizon 6.

2. **DEPTH1**
   - same model and goal;
   - depth fixed to 1.

3. **NO_GOAL**
   - same zero-valued model;
   - ordinary value planning without a goal seed.

4. **WRONG_GOAL**
   - requested-goal score is evaluated while supplying the opposite valid goal cue;
   - selected action should follow the supplied other goal, not remain fixed.

5. **BROKEN_GOAL_RECOGNITION**
   - lesion one necessary lower L1->L2 synapse under GOAL_A.

6. **BROKEN_ROUTE**
   - lesion the physical START->MID_A successor synapse.

7. **PI_PHASE_ROUTE**
   - shift that same synapse by pi.

8. **RESTORE**
   - exact saved synapse restored without relearning.

9. **IRRELEVANT_ROUTE_LESION**
   - lesion START->MID_B while requesting GOAL_A.

## Fresh causal interventions

Per sub-seed:

- first two held-out GOAL_A variants:
  - broken goal recognition = 20 decisions total;
  - broken route = 20;
  - pi phase = 20;
  - restore = 20.
- first held-out GOAL_A variant:
  - irrelevant competing-route lesion = 10.

## Reporting

Report:

- source/spec/authority/digest;
- sealed hierarchy/topology/motor/layout plan;
- FULL /80 and Wilson95;
- per-seed FULL /8;
- DEPTH1 /80;
- NO_GOAL requested-goal score /80;
- WRONG_GOAL follows supplied other goal /80;
- broken-goal /20;
- broken-route /20;
- pi-phase /20;
- restore /20;
- irrelevant /10;
- structure/reference violations;
- zero-outcome violations;
- transition-endpoint violations;
- REAL/fingerprint mutation violations;
- legacy graph violations;
- all-six-motor mask;
- Human Protection v1.1 regression;
- full regressions/Release.

## Frozen PASS thresholds

FRESH-G17 PASS requires all:

1. exactly 10 sub-seeds and FULL N=80;
2. FULL >= **76/80**;
3. Wilson95 lower bound >= **0.87**;
4. every sub-seed FULL >= **6/8**;
5. changing only goal cue causes the required goal-specific first-action switch in every sub-seed;
6. DEPTH1 requested-goal score <= **10/80**;
7. NO_GOAL requested-goal score <= **20/80**;
8. WRONG_GOAL follows the supplied other goal in >= **70/80**;
9. BROKEN_GOAL_RECOGNITION requested GOAL_A success <= **4/20**;
10. BROKEN_ROUTE GOAL_A success <= **4/20**;
11. PI_PHASE_ROUTE GOAL_A success <= **4/20**;
12. RESTORE GOAL_A success >= **19/20**;
13. IRRELEVANT_ROUTE_LESION preserves GOAL_A >= **9/10**;
14. abstraction structure/reference violations = 0;
15. factual transition outcome-value violations = 0;
16. physical transition endpoint violations = 0;
17. REAL/fingerprint mutation violations = 0;
18. legacy graph violations = 0;
19. all six opaque motor IDs exercised;
20. source guard PASS;
21. Human Protection v1.1 regression PASS;
22. G0-G16/P1-P5 regressions PASS;
23. Release build PASS.

A compile/workflow failure before `FRESH_G17_SEAL` is technical. A completed
authority-scored run missing any frozen cognitive threshold is a burned
scientific FAIL.

## Interpretation boundary

PASS establishes statistical bounded explicit goal-conditioned abstract planning.

It does not establish:

- autonomous goal invention;
- natural-language/semantic goals;
- goal-conditioned active information acquisition;
- stochastic/POMDP planning;
- AGI or consciousness.

The next gate after PASS is goal-conditioned active information acquisition:
explore selectively to resolve what matters for the current goal.
