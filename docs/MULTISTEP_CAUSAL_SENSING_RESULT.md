# Multi-step cognitive sensing — variable-depth phase-native carrier

Date: 2026-10-09  
Research branch: `research/beyond-intel4`  
Status: **OPEN DEVELOPMENT** — mechanism PASS (12/12); cold autonomous path discovery 4/4; end-to-end goal criterion 2/4. **Not a sealed or general AGI qualification.**

## Frozen history preserved

- Preregistered structural first-attempt [STRUCTURE-1](STRUCTURE1_PROTOCOL.md), [Actions 37948186490](https://github.com/kelbas2007/Aeterna-v1/actions/runs/37948186490) FAIL 0/4. It tested a two-action prerequisite, which the earlier model could not independently discover. The original experiment and verdict remain unchanged.
- Previous Fresh-2 [report](AUTONOMOUS_CYCLE_FRESH2_RESULT.md) passed a *direct-sensor* stochastic world family 4/4; it did not test arbitrary-length causal prerequisites.
- [Multistep design contract](MULTISTEP_CAUSAL_SENSING_PROTOCOL.md) was recorded before development.

## Architectural amendment, not a tenth module

The existing `PhaseTemporalEvidenceState` can now opt into a bounded physical graph of acquired
`PRE abstract cell → motor cell → POST abstract cell` relations. Each factual change grows or
updates a source-to-motor and motor-to-next-state physical phase synapse. Runtime executes
each next motor only through the existing protected `ScientificRuntime::step_unified`; a
carrier search over **conducting** links selects the next action from the *current factual
external state*. The bounded depth-8 search is generic; no 2/3/4/5-motor answer table or
hard-coded episode sampling schedule is supplied.

Physical independence and causal controls: [Actions 37954728543](https://github.com/kelbas2007/Aeterna-v1/actions/runs/37954728543), 3 independently randomized opaque assignments each for depth 2, 3, 4, 5. **12/12 mechanism controls pass.** All necessary synapse lesions remove the route; restoration and checkpoint recover it. Intermediate states do not masquerade as newly acquired cues.

**Cold discovery was not initially working.** [Actions 37954989940](https://github.com/kelbas2007/Aeterna-v1/actions/runs/37954989940) 0/4: only depth 2 autonomously discovered a complete path; depth 3–5 completed none. Independent log inspection found goal-completing motor 0 hijacked 186/192 first actions in depth 3 because a factual goal endpoint was being treated as a promising exploration frontier. General physical goal witness now uses only actual protected `task_outcome=1.0`, and its essential synapse lesion removes that authority; no private correct action labels. [Physical causal witness and regression](https://github.com/kelbas2007/Aeterna-v1/actions/runs/37956951637).

Another general fix restores factual **belief-conditioned trial counts** when a motor changes a noncue state: the one-step outcome UCB policy formerly regarded already attempted motors as forever untested in multistep mode. The new causal test confirms an actually failed outcome now moves its next motor hypothesis instead of repetitively selecting the same opaque index.

Last amendment uses the **length of the learned physically conducting path** to price further sensing when the rival beliefs imply different action consequences; it receives no supplied desired depth.

## Last open development run

[GitHub Actions 37958136237](https://github.com/kelbas2007/Aeterna-v1/actions/runs/37958136237), cognitive code `28764868d9f22891d0e8e30ce335e8b69fef25cb`, new fixed role/sample assignments inherited from the explicitly open development generator.

| Depth (executed actions per sample) | Completed physical chains during 192 plastic training episodes | Frozen correct /80 | First noisy cue oracle /80 | Fixed three-cue oracle /80 | Frozen episodes with 2+ real extra samples | Verdict (>=64 and > first-cue oracle) |
|---|---:|---:|---:|---:|---:|---|
| 2 | 375 | 59 | 55 | 59 | 80/80 | FAIL |
| 3 | 380 | 67 | 66 | 67 | 80/80 | PASS |
| 4 | 378 | 65 | 56 | 65 | 80/80 | PASS |
| 5 | 375 | 59 | 52 | 59 | 80/80 | FAIL |
| **Total** | **1508** | **250/320 (78.13%)** | **229/320** | **250/320** | **320/320** | **2/4** |

All four acquired a conducting physical final sensing action (readout 1.0), made 80/80 terminal commits, reported zero blocked or unsupported actions, and preserved frozen U1 parameters after checkpoint restart. Depth 5 deliberately reuses one opaque motor in different stages, requiring real sensory-context differentiation.

The open evaluator was iterated after observed failures. The final scores match the realized three-cue benchmark in aggregate and **do not demonstrate a generalized advantage over that benchmark**. This is one persistent training lifetime *per depth*, four separate organisms; the transferred generic visual and U1 foundation are not trained from scratch. The underlying environment remains a synthetic hidden-binary-cause family.

## Next meaningful boundary

Learn **value of information conditioned on remaining action budget and physical chain length**, without a host-selected sample count or sacrificing terminal execution. Depth 5 incurs five protected motor executions for every independent observation; with only 16 steps per episode, additional information has substantial opportunity cost. A structurally new frozen-world test must be preregistered only after the general mechanism is stabilized. Do not rerun the consumed STRUCTURE-1 authority attempt or call this diagnostic AGI.
