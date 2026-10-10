# EMBODIED-WORLD-1 — first matched independent Farama task evidence

Date: 2026-10-10. **VALID OPEN DEVELOPMENT FAIL** against the
predeclared full-task success criterion, but 1 independent outside
task actually completed. Run
[38048828798](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38048828798)
at exact source commit \`a619f214453c031a4a37a5390a50e82f25eeff97\`.
Keep first frozen-source historical benchmark results unchanged.

## Strictly matched experimental comparison

Same Rust cognitive source for both modes, SAME training seeds
54000..54011 and SAME heldout seeds 55000..55003, same independent
Farama MiniGrid 3.1.0 DoorKey-5x5 / MultiRoom-N2-S4 / Unlock physics,
one lifelong organism per arm across interleaved world families,
and same 128 actual externally screened actions per episode.
Human Protection and native U1 choose every motor. Zero staged
object proximity, zero evaluator-given key, zero motor demonstration,
zero teacher labels, zero hidden map/compass/mission transmitted to
the organism. Native cognitive checkpoint/restart and frozen
heldout model. Both arms receive the same matched random baseline.

| Real externally measured outcome | Unmodified self-object, no embodied mode | Embodied optic-flow learning mode |
|---|---:|---:|
| Training task completions /36 | **0** | **7** |
| Frozen heldout full task completions /12 | **0** | **1** |
| Matched random heldout completions /12 | **1** | **1** |
| Frozen heldout physical agent movements | **1** | **143** |
| Frozen heldout actual key pickups | **1** | **10** |
| Frozen heldout actual door openings | **1** | **3** |
| Learned local object effects | **3** | **3** |
| Internal optic-flow movement inferences | **0** | **425** |

Per world: 7/12 training successes and 1/4 heldout success
were exclusively in the independent **MiniGrid-Unlock-v0** task;
the successful heldout environment seed was **55002**.
DoorKey and MultiRoom both remained **0/12 training and 0/4
heldout**, even though physical manipulation/navigation occurred.
Independent actual reward was delivered by Farama after completing
Unlock, not fabricated by our evaluator. Inference count 425
reflects generic sensory shift detections across both training
and heldout, and should NOT be mistaken for actual physical
position changes. Physical movement counts come solely from the
hidden-side evaluation auditor and are NEVER supplied to cognition.

Exact CI result:
\`EMBODIED_PAIR_RESULT ablated_tasks=0/12 embodied_tasks=1/12 random=1/12 ablated_moves=1 embodied_moves=143 optic_shifts=425 key_pickups=10 doors=3 verdict=DEVELOPMENT_FAIL\`.

## Genuine improvement and remaining limitation

This is strong paired evidence that opt-in sensory-flow action
exploration changes behavior and increases actual navigation:
143 physical moves vs 1 in a matched same-seed ablation, and
a real autonomous outside Unlock goal was reached after
acquiring motor knowledge from actual real effects.
But **the heldout task result only matched random 1/12**;
the preregistered requirement >=6/12 and > ablation AND random
FAILED. It is NOT a qualified general intelligence or a
complete navigation/planning breakthrough. The navigation
motor-selection algorithm is explicitly written in Rust and
the observation already contains categorical tile classes.

Next scientifically meaningful boundary: extended memory of
world-relative locations and object persistence across unseen
egocentric frames; learned landmark/subgoal values after real
reward; navigation toward past observed objects, not just
preference for locally sensed movement. A new differently
seeded experiment must measure whole-task reward; do not tune
against this consumed dataset.
