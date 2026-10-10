# HISTORY-MEMORY-3 — causal native historical memory works; external historical use FAILS

Date: 2026-10-10. **NATIVE HISTORY ALIAS PASS (2/2 tests)**,
**INDEPENDENT EXTERNAL MEMORY WORLD DEVELOPMENT FAIL**.
[Exact GitHub Actions run 38067414320](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38067414320).
[Registered protocol](HISTORY_MEMORY3_PROTOCOL.md).

## Mechanism

The EvoPhase native generic controller now has an opt-in
`episodic_recall` mechanism that stores ACTUAL positive
external reward episodes as distributed
`(recalled subjective context, executed opaque motor)`
pairs. It uses signed competitive temporal eligibility
traces and the same conducting physical motor synapses,
U1/HP controls, no task/cue object name and no hidden
simulator map, position or goal labels.

This is a **hand-designed generic episodic memory
association** (bounded vector similarity), not biologically
accurate hippocampal replay or an SNN emergently
discovering general reasoning operations.

## Controlled native history requirement genuinely passed

The minimal 2-step controlled factual training experiment
presented TWO different earlier sensory cues and an
IDENTICAL final visible frame, with alternative actual
positive outcome actions. After frozen learned source:
- history A + same present frame → action **0**;
- history B + same present frame → action **1**;
- corresponding memory-less identical-frame policy → action **2**
  irrespective of preceding cue.
The distinction survived physical cognitive checkpoint and
restart. Test transcript:
`HISTORY_ALIAS_NATIVE_PASS same_current_image=true cueA_action=0 cueB_action=1 no_memory_action=2 checkpoint=true`.
This native test used controlled training trajectories and
known factual correct action examples; it establishes the
representational and retrieval *capability*, NOT the autonomous
acquisition of the correct route on its own.

## The real, externally independently authored MemoryS7 world

The independently maintained Farama MiniGrid 3.1.0
`MiniGrid-MemoryS7-v0` requires noticing a starting-room
object, traversing a corridor, and choosing the matching
object at the fork when the original cue has left view.
Each memory/no-memory EvoPhase carrier trained on SAME
128 actual world seeds 100000..100127; a single continuing
lifetime per arm, checkpoint/restart and learning frozen
before 24 new heldout worlds 101000..101023, 200
protected motor action maximum. A seeded random control
received the same 24 heldout worlds and budget.
The learner got ONLY public category-coded partial
observations, own U1/HP motor and actual Farama reward;
never mission text, object labels, target position or
simulator hidden map. Evaluator recorded hidden positions
ONLY after outcomes for diagnostic evidence.

| Actually completed full MemoryS7 tasks | Successes /24 |
|---|---:|
| Native generic learner **with episodic memory** | **10** |
| Identical native learner **without memory** | **10** |
| Matched seeded random motor | **8** |

Both cognitive arms had **64/128 training successes**
and **124 executed frozen actions** each. The opt-in
memory arm accumulated **9 distinct rewarded episodic
context/action records** (0 in memory-less control).
Its record store therefore really worked, yet did
not yield a different policy at the fork.

**Critical raw artifact diagnosis:** inspecting every one
of the 24 heldout episodes in BOTH arm JSON reports,
each organism reached the EXACT SAME terminal simulator
coordinate `[5,2]`, regardless of whether actual
`success_pos` was `[5,2]` or `[5,4]`.
Thus the 10 wins were due to selecting the same upper
branch on maps where it happened to be correct,
not using the starting object memory to identify the
correct exit. The episodes have 4–7 executed actions.
This is a strong refutation of the claim that the new
memory controller autonomously used the relevant past cue,
despite the valid native mechanism test. The true
benchmark verdict is **DEVELOPMENT FAIL**.

The only outcome labels/positions in native module
come from the actual PRE/action/POST and reward, never
the hidden evaluator state. The repeated wrong-branch
policy is an actual failed attempt, not an invalid
transport or fabricated reward report.

## Next genuine architecture barrier

The episode store contains only a positive *last*
action context at terminal success, NOT the causal
chain of decisions that led from original cue to the
correct exit. For this independent memory task, the
organism needs experience-dependent temporal credit
through an entire sequence and the ability to use both
successes AND failures in choosing earlier branches.
Just storing a terminal rewarded observation is not
a human-like memory of meaningful events.

Future research candidate: bounded autobiographical
sequence memory recording each factual PRE/opaque
motor/POST; retrospective credit after real end of
episode, including negative experience when an
action fails; source-agnostic retrieval conditioned
on learned event history. This MUST be independently
evaluated on new seeds and with a real episode
history ablation, not another preset correct-branch
table or a benchmark-specific MiniGrid route script.

No general intelligence claim is supported by this
result; keep first and second native-history failures,
and the successful native capacity test, together
with this actual external negative finding.
