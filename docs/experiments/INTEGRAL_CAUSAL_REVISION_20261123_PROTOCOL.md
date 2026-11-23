# Integral causal revision — frozen protocol, 23 November 2026

## Question and scope

Can the existing, unified AETERNA construction runtime acquire information by
choosing its own actions, encounter a contradiction after an unannounced world
change, revise its forecasts and recover goal performance? This is a new,
bounded engineering test of a disclosed synthetic world family. It is not an
independent external intelligence benchmark or a claim of general scientific
reasoning. The world resembles the existing causal sensor family, but adds a
real hidden rule reversal, prospective evidence and matched controls.

Production source is fixed to main commit
`01a35d0c69c3dec0340c753456ada4a7b7f221da`. No production, existing test, dependency
or toolchain file is changed. Only the evaluator, scorer, this protocol and a
dedicated workflow are added on `experiment/integral-causal-revision-20261123`.
The first execution commit, binary SHA256, workflow run ID, seed and all world
outcomes will be recorded. A failed capability verdict is retained unchanged.

## Cognition boundary

One `ScientificRuntime` starts without world experience in each of eight worlds.
It uses the existing `new_constructing`, agency-aware credit, self-directed
control and `CausalConfig` APIs, all enabled before the first action. There is
no additional solver, language model, host-written research policy or direct
training stream. Every non-random action uses the same protected
`step_constructing_with_action_limit` call and a single factual callback.

The actor receives only the initial/current observation, the same high-level
goal, all four equally available one-unit-cost actions, remaining episode
actions and the actual POST/reward/cost. It receives no action-role mapping,
hidden bit, change flag, true rule, preferred motor, expected outcome or
correct-action label. Read-only diagnostic beliefs are captured before action
execution and are never fed back as advice. Their observation/computation
overhead is outside the actor's configured planning budget.

Settings: observation width 2; four motors; planning depth 4; planning work
120,000 per choice; history depth 3; journal capacity 4,096; minimum training
128; synthesis search interval 1,000,000; other causal settings are defaults.
The long synthesis interval follows the existing causal test configuration:
this evaluates integrated causal memory and U1 arbitration, not acquisition of
a new VM program. The construction memory and planning contain authored
software; the experiment is not a claim that every operation is SNN-owned.

## World

Each world independently permutes four opaque motor IDs:

| Physical role, known only to environment/scorer | Consequence |
|---|---|
| Informative sensor | Reveals a stable opaque code for the episode's hidden bit |
| Distractor sensor | Returns an independent random binary measurement |
| Actuator 0 | Ends the episode, succeeding iff `0 == hidden XOR rule` |
| Actuator 1 | Ends the episode, succeeding iff `1 == hidden XOR rule` |

Seven unique opaque observation symbols denote start, two useful readings, two
noise readings, success and failure. Observation field 0 stays at namespace 0;
field 1 carries the symbol. The goal is exact equality of field 1 to the
success symbol. Symbol distance carries no physical progress information.

Every episode starts at the ordinary start observation, never an intermediate
learning state. The hidden bit varies independently by episode. Repeated useful
readings agree within an episode; distractor readings are independent. A terminal
actuator ends the episode whether successful or not; otherwise the episode ends
after four actions. Every actual action costs one. Reward is one only for an
observed terminal success and zero otherwise. Episode resets are factual world
resets and use the existing restart API; memory is retained.

The episode schedule and budget are experiment conditions, not a sequence of
research actions. The evaluator never selects a probe or orders a training
action. The same action interface is used for every episode.

## Fixed exposure and controls

The first-attempt GitHub workflow run ID supplies a fresh master seed after code
and protocol have been committed. Eight indexed worlds are derived without
selection, retries or rejecting difficult instances. Motor and observation
permutations, latent bits, random-control actions and noise use separated random
streams. Matched arms see the same latent/noise schedules, irrespective of which
actions they select.

| Phase | Episodes per world | Rule | Learning |
|---|---:|---:|---|
| `acquire` | 256 | 0 | Enabled, all actions self-selected |
| `original_held` | 64 | 0 | Frozen clone of acquired state |
| `adaptive` | 512 | 1 | Continuing from the acquired checkpoint |
| `frozen` | 512 | 1 | Same acquired checkpoint, model and policy learning frozen |
| `recovered_held` | 64 | 1 | Frozen, restored post-adaptation checkpoint |
| `shuffled_cue_held` | 64 | 1 | Same post-adaptation checkpoint, useful reading independent of hidden bit |
| `stale_held` | 64 | 1 | Frozen continuation checkpoint |
| `random_held` | 64 | 1 | Uniform random motor, same environment and maximum budget |

