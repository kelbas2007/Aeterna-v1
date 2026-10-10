# SELF-OBJECT-2 — independently scored real physical effects, new external worlds

Date 2026-10-10. **OPEN DEVELOPMENT PASS 24/24** for
strict object-interaction physics with no correct-motor tutor,
not a standalone qualification of human/general intelligence.
[Run 38046613579](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38046613579)
at the preregistered [protocol](SELF_OBJECT2_PROTOCOL.md).
Source revision is retained in the immutable GitHub run, and the
first SELF-OBJECT-1 report is independently preserved.

Train physical experiment seeds 44000..44009.
Frozen heldout seeds 45000..45011 in each of two *distinct
task families*: MiniGrid-Unlock key pickup and DoorKey locked
door opening after examiner supplied appropriate carried key.

The organism received NO correct action IDs, no demonstration,
no category names and no object-specific path or internal map.
On each real trial, it chose an opaque motor via U1, executed
through Human Protection, and received only the real public
partial MiniGrid observation/reward. Exactly 10 externally
executed motor probes yielded two learned functional
category→motor phase-synapse links. The model was
checkpointed/restarted and frozen for heldout.

Crucially the independent grader inspected the *actual* Farama
object state, not just a visual frame difference:
- KEY: the same physically existing key instance moved into the
  agent's actual carried inventory from no carried object;
- DOOR: the same physically existing door object's \`is_open\`
  changed from false to true.
These evaluator-only fields and expected action IDs were never sent
to the organism. The rerun reported
\`SELF_AFFORDANCE1 train_actions=10 motor_tuition=0 learned=2 frozen_interactions=24/24 blocked=0 goal_rewards=0\`.
All 24 scored successes are on actual simulator effects, and
none are simply rotations of the visible camera.

Control: native self-object effect experiment also showed first
learned motor with no labeled motor tuition, retained after
checkpoint and abolished by zero-weight phase-synapse lesion,
restored by synapse restoration, with U1/HP actual execution.

**Limits**: The examiner still staged target objects directly
ahead of the agent, and at locked doors supplied a valid carried
key as an examiner-prepared precondition. The single
known body-relative front tile is provided as an embodiment
geometry primitive. The shared MiniGrid category channel is
not RGB object discovery. The generic trial-search algorithm
was written in Rust rather than invented by the SNN.
The agent did not navigate, plan subgoals, discover that
a key was required for the door, or receive terminal task
reward. The next separate UNSTAGED-WORLD-1 challenge removes
all staging and separately measures genuine heldout game
goals, random baseline, object manipulation and movement.
