# UNSTAGED-WORLD-2 — fresh generic repeated-action arbitration diagnostic

Date 2026-10-10. Valid OPEN DEVELOPMENT **FAIL** on independent
seed family. [GitHub run 38046953463](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38046953463).
Predeclared [protocol](UNSTAGED_WORLD2_PROTOCOL.md).

Cognitive adjustment: generic transient episode record of already
executed visible-state/motor pairs suppresses repeating the
same mastered motor at the same factual scene, allowing a different
motor or existing factor exploration to enter U1. This record is
cleared on external episode reset and native checkpoint; it does
not pollute the long-term learned fingerprint. The physical
object category→motor memory is preserved. No game-specific action
meaning, map or motor scheduling was added.

Same independent Farama 3.1.0 three full world task families;
new independent training seeds **52000..52011**,
frozen heldout **53000..53003**, 128 protected motor steps,
paired random actions. No tutor, no staged object, no preloaded
key, no privileged simulator mission, map or goal identity.
One organism across ALL worlds, checkpoint/restart and frozen
evaluation.

Exact CI:
\`UNSTAGED_WORLD1 train_goals=0/36 frozen_goals=0/12 random=0/12 learned_effects=3 heldout_pickups=3 heldout_door_openings=2 heldout_moves=1\`
\`UNSTAGED_WORLD1_SUMMARY verdict=DEVELOPMENT_FAIL\`.

**No full task success** (0/12 versus random 0/12). The agent did
execute one real change of its physical position, three real
key pickups and two real door openings in heldout. These
partial physically measured events are useful evidence of
local capabilities but NOT a successful task.
The first run's 0 moves and this run's 1 move use DIFFERENT
seed families, so one cannot attribute the difference
solely to the code repair as a paired causal improvement.

The third major barrier is now explicitly **temporally extended
self-localization and object-relative navigation** with
credit for attaining distant goals under sparse reward,
not discovering one more opaque pickup/toggle motor. We
must retain partial-view temporal state across camera turns,
infer motion and visited locations, identify an unexplored
frontier, and allocate long-horizon actions while avoiding
repeated loops. A paired fixed-seed ablation of memory/control
is needed before scientific causality is claimed.

Neither this result nor supervised localized motor PASS
establishes independent DoorKey/MultiRoom task solving.
