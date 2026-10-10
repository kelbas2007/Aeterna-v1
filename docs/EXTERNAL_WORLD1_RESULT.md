# EXTERNAL-WORLD-1 — first actual independent Farama MiniGrid evidence

Date 2026-10-10. Status: **VALID EXTERNAL DEVELOPMENT FAIL 0/12**.
The external simulator is `minigrid==3.1.0` from Farama.

## Initial transport failure, not cognitive evidence
[Run 38033213634](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38033213634)
compiled and returned a green workflow, but 0 actions were executed
in every episode (native `NoSupportedAction`). Diagnosis: U1
proposal coalescing requires an attached U2 hypothesis ecology;
the new CLI had enabled U1 but not U2. This run is **INVALID** for
external cognition, regardless of its green CI.

## Corrected executable baseline
[Run 38033397140](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38033397140)
fixed U2, compiled and passed a new Rust unit test proving the
first motor is chosen, protection-screened and actually executed.
The Python simulator then ran 3 independent external tasks for 8
training episodes and 4 disjoint frozen seed episodes each.
Only public partial categorical 7x7x3 observations and factual
reward were supplied; no mission, hidden map or action hints.
Each training world had zero positive rewards and each heldout
world scored 0/4.

| Environment | EvoPhase training | EvoPhase frozen holdout | Seeded random holdout | Native factor rules |
|---|---:|---:|---:|---:|
| MiniGrid-Empty-5x5-v0 | 0/8 | 0/4 | 2/4 | 32 |
| MiniGrid-DoorKey-5x5-v0 | 0/8 | 0/4 | 0/4 | 32 |
| MiniGrid-MultiRoom-N2-S4-v0 | 0/8 | 0/4 | 0/4 | 32 |
| **Total** | **0/24** | **0/12** | **2/12** | |

Contradicted/no-op factual action observations: Empty 28,
DoorKey 95, MultiRoom 64. No protected action was blocked or
reported unsupported in this valid run; no early stop causes.
A maximum 32 distinct effect signatures was reached in every
task, with no acquired rewarded terminal observation.

The negative result proves that successful feature composition
on the prior *synthetic* task does **not** directly translate to
external partial-view exploration or DoorKey success. It does
not prove there is no useful cognition, nor that the model
cannot learn under longer curricula or revised exploration.
Do not hide the 0/12 under the green CI status.

## Next
A separate EXTERNAL-WORLD-2 OPEN diagnostic tests generic
goal-free exploration with new seeds and the same external
physics. Its result will not retroactively change these records.
