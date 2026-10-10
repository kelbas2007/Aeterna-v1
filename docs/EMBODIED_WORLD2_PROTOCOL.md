# EMBODIED-WORLD-2 — multiworld goal discovery with more external experience

2026-10-10. OPEN DEVELOPMENT, not fresh independent scientific authority.

The previous source-fixed paired [EMBODIED-WORLD-1](EMBODIED_WORLD1_RESULT.md)
showed 7/36 actual training task successes (all Unlock), 1/12
heldout (Unlock), versus 0/12 ablation and 1/12 random on the
SAME seeds. Actual heldout position changes were 143 vs 1,
with physically verified key/door interactions. This did NOT
meet the whole-task PASS threshold (>=6/12 and > random).
Those consumed seeds and results remain preserved.

This new experiment tests whether a longer sequence of
genuinely independent environments and longer action budget
allow the **same unchanged cognitive source** to acquire
more reusable task and sensor-flow experience. It is
NOT an algorithmic repair and contains NO game-specific
action scripts, map, goal coordinate, action/mission oracle or
environment-internal state in the native learner.

Physics is external Farama minigrid==3.1.0. One persistent
EvoPhase lifetime experiences FOUR families in this order:
Empty-5x5, DoorKey-5x5, MultiRoom-N2-S4, Unlock. Empty offers
a different externally governed navigation/goal environment,
not an artificially designed teaching transition.
Train seeds 56000..56011, interleaved across the four families,
48 training episodes total. Checkpoint/restart then frozen
heldout seeds 57000..57003 for each world (16 tasks).
Maximum **256 protected real external motor steps** per episode.
Paired unmodified self-object and embodied sensory-flow modes
receive exactly the same worlds, seeds, physics and budget.
Matched random baseline same heldout seeds/budget.

NO teacher labels/words, NO motor demonstrations, NO staged
objects, NO examiner-inserted inventory or hidden map.
The Rust organism sees only standard public MiniGrid partial
categorical tiles (588 binary channels), previous actual
protected action and factual reward. Evaluator privately
measures completion by positive real task reward and
actual physical key pickup/door opening/navigation.

Predeclared OPEN development PASS criterion:
- >= 6 / 16 actual frozen terminal task successes.
- score strictly exceeds both matched ablated source mode
  and matched random action baseline.
- >= 1 independently completed heldout DoorKey OR
  MultiRoom FULL task, not just the easier Empty/Unlock.
- at least one physically verified heldout key pickup and
  one actual door opening.
- valid full source run, nonzero protected actions, same
  heldout source/seeds and no privileged leak.
Fail otherwise; mere movement/event counts do not qualify.

Even a positive outcome is restricted to category-coded
partial MiniGrid and explicitly coded Rust optic-flow
exploration, not AGI/unsupervised natural language.
