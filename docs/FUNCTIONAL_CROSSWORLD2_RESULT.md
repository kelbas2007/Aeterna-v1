# FUNCTIONAL-CROSSWORLD-2 — factual transfer of supervised object motor across task families

Date 2026-10-10. Status: **OPEN DEVELOPMENT PASS 24/24 for staged
supervised physical object interaction**, NOT autonomous full-task
solution, NOT independent source-frozen scientific authority.
First source-run evidence:
[GitHub Actions run 38041785647](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38041785647),
run source tip \`a728db0fc472be70f8d490733c5dcd3067cbf1ff\`.

## Actual outcome

After two actual Farama MiniGrid motor demonstrations (one key
pickup from DoorKey and one door toggle from MultiRoom), the
SAME continuous EvoPhase organism stored two grounded
\`word→observed object-at-relative-tile→motor effect\` relations in
phase synapses. It checkpointed/restarted without retraining, and
learning remained frozen during heldout.

A different task family tested EACH previously taught relation:
- **KEY** learned in DoorKey-5x5: actual physical key pickup on
  12 new seeds of independent **MiniGrid-Unlock-v0**.
- **DOOR** learned in MultiRoom-N2-S4: actual physical toggle
  of a locked DoorKey-5x5 door in 12 new seeds. The EXAMINER
  supplied a carried matching key as a staged precondition,
  but did not inform cognition of that fact as a label or motor.

**24/24** heldout tests emitted a motor selected by the native U1,
passed through Human Protection, and had a real witnessed effect
on the object in the external simulator. No heldout teaching
motor IDs, action scripts or privileged internal map entered EvoPhase.
No terminal game/task rewards occurred.

Actual CI:
\`FUNCTIONAL_CROSSWORLD2_SUMMARY successful=24/24 rewards=0 verdict=DEVELOPMENT_PASS claim=STAGED_SUPERVISED_CROSSWORLD_SKILLS\`.

A separate native causal control,
[run 38041433823](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38041433823),
verified checkpoint, phase-synapse lesion/restore, no-op tuition
rejection, wrong-object refusal and U1/protected actual execution.

## What this genuinely means

This expands prior **word-only one-shot reference** into
**learned, physically grounded motor use**, and supports transfer of
a SUPERVISED object/motor relation to a *different task family*
within Farama MiniGrid's shared categorical sensor ontology.
The transfer was not due to a preloaded key-to-pickup
or door-to-toggle Rust table: an explicit tutor demonstration's
opaque motor was stored only after actual PRE/action/POST and
gated by physically conducting lexical/category→motor synapses.

**Critical caveats:** the tutor explicitly executed the desired
motor during training, which is strong supervision. The evaluator
staged the target object one tile ahead for EVERY heldout.
For the locked door it also staged an already-held matching key.
Thus the agent did **not** discover the action unaided, navigate
to either object, retrieve the key itself, infer the key-dependent
precondition, plan a causal subgoal chain, or win the game.
The external miniworld supplies symbolic categorical object type
IDs (not RGB recognition) globally consistent between world
families. Repeated heldout tests share the same visual category
ontology. This benchmark is intentionally an interaction-level
test, not an agentic task-level one. **0 task rewards** is not
overridden by 24/24 interaction accuracy.

The scientifically significant next gate must remove *examiner
staging and motor tuition* on a separately frozen source/seeds.
One lifelong organism must encounter a key and locked door,
discover effects by autonomous protected action, retain the key
while navigating out of view, decide that the key is required
for the door, then execute the complete chain in a fresh
outside maze. Include matched no-object-memory, no-affordance,
no-history and random baselines at equal physical action budgets.
