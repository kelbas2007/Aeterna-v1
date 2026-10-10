# HISTORY-MEMORY-1 — an independent environment that requires remembering the past

Date: 2026-10-10. OPEN DEVELOPMENT, not a frozen independent scientific authority.
Source and test are defined before first run and must not be modified
after inspecting heldout results for a retrospective claim.

## Why this test is different

Farama MiniGrid MemoryS7 is an **independently authored** RL environment.
A subject observes one of two objects in the starting room (key/ball),
moves down an occluding corridor, then chooses the matching object at
a split. Once the original cue scrolls out of its public partial view,
success can require retaining past information across steps. A wrong
turn is an immediate episode failure, not a minor local motor effect.

EvoPhase must NOT receive MemoryS7's mission text, starting object class,
correct exit, success_pos, failure_pos, global x/y, direction, internal map,
or a scripted motor route. Only the publicly observable **7x7x3
category-coded partial image bits**, the actual U1/Human Protection
permitted opaque motor, and the actual externally delivered reward
may enter the native organism. The evaluator reads hidden state ONLY
after actions to report physical outcomes; never feeds it back.
Object category input is already pre-symbolized, not learned RGB vision.

## Comparison, frozen and fair

Same source and same independent Farama MiniGrid version 3.1.0:
- Arm A: generic learned action policy, **memory OFF**.
- Arm B: identical generic policy, opt-in **developmental
  experience trace ON**.
- World: MiniGrid-MemoryS7-v0.
- Training seeds 80000..80127, 128 real continuous episodes
  per carrier; then native checkpoint/restart and learned-weight freeze.
- Fresh heldout seeds 81000..81023 (24), 200 physical U1/HP
  permitted actions/episode.
- Matched seeded random motor baseline on the same 24 worlds and budget.
- NO motor demonstrations, external tutor, staged objects,
  oracle cues, mission strings or privileged coordinates.
- Native history alias test separately probes whether two
  identical present frames but different prior cues may produce
  different actions after learning, and whether the same
  memory-less policy cannot encode that distinction.

Strict development success: memory arm completes >=12/24
heldout MemoryS7 episodes, strictly > both no-memory arm
and random, zero invalid agent transport, history alias unit
test PASS with real frozen recall and checkpoint. Otherwise
DEVELOPMENT_FAIL. The negative result must be retained as-is;
the test may reveal that a decaying history trace without
structured event binding is not sufficient.

History alias native unit test directly supplies controlled
factual 2-step sensorimotor/reward trajectories; its action
feedback is *training supervision* in a contrived minimal
native test, NOT evidence of autonomous discovery or real
MiniGrid memory performance. Only the independently maintained
Farama whole-task success counts as external competence.
