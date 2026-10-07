# G18 — GOAL-CONDITIONED ACTIVE ABSTRACT REASONING

Status: **PRE-REGISTERED BEFORE G18 IMPLEMENTATION**

Date: 2026-10-07

## Why G18 exists

G16 qualifies autonomous acquisition of a missing abstract transition model, but its
exploration objective is goal-agnostic: it values unknown/reachable frontier
structure in general.

G17 qualifies explicit goal-conditioned planning, but over an already acquired model.

G18 joins the two loops.

The organism must use the current raw goal observation to decide **which missing
abstract transition is worth physically testing now**.

## Architectural claim

The causal loop must be:

```text
raw current observation -> acquired current abstract cell
raw goal observation    -> acquired goal abstract cell
                         -> physical goal-relevance field over known model
                         -> goal-modulated epistemic frontier field
                         -> opaque physical exploratory action
                         -> factual next raw state
                         -> physical transition update
                         -> goal-conditioned replanning
```

The goal changes information acquisition without changing the learned world model
or the transferred P4 epistemic weights.

## Inherited drive

Reuse the same two learned P4 epistemic channels:

1. DIRECT_UNMODELLED
2. REACHABLE_FRONTIER

No new learned feature weight is introduced in G18.

The current raw goal supplies a multiplicative physical relevance field:

- seed goal abstract cell with charge 1;
- propagate relevance backward only through supported physical abstract transitions;
- for every acquired abstract state, multiply its unknown-action fraction by its
  goal relevance;
- propagate that goal-relevant unknown field backward through the same supported
  physical transition synapses.

The resulting goal-conditioned features are scored by the SAME transferred P4
drive synapses.

## Equal-score tie-break

If two actions have exactly equal goal-conditioned drive score, production may use
the same G16 generic supported-transition evidence tie-break.

It may not inspect:
- evaluator state IDs;
- goal IDs;
- correct action;
- shortcut identity;
- route list;
- hidden world law.

## Development substrate

Use the frozen 20x20 G17 substrate:

- 8 raw relation atoms;
- 4 physical L1 concepts;
- 6 physical L2 abstract states;
- 6 opaque motors;
- planning horizon 6;
- discount 0.95.

Two acquired L2 states act as goals A and B.

## Partial physical model

Before active interaction, learn a goal-agnostic partial model.

States:

- S0 start;
- S1 hub A;
- S2 hub B;
- S3 intermediate A;
- S4 goal A;
- S5 goal B.

For each opaque motor permutation:

- known S0 -> S1 transition through motor A;
- known S0 -> S2 transition through motor B;
- known S1 -> S3 -> S4 route;
- known S2 -> S5 route;
- all other non-shortcut motors at S0/S1/S2 are learned self-loops;
- one action at S1 remains deliberately unmodelled: shortcut A;
- one action at S2 remains deliberately unmodelled: shortcut B.

The evaluator world law is:

- shortcut A: S1 -> S4;
- shortcut B: S2 -> S5.

No shortcut tuple is supplied to cognition.

## Goal-conditioned acquisition episode

For each requested goal:

1. present raw S0 + raw goal cue;
2. allow at most **2 physical interactions**;
3. at each step choose with the G18 goal-conditioned active selector;
4. execute evaluator law;
5. update through ordinary factual abstract transition learning;
6. stop when the requested shortcut transition is structurally acquired.

The two-interaction bound requires:

- first action: navigate to the goal-relevant hub;
- second action: probe the unknown shortcut there.

## Development score

Test:

- both motor permutations;
- both goals;
- two held-out current/goal raw bindings.

FULL target shortcut acquisitions: **8**.

After acquisition freeze learning and require:

- planning from the relevant hub to the raw goal selects the newly learned shortcut;
- opposite goal cue targets the opposite hub/shortcut.

## Controls

