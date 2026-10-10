# EMBODIED-WORLD-2 — independent four-world, long-horizon whole-task results

Date 2026-10-10. Status: **VALID OPEN DEVELOPMENT FAIL for predeclared
combined complex-world gate**, but **7 actual successful FULL
external game tasks out of 16**. Green CI is not a qualification
verdict. Real Farama MiniGrid 3.1.0 action/reward evidence:
[GitHub Actions 38049301325](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38049301325)
at source commit \`6311fab778f9fa452e26217ad22024527f217a97\`.
[Predeclared protocol](EMBODIED_WORLD2_PROTOCOL.md).

## Controlled conditions

Both native EvoPhase arms and paired random comparison used
the **same real Farama-generated worlds**, **training seeds
56000..56011**, **heldout seeds 57000..57003**, **budget 256
actual protected motors/episode**, with one uninterrupted
organism per EvoPhase arm across all four task families.
Training 48 episodes, then native checkpoint/restart and
frozen learned cognition for the 16 heldout episodes.
All physical actions selected by native U1 and screened by
Human Protection. The organism received solely the public
partial 7×7×3 category-coded observation bits and factual
reward; no teacher word, motor demonstration, map,
internal position/compass, or examiner-arranged key/door.

| Full actual task success | Absent embodied exploration | Native embodied optic-flow learning |
|---|---:|---:|
| Training /48 | 8 | 15 |
| Frozen heldout /16 | 0 | **7** |
| Matched random heldout /16 | 1 | 1 |
| Heldout position changes | 18 | **211** |
| Heldout actual carried-key acquisitions | 3 | **14** |
| Heldout actual door openings | 3 | **6** |

## Exactly which real environments completed

| Environment | Embodied training /12 | Embodied heldout /4 | Frozen win seeds |
|---|---:|---:|---|
| MiniGrid-Empty-5x5-v0 | 7 | **4** | 57000, 57001, 57002, 57003 |
| MiniGrid-DoorKey-5x5-v0 | 0 | **0** | none |
| MiniGrid-MultiRoom-N2-S4-v0 | 0 | **0** | none |
| MiniGrid-Unlock-v0 | 8 | **3** | 57000, 57001, 57003 |

The actual MiniGrid engine, NOT EvoPhase's predictions, generated
positive terminal rewards on those seven heldout tasks. Heldout
task success outperformed the SAME-source no-navigation ablation
by 7 and the equally budgeted random agent by 6.
The agent executed 211 actual position changes and accomplished
14 actual key pickups plus six actual door-open events in heldout;
these are grounded simulator-object and position state audits,
never used by the agent for control. Optical-flow detection
reported 875 inferred transitions across the whole lifetime,
and MUST NOT be equated with actual physical movement count.

Actual retained stdout:
\`EMBODIED2_REAL_TASKS ablated=0/16 embodied=7/16 random=1/16 per_world={'MiniGrid-Empty-5x5-v0': 4, 'MiniGrid-DoorKey-5x5-v0': 0, 'MiniGrid-MultiRoom-N2-S4-v0': 0, 'MiniGrid-Unlock-v0': 3} moves=211 keys=14 doors=6 verdict=DEVELOPMENT_FAIL\`.

## Interpretation and remaining scientific challenge

These are the project's first repeatable autonomous solutions of
whole external tasks in multiple Farama environments **without
demo motors, staged objects, mission hint, hidden map or simulator
action/goal oracle**, with frozen holdout cognition. The success is
bounded to publicly category-coded egocentric MiniGrid and an
explicitly handcrafted Rust sensory-flow/novelty action policy
on physically learned motor statistics. It is not proof the
SNN independently invented a planner.

The strict criterion also REQUIRED >=1 completed DoorKey OR
MultiRoom task, not just relatively easy Empty/Unlock goals.
That part FAILED 0/8; the overall scientific verdict is
therefore **DEVELOPMENT FAIL**. The model does not yet
maintain a stable multi-room map, infer key-gate goal dependencies
over time, or use reward-grounded object/landmark planning.
Four unseen seeds per world are too few for general statistical
qualification. No unrelated world ontology or RGB transfer
was tested.

Next required independent challenge: reward-grounded spatial
memory and subgoal composition through camera rotations and
object absence; exact same-seed matched no-map and no-phrase
ablations, first-attempt sealed source, and real DoorKey plus
MultiRoom heldout task rewards as indispensable success evidence.
