# COLD-GOAL-FRESH1 — first-attempt self-acquired causal model and replanning

Date: 2026-10-10
Branch: `research/beyond-intel4`
Verdict: **BOUNDED DEVELOPMENT PASS, 12/12 first-attempt fresh role assignments**. NOT AGI, NOT novel-structure independent qualification.

## Evidence and temporal order

1. [OPEN development protocol](COLD_GOAL1_PROTOCOL.md) recorded before adding cold acquisition code, fixed source-family, no motor/path tuition, episode/action budgets and initial 4/4 criterion.
2. First development test [GitHub Actions 38025025890](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38025025890): **4/4** on the exposed seed `0xC01D_2026_6E7A_0001`, six physically acquired changing-state links in each arm, four-step intact frozen goal and changed-law recovery per arm. Earlier run 38024959744 was a Rust test-harness inclusion compilation failure, not a cognitive verdict; its error was fixed in the open test, not recast.
3. [Fresh1 preregistration](COLD_GOAL_FRESH1_PROTOCOL.md) with a NEW seed `0xC01D_2026_FE57_1010`, twelve new opaque visual/motor role assignments, **unchanged cognitive source** `1405d2f2af9b5ba3a1eb76481488f0f9d96c5173:src`, identical 192-episode and 16-action acquisition budget, 12-step test limit. Workflow checks the frozen tree SHA, the test blob hash, seed and first-run attempt before executing.
4. [First attempt frozen-source CI 38025296916](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38025296916): SUCCESS, `COLD_GOAL_FRESH1_SUMMARY passed=12/12 verdict=DEVELOPMENT_PASS`. Runner job 114134758031; [source test](../tests/cold_goal_fresh1.rs).

## Actual evidence, per world

| Arm | Unique factual transition edges acquired | Training successful goals /192 | Frozen goal reached | Frozen actions | After unforeseen interior edge failure | Physical lesion/restore and frozen U1 |
|---|---:|---:|---|---:|---|---|
| 0 | 6 | 185 | Yes | 4 | Reached goal, 8 actions | PASS |
| 1 | 6 | 185 | Yes | 4 | Reached goal, 11 actions | PASS |
| 2 | 6 | 186 | Yes | 4 | Reached goal, 10 actions | PASS |
| 3 | 6 | 184 | Yes | 4 | Reached goal, 7 actions | PASS |
| 4 | 6 | 186 | Yes | 4 | Reached goal, 11 actions | PASS |
| 5 | 6 | 185 | Yes | 4 | Reached goal, 9 actions | PASS |
| 6 | 6 | 185 | Yes | 4 | Reached goal, 11 actions | PASS |
| 7 | 6 | 186 | Yes | 4 | Reached goal, 11 actions | PASS |
| 8 | 6 | 186 | Yes | 4 | Reached goal, 11 actions | PASS |
| 9 | 6 | 185 | Yes | 4 | Reached goal, 7 actions | PASS |
| 10 | 6 | 185 | Yes | 4 | Reached goal, 7 actions | PASS |
| 11 | 6 | 184 | Yes | 4 | Reached goal, 8 actions | PASS |

The extra actions after drift are genuine protected exploratory and replanning acts, not an injected detour. In all 12 arms the originally selected interior motor actually returned a no-op, causing one observed contradiction. The succeeding behavior used both self-discovered detour motors and reached the original goal within the predeclared twelve-action bound. No blocked or unavailable actions during acquisition or heldout in any arm.

All 12 arms:
- The six changing-state causal edges began at **zero**. They were acquired exclusively through organism-selected motor invocations and factual PRE/action/POST. No evaluator-taught edge list, motor role IDs, correct route or exploration curriculum was passed into EvoPhase.
- The exact goal was presented as a raw, already recognizable sensory state. The current goal cell is context, not an imported transition.
- Each arm was one **separate** continuous 192-episode learner, not one lifetime spanning all 12 worlds.
- Physical source→motor synapse lesion of the acquired initial goal path removed the route, exact restoration returned it.
- Checkpoint/restart preserved the acquired model. The frozen U1 meta parameters did not change while evaluating the intact goal.
- Under drift, model learning was re-enabled and only one causal interior world transition changed. The organism executed a failed step, disabled contradicted physical model support, re-ran its bounded route search and reached the goal using other factual links.

## Architecture change

Opt-in `PhaseTemporalEvidenceState` now maintains the user-supplied raw goal's recognized carrier cell; it offers physical novelty frontiers using bounded per-state action-trial counts and conducting acquired transitions, and learns changed-state edges only from its own protected factual action outcomes. The same acquired graph supports goal-conditioned multi-step composition and model revision. Existing U1 arbitration and Human Protection remain in the path.

The bounded frontier search and goal route search are Rust algorithms operating on EvoPhase-owned physical synapses. **This is not evidence that the SNN invented BFS, its objective, or its exploration strategy.** The goal and fixed action budget are externally specified by the test; autonomous task selection remains unproved.

## Scientific limitations and next barrier

The twelve new assignments reuse the **same fixed deterministic six-state/six-motor topology**. This is a stronger transfer confirmation than replaying the exposed development seed, but is NOT transfer to unseen graph topology, stochastic transition laws, misleading/noisy state identity, open-world affordances, multiple simultaneous goals, or human-real-world safety. It starts from an experienced generic perceptual abstraction/U1 foundation, not a network devoid of all prior learning.

**Next high-value boundary:** preregister a genuinely different unknown topology, competing goals, action cost, changing transition uncertainty and longer dependencies; test whether one organism (not separate agents per arm) can autonomously discover and reuse/revise the acquired causal operations without a hand-coded exploration-frontier winner. Preserve the negative FRONTIER-1, STRUCTURE-1, and stochastic depth 2–5 qualification verdicts independently.
