# EXTERNAL-WORLD-1 — independent Farama MiniGrid integration

Date: 2026-10-10. Status: OPEN DEVELOPMENT, NOT scientific qualification.
The environment logic is installed from the external
[Farama-Foundation/Minigrid](https://github.com/Farama-Foundation/Minigrid)
package, pinned at \`minigrid==3.1.0\`. Our repo does not define its task
transitions. Three external registered environments:

- \`MiniGrid-Empty-5x5-v0\`: first navigation and sparse reward.
- \`MiniGrid-DoorKey-5x5-v0\`: real external key pickup, door unlock and exit.
- \`MiniGrid-MultiRoom-N2-S4-v0\`: multiroom navigation through doors.

## Live executable boundary

\`src/bin/external_world_agent.rs\` is a persistent independent Rust process,
holding the same phase-native EvoPhase organism across episodes.
\`scripts/external_minigrid_benchmark.py\` instantiates external Gymnasium
objects in another process. It sends only the standard PUBLIC PARTIAL
observation \`obs["image"]\`, encoded losslessly as four binary bits for
each of 7x7x3 tile fields. There are 588 binary sensory channels.
The mission text, simulator internal map, agent absolute coordinates,
game object API, and correct action or transition scripts NEVER enter
EvoPhase. Its motors are the public Gymnasium discrete 0..6 actions,
delivered only after \`ScientificRuntime::step_unified\` selects with
the existing U1 and screens using Human Protection.

After an action has actually executed, the Python host returns only
the fresh public observation and bounded immediate reward. Positive
reward, if seen, can record a factual terminal sensory goal inside
the same phase-native carrier. It is not imported from a simulator
goal table. Resetting starts a new factual episode but does not reset
carrier knowledge. The first eight episodes use seeds 100..107 and
learning enabled; the four frozen tests use seeds 900..903.
The entire source/evaluator is visible, so this is an OPEN development
diagnostic only, not unconsumed independent scientific authority.
Every episode has 64 protected external action steps or the
environment's own termination. Matched seeded random motors evaluate
the SAME heldout seed set and action budgets.

Run locally on Linux with Rust 1.99.0 and Python 3.10+:

\`\`\`bash
python3 -m pip install 'minigrid==3.1.0'
cargo build --locked --release --bin external_world_agent
python3 scripts/external_minigrid_benchmark.py \
  --agent target/release/external_world_agent \
  --output external-world1.json --budget 64
\`\`\`

CI runs the same steps and uploads both logs and full JSON. A green
compilation/test job does not establish a cognitive SUCCESS. The
benchmark ALWAYS reports the exact train/frozen evaluation/random
counts, including zero success, and an explicitly unqualified
\`MEASURED_NO_SCIENTIFIC_PASS\` verdict.

## Critical limitations

These are real independent *software* worlds, not the physical real
world. Default MiniGrid \`image\` is a categorical tile encoding, NOT
raw RGB pixels, not continuous-object vision, and not randomly
changing visual feature IDs. The one-hot-like binary serialization
provides positional structure but NO object semantics. The current
raw factor learner is a handcrafted Rust effect/guard induction
algorithm with bounded BFS; no evidence yet that it will achieve
non-zero external benchmark scores. The sparse terminal reward
goal may be unlearnable on hard worlds. Success on DoorKey would
be meaningful but must be confirmed against heldout and baselines;
zero success is an expected and publishable possible result.

Next after honest evidence: partial-view temporal object permanence,
multistep policy credit under sparse reward, rotation-invariant
feature binding, then pixel RGB mode and Crafter (64x64x3, 17
actions) with equal external evaluation criteria. Do not substitute
MiniGrid tile coordinates for true learned object identities.
