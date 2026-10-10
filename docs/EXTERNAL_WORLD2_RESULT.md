# EXTERNAL-WORLD-2 — real independent MiniGrid exploration result

Date 2026-10-10. OPEN DEVELOPMENT result, not frozen scientific evidence.
[Actions run 38033662625](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38033662625) 
compiled, passed the protected-first-motor unit test, executed the
three independent Farama MiniGrid 3.1.0 simulations and stored JSON/logs.

## New seed evidence (not reusing the first evaluation)

Training seeds per world: 1100..1107 (8); frozen heldout: 1900..1903 (4).
Budget 64 actual protected motor actions per episode, with separate
persistent carrier per external environment. Random baseline received
the same heldout world seeds and step budget. The goal is signalled
only by actual external positive reward. The simulator's mission,
internal map, agent position, transition rules and action labels
were not supplied to cognition.

| Real Farama environment | Training successes /8 | Frozen heldout /4 | Matched random /4 | Acquired effect rules |
|---|---:|---:|---:|---:|
| Empty-5x5 | 1 | 0 | 2 | 28 |
| DoorKey-5x5 | 0 | 0 | 0 | 32 |
| MultiRoom-N2-S4 | 0 | 0 | 0 | 32 |
| **Total** | **1/24** | **0/12** | **2/12** | |

The one training success on Empty was a genuinely executed outside
rewarded episode: the persistent organism recorded one positive
rewarded observation. It did **not** reproduce that reward on any
heldout fresh world. DoorKey and MultiRoom yielded no observed
positive rewards. The test is a **DEVELOPMENT FAIL for independent
external heldout task transfer**, despite green CI and all
protected actuations functioning correctly.

New generic exploration, compared with the first external attempt,
no longer uses the same lowest-index motor for every novel camera
frame. It scores factual local/global action trials, physical
non-no-op rates, predicted novel consequences and a deterministic
carrier-derived tie break. This is an implemented Rust heuristic;
it is NOT self-invented by SNN. The original run used different
seeds and must NOT be interpreted as paired statistical improvement.
The random control continues to outperform EvoPhase on the easy
heldout worlds.

## Architectural diagnosis

Partial egocentric 7x7 observations change drastically during turns,
making full-raw factored effects highly context specific. The
limited library of rules saturates under sparse reward, and a
single rewarded frame is not enough to generate a transferable
long-horizon objective. Raw-bit changes do not yet induce persistent
object identity, spatial localization or action-value propagation.
The next substantive intervention must address persistent
object-relative state and intrinsically chosen multistep subgoals
with actual reward credit, not simply add more global rules.

Do NOT add MiniGrid-specific motor scripts, privileged maps,
rewarding terminal pixels as preloaded goals, or tuned holdout
seeds to claim success. Preserve both external failures and
require a new blind frozen gate before qualified PASS.
