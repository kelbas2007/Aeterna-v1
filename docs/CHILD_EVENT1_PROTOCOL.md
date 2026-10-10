# CHILD-EVENT-1 — causal autobiography, success AND failed episodes

Date 2026-10-10. OPEN DEVELOPMENT. Previously source-qualified
HISTORY-MEMORY-3 obtained a successful native history alias control
but **no transfer to independent Farama MemoryS7**: memory 10/24,
no memory 10/24, random 8/24. Both organisms chose upper exit
in all heldout episodes. Results retained in
[HISTORY_MEMORY3_RESULT.md](HISTORY_MEMORY3_RESULT.md).
This must not be rebranded as success.

## Hypothesis
The old learned policy stored only (subjective current experience,
last action) when a terminal success arrived. It never remembered
what it had done BEFORE reaching the fork, and ignored terminated
unrewarded episodes. In developmentally grounded learning an episode
should be represented as linked experiential events (observed PRE,
opaque executed motor, factual POST, bounded external outcome),
with consequences reflected in past decision contexts.

## General mechanism under experiment
Optional \`PhaseGeneralPolicy.sequence_replay\` must attach to opt-in
experience history and existing episodic recall. It stores a
bounded ACTUAL sequence of prior contextual observations and
executed actions, never hidden simulator state. On factual
positive external reward it credits the previous up-to-20
events (including their earlier subjective cue contexts);
on a real episode ending without reward it stores those
preceding events as unsuccessful, not as positive guidance.
The next action readout can retrieve similar experienced
contexts. For contradictory histories, success/failure counts
affect the learned association strength.

No action-name, correct left/right branch, task title,
object role, external map/coordinates, labels or imagined
reward enters the native organism. The mathematical
credit/retrospective retrieve algorithm remains **handwritten
Rust** and is neither emergent SNN learning nor human brain
emulation.

## Tests and independent physics
Native tests:
- History alias: two different witnessed prior cues with the
  same final observation require distinct learned frozen motors;
  memoryless identical-frame control must remain unable to.
- Two factual multistep trajectories: real terminal reward
  records *earlier* PRE/action associations; an un-rewarded
  ended episode records actual negative associations. Native
  checkpoint persists acquired associations but clears
  ongoing episode buffers, no fabricated task reward.
- Physical motor synapse lesion blocks recalled output.
- Frozen readout does not alter persistent knowledge fingerprint.

External benchmark, not simulator-embedded physics:
- Exact independent Farama \`MiniGrid-MemoryS7-v0\`,
  publicly exposed partial categorical 7×7×3 observation,
  *no mission*, no hidden original object type, no target exit,
  no correct route, no experimenter demonstrations.
- One continuing native organism per arm, 128 training
  episodes on **110000..110127**, checkpoint/restart, frozen
  24 holdout episodes on **111000..111023**.
- 200 protected motor actions budget/episode.
- Arm A: learned policy **without any episode memory**.
- Arm B: EXACT same learned policy with real experience
  memory, factual recalled episodes and autobiographical
  *whole-sequence* positive/negative outcomes.
- Seeded random action control same seeds/budget.
- Human Protection/U1 unchanged. Scored ONLY by actual
  independent environment terminal reward. The auditor
  may inspect hidden coordinates to diagnose, but they
  never enter organism.

Strict OPEN development gate: >=12/24 real rewarded frozen
full tasks, > both matched memoryless and random,
with history-native lesion/freeze tests passed and
nonzero physical protected execution. Failure must be
documented and not described as a general-intelligence
breakthrough. Native training examples alone do NOT
qualify real acquisition of a causal memory.

Source must be frozen for independent replication after
any successful first development run. No retrospective
seed tuning or world-specific hardcoded fork selection.
