# INTEL-2 FAIL1 ERRATUM — INVALID_AFTER_SEAL W3 EVALUATOR LIFECYCLE

Date: 2026-10-08

## Historical run

Original frozen verdict run: `37687243350`
Frozen cognitive core: `07b44fb8e00837568bc9655e760eb031d1944dff`
Burned authority seed: `37687243350`

The original result file recorded INTEL-2 FAIL at World C after `NoSupportedAction`.

That historical run and its burned pack remain preserved.

## Post-verdict diagnosis

A diagnostic replay used the SAME burned authority seed but was explicitly non-scoring and restored the exact frozen cognitive source before execution.

Diagnostic run: `37727919816`.

Immediately before the line that the evaluator later reported as `World C unified action: NoSupportedAction`, the frozen runtime reported:

```text
INTEL2_POSTMORTEM_C_GOAL_REACHED k=7 state=16 target=16 trials=2
```

The evaluator helper `step_u` maps `StepOutcome::GoalReached` to `Err(RuntimeError::NoSupportedAction)`.

World C acquisition intentionally continued across repeated junction trials, but after the organism successfully reached the raw goal the evaluator did not begin a new episode. It called `step_unified` again while the current factual state already equalled the goal. The runtime correctly returned `GoalReached`; the evaluator misclassified that terminal condition as absence of a supported cognitive action and aborted.

Therefore the original W3 stop is **not evidence that the frozen unified architecture lacked a supported proposal at that point**.

## Correct scientific classification

The original pack was exposed at `INTEL2_SEAL`, so it remains permanently burned and cannot be converted to PASS.

Correct classification of the W3 stopping event:

**INVALID_AFTER_SEAL_EVALUATOR_TERMINAL_LIFECYCLE**

The project must not advertise the old run as an INTEL-2 PASS.

It also must not continue to cite that specific `NoSupportedAction` as a demonstrated cognitive failure.

## Prospective correction

A new independently seeded INTEL-2R1 may be run on the identical frozen cognitive SHA.

Only World C evaluator episode handling changes:

- after a junction decision reaches either terminal goal or terminal dead state,
- the environment begins a new episode by supplying a fresh reset-state observation,
- no host-selected motor is fabricated,
- no terminal->reset transition is learned,
- cognitive state, U1/U2 ecology, representation learning and all other world laws remain persistent.

This correction must be preregistered before the new authority pack is exposed.

All original thresholds remain unchanged.
