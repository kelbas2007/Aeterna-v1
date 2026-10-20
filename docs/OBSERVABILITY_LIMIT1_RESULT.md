# OBSERVABILITY-LIMIT1 — exposing a true physical state sensor REGRESSES current AETERNA

**Completed developer-disclosed ENGINEERING experiment, 2026-10-20.
Verdict: `NEGATIVE_SENSOR_ADDITION_UNDER_EXISTING_U1`.**
The [protocol](OBSERVABILITY_LIMIT1_PROTOCOL.md) was committed FIRST
[`10c98b4b`](https://github.com/kelbas2007/Aeterna-v1/commit/10c98b4b482c5286055c0d63d85ac28f2e2b4862),
before the sensor comparator source
[`5faf58ce`](https://github.com/kelbas2007/Aeterna-v1/commit/5faf58ce62a22d6854a2bdfc97648f4e635fa328).
No cognitive code was changed, no new sensor was invented by AETERNA.

[Exact Linux/Windows test run **39413022974**](https://github.com/kelbas2007/Aeterna-v1/actions/runs/39413022974):
both OS **2/2 tests PASS**, identical numerical metrics; source audit
PASS, original 81 source-bound files intact, original RSP2 unchanged.
Evidence artifacts:
[Linux](https://github.com/kelbas2007/Aeterna-v1/actions/runs/39413022974/artifacts/12260367456)
and [Windows](https://github.com/kelbas2007/Aeterna-v1/actions/runs/39413022974/artifacts/12259309711)
hold actual PRE/action/POST evidence and complete real action/goal traces.

Same 36 source-disclosed world/permutation pairs, same 24 episodes/world
(864 episodes per arm), max 18 real actions/episode, zero human-scheduled
training motors, all four motor offers continuously available, same
goal mask `[Some(1),None]`, same U1/Learner/Complexity Necessity and
active history experiment settings, same 2 input sensor cells, same
bounded computation and HumanProtection on **every** action.
Only difference: BLIND second channel is actually constant 0;
SENSORED second channel faithfully exposes the world's latent-stage
integer AFTER each executed physical motor. This is an
*environment-supplied* measurement, not a learned or sensed latent
variable manufactured by the organism.

| Real goal family | Blind successes | True-stage sensor successes | Blind physical actions | Sensored physical actions |
|---|---:|---:|---:|---:|
| 4-motor prerequisites (12 worlds, 288 eps) | 195 | **18** | 2,645 | 5,061 |
| 5-motor repeated prerequisites (12 worlds, 288 eps) | 8 | **1** | 5,125 | 5,182 |
| Familiar 3-motor prerequisite (6 worlds, 144 eps) | 140 | **31** | 576 | 2,353 |
| Direct-goal negative control (6 worlds, 144 eps) | 144 | **144** | 279 | 279 |
| **Total /864** | **487** | **194** | **8,625** | **12,875** |

Loss of 293 solved episodes and +4,250 actual motor actions.
**18/36 paired worlds regressed**; early-six 101→46, last-six
132→52. Full actual completed acquired own-option counts
(depth 3,4,5): BLIND **137,184,2**; RICH **0,0,0**. This is an
architectural adaptation failure, not evidence that the true sensor
was false or a new scientific one-use qualification.

## Probable mechanism; explicitly a hypothesis, not yet proven

`carrier_autobiographical_option_metered` and the current
`carrier_variable_depth_option_metered` identify their preparatory
motor sequence by requiring `PRE == POST` across the **entire**
observation vector. This holds when the only signal is flat progress,
but when physically meaningful latent preparation becomes *visible*,
every preparatory transition changes a different coordinate.
The very signal that would allow ordinary state-aware goal planning
suppresses the old invisible-preparation macro recognizer. Thus
extra data can make the behavior policy WORSE.

This effect alone does not prove the full causal attribution:
the additional coordinate also changes factual model supports,
native U1 credit features and exploratory novelty.
The next test must isolate whether opting into the **already-existing**
certified factual transition planner restores performance on rich
physical observations, without changing the core source or the
world/seed. If it does not, the actual failure is broader than
a missing planner switch and demands a change to state abstraction,
action-history model and goal-conditional credit.

## Integrity limits

This is a repeated **exposed**, deterministic, finite-state synthetic
family, not fresh independent generalization. Sensored stage is an
intentionally privileged direct measuring instrument; AETERNA does NOT
autonomously request, invent or add the physical sensor. No general
POMDP or AGI claim. The existing one-use SELF-ORIGINATED-OPTION1
science score is unchanged, and its exact first seed must not be
repeated to reinterpret this failure.
