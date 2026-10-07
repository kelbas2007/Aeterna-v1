# INTEL-1 REPAIR-2 — FROZEN EXPLOITATION PRECEDENCE

Status: **PREREGISTERED BEFORE REPAIR-2 SOURCE CHANGE**

Date: 2026-10-07

## Single permitted change

Inside `choose_phase_native_goal_active_action`:

When `PhaseNativeConfig.learning_enabled == false`, do not score epistemic unknown/frontier features.

Instead directly invoke the existing physical goal recurrence from the recognized abstract entry to the recognized raw goal and return its supported action.

When learning is enabled, G18 goal-conditioned active information acquisition remains unchanged.

## Rationale

Learning-frozen operation is an explicit exploitation mode. An unmodelled action has epistemic value only if the organism is allowed to acquire its consequence. With learning frozen, choosing it instead of an existing supported goal plan is inconsistent.

## Required witnesses

1. Burned W1 replay: known physical plan action equals runtime action at every replayed state and reaches goal <=20.
2. Goal-active learning-enabled G18/G19 behavior unchanged.
3. G16–G23, Human Protection and Repair-1 regressions PASS.
4. Release PASS.
5. No evaluator/world constant in production change.

After verification, freeze a new cognitive SHA and use a new authority seed for another INTEL run.
