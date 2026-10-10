# CHILD-EVENT-1 — actual causal autobiography stores entire episodes, but external cue-use FAIL

2026-10-10. **NATIVE TEST PASS; EXTERNAL MEMORY CAUSAL-USE FAIL.**
The first raw CI scorer printed \`DEVELOPMENT_PASS\` for **14/24**
task rewards, but it missed a crucial disqualifying behavior.
The persisted raw artifact was inspected at the level of each
heldout final actual simulator position: **all 24 episodes
selected the same lower side, regardless of initial cue**.
We therefore explicitly OVERRIDE the earlier numeric
score and record a **VALID DIAGNOSTIC DEVELOPMENT FAIL**.
Neither green CI nor 14 wins is evidence of learned cue
conditioning.

Evidence:
[GitHub run 38068337538](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38068337538),
exact first source commit \`855b58c1d3e680509ce629b02731d78f2a453eb0\`,
raw artifact \`child-event1-autobiographical-causal-history\`.
[Predeclared initial protocol](CHILD_EVENT1_PROTOCOL.md).

## What genuinely worked

Generic phase-native learner implemented optional bounded
\`episode_trace\` of ACTUAL subjective PRE contexts and
executed opaque motor IDs. On real positive external reward,
the previous at-most-20 events are recorded as successful
autobiographical context/action associations. At the start
of a new learning episode whose preceding history had no
positive reward, those actually executed events become
unsuccessful experience associations. There is NO teacher
correct-action oracle, success coordinate, environment title,
mission, external state, object name, or prescribed route
inside the native learner.

Native controls PASSED:
- \`AUTOBIOGRAPHICAL_REPLAY_NATIVE_PASS whole_success_events=2
  failed_events=2 checkpoint=true frozen=true motor_lesion=true\`
- \`HISTORY_ALIAS_NATIVE_PASS same_current_image=true
  cueA_action=0 cueB_action=1 no_memory_action=2 checkpoint=true\`
- 1/1 whole-event native check, 2/2 history checks,
  2/2 general-policy regression checks.

This establishes that the mechanism is *representationally
capable* of distinguishable histories and that rewards/failed
episodes really affect persistent learned event memory,
not that it independently uses the information.

## Genuine outside task, fixed experimental conditions

External independently maintained Farama \`minigrid==3.1.0\`
\`MiniGrid-MemoryS7-v0\`. 128 real training episodes
seeds 110000..110127, then carrier checkpoint and frozen
24 heldout seeds 111000..111023, max 200 protected
motor commands per episode, matched source/control/worlds.
No hidden simulator state or mission entered cognition.

| Outcome | Same learned policy without event memory | Event sequence replay |
|---|---:|---:|
| Actual task success /24 | **10** | **14** |
| Random actions /24 | **6** | **6** |
| Training task success /128 | 60 | 63 |
| Distinct successful context/action memories | 0 | 64 |
| Unsuccessful witnessed associations | 0 | 457 |
| Chosen heldout exit positions | \`(5,2)\` 24/24 | \`(5,4)\` 24/24 |

The task's hidden truth was upper \`(5,2)\` on 10/24
and lower \`(5,4)\` on 14/24 of these heldout worlds.
The event-memory agent chose lower in **every case**.
Its 14 successes simply matched the more frequent lower
target; the memoryless agent always chose upper and won 10.
Neither actually conditioned its branch on the remembered
starting cue.

Therefore **the external advantage is an artifact of
target-side imbalance, not causal-use intelligence**.
A better scoring criterion must require BOTH exit branches,
correct choice conditioned on the starting cue, and
outperformance against the best fixed-side majority
baseline — not merely a high aggregate reward count.

## What this changes scientifically

The current general event replay is a programmer-defined
associative store; its positive/negative credits can still
collapse to a fixed preferred action. This is NOT childlike
self-generated causal inference or reliably using a memory
of something now outside view. It is still impressive that
one carrier stores both rewarded and unrewarded episodes,
but usefulness requires a *recalled distinction controlling
action*, not simply a store of episodes.

The next test should **not change the source code**
to fit this consumed result. Instead pin exact source
commit \`855b58c1...\`, use fully unconsumed future seeds,
and require verified conditional branch choices and
> best constant upper/lower strategy. If it fails
again, the route is fundamentally inadequate rather
than "nearly AGI".
