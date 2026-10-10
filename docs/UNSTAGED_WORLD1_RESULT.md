# UNSTAGED-WORLD-1 — first real without-object-staging external task evidence

Date 2026-10-10. Source pinned to first run
[38046685780](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38046685780).
**Valid OPEN DEVELOPMENT FAIL for full external-world tasks**.

Farama MiniGrid 3.1.0, no teacher, no supervised motor, no
object/precondition placement, one persistent EvoPhase lifetime
interleaved among DoorKey-5x5, MultiRoom-N2-S4 and Unlock.
The agent receives only public partial 7x7x3 symbolic categorical
sensory frames serialized as 588 binary channels, executed native U1
action and actual bounded task reward through Human Protection.
Training seeds 50000..50011 (36 episodes total),
then checkpoint and frozen heldout 51000..51003 (12 total),
128 permitted motor steps per episode. Matched random actions
on identical heldout seeds/budget.

Exact published CI:
\`UNSTAGED_WORLD1 train_goals=0/36 frozen_goals=0/12 random=1/12 learned_effects=3 heldout_pickups=191 heldout_door_openings=0 heldout_moves=0\`
\`UNSTAGED_WORLD1_SUMMARY verdict=DEVELOPMENT_FAIL\`

All 12 heldout tasks were FAILED, random got 1/12.
The independent simulator auditor observed repeated key pickup,
zero door openings and, most importantly, **ZERO changes of
physical agent position** during heldout. The local learned
category→motor link therefore monopolized the controller,
preventing exploration/navigation. This is a precisely
diagnosed behavioral defect and not full intelligence
despite learned real object motor effects. The count of
191 pickups reflects repeated manipulation; it is NOT
191 different successfully completed tasks or objects.

The run stays immutable. Later code cannot change this evidence.
