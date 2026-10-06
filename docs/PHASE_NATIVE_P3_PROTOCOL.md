# P3 — AUTONOMOUS PHASE-NATIVE ACQUISITION

Status: **PRE-REGISTERED BEFORE P3 IMPLEMENTATION**

Date: 2026-10-06

## Question

Can one cold Aeterna-v1 organism acquire the transition knowledge required for a delayed goal by selecting its own physical interactions, without receiving a prepared transition curriculum or route script, and then preserve that acquired capability across a restart-like checkpoint restore?

P3 is a bounded autonomy gate. It is not an AGI claim.

## Starting state

Each scored organism starts with:
- raw 12x12 perception attached;
- opaque motor IDs;
- phase-native P1/P2 enabled;
- no learned phase-native receptors for the world;
- no learned phase-native transition circuits for the world;
- no graph planner;
- no prepared transition tuples;
- no demonstrated solution route.

The environment alone contains the hidden transition law.

## Allowed external shell

The shell may:
- present the current raw raster;
- execute the opaque motor selected by the organism;
- return the factual POST raster;
- return factual bounded value / Need;
- reset the physical episode after a terminal state.

The shell may not:
- choose the organism's action;
- enumerate transition tuples for learning;
- reveal the correct route;
- reveal the correct motor at any state;
- provide state IDs, graph edges, distance-to-goal, or hidden-law labels to cognition.

## Phase-native exploration requirement

P3 must add an explicit phase-native autonomous acquisition path.

The action selector must operate from acquired state inside EvoPhase and use the same physical cells/synapses as P1/P2.

Required behavior:
1. At a newly observed state, unmodelled opaque actions have intrinsic epistemic value.
2. After a transition is factually acquired, that action is no longer treated as unmodelled at that state.
3. Novelty at a deeper known receptor can propagate backward through already acquired phase-native successor connections.
4. This allows the organism to traverse known transitions back toward a frontier that still contains unknown actions.
5. When no reachable frontier remains and reward knowledge exists, ordinary P1 value propagation may exploit the acquired model.

A host-side BFS/DFS/frontier, route table, state graph or correct-action map is forbidden in the production P3 selector.

## Autonomous world family

The primary mechanism family is a reset-chain world.

For a hidden chain of length 3–5:
- each nonterminal state has three opaque actions;
- one hidden action advances one state;
- the other two return to the start state;
- the advancing motor may differ at every state;
- reaching the terminal state produces factual value 1.0;
- no other transition produces positive value.

The organism is never shown the advancing action.

Because wrong actions reset the environment, reaching deeper unexplored states repeatedly requires reuse of already acquired transitions. A selector that only tries unknown actions at the immediately visible state is insufficient once the start state's actions are all known.

## Acquisition phase

The cold organism receives an interaction budget.

At each physical step:
1. current REAL raster is already present;
2. the organism selects an action through the P3 phase-native autonomous selector;
3. the shell executes that opaque motor;
4. the factual POST/value are passed to `observe_phase_native_action_result`;
5. the learned P2 forward model is updated only from that actual interaction.

Acquisition stops on first terminal reward or budget exhaustion.

Report:
- physical interactions to first reward;
- number of acquired receptors;
- number of acquired transition circuits;
- no legacy planning transitions.

## Goal-use evaluation

After first reward:
- learning is frozen;
- start from an unseen absolute raster translation;
- the organism must reach the same factual goal using ordinary phase-native planning/action prediction;
- no exploration hints are allowed during this scored exploitation episode.

The evaluator checks only factual task success and physical interaction cost.

## Restart persistence

After successful acquisition:
1. create an opaque phase-native checkpoint containing acquired physical state;
2. create a new cold `EvoPhase` with matching substrate dimensions;
3. restore the checkpoint;
4. do not copy current REAL observation;
5. freeze learning;
6. solve the held-out translated start again.

The restored carrier fingerprint must equal the pre-checkpoint learned fingerprint before a new REAL observation is loaded.

A plain clone of the original `EvoPhase` is not accepted as the persistence witness.

## Factual revision extension

After the initial world is solved, a changed-law variant inserts one previously unseen detour state on one formerly valid advancing transition.

The same formerly correct motor now factually leads to the detour. The detour has three opaque actions and exactly one reconnects to the old chain; the others reset to start.

Requirements:
- the organism first attempts behavior based on the old acquired model;
- factual prediction error/revision records the changed successor;
- no route replacement is supplied by the evaluator;
- autonomous P3 exploration must discover the detour's useful action;
- the organism must regain the goal within a bounded additional interaction budget;
- a frozen matched copy must remain unable to acquire the new detour transition;
- unchanged old chain knowledge must remain usable.

## Controls

Matched controls:
1. **NO_LEARNING** — phase-native learning disabled from cold start.
2. **NO_STRUCTURAL_GROWTH** — no new phase-native receptors/circuits.
3. **ZERO_PHASE_LEARNING** — phase offsets cannot be acquired.
4. **ZERO_WEIGHT_LEARNING** — transition strengths cannot be acquired.
5. **RANDOM_ACTION** — same environment and budget, but evaluator supplies a seeded uniform random action only as a diagnostic baseline. It is not cognition and cannot count as P3 success.
6. **DIRECT_ONLY_EXPLORATION** — diagnostic selector may choose unknown actions only at the current receptor and cannot propagate frontier novelty backward. This tests the necessity of multi-step novelty propagation in reset worlds.

## Deterministic preflight acceptance

Before any one-use fresh qualification, at least 12 deterministic hidden worlds must satisfy all:

- cold native organism has zero learned world circuits before interaction;
- no transition curriculum function is called;
- autonomous native selector chooses 100% of acquisition actions;
- native organism reaches first reward within 60 physical interactions;
- frozen exploitation solves from an unseen translation;
- checkpoint-restored cold instance also solves from the unseen translation;
- no legacy graph transition table exists;
- DIRECT_ONLY_EXPLORATION is strictly worse on at least one reset-chain world;
- NO_LEARNING, NO_STRUCTURAL_GROWTH, ZERO_PHASE_LEARNING and ZERO_WEIGHT_LEARNING do not match full P3 success;
- changed-law detour is autonomously repaired within 40 additional interactions;
- frozen matched copy fails to learn the detour;
- unchanged transition predictions remain valid;
- full regressions and Release build pass.

## Fresh qualification

Only after deterministic preflight passes may a one-use fresh P3 pack be enabled.

Minimum pack:
- 10 authority-derived sub-seeds;
- 8 worlds per sub-seed;
- N = 80;
- chain lengths 3–5;
- independent opaque action permutations by state;
- unseen absolute translations for exploitation;
- changed-law detour location/action derived from the authority seed;
- world parameters/digest logged before acquisition/scoring.

PASS thresholds are to be frozen in a separate fresh P3 qualification section before its first authority run. The deterministic preflight is not itself a statistical generalization claim.

## Interpretation boundary

A P3 PASS would establish bounded autonomous acquisition and reuse of a deterministic transition model under explicit physical interaction budgets.

It would still not establish:
- arbitrary open-world exploration;
- learned intrinsic motivation;
- stochastic/POMDP reasoning;
- language;
- unrestricted concept invention;
- recursive program synthesis;
- AGI or consciousness.
