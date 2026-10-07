# INTEL-1R1 — second frozen-core verdict

Date: 2026-10-07
Verdict: **FAIL — earliest remaining criterion failure is frozen reuse/exploitation.**

Run: `37660021524`
Frozen core: `397e57bcceae47b59275eac9fbfa865e2d6238da`
Authority seed / burned pack: `37660021524`.

Observed before failure:

- W1 unknown navigation acquired: PASS, goal reached in 15 actions;
- W1 frozen translated revisit: FAIL, 20-action budget exhausted;
- W2 unknown causal-machine sequence acquired: PASS, goal reached in 13 actions;
- W2 frozen translated revisit: FAIL, 20-action budget exhausted.

The run later encountered NoSupportedAction before scoring W3, but W1 reuse is the earliest frozen INTEL criterion already missed and therefore the next causal bottleneck.

## Burned-pack diagnostic

A separate diagnostic replayed the burned W1 law without changing cognition.

The physical goal planner itself returned the correct route:
- state0 -> action 5, predicted value 0.857375, depth 3;
- state1 -> action 3, predicted value 0.9025, depth 2.

Unified runtime selected action 5 correctly at state0, then repeatedly selected action 4 at state1 while labeling the mode GoalDirectedAction.

There were no compositional/perceptual/context candidates at that state.

Root cause: `choose_phase_native_goal_active_action` continues to score epistemic unknown/frontier actions even when native learning is frozen. Thus an unmodelled action can override an already-supported goal plan during explicit exploitation.

Knowledge retention is therefore present; exploitation arbitration is wrong in frozen mode.

## Next repair rule

No G24 is opened. Repair-2 may ONLY make frozen goal-active mode prefer the existing supported physical goal plan, with no new world/task information.
