# FUNCTIONAL-GROUNDING-1 — externally witnessed object motor meaning

Date 2026-10-10. Branch \`research/beyond-intel4\`.
**OPEN DEVELOPMENT PASS for the supervised interaction criterion**, not independent
AGI qualification and not full external-world task solving.
[GitHub Actions run 38041549234](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38041549234)
at first run source commit \`0fd959355640aac841849c94ed0c5fbc5090af01\`.

## Verified facts

The independently installed Farama MiniGrid environment executed exactly
**two tutor-controlled training motors**, one witnessed \`key\` pickup from
DoorKey-5x5 and one witnessed \`door\` toggle in MultiRoom-N2-S4.
The learner received public partial observation before and after,
a deictically taught visible word and the *opaque ID of the motor
actually executed*. It did NOT receive internal object ID, coordinate,
action-name string, hidden map, or stored world model.
Both local effects were accepted and saved as physically conducting
lexical-object-motor synapses in the continuing EvoPhase carrier.

After a cognition checkpoint/restart and frozen learning, a hidden
test/evaluator placed the target object immediately in front of
the agent on **12 fresh DoorKey seeds for keys and 12 fresh
MultiRoom seeds for doors**. The evaluator supplied only the target
word, never an action ID. U1 selected the acquired motor and Human
Protection screened actual execution. In all **24/24** episodes:
the chosen motor matched the evaluator's action truth and the REAL
external MiniGrid observation showed the pointed object changed.
No task terminal reward was achieved by this staged interaction.

Actual log:
\`FUNCTIONAL_EXTERNAL1 teacher=2 witnesses=[True, True] affordances=2 object_interactions=24/24 goal_rewards=0\`
\`FUNCTIONAL_EXTERNAL1_SUMMARY verdict=DEVELOPMENT_PASS claim=SUPERVISED_MOTOR_AFFORDANCE_NOT_AUTONOMOUS_TASK_SOLVING\`.

Matched native unit test [38041433823](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38041433823)
also passed: a no-op could not teach an affordance, the wrong visible
object did not attract the learned motor, an altered appearance/state
preserved useful recognition, checkpoint retained cognition,
physical synapse lesion eliminated the learned motor response and
restoration returned it; the motor was selected/executed via U1 and
Human Protection rather than bypassing either.

## Qualification and critical shortcomings

**This is motor-supervised grounding, not autonomous functional reasoning.**
A researcher demonstrated the correct action on each object, and both
training and heldout evaluation explicitly staged objects in the agent's
immediate egocentric front tile. None of these 24 tests required
autonomous key search, movement, unlocking a locked door, or collecting
a delayed task reward. Results do not establish cross-world *affordance*
transfer between different MiniGrid families for the same object role,
only retention and reuse on new seed instances of each training world.
MiniGrid already supplies categorical object-type tokens rather than RGB
images. The learned effect recognizer is an explicitly programmed Rust
witness filter with physical synapse gating. The tutor's motor and word
are explicit training information, not self-discovered.

Next required evidence: test a word/action learned in one external task
family against an object with the same grounded role in an unseen
independent task family and newly staged preconditions; then remove
the tutor's motor, remove object staging, acquire spatial permanence,
and attempt full autonomous DoorKey completion with a fair random
comparison. Retain all failed transfer cases.
