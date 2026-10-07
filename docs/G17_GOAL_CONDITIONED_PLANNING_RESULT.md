# G17 — goal-conditioned abstract phase-native planning

Date: 2026-10-07

## Verdict

**MECHANISM PASS — one unchanged zero-reward physical world model selects different actions from different raw goal observations.**

Workflow: `37617083481`  
Scored source: `9f87bc44d71209942a9f35735f68ebad707fd3cb`  
Protocol: `docs/G17_GOAL_CONDITIONED_PLANNING_PROTOCOL.md`

## Mechanism result

Across two opaque goal-planning motor permutations:

- FULL_GOAL_PLANNING: **8/8**;
- DEPTH1 requested-goal score: **0/8**;
- NO_GOAL goal-specific score: **0/8**;
- supplying the opposite valid goal made the planner follow that other goal: **8/8**;
- BROKEN_GOAL_RECOGNITION GOAL-A success: **0/4**;
- BROKEN_ROUTE GOAL-A success: **0/4**;
- PI_PHASE_ROUTE GOAL-A success: **0/4**;
- exact RESTORE without relearning: **8/8**;
- IRRELEVANT competing-route lesion preserved selected goal: **2/2**;
- source guard PASS;
- Human Protection v1.1 regression PASS;
- G16 regression PASS;
- full optimized regressions PASS;
- Release build PASS.

Every learned G17 abstract transition carried factual value 0. The selected action therefore came from the requested goal cell and learned physical transition topology, not a reward label embedded in the model.

## Architectural meaning

The causal chain is:

```text
raw current observation -> acquired current abstract cell
raw goal observation    -> acquired goal abstract cell
                         -> imagined goal-cell excitation
                         -> backward physical propagation through learned
                            abstract successor/afferent synapses
                         -> opaque motor choice
```

The same transition model is reused for both goals.

Changing only the goal observation changes the first action.

No host graph/search, evaluator goal ID, correct-action mapping or separate reward retraining is used.

## Boundary

This establishes bounded explicit goal-conditioned abstract planning.

It does not establish autonomous goal invention, natural-language goals, stochastic planning, or goal-conditioned active information acquisition.

The active evidence gate is FRESH-G17.
