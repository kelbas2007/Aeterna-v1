# CROSSWORLD-OBJECT-2 — one deictic lesson per word, fresh external worlds

2026-10-10, OPEN DEVELOPMENT PASS for narrow categorical lexical reference.
[Real independent Farama run 38037399748](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38037399748).
Source at exact run commit \`dc57bdb66a32a9c3a3d51519563b8aa3eddcf3e7\`.
Not a frozen independent scientific/AGI authority gate.

## Actual evidence
- Training: twelve DoorKey-5x5 seeds 6100..6111.
- Heldout: eight new seeds 7100..7107 in EACH of DoorKey-5x5,
  MultiRoom-N2-S4, Empty-5x5; no lessons during holdout.
- A **single continuing Rust EvoPhase organism** was never rebuilt
  between external environments. Cognition checkpoint/restarted once,
  retained object and word memory, and was frozen during heldout.
- **Exactly one accepted pointed learning exposure** for \`key\`,
  exactly one for \`door\`. No repeated correction/tutor thereafter.
- Across 2,112 physically screened/issued simulator actions,
  780 training frames, five visual categories in memory,
  words=2, the frozen learner correctly returned
  **773/773 visible word-object referents**, 0 false positives.
- **110/110 doors in independently generated MultiRoom environments**
  were found using the DoorKey-taught word, without a MultiRoom
  teacher. Unknown/untrained word behavior and the physical
  phase-synapse lesion/restore and checkpoint invariants were
  separately verified by two native controls.
- **0 positive task rewards**, therefore **still no independent
  cross-world planning or motor-affordance transfer**.

Raw exact CI:
\`CROSSWORLD_OBJECT teacher={'key': 1, 'door': 1} words=2 visual_classes=5 frames=780 actions=2112 visible=773 correct=773 false_positive=0 crossworld_door=110/110\`
\`CROSSWORLD_OBJECT_SUMMARY verdict=DEVELOPMENT_PASS task_rewards=0 claim=SUPERVISED_CATEGORICAL_REFERENCE_NOT_AGI\`.

## Interpretation boundary

MiniGrid already supplies an **object-type categorical sensor field**
stable across these independently seeded environments, and our
handwritten recognizer uses that field as its invariant signature.
The TEACHER uses the simulator's public categorical \`OBJECT_TO_IDX\`
for choosing one visible object to point at, but never gives that
index to the agent: the only lesson is \`{word, tile_index}\`.
The proof is **one-shot supervised naming and transferable reference
across positions, color changes, open/closed state and worlds sharing
a predefined categorical object ontology**. This is NOT discovery
of object categories from RGB, not abstract symbolic language
reasoning, not knowledge that a key opens a door, not autonomous
task competence. One-shot association alone may be implementable
by a simple lookup baseline.

The next real cognitive barrier is to tie this referenced
object identity to observed action affordances and goal-relevant
relations, then recognize the same physical objects from raw RGB
with heldout appearances / shifted ontology, and solve independent
DoorKey/MultiRoom objectives. No claim of a successful learned
task policy is justified by these lexical results.
