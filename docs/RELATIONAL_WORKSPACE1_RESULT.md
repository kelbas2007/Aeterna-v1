# RELATIONAL-WORKSPACE-1 — unique-first candidate assumption was falsified by real observations

2026-10-10. **Native physical tests PASS; actual paired Farama
MemoryS7 CAUSAL USE FAIL**.
[First actual Actions 38074563658](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38074563658)
with [predeclared contract](RELATIONAL_WORKSPACE1_PROTOCOL.md).

## What the source actually changed

The same general learned-only EvoPhase policy received an
opt-in 96-dimensional *relational feature representation*
which stores an initially observed rare foreground appearance
and compares later visually similar candidates after occlusion.
No motor is selected by this operator: all action choices remain
with the generic learned policy, subject to U1 and
Human Protection. A physical phase relay synapse determines
whether the history-conditioned relational readout is available.
There are no task-specific key/ball IDs, motor/route scripts,
correct-branch tables or hidden simulator coordinates
inside the learner. Raw MiniGrid observations remain
PRE-CATEGORIZED categorical-tile data, not RGB learned vision.

Native tests PASSED 2/2, including different cue history
with same present image, 10 fully occluded frames,
physical synapse lesion/restore and native checkpoint
protection. The first implementation was restricted
to forming a subject only when exactly ONE rare foreground
visual candidate was identifiable in the INITIAL frame.

## Independent live software-world diagnosis

Two independent carrier lifetimes were compared across
128 training episodes (seeds 150000..150127) and 32 NEW
balanced heldout layouts (151000..151031) with
both original and causally cue-swapped variant
per map (64 heldout decisions per arm). Mission,
map, correct exit, cue ID and evaluator states were never
sent to the Rust organism. Actual independent Farama
MiniGrid 3.1.0 reward and final physical positions
scored the outcome. Frozen native checkpoint restored
before each cue-swapped heldout episode.

| Actual outcome | Generic episode memory only | Opt-in unique-subject relational representation |
|---|---:|---:|
| Rewarded heldout tasks /64 | 32 | 32 |
| Pairs correctly solved on both cues /32 | 0 | 0 |
| Pairs where physical exit changed after cue exchange | 0 | 0 |
| Pairs where original cue reached sensors | 8 | 8 |
| Pairs where new relational subject was actually bound | n/a | **0** |
| Relational current view had an identity match | n/a | 0 |

Both policies chose an unchanged exit under every
counterfactual cue substitution. On 24/32 pairs
the decisive original cue never reached input at all.

**Essential causal failure diagnosis:** although 8/32
pairs actually showed the starting cue, the stricter
unique-subject rule bound ZERO subjects on actual
observations, because other rare features co-occurred.
Thus the module was unable to create the memory in
real scenes and should NOT be described as an
acquired real-world relational intelligence capability.
The task failure is not solved by the valid synthetic
native controls.

The next open experiment, [RELATIONAL-HYPOTHESES-2](RELATIONAL_HYPOTHESES2_PROTOCOL.md),
changes only the REPRESENTATION of uncertain
observation content to a bounded hypothesis set
instead of demanding a single uncontested cue.
No new motor route/scene-specific correct answer is installed.
Any PASS must prove actual counterfactual cue-use on
new heldout seeds, not just nonzero memory readout.
