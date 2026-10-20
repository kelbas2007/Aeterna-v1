# OBSERVABILITY-LIMIT1 — is five-action failure caused by missing sensory distinctions?

Date: 2026-10-20. PR11 previously integrated source `3d5c45b90471226704e6bed716005656dc87ac34`.
**Preregistered BEFORE implementing ANY new evaluator code or inspecting outcomes.**
This is a *developer-disclosed causal sensor ablation*, not a fresh
source-blinded science exam or a cognitive model improvement.

## Hypothesis and scientific constraint
In a deterministic hidden-precondition world in which ALL physically
executed preparation actions map to the exact same visible progress
coordinate 0, no observational evidence separates different latent
preparation stages until final success/failure. Trying an isolated
motor and receiving the SAME observation cannot identify hidden state
no matter how intelligent the motor chooser is. The learner may need
either a discriminating multi-motor experiment or an additional
legitimate sensor. Active automata model-learning depends on actual
observable output distinctions, resettable membership experiments
and counterexamples (background: AALpy and LearnLib).

This experiment tests the consequence of **providing a true separate
sensor**, not whether AETERNA can invent or request one. A privileged
evaluator-internal stage label is NEVER handed to cognition as a
semantic rule. It is a numeric actual POST channel, which a physical
world would need to instrument.

## Precommitted matched controls
- Same 36 SOURCE-DISCLOSED worlds × 24 goal episodes with maximum
  18 actual actions, 12 distinct motor-role maps for 4-step and
  5-step worlds, 6 each for 3-step and direct-goal controls.
- Both arms configure *identical* two-sensor `ScientificRuntime`
  with the already implemented complexity-need gate and
  contextual-history exploration. Same zero factual history, zero
  U1 native weights, no scheduled teacher action, all four motor
  offers each step, no latent role labels.
- Goal mask `[Some(1),None]` (only actual progress is a goal).
- Blind arm: second channel is always actual flat 0. Sensored arm:
  second channel is the world's **actual latent-stage integer** after
  each physical motor (value is not an inferred/simulated prediction).
  Sensored stage is initialized/reset to zero; no information from
  later unseen actions, future stage or motor identity leaks.
- Sensor precision is perfect, **not** a model of noisy realistic
  perception. Difference in agent performance is observational
  availability plus its endogenous learned policy.
- Every motor permitted by HumanProtection independently; each
  POST actual cost=1 and journal entry recorded; external reset
  explicitly acknowledged. Same default bounded work and checkpoint
  roundtrips.
- The world family, role maps, horizons and parameters are public and
  reused from earlier experiments: **NO independent qualification**.

## Precommitted measurements and gates
Measure goal successes, physical actions, first-six vs last-six
successes, native U1 selected contextual motor probes, completed 3/4/5
macros, and journal/checkpoint integrity by family and by world,
preserving complete actual traces and protocol source SHA.

**Positive SENSOR-UTILITY verdict only if:**
- Sensored five-action family > blind five-action family in successes,
  and at least 12/288 five-action successful episodes (prior disclosed
  main source with progress-only reached just 8/288);
- no lower success rates in the 3-action and direct-goal families;
- overall sensored goal solves > blind goals and external action
  cost does not rise more than 10%.

If the additional channel fails, record a strict
`NEGATIVE_SENSOR_ADDITION_UNDER_EXISTING_U1` (no adjusting the
sensor mapping, reward or seeds until this verdict is logged).
Positive means richer **environment-provided observation** helps.
It does not show the cognitive organism self-developed a sensor,
requested it, constructed a hidden-state model, or solved a third-party
POMDP. A future real-sensor/inspection-action trial is needed to
demonstrate agent-directed active perception.

## Next possible barrier depending on result
- If sensor utility positive, learn a *need-based* sensor query
  conditioned on rival state explanations and actual cost.
- If negative, the bottleneck lies also in learned state abstraction,
  prediction, credit or action selection, not solely missing data;
  investigate why a true available state signal is ignored.
Do not merge a scientific claim based on source-disclosed tuning.
