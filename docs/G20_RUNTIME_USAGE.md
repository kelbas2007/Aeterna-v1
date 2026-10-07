# Using the continual reasoning runtime

Entry point: `aeterna_v1::scientific_runtime::ScientificRuntime`.
Implementation: `src/scientific_runtime.rs`.
Deterministic integration test: `tests/g20_continual_reasoning.rs`.

## Lifecycle

1. Construct one `ScientificRuntime::new(acquired_evo)` from a native EvoPhase. The current integration witness uses acquired representations and prior factual transitions; construction does not synthesize knowledge for a new domain.
2. Supply an actual observation through `observe_external(&raw_observation)`.
3. Set a raw, already recognizable goal through `set_goal(&raw_goal)`.
4. Call `step(assess, execute)` repeatedly. `assess` receives an action proposal and returns optional external hazard evidence. `None` is a block. `execute` receives an authorized action and returns the actual raw next observation or an execution error.
5. Change the goal without replacing the runtime or retraining the model. The physical environment continues from its actual current position.
6. `restart_cognition()` restores only an in-memory native checkpoint. It preserves the runtime's protection latch, goal and monotonic audit sequence, but deliberately clears the current observation. Obtain and supply a new actual observation before acting again.

`StepOutcome::GoalReached` means the sensed abstract state matches the raw goal. It does not automatically invent another goal or actuate a stop motor. A deployment must separately define safe stop behavior.

`StepOutcome::Blocked` never calls `execute` and never supplies an artificial observation to the learner.

`StepOutcome::ExecutionFault` latches the runtime gate and requires renewed external sensing. The adapter must not blindly retry an action whose outcome is unknown. Returned execution errors and unusable POST observations are covered; process aborts, panics, power loss and device firmware behavior are not a qualified fault-containment boundary in this version.

`external_operator_reset()` is an explicit host operation, not a native cognitive motor. The application must authenticate who can call it.

## Inspecting the mechanism

The runtime proposes native rival discrimination first, then falls through to the inherited goal-conditioned active selector/planning path. This arbitration rule is programmer-defined. It contains no task labels, evaluator action mapping, or additional graph solver.

Successful factual POST observations enter the inherited physical rival-revision update. In this deterministic witness a contradicted successor loses executable weight but remains structurally addressable with revision provenance.

The bounded audit ring stores 256 operational events. This ring is not the organism's learned memory and not a durable security log. `organism()` exposes read-only diagnostics; no mutable carrier or protection handle is exposed by the wrapper.

Run the integration checks without consuming any fresh qualification pack:

```sh
cargo test --release --test g20_continual_reasoning g20_ -- --nocapture
cargo test --release --test human_protection -- --nocapture
```

The G20 test reuses the G19 abstraction/source-drive fixture. Imported historical G19 tests are still ordinary regression tests, and their fresh-pack test remains ignored by default. G20's two deterministic lifetimes must not be advertised as 16 independent domains or a statistically representative intelligence benchmark.

## Deployment boundaries

This software wrapper restricts the callback it owns. It is not a security sandbox against a privileged host that bypasses it. Give cognition no separate hardware handle, file/network command route or operator credential. Do not deploy this model on hazardous equipment on the strength of these tests.

The existing Human Protection thresholds operate on supplied numbers. No person detector, calibrated risk estimator, independent emergency relay, actuator deadline, authenticated reset service or durable restart interlock is added by G20. The numeric constants are not approved safety limits.

This version accepts recognizable raw states and deterministic local transitions. New perceptual vocabulary, in-lifetime abstraction invention, noisy belief revision, durable disk persistence and open-ended goal formation remain separate engineering/research requirements.
