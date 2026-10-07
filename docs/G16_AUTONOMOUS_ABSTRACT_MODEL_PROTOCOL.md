# G16 — AUTONOMOUS ABSTRACT MODEL ACQUISITION

Status: **PRE-REGISTERED BEFORE G16 IMPLEMENTATION**

Date: 2026-10-07

## Why G16 exists

G15 statistically qualified planning through a learned physical abstract transition model, but every transition in that model was supplied as a prepared factual tuition set.

G16 removes that curriculum.

The target organism starts with acquired physical abstract state representations and a transferred learned P4 exploration drive, but with:

- zero native abstract transition circuits;
- zero legacy graph transitions;
- no route list;
- no action-law table inside cognition.

The organism must choose its own physical actions, observe only factual raw next-state + factual value, build the missing abstract transition model, discover the delayed reward, and then exploit its own frozen model on held-out raw bindings.

## Architectural claim

The causal loop must be:

```text
raw factual state
 -> physically active acquired abstract state
 -> learned physical exploration drive over missing/reachable abstract transitions
 -> opaque physical action
 -> environment factual next raw state + factual value
 -> phase-native abstract transition learning
 -> updated physical frontier/model
 -> ...
 -> frozen abstract model-based planning
 -> delayed-reward action choice
```

No evaluator route/action law enters cognition.

## Source meta-drive

Before the target lifetime, the P4 two-feature physical drive is meta-trained on a fixed bounded source family of short deterministic reset worlds.

Only the drive checkpoint transfers:

- two learned drive synapse parameters;
- drive observation count/config.

The following MUST NOT transfer:

- source receptors;
- source transition circuits;
- source sensory decoders;
- source REAL state;
- source world/action laws.

Target drive learning is frozen.

This asks whether the learned epistemic weighting learned in lower/raw state space transfers to acquired abstract state space.

## Target abstract substrate

Development witness uses:

- raw 20x20 raster;
- HDC dimension 192;
- 6 opaque motors;
- local relation radius 4;
- six acquired physical L2 abstract states;
- planning horizon 6;
- discount 0.95.

The six L2 states are acquired before the target transition lifetime by the same physical abstraction substrate used in G15.

No target transition circuit exists at target lifetime start.

## Abstract exploration features

The existing P4 inherited feature vocabulary is reused, not redesigned:

1. **DIRECT_UNMODELLED**
   - the current physical abstract state has no supported transition for this opaque action.

2. **REACHABLE_FRONTIER**
   - this action has a learned physical abstract transition whose successor can physically reach an abstract state that still has unmodelled actions.

The feature values are computed only from:

- current physically active abstract cell;
- acquired native transition circuits;
- actual phase-sensitive conductance;
- the set of acquired physical abstract cells at the same representational level.

The transferred P4 drive weights score those features through the same physical drive synapses.

No raw state ID, route depth, correct action, or evaluator frontier label is available.

## Target world family

Each world uses 4-6 of the six acquired abstract states.

At every nonterminal state exactly two target planning motors are behaviorally relevant; all six motors remain opaque to cognition.

From start S0:

- one action gives an immediate terminal factual value **0.55** and resets on the next episode;
- the other enters a delayed chain with immediate value 0.

At each intermediate chain state:

- exactly one of the two planning actions advances;
- the other resets to S0 with value 0.

The advancing motor is independently selected per state.

The final advancing transition yields factual value **1.0**.

Thus a greedy model that stops after discovering the 0.55 branch is suboptimal; the organism must deliberately gather deeper transition information.

Episode reset is an external environment boundary only. No transition tuple is injected during reset.

## Ordinary target interaction

For each physical interaction:

1. environment presents current raw state;
2. organism chooses an opaque action through the abstract learned-drive selector;
3. environment executes it;
4. organism receives only factual next raw state and scalar factual value;
5. organism updates the physical abstract transition circuit from that actual experience;
6. the drive TD update receives only whether the physical model structurally gained a new transition and the physical successor abstract state;
7. if environment terminates/resets, a new factual start observation is presented.

No prepared transition curriculum is allowed.

## Development worlds

Exactly 12 deterministic target worlds:

- chain lengths 3, 4 and 5;
- both start-action permutations;
- varied advancing-action patterns;
- varied raw binding offsets.

Per-world interaction budget: **80**.

After autonomous acquisition:

- freeze transition learning;
- freeze drive learning/readout for scoring;
- evaluate the start state at an unseen raw binding;
- FULL must choose the delayed first action using abstract model-based planning.

## Matched controls

1. **FULL_LEARNED_ABSTRACT_DRIVE**
   - transferred learned P4 drive;
   - physical abstract transition learning enabled.

2. **ZERO_DRIVE**
   - same physical abstract substrate/model learner;
   - drive weights remain zero.

3. **FRONTIER_LESION**
   - transferred learned drive;
   - REACHABLE_FRONTIER physical drive synapse lesioned after restore.

4. **DIRECT_ONLY**
   - may choose locally unmodelled current-state actions;
   - may not propagate reachable abstract frontier value backward.

5. **SEEDED_RANDOM**
   - external seeded uniform motor choice;
   - diagnostic only.

6. **NO_TRANSITION_LEARNING**
   - action selection receives factual interactions but cannot update target transition circuits.

7. **NO_GROWTH**
   - structural transition recruitment disabled during target lifetime.

## Required persistence / ownership

For FULL:

- target starts with native transition circuit count = 0;
- legacy graph transition count = 0;
- transferred drive weights are unchanged throughout target lifetime;
- target transition circuits begin/end only on acquired abstract cells;
- after acquisition a phase-native checkpoint/restart must retain the abstract transition model and still solve held-out exploitation after a fresh raw observation.

## Mechanism PASS thresholds

PASS requires all:

1. FULL autonomously reaches factual value 1.0 in **12/12** target worlds within 80 interactions.
2. FULL frozen held-out abstract planning chooses the delayed first action in **12/12**.
3. Restarted FULL chooses the same delayed first action in **12/12**.
4. FULL starts every target with 0 native transition circuits and 0 legacy graph transitions.
5. FULL transferred drive weights remain unchanged during target acquisition.
6. FULL target circuits have zero non-abstract endpoint violations.
7. ZERO_DRIVE autonomously reaches delayed reward in <= **3/12**.
8. FRONTIER_LESION reaches delayed reward in <= **3/12**.
9. DIRECT_ONLY reaches delayed reward in <= **4/12**.
10. NO_TRANSITION_LEARNING = **0/12** frozen held-out delayed plans.
11. NO_GROWTH = **0/12** frozen held-out delayed plans.
12. SEEDED_RANDOM is reported with acquisition cost but cannot count toward PASS.
13. Mean FULL interactions to first delayed reward <= **45**.
14. Source guard confirms abstract learned-drive selector has no evaluator route/action/depth mapping and no legacy planner/search fallback.
15. G0-G15 / P1-P5 regressions PASS.
16. Release build PASS.

## Causal interpretation boundary

PASS would establish bounded autonomous acquisition of a physical abstract transition model using a previously learned exploration drive, followed by physical abstract planning.

It would still not establish:

- task-specific goal-conditioned information value;
- stochastic/POMDP active learning;
- autonomous invention of the exploration feature vocabulary;
- arbitrary goals;
- AGI or consciousness.

A separate one-use fresh G16 pack is required before statistical qualification.
