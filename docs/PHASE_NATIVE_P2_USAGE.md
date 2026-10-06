# P2 usage and interpretation

P2 extends the explicit phase-native backend. It does not silently switch the legacy G8/G9 backend or claim that every cognitive subsystem is physical-native.

## Ordinary interaction boundary

The implementation exposes these methods directly on EvoPhase:

```rust
// Assumes an attached raw-raster field and phase-native planning enabled.
// Acquisition input is a factual transition, not an answer or route script.
let error_before_update = evo.observe_phase_native_forward_transition(
    &observed_pre, opaque_motor, &observed_post, factual_value,
);

// A real observation starts the ordinary action loop.
evo.observe_initial_real(&observation, false);
if let Some((decision, prediction)) = evo.choose_phase_native_action_with_prediction() {
    // prediction is already available here, BEFORE the external world acts.
    // Only the trusted actuator/world shell performs this physical step.
    let (actual_post, actual_value) = world.step(decision.first_action);
    let sensory_error = evo.observe_phase_native_action_result(
        decision.first_action, &actual_post, actual_value,
    );
}
```

This is API illustration, not a standalone program or an externally executed deployment. The shell implements only the world/actuator and factual observations. It must not substitute a correct action when the carrier abstains.

P1 shared-synapse value propagation chooses the action. P2 shared-synapse forward propagation predicts its consequence. Factual result ingestion computes pre-update relational prediction error, optionally revises the learned network, then advances REAL. With learning disabled, only the externally supplied REAL observation advances; learned parameters do not.

`imagine_phase_native_actions(pre, actions)` can additionally predict a supplied opaque action sequence. Only the initial PRE is supplied. Intermediate future states are carried by the evolving imagined membrane buffer, not evaluator callbacks or stored target-vector copying. A missing or ambiguous learned link returns the valid prediction prefix. A sequence longer than the configured native horizon returns an empty result rather than silently exceeding the budget.

## What is generated

The output is a numeric sensory pattern reconstructed through receptor-to-sensory synapses. These connections learn from actual sensory values. Their weights store amplitudes, and phase offsets calibrate transmission. The continuation kernel cannot access receptor prototypes; input recognition outside that kernel still uses the existing HDC representation.

The P2 experiment teaches each sensory pattern at a single origin. A translated input can activate the same relational receptor, but the decoder regenerates the pattern in the **tuition coordinate frame**, not necessarily the current observer's absolute coordinates. Predictions are therefore evaluated by their relational encoding. There is no claim of learned spatial registration or translation-equivariant image rendering.

If the same relational receptor is trained on many unregistered pixel locations, this simple decoder can average incompatible pixel patterns. Supporting arbitrary moving cameras requires a learned/factually grounded binding or coordinate-frame mechanism; it is not fixed by the P2 test.

Generation here is learned associative sensory reconstruction and multi-step state continuation. It does not create novel objects, invent transition laws, or render unrestricted scenes. The bounded synchronous complex-current rule is inherited, not a learned reasoning algorithm or a demonstrated biological oscillator.

## Revision scope

P2 assumes fully observed deterministic state/action consequences. A new factual successor attenuates the contradicted afferent while retaining the old circuit addresses and contradiction records; a supported new successor circuit is acquired. Unchanged state/action links are not globally erased. Ambiguous simultaneous successor activation abstains.

This is not sufficient for stochastic worlds, hidden context, or deliberate retention of several competing laws. Those require a conditional belief model rather than applying deterministic overwrite/suppression indiscriminately.

## Full intelligence objective remains open

The implemented loop is a component of a persistent learner: experience -> acquired model -> forecast -> action -> observed error -> revision. It is not the entire intelligence objective.

A subsequent integrated acquisition task must remove the supplied transition curriculum: start cold, choose informative physical interactions inside the carrier, learn discriminative state representations, compose experience for goals not directly rehearsed, preserve old abilities while revising contradicted dependencies, and retain that state across restart. Measure goal achievement and interactions used, not only intervention tests. Do not confuse this independent Aeterna-v1 research line with the more developed separate Codex-AETERNA project.
