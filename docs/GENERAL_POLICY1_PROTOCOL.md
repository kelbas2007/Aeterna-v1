# GENERAL-POLICY-1 — replacement rather than another task-specific mechanism

Date: 2026-10-10. **OPEN DEVELOPMENT**; architecture-source and seeds declared before any external result, not sealed independent science.

## Research question and falsification

EvoPhase's 7/16 externally rewarded heldout Farama tasks in
[EMBODIED-WORLD-2](EMBODIED_WORLD2_RESULT.md) were achieved by hand-authored
Rust choices between "rare object", translation, turn and exploration.
This is NOT a sufficient general-intelligence substrate. Can a single
task-agnostic learnable controller, receiving ONLY raw observation,
its actually executed opaque action, the subsequent raw observation
and external reward, acquire a transferable policy *without*
these manually authored object/spatial decisions?

The proposed **architectural replacement** activates
\`PhaseNativeState.general_policy\` and makes the
\`collect_phase_native_unified_proposals\` path return ONLY this policy's
motor: it bypasses all authored object, navigation, factor, symbolic,
instruction or world-conditioned proposal sources. It does not add
another contender to the prior selector; it **replaces the source of
action proposals**. U1 and Human Protection still screen EVERY motor.
It lives in the same physical EvoPhase carrier and checkpoint.

## General learning rule — what is truly still programmed

- Generic binary raw sensory input → fixed signed distributed hash
  representation of active channels (96-dimensional, no object class,
  geometry, task name, mission, agent pose, motor name or map).
- Each opaque motor owns learned value weights and an action-conditioned
  predictive feature mean. Novel transition error provides small
  task-agnostic intrinsic curiosity without fabricated terminal reward.
- Eligibility traces across preceding real actions (decay 0.91) allocate
  a **real externally delivered delayed reward** to a causal sequence.
  This is a programmed TD-like plasticity heuristic, *not* a
  learning rule invented by a neural network.
- Real phase synapse to every motor is a necessary actuation gate.
  Artificial zero-weight lesion must prevent its physical motor.
  Checkpoint persists acquired value/prediction weights and links.
  Transient visit/eligibility history resets across episodes/restarts.
- Native learning freezes during heldout. Episode-local action
  attempts may continue to avoid identical-frame perseveration.
  All externally issued actions still pass U1/HP.

Unlike authored exploration routines, there is **no object ontology,
key/door affordance, hand-selected pickup/toggle/forward motor,
map, BFS, camera-rotation search, word or external task-specific
feature** in the replacement controller. This is a genuine
change of the control architecture with an honest constraint:
the distributed hash and reward/curiosity updates themselves
remain fixed Rust algorithms, not self-emergent AGI.

## Independent outside benchmark, matched intervention

Use original independent Farama MiniGrid 3.1.0 code, the usual real
external partially observed categorical frame of 588 binary sensor
bits. 4 world families:
\`MiniGrid-Empty-5x5-v0\`, \`MiniGrid-DoorKey-5x5-v0\`,
\`MiniGrid-MultiRoom-N2-S4-v0\`, \`MiniGrid-Unlock-v0\`.
48 interleaved learning episodes (training seeds 62000..62011)
in ONE persistent Rust organism per arm. Native checkpoint,
then frozen heldout 16 episodes (63000..63003 each).
256 real permitted motor actions budget per episode.

Controls run SAME source, world order and seeds:
A. Opt-in general learned replacement, no authored task strategies.
B. Previous stronger embodied Rust hand-authored reference;
C. Matched seeded random actions.
No training or heldout object staging, carrying interventions,
correct motor examples, goal positions, mission strings,
internal simulator map/agent location or action meanings enter
cognition. The simulator evaluates actual external terminal
reward plus hidden-side physical event counters; those counters
are never transmitted to cognition.

**Strict OPEN development success**: >=6/16 actual frozen complete task
successes, strictly more than both matched authored embodied baseline
and random, and >=1 heldout DoorKey or MultiRoom complete task reward.
Also basic native physics/eligibility/lesion/frozen controls PASS,
and actual U1+HP executed physical actions. Otherwise record
\`DEVELOPMENT_FAIL\`, even if unsupervised novelty/updates increase.

This is one development run on reserved seeds, not a qualification
against source-unconsumed independently heldout tests. Do not tune
against these first results and then claim a clean independent gate.

## Why this is important even if it fails

If successful, it would show learned generic action preferences and
delayed reward credit can REPLACE heavily engineered strategy-specific
control on a small independent external task battery. It would
not establish learned object language, pixel RGB vision, arbitrary
new tool invention or AGI.

If the source compiles but frozen tasks fail relative to the
handwritten baseline, this is a **negative architectural result**:
the proposed generic plasticity/representation lacks sufficient
generalization or temporal causal structure. The correct next
step would be to rethink the REPRESENTATION and substrate learning,
not add hard-coded key/door routes to win a benchmark.
