# FUNCTIONAL-GROUNDING-1 — word→object→factual action (open supervised benchmark)

Date 2026-10-10. Research branch \`research/beyond-intel4\`.
**OPEN DEVELOPMENT**, not independent/frozen scientific evidence.

## Motivation
Earlier one-shot lexicon on a single continuing organism succeeded on
773/773 categorical visual references across DoorKey/MultiRoom/Empty, but
0 task rewards. The missing capability is that a named *physical object*
must relate to what particular motor can actually change it.

## Boundaries and assumptions
The environment is **real external Farama MiniGrid 3.1.0 code**,
not an embedded AETERNA-written transition table.
The public observation is its partial 7×7×3 categorical tile image
serialized into 588 binary bits. The origin of object identity is
explicitly given by MiniGrid's first category channel, not raw RGB
perceptual discovery. Human Protection is unchanged for *heldout*
motor choice and execution.

There are TWO explicit, human-demonstrator-style training interventions:
1. Point at a visible key in DoorKey-5x5 and say \`key\`, stage the key
   one square in front, execute an actual MiniGrid motor to pick it up.
2. Point at a visible door in MultiRoom-N2-S4 and say \`door\`, stage the
   door one square in front, execute an actual MiniGrid motor to toggle.

The scripted demonstrator is allowed to inspect the external simulator
to *stage a teaching/exam opportunity* and select a teaching action.
The EvoPhase organism receives only the raw public PRE, the already
learned word + pointed relative tile, the opaque ACTION ID that was
actually executed, and the real external public POST. It NEVER
receives the object's internal simulator coordinates, class ID,
action semantics, map, rewards table, or navigation solution.
This is **explicitly motor-supervised** causal grounding. It is NOT
spontaneous autonomous motor discovery or independent planning.

Generic native induction accepts only a local object change or an
observed relocation of the same visual category to a second tile,
with unchanged view elsewhere. Occlusion-reveal changes are allowed
only if other modified tiles were previously all-zero unobserved
cells. This rejects most camera-movement false positives.
Each factual \`word/category @ egocentric tile → motor\` effect is
supported by an actual phase synapse from lexical cell to motor.
Both word link and motor-effect link must conduct on readout.
No simulator category/action names are embedded in Rust.

## Blind heldout
The same organism and carrier checkpoint are restored after two
demonstrations. Training is frozen. On 12 fresh environment seeds
22000..22011 per object type (24 total), the evaluator stages a
fresh real object one tile ahead and sends ONLY the word intent.
No action ID/demo is sent. The native U1 chooses the acquired motor,
Human Protection must authorize it, and the real external simulator
provides the resulting POST. Success requires correct motor *and*
factual change of the pointed tile. The independent evaluator may
inspect ground-truth motor ID for scoring only. Zero successes must
be preserved, not described as passing.

Strict OPEN development target: acquire 2/2 factual action witnesses,
preserve both after native restart and frozen heldout, and succeed
24/24 heldout object interactions with zero unsupported/blocked actions.
Causal controls: native zero-weight link lesion must remove qualified
word-action policy, restore must recover; wrong object at same relative
tile must not reuse motor; no-op/unchanged PRE/POST cannot teach
affordance. These controls run in a separate native test.

## Critical interpretation
Even 24/24 does **not** demonstrate full DoorKey or MultiRoom success:
the *evaluator staged object proximity and explicitly demonstrated the
correct motor in training*. It would establish only actual
externally grounded word-conditioned motor-effect reuse at a fixed
relative visual position on shared symbolic categorical input, with
the existing U1/protection boundary. Autonomous navigation, resource
persistence, door unlocking prerequisites and RGB visual concept
discovery remain open. Motor semantics are learned from *a supplied
demonstration*, rather than self-initiated exploration; this is a
restricted curriculum, not general autonomous intelligence.
