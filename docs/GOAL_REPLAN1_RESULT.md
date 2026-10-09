# GOAL-REPLAN-1 — factual 4-step goal planning and changed-law recovery

Date: 2026-10-09  
Status: **OPEN DEVELOPMENT PASS** (mechanism 4/4; frozen-source fresh transfer 12/12). **NOT cold autonomous acquisition or AGI.**

## First integrated mechanism experiment

[Goal-conditioned physical replanning GitHub Actions 37973110682](https://github.com/kelbas2007/Aeterna-v1/actions/runs/37973110682) PASSED 4/4 independently shuffled opaque assignments, on research code commit `ec3bf3827e449ad37569e90e4f59842544b927e3`. Test: [`goal_replan_physical.rs`](../tests/goal_replan_physical.rs).

The same physical EvoPhase organism per arm executed:
- A **four-step goal path** assembled by a bounded generic search through previously acquired, physically conducting state→motor→state transitions.
- A second episode in which an intermediate transition unexpectedly returned the **same state**. The factual POST physically retired the contradicted transition, not its unrelated neighboring links.
- A recomputed alternate route reusing two acquired but **never full-route demonstrated** detour transitions. Goal was reached with six actions, including the contradicted action.
- Real actuation through Human Protection and the ordinary `ScientificRuntime::step_unified`; no motor sequence or causal law-change flag was supplied to cognition.
- Causal lesion of the essential first synapse abolished the physical route and exact restoration recovered it. Checkpoint/restart preserved the original acquired policy and U1 weights.

### Individual development trajectories

| Arm | Initial 4-step route | After hidden law change: 6 executed actions | Achieved both goals |
|---|---|---|---|
| 0 | 4, 3, 0, 2 | 4, 3 (no-op), 5, 1, 0, 2 | Yes |
| 1 | 2, 4, 1, 0 | 2, 4 (no-op), 5, 3, 1, 0 | Yes |
| 2 | 1, 5, 3, 4 | 1, 5 (no-op), 0, 2, 3, 4 | Yes |
| 3 | 1, 2, 3, 4 | 1, 2 (no-op), 5, 0, 3, 4 | Yes |

## Source-frozen first-attempt fresh transfer

[Pre-registered protocol](GOAL_REPLAN_FRESH1_PROTOCOL.md): new unused seed `0x6A17_2026_11FA_9001`, **12** new opaque state and motor role assignments, source `src/` tree pinned exactly to `ec3bf3827e449ad37569e90e4f59842544b927e3:src`.

[First-attempt GitHub Actions 37973556878](https://github.com/kelbas2007/Aeterna-v1/actions/runs/37973556878): **SUCCESS, 12/12**. The workflow verified source-tree hash, test blob hash and preregistered seed before executing the complete untouched test. Log:

```text
GOAL_REPLAN_FRESH_SUMMARY intact=12/12 drift_recovered=12/12 verdict=DEVELOPMENT_PASS
```

Each fresh arm performed the 4-step intact physical route, saw exactly one unexpected factual no-op, changed its action plan and executed the 6-step path to the same goal. Checkpoint and physically necessary first-link lesion/restore were tested. All 12 passed without altering the frozen cognitive source.

## What this proves and what it does not

**Proved within this bounded mechanism family:** previously acquired physical transition knowledge can be composed online into a goal-directed sequence, used as normal protected external actions, invalidated on factual contradiction and recomposed from the current state. This holds for 12 never-before-tested opaque motor/state assignments under the frozen implementation.

**Not proved:** discovery of all component transitions from cold unguided experience in the goal-planning experiment; separate logical prerequisites or partial-order effects beyond the fixed 6-state topology; unbounded horizons; general AGI; multi-goal long-lived adaptation; robust/noisy transition uncertainty; deployable physical-world safety. The source includes a bounded BFS search in Rust over EvoPhase-owned conducting physical links. This is evidence of goal-conditioned causal planning in the narrow experiment, **not proof that SNN synaptic dynamics themselves invent the graph-search algorithm**.

The prior stochastic depth 2–5 sensing diagnostics remain 2/4 end-to-end under their own criteria, and the earlier STRUCTURE-1 first-attempt FAIL 0/4 remains intact. These new deterministic goal-route experiments do not rescore either result.

## Next independent boundary

A single persistent organism must **autonomously acquire its own prerequisite transitions**, compose a new multi-step goal path with no guided edge acquisition, then revise a law-shifted route while keeping state/action evidence truthful. Pre-register a new and structurally different cold task and causal controls before any independent scientific qualification.
