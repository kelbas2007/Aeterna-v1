# CROSSWORLD-OBJECT-1 — one lifetime, visual memory and pointed-language grounding

Status: OPEN development, 2026-10-10; not independent AGI evidence.

## Why
External MiniGrid 1 and 2 instantiated a fresh organism for each world
family and scored 0/12 frozen goals, so by construction no learner
carried a grounded object or word between task families. The model had
an 8-bit object/category visual field in the public tile image, but the
factor learner treated the entire moving 7x7 view as volatile bits.

## Mechanism and causal boundary
New optional \`PhaseGroundedObjects\` is stored inside the SAME
\`PhaseNativeState\` and phase checkpoint as the factor model.
From factual 7x7x3x4 encoded frames it passively records candidate
tile-appearance signatures independent of positions; colors and
open/closed states may vary when only the first channel is chosen
as the identity key. It is a categorical sensor code already supplied
by MiniGrid, **not** an unsupervised object detector from RGB pixels.
A word is not invented from co-occurrence: an external teacher gives
a genuine *point and word* pair from an actually visible observation.
This is explicit word supervision, **not** the organism understanding
language or receiving the simulator's mission strings.

Native learned entries are mapped through a REAL conducting
\`visual-category-cell → lexical-cell\` phase synapse. If this synapse
is physically lesioned, recognition must disappear and return on
restore. The whole object/lexical state is included in the native
checkpoint and learned-fingerprint. Unknown/untrained words fail closed.

## Real external evaluation
A *single Rust process and organism*, never reinitialized during the
experiment, receives up to twelve DoorKey-5x5 episodes with train
seeds 4100..4111. A teacher INSPECTS ONLY public observed
\`obs["image"]\` via MiniGrid's publicly documented categorical tokens
to decide whether a **currently visible** key or door can be pointed
at. The only lesson sent into EvoPhase is \`{word, tile_index}\`.
It never receives \`OBJECT_TO_IDX\`, task/mission/map, object identity
from hidden simulator state, correct motor, or route.

After learning, cognition is checkpointed and restarted; testing
follows on new DoorKey-5x5, *different independent MultiRoom-N2-S4*
and Empty-5x5 worlds (seeds 5100..5107). No lessons are sent in
evaluation and learning is frozen. The agent still performs every
protected action in the world; each new public observation is queried
for referents via word only. The evaluator knows visible public
category indices to score precision and recall. All recognition
actions use the same visual sensor format across world types.

The predeclared strict *development* criterion: both key and door
labels must have been visually taught; native words remain 2 through
restart and holdout, actual external actions >0, heldout object
word reference has 100% precision/recall on sampled visible
instances, and at least one correctly located door from a
MultiRoom world NEVER used during language lessons.

A failure from absent teacher exposure is a real failed experiment,
not a reason to supply privileged object positions or modify the
heldout seed after seeing a result. The code always prints the
verdict and uploads JSON/console logs even if it is FAIL.

## Scientific limits
This is **supervised lexical reference transfer on a pre-symbolized
categorical visual observation**, not autonomous invention of a
concept for “key,” not grounded semantics of unlocking, not task
learning, and not an RGB object detector. Matching one type value
between worlds exploits MiniGrid's global type-code convention; it
does not establish transfer to another game's perceptual ontology.
Task reward remains separately measured and must not be conflated
with lexical-reference accuracy.

Next: true perceptual object grouping from RGB, relative spatial
memory and learned object affordances linked to words; paired
withheld ontology/appearance changes under the same action budget.
