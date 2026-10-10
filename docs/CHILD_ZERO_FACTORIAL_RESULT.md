# CHILD-ZERO-1b — immutable-source 2×2 ablation of innate priors and lived memory

Date 2026-10-10. **OPEN COMPONENT ABLATION**, not clean independent
heldout seed replication or a proof of the human infant's mind.
Original combined experiment:
[CHILD-ZERO-1 run 38063925133](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38063925133).
Single-factor run:
[GitHub 38064344812](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38064344812).

To prevent post-hoc algorithm tuning, this factorial experiment
explicitly checked out the exact immutable original source
\`c6a09132b46c55f6d92c85efff0c4138c519265f\`,
and compared the TWO missing single-factor arms against
the two already consumed arms on EXACT identical training
seeds 72000..72011, frozen heldout 73000..73003,
and fixed 256-step budget per independent Farama
MiniGrid environment. Four real world families:
Empty-5x5, DoorKey-5x5, MultiRoom-N2-S4 and Unlock.
No world internal state, teacher action/word/goal, staged
objects, map, hidden position/compass, route or action
meaning was passed to the physical EvePhase controller.
U1/HP mediated all actually executed motors.

## All four combinations, unchanged source and same seeds

| Initial innate physical biases | Subjective experienced-events memory | Full real externally rewarded heldout tasks /16 | Empty /4 | DoorKey /4 | MultiRoom /4 | Unlock /4 |
|---|---|---:|---:|---:|---:|---:|
| OFF | OFF | **6** | 0 | 2 | 3 | 1 |
| ON | OFF | **4** | 0 | 0 | 3 | 1 |
| OFF | ON | **5** | 0 | 2 | 3 | 0 |
| ON | ON | **9** | 4 | 4 | 0 | 1 |

The random matched baseline was 2/16.

Raw first test:
\`CHILD_ZERO1_REAL_TASKS child=9/16 no_innate=6/16 random=2/16 ... verdict=DEVELOPMENT_PASS\`

Raw immutable-source factorial:
\`CHILD_ZERO_FACTORIAL same_source=true same_seeds=true base=6/16 combined=9/16 innate_only=4/16 memory_only=5/16 innate_per_world={'MiniGrid-Empty-5x5-v0': 0, 'MiniGrid-DoorKey-5x5-v0': 0, 'MiniGrid-MultiRoom-N2-S4-v0': 3, 'MiniGrid-Unlock-v0': 1} memory_per_world={'MiniGrid-Empty-5x5-v0': 0, 'MiniGrid-DoorKey-5x5-v0': 2, 'MiniGrid-MultiRoom-N2-S4-v0': 3, 'MiniGrid-Unlock-v0': 0} claim=OPEN_COMPONENT_ABLATION_NOT_INDEPENDENT_REPLICATION\`.

## Crucial causal inference

The inherited prior block ALONE was worse than the base learner
(4/16 vs 6/16) on these seeds. Episode memory ALONE was also
worse (5/16 vs 6/16). The COMBINATION was the best (9/16),
showing a **non-additive, context-dependent interaction**.
Its four Empty successes and four DoorKey successes coincided
with zero MultiRoom successes, while the other arms all solved
three MultiRoom episodes. Therefore it would be INCORRECT
to claim either innate priors or a memory trace is proven
individually effective across environments, or that the system
has been endowed with actual human infant intelligence.
The four combinations share all the same seeds and a source,
so their behavior is paired, but this result is a SINGLE
small seed family and highly nonlinear action-learning
dynamics. It could be lucky stochastic trajectory alignment.
A source-sealed independent seed replication and per-prior
physical lesion controls are required for any stronger claim.

Only generic novelty/contingency/regulation functions were
meaningfully exercised on binary task observations. Readiness
for human speech/faces is currently NOT actual perception,
and rough magnitude/continuity slots need further functional
grounding. A human has biological autonomic needs,
body-specific reflex circuits, prenatal language experience,
and physical proprioception that are not modeled by merely
configuring eight numeric phase links.

## Engineering verdict

Preserve both on/off settings and DO NOT make combined
innate+memory modes the unquestioned default until fresh
independent evidence shows durable benefit. Do NOT append
MiniGrid-specific reflexes or a ready-made adult object
concept table as a repair for these 16 tasks.
The correct research target remains a self-organizing,
history-dependent, multi-sensory, causally grounded
developing organism, with predictions, embodied learning
and new skills acquired from real interaction.
