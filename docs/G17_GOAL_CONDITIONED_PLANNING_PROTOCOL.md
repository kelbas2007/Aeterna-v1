# G17 — GOAL-CONDITIONED ABSTRACT PHASE-NATIVE PLANNING

Status: **PRE-REGISTERED BEFORE G17 IMPLEMENTATION**

Date: 2026-10-07

## Why G17 exists

G15/G16 qualify learned abstract future models and autonomous acquisition, but the
decision objective is still a factual scalar value embedded in the learned world.

G17 introduces an explicit current goal as an already-recognizable raw abstract
state. The same learned transition model must support different choices when the
goal changes, without relearning transitions or reward values.

## Core causal chain

```text
raw current observation -> acquired current abstract cell
raw goal observation    -> acquired goal abstract cell
                         -> seed goal cell internally
                         -> physical backward propagation through learned
                            abstract transition circuits
                         -> opaque motor choice
```

The transition model is goal-agnostic.

## Production requirements

Extend the same phase-native physical recurrence.

A new goal-conditioned planner may receive:

- current raw sensory observation;
- goal raw sensory observation.

It may resolve each to an acquired physical abstract cell.

It MUST NOT receive:

- evaluator state IDs;
- correct action;
- route list;
- goal ID;
- host graph/search output.

Goal planning must not modify REAL factual state or learned weights.

## Development abstraction substrate

Use the frozen 20x20 G15/G16 abstract substrate and six acquired L2 physical states.

Learn a goal-agnostic transition model with all factual transition values set to 0.

The model must include at least two competing routes from the same start:

- route to goal A begins with opaque motor X;
- route to goal B begins with opaque motor Y;
- X != Y.

Both goals are acquired physical abstract states.

## Goal recurrence

For planning only:

1. set goal physical cell charge to 1.0 in imagined membranes;
2. propagate charge backward through supported phase-native successor/afferent
   synapses for the configured horizon;
3. read motor potentials only from circuits leaving the current abstract cell;
4. select the strongest opaque motor.

No factual outcome weight is required for the goal signal.

## Held-out transfer

Transition tuition uses one raw binding per abstract state.

Goal-conditioned planning uses raw current and raw goal bindings absent from
transition tuition.

No transition or abstraction relearning is allowed at scoring time.

## Development score

Test both opaque motor permutations and two distinct goals from the same start.

For each motor permutation:

- 2 goals;
- 2 held-out raw binding variants per goal.

Total FULL score: **8 decisions**.

Each goal pair must require different first actions.

## Controls

1. **FULL_GOAL_PLANNING**
   - same transition model;
   - raw current + raw goal;
   - sufficient horizon.

2. **NO_GOAL**
   - same model/current state;
   - no goal seed;
   - cannot satisfy the goal-specific decision criterion.

3. **WRONG_GOAL**
   - give the other valid goal cue;
   - action must switch accordingly rather than remain fixed.

4. **DEPTH1**
   - same goal/model;
   - route requiring >=2 abstract transitions must fail when horizon=1.

5. **BROKEN_GOAL_RECOGNITION**
   - lesion one necessary lower abstraction synapse beneath the raw goal cue.

6. **BROKEN_ROUTE**
   - lesion one necessary physical successor synapse on the route to the selected goal.

7. **PI_PHASE_ROUTE**
   - shift that same route synapse by pi.

8. **RESTORE**
   - restore exact saved synapse without relearning.

9. **IRRELEVANT_ROUTE_LESION**
   - lesion a transition used only by the competing goal route;
   - selected goal action must remain correct.

## Mechanism PASS thresholds

PASS requires all:

1. six acquired L2 physical abstract states before transition tuition;
2. learned transition circuits begin/end only on acquired abstract cells;
3. factual transition values for the G17 model are all 0;
4. FULL_GOAL_PLANNING = **8/8**;
5. changing only the raw goal cue changes the selected first action in both motor permutations;
6. NO_GOAL goal-specific score <= **2/8**;
7. WRONG_GOAL follows the wrong/other goal rather than the requested one in >= **6/8**;
8. DEPTH1 requested-goal score <= **2/8**;
9. BROKEN_GOAL_RECOGNITION loses >=2/8;
10. BROKEN_ROUTE loses >=2/8;
11. PI_PHASE_ROUTE loses >=2/8;
12. exact RESTORE returns 8/8 without relearning;
13. IRRELEVANT_ROUTE_LESION preserves selected goal in both motor permutations;
14. held-out raw goal/current bindings require no relearning;
15. REAL factual frame unchanged by planning;
16. learned-state fingerprint unchanged by planning;
17. legacy graph transition count = 0;
18. source guard finds no evaluator state/goal IDs, correct-action mapping, BFS/DFS/Dijkstra/A*, route table or EvoImaginationPlanner fallback;
19. Human Protection v1.1 regression PASS;
20. G0-G16/P1-P5 regressions PASS;
21. Release build PASS.

## Fresh qualification

A separate one-use fresh pack must randomize:

- primitive abstraction hierarchy;
- six abstract-state identities;
- transition topology;
- two or more requested goals per world;
- opaque motor IDs;
- route lengths 2..4;
- raw current/goal tuition and held-out bindings;
- transition tuition order.

Fresh N >=80 goal-conditioned decisions across >=10 sub-seeds.

## Interpretation boundary

PASS establishes bounded explicit goal-conditioned abstract planning over a
learned physical world model.

It does not establish:

- autonomous goal invention;
- semantic/natural-language goals;
- stochastic planning;
- goal-conditioned active information acquisition;
- AGI or consciousness.
