# G18 — goal-conditioned active abstract reasoning

Date: 2026-10-07

## Verdict

**MECHANISM PASS — current goal selectively controls which missing physical abstract fact is acquired.**

Workflow: `37627583024`  
Scored source: `c60e518c233f7333330922c9e15a8c22a567a1d5`  
Protocol: `docs/G18_GOAL_CONDITIONED_ACTIVE_REASONING_PROTOCOL.md`

## Result

Across both opaque motor permutations, both requested goals and two held-out raw bindings per goal:

- FULL requested shortcut acquisition: **8/8**;
- correct first navigation to requested-goal hub: **8/8**;
- correct second probe of requested unknown shortcut: **8/8**;
- frozen post-acquisition plan chooses learned shortcut: **8/8**;
- GENERAL_FRONTIER goal-specific acquisition: **4/8**;
- NO_GOAL: **0/8**;
- opposite valid goal targets the opposite shortcut: **8/8**;
- BROKEN_GOAL_RECOGNITION: **0/8**;
- BROKEN_GOAL_ROUTE: **0/8**;
- PI_PHASE_GOAL_ROUTE: **0/8**;
- exact RESTORE: **8/8**;
- IRRELEVANT_BRANCH_LESION: **8/8** preserved;
- NO_TRANSITION_LEARNING post-plan shortcut: **0/8**;
- source guard PASS;
- Human Protection, G16, G17, full optimized regressions and Release PASS.

## Meaning

G16 asks: "what is unknown?"

G18 adds: "which unknown fact matters for THIS current goal?"

The same transferred two-weight P4 drive is retained. G18 does not add a task-specific learned exploration weight. A physically recognized raw goal produces a backward relevance field through already-known abstract transition synapses. That field gates unknown/reachable frontier value before the existing learned drive weights score it.

The world contains two simultaneously unmodelled shortcuts. FULL acquires only the shortcut relevant to the supplied raw goal within the two-action budget. Goal-agnostic frontier exploration can target only one of the two goals under the same budget.

## Boundary

This is bounded goal-conditioned active information acquisition. It is not autonomous goal invention, stochastic experimental design, language semantics, unrestricted science or AGI.
