# CROSSWORLD-OBJECT-1 — external lexical referents in one lifetime

Date 2026-10-10. OPEN DEVELOPMENT. Not independently qualified AGI.
Evidence: [Actions run 38037219734](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38037219734)
at checked test source commit \`a005f7687e407f88b7395104cb2bed29636b9680\`.

## Factual result
\`CROSSWORLD_OBJECT_SUMMARY verdict=DEVELOPMENT_PASS task_rewards=0 claim=SUPERVISED_CATEGORICAL_REFERENCE_NOT_AGI\`

A **single persistent** EvoPhase Rust process, not three reinitialized
organisms, executed 2112 actions in independent Farama MiniGrid worlds.
The training environment was DoorKey-5x5, seeds 4100..4111,
while frozen heldout was DoorKey, MultiRoom-N2-S4 and Empty,
seeds 5100..5107. Cognition checkpointed/restarted between
training and evaluation. The native carrier retained 2 word links
and 6 candidate visual categories in 780 learned observational
frames. The actual deictic tutor produced 596 accepted "key"
and 391 "door" presentations, always pointing to a CURRENTLY
visible tile in public partial observation. No teacher calls
occurred in frozen heldout.

Visible reference scoring in heldout:
- 725 / 725 visible object instances returned the correct referent.
- Zero incorrect predicted referents.
- 115 / 115 visible doors in the **different MultiRoom family**
  were found by the grounded word from DoorKey lessons.
- Object word readout survived native checkpoint + restart.
- Matched unit tests verified that physically zeroing the word's
  phase synapse deletes its readout, and restoring reinstates it.
- Task rewards across heldout were **0**, so no evidence of
  independent DoorKey/MultiRoom solving emerged.

## Precisely what is and isn't grounded
The spoken string was EXPLICITLY paired by a deictic teacher with a
pointed tile. The learner never received simulator object type IDs,
mission strings or task plans. However the TEACHER knows object types
using MiniGrid's public categorical image codes, and the underlying
visual appearance signature is the first categorical channel, stable
across these MiniGrid environments. Therefore perfect matching
across seeds and world families is **narrow cross-world reuse of
supervised pre-categorized referents**, not autonomous discovery
from raw RGB or understanding of what a key does.
Repeated presentation is a confound; the separate preregistered
\`CROSSWORLD-OBJECT-2\` one-presentation test explicitly addresses it.

The mechanism is a handcrafted stable visual signature and phase
synapse-backed word mapping, not an emergent language faculty.
The next boundary requires learning motor affordances and relational
dependencies of those visual referents, then generalization to
different visual ontologies/RGB and actual task performance.
