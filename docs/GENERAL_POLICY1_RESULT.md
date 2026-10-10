# GENERAL-POLICY-1 — authored task controller replaced by learned distributed temporal policy

Date: 2026-10-10. **OPEN DEVELOPMENT PASS under preregistered criterion**,
not independent AGI proof. First source-fixed and matched outside run:
[GitHub Actions 38053197738](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38053197738),
**exact immutable source commit**
\`58d204089eff50a36793397bad3407f30b249e56\`.
[Protocol fixed before run](GENERAL_POLICY1_PROTOCOL.md).

## Structural difference from earlier incremental patches

One new **opt-in alternative decision architecture** returns immediately
from native \`collect_phase_native_unified_proposals\` and does NOT run
the manually designed object-affordance, goal-plan, spatial-navigation,
factor-effect, word-intent or object-class proposal generators.
It is NOT just one more module competing to control the motor.
All motors still go through the preexisting U1 arbiter and Human Protection.

The replacement uses a fixed 96-dimensional distributed random signed
hash of binary raw active sensory channels (not object categories,
spatial coordinates, MiniGrid class IDs, task title or path lookup).
It learns each opaque action's preference and an action-conditioned
feature predictor. Generic prediction surprise is a small
intrinsic acquisition signal. Real delivered task rewards are
propagated into previous action preferences through 0.91
eligibility-trace decay. Learned values and phase conducting motor
links are checkpointed; episodic visits and traces are erased
at restart and ignored by knowledge fingerprint.
Frozen heldout does not update learned values, demonstrated by
separate native tests; zeroing all physical motor links blocks
action choice. These learning formulas ARE programmed Rust
procedures rather than operations independently discovered by SNN.

## Original matched physics and outcome

One continuous native organism per comparison arm trained over
four independently maintained Farama MiniGrid 3.1.0 software tasks
in interleaved order, 12 learning seeds 62000..62011 per task:
48 learning episodes and at most 256 real protected motor
actions per episode. Then checkpoint/restart and **frozen** test
on new seeds 63000..63003 (4 per world, 16 total).
No staged objects, supplied keys, hidden maps/compass,
correct motors, instruction strings, goal positions or
demonstrations. Evaluator looked at environment-internal
state solely to count physical actions and actual reward,
NEVER to control the agent. Matched authored policy arm
had identical source, seeds, worlds and budget; matched
seeded random baseline was also recorded.

| Actual external task outcome | General learned replacement | Authored embodied selector | Random |
|---|---:|---:|---:|
| Total frozen heldout success /16 | **9** | 8 | 2 |
| MiniGrid-Empty-5x5-v0 /4 | **4** | 4 | — |
| MiniGrid-DoorKey-5x5-v0 /4 | **3** | 0 | — |
| MiniGrid-MultiRoom-N2-S4-v0 /4 | **0** | 0 | — |
| MiniGrid-Unlock-v0 /4 | **2** | 4 | — |
| Training successes /48 | **20** | 15 | n/a |
| Heldout actual physical movements | **241** | 166 | n/a |
| Heldout actual key pickups | **8** | 8 | n/a |
| Heldout actual door openings | **8** | 7 | n/a |

General-policy learning occurred on **6,152 factual transitions** and
**20 positive real environment reward events** in training. The
nine positive heldout rewards came from the external game engine,
not from agent predictions:
- Empty wins at 63000, 63001, 63002, 63003.
- DoorKey wins at 63000, 63001, 63002.
- Unlock wins at 63002, 63003.
- MultiRoom no wins. No task-specific action scripts/route hints
  were used for these tasks.

\`GENERAL_POLICY1_RESULT learned=9/16 authored=8/16 random=2/16 learned_training=20/48 per_world={'MiniGrid-Empty-5x5-v0': 4, 'MiniGrid-DoorKey-5x5-v0': 3, 'MiniGrid-MultiRoom-N2-S4-v0': 0, 'MiniGrid-Unlock-v0': 2} reward_updates=6152 real_reward_events=20 real_moves=241 verdict=DEVELOPMENT_PASS\`

[Native controls 2/2 and unchanged older object controls 3/3](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38053197760).

## Interpretation, no overclaim

**This IS a fundamental experiment changing the controller and
showing actual full external DoorKey completion without manually
coded key/door/motor strategy.** It is NOT evidence of AGI:
only 3/4 DoorKey, 0/4 MultiRoom; it uses a fixed, programmed
feature hashing / plasticity rule with pre-categorized partial
MiniGrid sensor data, not continuous raw RGB and not independent
learned ontology; its success is on a small set of four
independent but related Farama gridworld families. The
general learned policy exceeds the strong authored benchmark by
**only 1/16**, and its smaller world-by-world differences could
reflect small-sample variance. It also performs WORSE on
Unlock (2/4 vs authored 4/4). It is premature to infer reliable
transfer superiority.

The specific important finding is that authored task-rule
selection is **not necessary** for SOME observed external
multi-step task success, including actual DoorKey, under a
generic distributed temporal-reward learner. More rigorous
claims require a second clean source-pinned replication on
unconsumed seeds, with NO source edits and matched controls;
this is registered separately as
[GENERAL-POLICY-2](../.github/workflows/general-policy2-frozen-source.yml).

If the frozen replication fails, qualify the experiment as an
unstable candidate, NOT a broad breakthrough. Never tune
this run's consumed seeds and then call it heldout authority.
