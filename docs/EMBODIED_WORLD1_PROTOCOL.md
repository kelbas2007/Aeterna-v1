# EMBODIED-WORLD-1 — learned egocentric visual motion across independent tasks

Date: 2026-10-10. OPEN DEVELOPMENT, no frozen independent scientific authority.

The user requires actual full-task completion on outside environments, not
isolated local object manipulation. The previous UNSTAGED-WORLD-1
[38046685780](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38046685780)
and UNSTAGED-WORLD-2
[38046953463](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38046953463)
remained full task FAIL 0/12. UNSTAGED-WORLD-2 moved the agent only
once across 12 frozen episodes. Local object/camera interaction dominated
motor arbitration, and goal-free exploration was not spatially coherent.

## Modification
An opt-in \`PhaseEmbodiedNavigator\` lives IN the same persistent native
grounded-object state, not in the Python evaluator. It estimates whether
a **real protected motor's actual PRE/POST public partial visual frame**
contains the one-tile ego-translation optic pattern (overlapping tile
features shifted toward the body). The algorithm receives no simulator
agent position/direction, motor meaning or map data. From factual
experience it records which opaque motors tend to cause view translation,
change a scene without translation or produce no observable change.
The sensor coordinate system is the same public partial 7×7 tile
format as previous MiniGrid projects. The inference is a generic
hand-coded sensory-flow rule in Rust, not a neural invention or
3D perception.

The same U1 and Human Protection still execute every external motor.
It arbitrates between BOUNDED local object experimentation for rare
visible front-tile appearance patterns and novelty-driven exploration
of non-object scenes. The explorer favors an actually supported motor
that repeatedly generates a forward-like image translation; all
choices are based only on actual sensory outcome statistics, episode
observation-action novelty and generic per-action coverage.
Factual motor trials persist in checkpoint; transient per-episode
view/action history does not. Learned object affordances remain intact.

## Paired actual simulator comparison
Same source and SAME train/heldout seeds for ablated and active mode:
- Farama MiniGrid 3.1.0 DoorKey-5x5, MultiRoom-N2-S4, Unlock.
- Training seeds 54000..54011 per family, interleaved ONE continuing
  organism; 128 protected steps per episode.
- Native checkpoint/restart, frozen heldout seeds 55000..55003
  per family (12 episodes), still ONE organism.
- NO object staging, no carried-key insertion, no teacher words, no
  motor examples, no mission texts, no environment map or hidden
  coordinates sent to cognition.
- Baseline arm: exactly the existing native self-object experiment
  WITHOUT the embodied optic-flow mode. Proposed intervention:
  same underlying model with opt-in real visual motion acquisition.
- Independently seeded random control gets identical heldout worlds
  and step budget. Actual terminal simulator reward is the only
  complete-task success criterion, while actual changes of agent
  absolute location, carrying a key and opening a door are measured
  by the evaluation process only, never sent to EvoPhase.

Predeclared OPEN development claim: to call this full-task success,
the embodied organism must achieve >=6/12 terminal task rewards
and exceed both paired ablated and random scores, plus actual
heldout key pickup and door opening. Otherwise DEVELOPMENT_FAIL,
even if its movement or object actions improve. Paired movement
and visible-motion evidence are diagnostic only. The validity gate
requires nonzero protected action executions, strict mode and
seed equality, no use of privileged simulator state inside agent.

Do not convert green CI into a cognitive PASS. Save every log as first
actual development evidence. Preserve earlier consumed failures.

## Limitations
Categorical MiniGrid public tile sensor, known field geometry,
handwritten generic optic-flow/novelty search. Does NOT qualify raw
RGB object discovery, independently induced movement operators,
robust probabilistic spatial map, abstract goal language or AGI.
