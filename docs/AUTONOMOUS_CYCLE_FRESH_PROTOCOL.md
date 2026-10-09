# AETERNA EvoPhase autonomous-cycle fresh-seed confirmation
Status: PREREGISTERED BEFORE FIRST FRESH EVALUATOR RUN
Date: 2026-10-09
Branch: `research/beyond-intel4`

## Cognitive source freeze
Last qualified development source tree before this protocol:
`b5a21f5d6a761b463ecacc6dfcbb5a5c65895fca`.
No modifications to `src/` are authorized during the fresh check.
The first visible development run [37932291441](https://github.com/kelbas2007/Aeterna-v1/actions/runs/37932291441)
reported 70/80, 74/80, 70/80, 72/80 in four known stochastic arms. These
arms have been iterated during architectural development and are **not** an
independent source of generality.

## Fresh challenge preregistration
- Different fixed seed: `0xF2A7_2026_19C4_5B0E`. This is NOT the exposed `0xA5C3_2026_51A9_D13B` seed.
- Exactly four independently randomized 24-state cue/goal/failure and
  six-opaque-motor role assignments from that seed.
- Same structural generator family and 70% independently noisy clues as the
  development test. The cognitive code must never receive hidden side, action
  roles, seed, or task-specific learning schedule.
- Per arm one uninterrupted 192-episode A(64)→B(64)→A(64) training lifetime;
  each episode has at most 10 physically permitted executed actions.
  80 separate balanced heldout episodes; model learning frozen; checkpoint
  restart between training and heldout; physical belief and U1 policy retained.
- Same sensory/goal builder, goal law switch, U1 transfer baseline, protected
  `ScientificRuntime::step_unified`, and current opt-in autonomy settings.
- PRE/POST are factual, no post hoc intervention or adaptive evaluator
  training. Historical failed stochastic tests remain failed.

## First-attempt pass thresholds
All four arms must independently satisfy:
1. >=64/80 correct hidden-goal commitments; strictly better than the respective
   realized first-noisy-cue oracle under identical heldout hidden labels.
2. >=40/80 episodes with two or more self-executed extra samples.
3. Two discovered cue-source identities and positive conducting physical
   sensory motor affordance acquired during training.
4. Zero blocked/unsafe/unsupported actions in the protected runner.
5. Frozen U1 physical parameters unchanged throughout heldout; checkpoint
   preserves the acquired autonomous exploration contract.

Any failure is **FRESH DEVELOPMENT FAIL**, not a cue to rerun with a new seed.
Code compilation/pass does not promote the result. There is no independent
open-world AGI claim: this only measures new stochastic role assignments in a
still-synthetic task family on a prepared vision/phase foundation.

## Evidence recording
Report each fresh arm separately: hidden reward correct/80, first and
three-cue oracles, raw sensor actions, >=2 sample episodes, physical sensor
conductance, proposal policy agreement, training actions, safety blocks and
checkpoint correctness. Link first GitHub Actions run and exact source SHA in a
result file; do not edit source, generator or thresholds after observing output.
