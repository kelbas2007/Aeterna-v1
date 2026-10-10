# UNSTAGED-WORLD-1 — first unassisted crossworld lifetime

Date 2026-10-10; OPEN DEVELOPMENT, not frozen scientific authority.

This experiment is deliberately *more difficult* than all preceding
successful object-interaction tests. No evaluator-staged objects,
no preloaded key, no word pointing, no motor demonstrations, no
navigator/operator schedule. An external Farama 3.1.0 simulator
returns only public partial 7x7x3 categorical image and actual
reward to the native EvoPhase organism.

One persistent Rust process/phase carrier first experiences
independent DoorKey-5x5, MultiRoom-N2-S4 and Unlock, with 12
interleaved training seed episodes per family (50000..50011);
then a real checkpoint/restart and FOUR frozen heldout seeds
(51000..51003) from each family. Each episode has a budget of
128 protected externally executed actions. World order and seed
generation are evaluator setup, but the organism receives no
world name, mission string, object type, hidden map, correct motor,
goal coordinates, reward target location or experimental subgoal.
The body-relative front tile geometry is fixed and generic across
all worlds, allowing self-motor experiments rather than supplying
object-specific action affordances.

The simulator evaluator (NOT EvoPhase) separately inspects real
key pickups, door openings and successful movement as ground
truth, in addition to actual game terminal reward. Training and
heldout use the same continuing organism instead of three new
models. Frozen heldout changes neither memories nor meta weights.
Paired seeded random motors get the same heldout seeds and physical
action budgets.

Predeclared OPEN development criterion:
- >= 6/12 FULL task terminal successes in heldout, strictly > random.
- >= 1 actually acquired carried key and >= 1 actually opened door
  during heldout.
- Proven protected action execution, no privileged state leak.
Otherwise FAIL, even if the organism acquired many object or
factor rules. All negative results retained. This is an honest
difficulty test, not a guarantee of near-term solution.

Even PASS would show bounded success with categorical MiniGrid
sensor and hand-coded Rust goal-free exploration, not RGB
perception or unsupervised creation of language.
