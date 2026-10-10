# GENERAL-POLICY-2 — source-immutable new-seed replication FAIL

Date: 2026-10-10. **REPLICATION FAIL**.
[Exact GitHub Actions run 38053677397](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38053677397).
The workflow explicitly checked out the **unmodified original source
commit \`58d204089eff50a36793397bad3407f30b249e56\`**
(the exact successful GENERAL-POLICY-1 executable source).
No controller revision, hyperparameter tuning, feature changes,
motor labels or training hint could enter between first experiment
and this independent seed replication.

## Two genuinely different seed families

| Outcome | First run 38053197738 | New frozen-source replication 38053677397 |
|---|---:|---:|
| Learned controller actual full task successes /16 | **9** | **2** |
| Matched stronger authored embodied baseline /16 | 8 | 4 |
| Paired random control /16 | 2 | 2 |
| Learned training game successes /48 | 20 | 12 |
| Actually rewarded training episodes | 20 | 12 |
| Native factual weight updates | 6152 | 6746 |
| MiniGrid-Empty-5x5 heldout /4 | 4 | 0 |
| MiniGrid-DoorKey-5x5 heldout /4 | 3 | 0 |
| MiniGrid-MultiRoom-N2-S4 heldout /4 | 0 | **2** |
| MiniGrid-Unlock heldout /4 | 2 | 0 |
| Actual heldout physical moves | 241 | 137 |

Second run used train seeds 66000..66011 per family,
and completely distinct heldout seeds 67000..67003
per family. SAME 4 external Farama minigrid==3.1.0
worlds with 256 protected actions/episode,
one continuing organism per arm across interleaved
training worlds, checkpoint/restart and frozen heldout.
No maps, action meanings, reward location, teacher examples,
staged keys/doors or privileged evaluator state entered
the Rust process. All externally generated terminal rewards
are real and results are scored even when they fail.

Actual preserved output:
\`GENERAL_POLICY2_FROZEN_RESULT learned=2/16 authored=4/16 random=2/16 complex_wins=2/8 per_world={'MiniGrid-Empty-5x5-v0': 0, 'MiniGrid-DoorKey-5x5-v0': 0, 'MiniGrid-MultiRoom-N2-S4-v0': 2, 'MiniGrid-Unlock-v0': 0} reward_events=12 verdict=REPLICATION_FAIL\`.

## Scientific inference

The generic replacement is IMPLEMENTED, passes native delayed
external reward-credit/freeze/checkpoint/physical motor lesion
controls, and DOES sometimes complete full complex independent
external tasks without handwritten object/maze rules.
Importantly, it produced two real successes in MultiRoom,
which no previous open heldout agent had achieved in this series.
Nevertheless its **measured whole-task superiority is not robust**:
on the unconsumed seeds the new learned controller performed
**at random-level 2/16 and below the authored controller 4/16**.
The preregistered claim of reproducible *superiority to both
controls* is thus REFUTED by the first honest replication.
Do not cite 9/16 from original run alone as a stable general
intelligence breakthrough.

## Why this motivates a more fundamental change

The distributed feature hash is a function only of the CURRENT
egocentric sensor image; it has no learned latent continuity
through object disappearance, turning or differing histories
with identical current views. Eligibility traces assign
delayed reward backward but are only a learning-gradient memory;
they are not a **belief state** for making conditional choices
later. Task success hinges on navigation and causal objectives
requiring temporal state. The same visible frame after a different
past can demand a different action; a reactive image-only policy
cannot represent the distinction except through incidental
clock-based pseudo-randomness. This is an architectural
expressivity limitation, not just an insufficient number of
local skill templates.

A next permissible fundamental research step must modify
the INTERNAL REPRESENTATION/LEARNING DYNAMICS to build
a self-supervised, history-dependent predictive state and
discover reusable transitions or latent operations. It must
retain the no-task-script controller replacement. Demonstrate
the need for memory on a withheld-history POMDP first, and
then independently replicate full external task outcomes on
separately reserved world/seed families. Do NOT patch exact
MiniGrid game motor sequences or retune using the consumed
62000/63000 and 66000/67000 seed batches.
