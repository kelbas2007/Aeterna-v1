# CHILD-ZERO-WORLD-1 — newborn-like predispositions, lived history memory and real external tasks

Date: 2026-10-10. **OPEN DEVELOPMENT PASS for matched total-task target.**
Not proof of all innate concepts, newborn brain simulation, AGI,
general transfer outside the MiniGrid observation ontology, or
statistically robust performance.
[Original real-world GitHub Actions 38063925133](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38063925133)
used source \`c6a09132b46c55f6d92c85efff0c4138c519265f\`
and a declared test protocol
[CHILD_ZERO_WORLD1_PROTOCOL.md](CHILD_ZERO_WORLD1_PROTOCOL.md).

## What actually changed

A SINGLE opt-in learned general policy replaced the prior
hand-authored per-object or per-world action source in both
experiment and control. The experimental learner additionally
started with EIGHT initialized, lesionable PHYSICAL phase connections
covering weak continuity/orienting/agency/habituation/
magnitude/regulation and social/speech readiness.
Only novelty, motor contingency and fatigue/habituation have
functional influence for generic binary MiniGrid observations;
social and speech are NOT exercised by fake signals.
The experimental controller also received an internal
slow leaky trace of its *own experienced sensory history*.
No category-labelled concepts, action scripts, motor semantics,
hidden simulator pose, map, mission, language teacher or externally
staged key/door were ever supplied to cognition.

The learned controller was trained by actual own motor
PRE → opaque action → actual POST plus environment's bounded
reward under preexisting U1 and Human Protection. Its
own episodic memory was reset and the learned weights frozen
on heldout; the birth priors were present before any experience.

## Exact independently measured outcome

Farama MiniGrid 3.1.0, four independently maintained gridworld
families, training seeds 72000..72011 (48 episodes total),
heldout seeds 73000..73003 (16 episodes), fixed physical
budget 256 permitted actions/episode, same source/version
and same seed set between both arms. No sensory-mission or
privileged scoring information entered the agent.

| Actual completed FULL heldout tasks | Generic learner only | Generic learner + innate predispositions + lived experience memory |
|---|---:|---:|
| Total /16 | **6** | **9** |
| MiniGrid-Empty-5x5-v0 /4 | 0 | **4** |
| MiniGrid-DoorKey-5x5-v0 /4 | 2 | **4** |
| MiniGrid-MultiRoom-N2-S4-v0 /4 | 3 | **0** |
| MiniGrid-Unlock-v0 /4 | 1 | 1 |
| Training task completions /48 | 12 | 20 |
| Real key pickups in frozen heldout | 9 | 13 |
| Real door openings in frozen heldout | 6 | 8 |
| Actual agent movements in frozen heldout | 265 | 213 |
| Positive environment training rewards | 12 | 20 |

Matched seeded random baseline had 2/16 completed tasks.

Exact source log:
\`CHILD_ZERO1_REAL_TASKS child=9/16 no_innate=6/16 random=2/16 child_training=20/48 per_world={'MiniGrid-Empty-5x5-v0': 4, 'MiniGrid-DoorKey-5x5-v0': 4, 'MiniGrid-MultiRoom-N2-S4-v0': 0, 'MiniGrid-Unlock-v0': 1} innate_events=5910 trained_reward_events=20 verdict=DEVELOPMENT_PASS\`.

The experimental agent met the preregistered open criterion:
>=6/16 actual full task completions, > paired identical-source
base and > matched random, and genuine complex DoorKey success.
The null agent unexpectedly did BETTER on MultiRoom 3/4,
so the predispositions may have changed the balance of
exploration rather than uniformly improving it.

## Interpretation boundary, causal confound and next test

**CRITICAL:** TWO architectural changes were enabled
SIMULTANEOUSLY: inherited innate priors AND history-dependent
working memory. This first comparison cannot say which
contributed to 9/16. The initial score also did not beat
the earlier general-policy 9/16 from different seed families.
The benchmark includes pre-categorized MiniGrid sensory tokens,
not raw RGB, language, human social interaction, or biological
needs. A 16-task heldout is a small sample and cannot justify a
broad superiority/general intelligence assertion.

Source-pinned follow-up
[CHILD-ZERO-1b](../.github/workflows/child-zero-factorial.yml)
is restricted to just innate-only and memory-only arms
on the SAME seeds and same code. It is an OPEN component
ablation (not independently heldout research replication).
A true claim of robust developmental benefits would require
source-frozen replication on fresh unseen worlds/seeds,
independent task families, full history-sensitive inference
and physical circuit ablations.

No existing mature human language, face detector, object
ontology, physical semantics or "innate moral concepts"
were installed in EvoPhase. It is a functional infant-inspired
research architecture, not a child.
