# G15 — ABSTRACT PHASE-NATIVE MODEL-BASED PLANNING

Status: **PRE-REGISTERED BEFORE G15 IMPLEMENTATION**

Date: 2026-10-07

## Why G15 exists

G10-G14 establish increasingly strong physical abstraction:

- composite concept construction;
- physical concept execution;
- recursive concepts;
- self-triggered escalation;
- one generic depth mechanism;
- self-selected representational depth.

Those capabilities are still primarily used for factual action classification.

G15 connects acquired abstraction to **future-directed cognition**.

The organism must use acquired physical concept cells as states of a learned phase-native transition model and choose a delayed-reward action by propagating value through multiple learned abstract transitions.

## Core causal chain

```text
raw observation
 -> acquired physical abstract-state cell
 -> learned physical abstract transition
 -> future abstract-state cell
 -> further learned transition(s)
 -> delayed factual value
 -> backward physical value propagation
 -> opaque motor choice
```

Task-level planning must not use a host graph, transition table, route enumeration, evaluator state ID or evaluator goal ID.

## Production architecture requirement

Extend the existing P1 physical transition/value mechanism rather than introducing an independent graph planner.

Required design:

1. raw sensory input activates an already acquired physical concept/deep node;
2. that physical cell is used directly as the state endpoint of phase-native transition circuits;
3. factual interaction learns:
   - abstract-state cell -> relay;
   - relay -> next abstract-state cell;
   - relay -> opaque motor;
   - relay -> factual outcome;
4. planning begins from the currently active abstract-state cell;
5. the existing local physical value recurrence propagates delayed value through those abstract successor cells;
6. selected motor arises from physical motor potentials.

A host-side function may recognize which acquired abstract cell is active from raw input. It may not map that cell to a task answer or route.

## Development abstraction substrate

Use the frozen 20x20 relation substrate and acquire four reusable physical L2 concepts with the G13/G14 generic abstraction mechanism.

These four acquired L2 cells are evaluator-referred to as S0/S1/S2/S3 only in tests. Production receives no S labels.

Every abstract state must be recognizable from at least two raw spatial bindings:
- tuition binding;
- held-out binding not used while learning abstract transitions.

The abstract transition model is learned only once from tuition bindings.

## Development world

Two opaque planning actions are used.

From S0:

- action A: terminal/absorbing branch with immediate factual value **0.55**;
- action B: S1 with immediate value **0.00**.

Delayed branch:

- S1 -> S2 with value **0.00**;
- S2 -> S3 with value **1.00**.

Choose discount **0.95**.

Thus:
- depth-1 / immediate choice prefers A (0.55 > 0);
- adequate multi-step physical propagation prefers B because delayed value reaches S0 with discounted value >0.55.

Additional factual transitions are included so both opaque actions have learned outcomes at every abstract state and no missing-transition heuristic can identify the correct route.

The evaluator may permute the two opaque action IDs.

## Factual tuition

For every abstract-state/action transition:

- present raw PRE;
- execute opaque action in evaluator world;
- present factual raw POST;
- present factual bounded value.

No imagined output is used as tuition.

The model sees transition facts, not a solved action sequence.

After tuition:
- freeze all transition learning;
- use only held-out raw bindings for planning/scoring.

## Required planning behavior

Test both opaque-action permutations.

FULL_ABSTRACT_PLANNING must:

- choose delayed branch at S0;
- report selected physical planning depth >=3;
- leave factual REAL state unchanged while planning;
- preserve acquired abstract/transition weights during imagined evaluation.

Total mechanism FULL score: **8/8** over two motor permutations and four held-out binding/nuisance variants.

## Controls

1. **FULL_ABSTRACT_PLANNING**
   - acquired abstraction;
   - physical abstract transition circuits;
   - horizon >=3.

2. **DEPTH1**
   - same learned physical abstract model;
   - horizon fixed to 1;
   - must prefer immediate 0.55 branch.

3. **NO_ABSTRACT_MODEL**
   - same acquired perceptual abstraction;
   - abstract transition tuition disabled;
   - must not solve delayed choice.

4. **BROKEN_ABSTRACT_STATE**
   - lesion one necessary lower concept/deep synapse so the current held-out raw scene no longer activates its required abstract-state path.

5. **BROKEN_TRANSITION**
   - lesion one necessary physical abstract successor synapse on the delayed route.

6. **PI_PHASE_TRANSITION**
   - shift that same successor synapse phase by pi.

7. **RESTORE**
   - restore exact saved successor synapse without retraining.

8. **IRRELEVANT_LESION**
   - lesion a transition not on the selected delayed route and require selected start action to remain correct.

## Transfer requirement

Abstract transition tuition is performed using one set of raw bindings.

Held-out planning uses different raw bindings of the same learned abstract states.

The transition model must not be relearned on held-out bindings.

## Source guard

The production abstract-planning path must contain no:

- evaluator S0/S1/S2/S3 identifiers;
- correct action mapping;
- route list;
- BFS/DFS/Dijkstra/A*;
- transition hash/map/table used for task-level planning;
- legacy `EvoImaginationPlanner` fallback.

It may use:
- acquired physical abstract cell addresses;
- shared physical PhaseSynapse arrays;
- phase coherence/conductance;
- local bounded value recurrence;
- local motor competition.

## Mechanism PASS thresholds

PASS requires all:

1. Four physical abstract state concepts acquired before transition tuition.
2. Abstract transition tuition creates physical circuits whose endpoints are acquired abstract cells.
3. FULL = **8/8**.
4. Every successful FULL decision selects the delayed branch.
5. Every successful FULL decision has selected depth >=3.
6. DEPTH1 delayed-branch score <= **1/8**.
7. NO_ABSTRACT_MODEL delayed-branch score = **0/8**.
8. BROKEN_ABSTRACT_STATE loses >=2/8.
9. BROKEN_TRANSITION loses >=2/8.
10. PI_PHASE_TRANSITION loses >=2/8.
11. Exact RESTORE returns **8/8** without retraining.
12. IRRELEVANT_LESION preserves target in both motor permutations.
13. Held-out raw bindings require no transition relearning.
14. Pure planning changes neither REAL factual state nor learned-state fingerprint.
15. Source guard PASS.
16. G0-G14 / P1-P5 regressions PASS.
17. Release build PASS.

## Fresh qualification after mechanism PASS

A separate one-use authority pack must randomize:

- primitive relation hierarchy used to form abstract states;
- raw tuition and held-out bindings;
- opaque action IDs;
- delayed-route length 2..4 transitions after the first action;
- immediate distractor value in a range where depth-1 prefers it but full discounted delayed value prefers the long branch;
- order of factual transition tuition.

Fresh N >=80 start-state decisions across >=10 sub-seeds.

Required fresh controls include:
- DEPTH1;
- no abstract model;
- necessary abstract-state lesion;
- necessary abstract-transition lesion;
- pi phase shift;
- exact restoration;
- irrelevant lesion.

## Interpretation boundary

PASS would establish bounded **abstract model-based planning**: acquired concepts serve as reusable physical states in a learned multi-step future model.

It would not establish:
- open-ended goal invention;
- general search over arbitrary programs;
- unrestricted stochastic planning;
- natural-language reasoning;
- AGI or consciousness.
