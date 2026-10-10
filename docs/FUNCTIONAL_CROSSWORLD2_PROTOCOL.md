# FUNCTIONAL CROSSWORLD-2 — different outside task families with staged interaction

2026-10-10; OPEN development, not source-frozen scientific authority.
Previous [FUNCTIONAL-GROUNDING-1](FUNCTIONAL_GROUNDING1_RESULT.md)
demonstrated 24/24 correct fresh-seed interactions but tested
the same environment families in which each motor was taught.
This test requires a new externally generated world family for
the *same* acquired word/motor relation.

A SINGLE persistent EvoPhase runtime receives two explicit
training demonstrations: key pickup from MiniGrid-DoorKey-5x5
and door toggle from MiniGrid-MultiRoom-N2-S4. The tutor stages
each visible object in the adjacent egocentric front tile,
points and utters a word, executes the correct motor once in
the real outside environment and transmits observed PRE/POST,
motor ID and the pointed word. This is supervised motor tuition,
not autonomous discovery.

After cognition checkpoint/restart and frozen learning:
- A key from the DIFFERENT \`MiniGrid-Unlock-v0\` task family is
  experimentally staged one tile in front. Test wants the
  word \`key\` to select the formerly learned pickup motor.
- A locked door from DIFFERENT \`MiniGrid-DoorKey-5x5-v0\`
  family is experimentally staged at that same relative tile.
  The EVALUATOR externally inserts an appropriate already-carried
  key, so toggle can open the locked door. Test gives EvoPhase
  ONLY the word \`door\`, no action ID or carrying label.
- 12 new heldout seeds 33000..33011 for each of the 2
  environments. All motor decisions must go through U1
  and Human Protection. The native agent does not see
  simulator coordinates, object class names or precondition
  preparation. The scoring harness alone compares actual
  returned action and public local target-tile change.

Required open development success: both training witnesses
must be real, checkpoint must retain 2 learned affordances,
24/24 heldout externally executed interactions must choose
the correct motor and measurably change the target tile.
CI must print actual scores and preserve FAIL outcomes.

Caveat: the simulator provides the SAME categorical object
ontology in all worlds. The heldout world explicitly stages
object proximity (and a carried key for a locked door).
Passing proves reuse of a **supervised** object/motor skill
between independently created task families; NOT autonomous
navigation, not intrinsic subgoals, not inferred need to
acquire the key, not learning physical causality from RGB,
and not full DoorKey task success. The task reward is
recorded separately.