1. **FULL_GOAL_ACTIVE**
   - raw current + raw goal;
   - transferred learned P4 drive;
   - goal-conditioned relevance/frontier modulation.

2. **GENERAL_FRONTIER**
   - same partial model and drive;
   - ordinary G16 goal-agnostic selector.

3. **NO_GOAL**
   - no goal relevance field;
   - cannot satisfy goal-specific shortcut criterion.

4. **WRONG_GOAL**
   - supply the other valid raw goal cue;
   - must target the other shortcut rather than the requested one.

5. **BROKEN_GOAL_RECOGNITION**
   - lesion a physical lower abstraction synapse beneath the goal cue.

6. **BROKEN_GOAL_ROUTE**
   - lesion a supported physical transition needed to propagate relevance from the
     requested goal back to its hub/start.

7. **PI_PHASE_GOAL_ROUTE**
   - shift that same route successor synapse by pi.

8. **RESTORE**
   - exact synapse restore without relearning.

9. **IRRELEVANT_BRANCH_LESION**
   - lesion only the competing goal branch;
   - requested-goal shortcut acquisition must remain correct.

10. **NO_TRANSITION_LEARNING**
    - exploration may execute, but shortcut fact cannot be installed.

## Physical ownership requirements

FULL selector must derive all scores only from:

- current physically active abstract cell;
- physically recognized raw goal cell;
- supported phase-native abstract transition circuits;
- actual phase-sensitive conductance;
- transferred P4 drive synapses;
- physical unknown-action evidence.

No host graph, BFS/DFS/Dijkstra/A*, route list, state/goal table or evaluator shortcut map.

## Mechanism PASS thresholds

PASS requires all:

1. FULL acquires requested shortcut in **8/8** within 2 interactions.
2. FULL first physical action reaches the correct goal-relevant hub in **8/8**.
3. FULL second physical action probes the correct unmodelled shortcut in **8/8**.
4. Frozen post-acquisition goal plan from the relevant hub chooses learned shortcut in **8/8**.
5. GENERAL_FRONTIER goal-specific shortcut success <= **4/8** within the same 2-interaction budget.
6. NO_GOAL goal-specific shortcut success <= **2/8**.
7. WRONG_GOAL follows the opposite goal shortcut in >= **6/8**.
8. BROKEN_GOAL_RECOGNITION requested shortcut success <= **2/8**.
9. BROKEN_GOAL_ROUTE requested shortcut success <= **4/8**.
10. PI_PHASE_GOAL_ROUTE requested shortcut success <= **4/8**.
11. Exact RESTORE returns FULL shortcut acquisition = **8/8**.
12. IRRELEVANT_BRANCH_LESION preserves requested shortcut acquisition in both motor permutations.
13. NO_TRANSITION_LEARNING post-acquisition shortcut plan = **0/8**.
14. Transferred drive weights remain unchanged.
15. Transition endpoints remain acquired abstract cells.
16. REAL factual frame only advances through factual interaction APIs.
17. Legacy graph transition count = 0.
18. Source guard PASS.
19. Human Protection v1.1 regression PASS.
20. G0-G17 / P1-P5 regressions PASS.
21. Release build PASS.

## Fresh qualification

Fresh G18 must randomize:

- abstraction hierarchy;
- start/hub/intermediate/goal state identities;
- two goal identities;
- navigation/shortcut motor IDs;
- known route topology;
- raw tuition and held-out bindings;
- transition tuition order.

At least:

- 10 sub-seeds;
- 80 goal-conditioned acquisition episodes;
- 80 post-acquisition goal plans.

Fresh must report information selectivity: irrelevant unknown transitions acquired before
the requested shortcut.

## Interpretation boundary

PASS establishes bounded goal-conditioned active information acquisition.

It does not establish:

- autonomous goal invention;
- semantic language goals;
- stochastic/POMDP information gain;
- autonomous invention of epistemic feature vocabulary;
- unrestricted scientific reasoning;
- AGI or consciousness.
