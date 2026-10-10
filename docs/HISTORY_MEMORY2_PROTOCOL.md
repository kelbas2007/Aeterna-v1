# HISTORY-MEMORY-2 — competitive temporal credit on true delayed-cue worlds

2026-10-10. OPEN DEVELOPMENT, not independent final scientific qualification.

The first [HISTORY-MEMORY-1](HISTORY_MEMORY1_PROTOCOL.md) ATTEMPT
[GitHub 38066838257](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38066838257)
failed its native history-alias test: two different prior cues,
then IDENTICAL current sensory frame, both led to action 2. This
is a real cognitive FAIL rather than a missing compiler dependency.
The external Farama MemoryS7 stage was **NOT EXECUTED** in that
first attempt; CI stopped at the necessary native test.

Cause hypothesis from algorithm: reward eligibility in the original
generic learner always credited the selected motor *positively*
on each step, so a frequent preparation motor received credit
from many subsequent successes without learning which action
competed correctly in the final ambiguous choice.

This new preregistered OPEN iteration changes one GENERAL
plasticity rule rather than adding any object/motor/world
specific router:
- At factual PRE/action, a signed trace strengthens eligibility
  of the ACTUALLY selected motor, and records negative eligibility
  for alternatives at the same factual PRE history.
- When the REAL external outcome arrives, it propagates through
  those signed traces across preceding steps, strengthening
  successful context-specific choices and inhibiting actions
  that competed in that same factual state.
- No evaluator-supplied correct action label, hidden cue class,
  object ID, mission, target exit or motor ID enters EvoPhase.
  This is hand-designed **competitive learning**, not a spontaneous
  novel SNN synaptic rule.

First disqualifying gate is SAME controlled native two-history
test: different factual prior cue, later SAME visible input,
genuinely different frozen policy action; without memory the
action at identical input must be identical. Synaptic gate,
checkpoint and freeze still required.

If it passes, run independent Farama MiniGrid 3.1.0
\`MiniGrid-MemoryS7-v0\`: 128 real training episodes on NEVER
used seeds 90000..90127, same continuing EvoPhase organism,
then checkpoint/restart and frozen heldout on 24
seeds 91000..91023, budget 200 real U1/HP actions per episode.
Compare generic policy with history memory ON/OFF, and matched
seeded random baseline. Farama's *actual task terminal
reward* is the only success measure; no staged objects or
motor demonstrations. The hidden starting object and target
exit are evaluator-only and are NOT transmitted.

OPEN PASS requires >=12/24 full heldout goals, strictly >
both no-memory and random, with positive native history
alias causal test. Otherwise DEVELOPMENT_FAIL. Even a pass
in this single related gridworld would not show human child
concept learning. Future independence requires other
perception/input ontologies and source-frozen reruns.