The unannounced inversion is applied at the ordinary episode boundary after
acquisition. No extra reset, forgetting, new goal or hint accompanies it. A world
that fails acquisition still completes every subsequent phase and stays in all
denominators. Original validation is a clone and cannot teach the continuing
actor. Adaptive and frozen arms have the same starting acquired state and
512-episode opportunity. Evaluation clones cannot contaminate one another.
The sensor control preserves possible symbols and all action affordances while
destroying their relationship with success. Random control includes its actual
timeouts and actions, rather than an assumed 50% score.

## Evidence, integrity and success gates

For every step, save PRE, all prospective action outcome distributions, support,
unknown mass, the last three within-episode transitions, selected action/source,
native proposal fields, POST, cost, reward, event and learned fingerprints.
Hidden state/roles are added only to the evaluator's trace for independent
scoring. Save all failures and timeouts. Preserve per-world and per-phase goal,
action, measurement, source and memory reports.

Integrity requires exact correspondence of one runtime-selected motor to one
physical callback, unchanged action availability, factual transition/cost counts,
zero externally forced actions, exact checkpoint restoration and unchanged learned
fingerprints/transitions in frozen phases. The production agency credit counter
only increments while its controller is learning: check self-selected increments
against callbacks in learning phases and unchanged agency counters in frozen
phases; independently audit every frozen callback against the returned decision.
The separate scorer also reconstructs legal world transitions and episode totals.

A **complete loop in one world** requires all of the following:

1. Frozen original validation succeeds in at least 75% of its 64 episodes.
2. During adaptive exposure after inversion, a selected terminal action fails
   despite a before-action success probability of at least 0.75 and at least
   three supporting observations.
3. After that contradiction, the runtime itself selects an informative sensor.
4. In recovered held evaluation, on an exactly repeated PRE/history/action
   context, the prediction for the previously failed action shifts toward
   failure by at least 0.25 and reaches at least 0.60. This must appear in at least
   three distinct held episodes. All action forecasts are available even when
   the obsolete action is now avoided.
5. Recovered held success is at least 75%, with at least three successful held
   episodes that included the useful sensor, and all integrity checks pass.

The overall capability verdict is **PASS** only when:

- All eight worlds and all phases finish with valid evidence.
- Aggregate original and recovered held success each reach at least 80%.
- At least six of eight worlds contain the complete evidence chain above.
- Aggregate recovered held success exceeds each of stale, shuffled-cue and
  random held success by at least 20 percentage points.

Thresholds and budgets are fixed before the first evaluation. An executed trial
missing any capability gate is **FAIL**. Broken infrastructure, inconsistent
evidence or incomplete worlds give **INVALID**, never a capability PASS.
Successful compilation or evidence collection alone is not success on the task.

Source `causal-belief-experiment`, `selected_fields[2]`, changed fingerprints and
the `useful_experiments` counter are diagnostics, not sufficient success gates.
The exposed hypothesis weights are over history depths, not newly invented
scientific theories. A passing frozen control gap attributes a benefit to broad
continued learning, not uniquely to a single isolated mechanism.

## Execution and reporting

Build the evaluator with the repository-pinned Rust toolchain and locked
dependencies. Check production files against the base commit and record the
compiled executable hash before drawing any world. Run all eight worlds with
two parallel workers, then score the persisted trajectories. Do not retune code,
thresholds, seeds or budgets after seeing capability outcomes.

```sh
cargo build --locked --release --example integral_causal_revision_eval
python3 scripts/run_integral_causal_revision.py \
  --binary target/release/examples/integral_causal_revision_eval \
  --seed "$GITHUB_RUN_ID" --workers 2 --output integral-evidence
```

`verdict.json` and `RESULT.md` summarize the exact frozen gates. Per-world JSON
and JSONL retain the detailed evidence. Re-scoring saved files is allowed and
does not consume new experience. Any genuine harness defect and its correction
must be disclosed; an exposed trial never becomes unseen evidence again.
No production repair or merge into main is part of this experiment.
