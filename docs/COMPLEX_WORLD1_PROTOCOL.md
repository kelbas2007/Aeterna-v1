# COMPLEX-WORLD-1 — unknown prerequisite labyrinth, cold goal acquisition

Date: 2026-10-10
Branch: `research/beyond-intel4`
Status: **OPEN DEVELOPMENT PROTOCOL, RECORDED BEFORE EXECUTION**.
Source prior to new development: `da24713a8751ce8cf21ef7f431799effbcb210a1`.
Do not relabel it as an independently frozen AGI qualification.

## What makes this a new causal structure

Not another permutation of the previously exposed six-state, four-step graph.
A 14-state world with six opaque motors, one nine-step mandatory route,
a key acquisition and subsequent generator activation before the gated exit,
a physically reachable but misleading power-first path, recoverable traps,
state-dependent motor reuse, and a detour around an interior transition.
The environmental causal law is hidden in the evaluator and is NEVER
provided as an edge list, motor names, state ranks or lesson schedule.

Synthetic evaluator-private logical stages:
- Start (0) → foyer (1) → key room (2) → **key acquired** (3) →
  generator access (4) → **power on with key** (5) →
  gate approach (6) → secured corridor (7) → exit hall (8) → goal (9).
- Detour: key-acquired (3) → alternative generator route (10) → powered-with-key (5).
- Wrong-first branches: foyer (1) → premature power room (11) → trap (12) → foyer (1), and foyer (1) → dead end (13) → foyer (1).
- Reversible but unproductive detours from the gate/corridor are allowed.
- Actions are all six opaque motors, shuffled across separate arms; the SAME motor may cause different effects in different states.
- No PRE/action/POST demonstrations in the target. Goal is a recognizable raw sensory state. Factual POST and bounded success reward exclusively from protected external execution.

## Training and evaluation

Fixed open-seed `0xB17E_2026_0C0D_0101`, four never-before-used visual/motor
role assignments. General pretrained raw perception and U1 checkpoint from
prior research are explicitly permitted (NOT from-zero AGI). Each arm runs
up to 256 bounded training episodes, max 24 executed safe actions per episode,
one continuous physical organism (four arms are four independent organisms).
No evaluator selects an action. Track actual discoveries of the nine base
chain edges plus the two detour edges, plus any trap/exit edges.

Checkpoint/restart and model freeze for a fresh intact nine-step goal
within 16 actions. Then in the SAME organism, resume learning and silently
block key→generator transition (3→4), without a change flag to cognition.
An actual no-op on the expected action must physically withdraw the obsolete
transition. Replan through the 3→10→5 detour and reach the goal within 20
protected actions. Check causal lesion and exact restore of the initial
essential physical synapse; no metadata-only route survival.

## Predeclared development criterion

All four arms must (1) learn >=11 distinct nontrivial physical transitions
from self-chosen factual motor actions including the entire original route
and the alternate detour, (2) learn at least one distractor/trap branch,
(3) reach the original goal in exactly nine protected actions in frozen
evaluation with unchanged U1 weights, (4) after unexpected changed-law
no-op, dynamically reach goal via BOTH detour edges within twenty actions,
(5) survive checkpoint, pass physical lesion/restore and have zero
unavailable/blocked actions. This is an **open development** generator:
failures must be recorded and investigated without changing its seed,
topology or thresholds retroactively.

Two limitations are explicit: the underlying six-motor, 14-state environment
is still synthetic, and EvoPhase's bounded graph traversal/frontier search
is Rust code over acquired phase-synaptic weights. Success would not prove
general AGI, unknown-language reasoning or neural invention of graph-search
algorithms. A separate source-frozen first-attempt unseen-topology
qualification must follow any successful open development.
