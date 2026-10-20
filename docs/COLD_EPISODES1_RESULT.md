# COLD-EPISODES1 — self-selected multi-episode causal acquisition, development PASS

Date 2026-10-19. **Open/source-disclosed engineering result; NOT independent-world science qualification.**

## Provenance
- Protocol committed BEFORE the implementation: `a5c1578d80f6471963ca585ef4c6ecf7dc94de12` ([contract](COLD_EPISODES1_DEVELOPMENT.md)).
- Exact first diagnostic source commit: `77a5dc6a165b69d49b942bae755eb98370ae3f6e`.
- [Linux GitHub Actions first executed diagnostic](https://github.com/kelbas2007/Aeterna-v1/actions/runs/39316680184): **SUCCESS**. Stdout `COLD_EPISODES1_RESULT` contains complete per-episode actions/successes, all modes, weights and safety checks.
- The world has one numeric sensor and an evaluator-only readiness variable. It runs 24 consecutive goal-1 episodes for each of two exposed motor-role maps. All FOUR actions remain freely offered on EVERY step, with real HumanProtection permit and actual external PRE/action/POST. There are **zero tutor-selected training motors**.
- The evaluator starts the next physical WORLD episode after goal success or action cap, then sends ONLY the actual new sensor observation to `observe_constructing_external_restart`. No motor is selected during this reset; the Learner journal, synapses, resource costs and safety audit persist. Each actual first action after reset carries `restart=true`, ensuring causal-history predicates do not cross episode boundaries.

## Actual observed outcomes — each mode 48 episodes (2 physical role orders × 24)

| Mode | Combined tasks solved / 48 | Total protected actions | Selected acquired history goal plans | Selected epistemic probes |
| --- | ---: | ---: | ---: | ---: |
| A. Factual U1 only | **30** | **566** | 0 | 0 |
| B. Factual + one-action learned causal history | **33** | **456** | 28 | 0 |
| C. Factual + causal history + self-selected experiments | **44** | **237** | **58** | **26** |
| D. C + goal-only instrumental-credit prototype | **11** | **761** | **0** | 47 |

By motor mapping:
- `[0,1,2,3]`: A 16/24 (292 acts), B 19/24 (192), C **21/24 (129)**, D **1/24 (418)**.
- `[2,0,3,1]`: A 14/24 (274), B 14/24 (264), C **23/24 (108)**, D **10/24 (343)**.

Mode C compared with A improved goal success 30→44/48 (+14 tasks, 62.5→91.67%) and reduced actually executed actions 566→237 (329 fewer, about 58.1%). The first-six episode successes were 4/6 and 5/6 in mode C (total **9/12**); the last six were **6/6 and 6/6** (total **12/12**). The last six episodes of both worlds mostly required exactly two physical actions, matching the nontrivial hidden preparation+execution structure.

Mode D's experimentally authored goal-only credit isolation was **harmful**, not a qualified improvement. It accumulated only 1 and 10 actual goal-progress credits, selected zero history plans, and suffered complete failure during the final six episodes of both role maps. This negative is preserved; D must **NOT** be treated as an improved default or pushed to `main`.

## What was learned and what was NOT

- AETERNA did select its own protected exploratory motor actions and later successfully used **58 actual selected/executed acquired history action plans**, rather than merely offering them.
- It solved 44 successful episodes in a single persistent lifetime with acquired causal facts and native U1 meta-synapse learning, no forced teacher action schedule. This is evidence of useful *within-world* autonomy, not just a scripted success demonstration.
- The new external episode observation API restored real initial sensory state while leaving previous cognitive facts and safety audit untouched; invalid width, HumanProtection emergency latch and source checkpoint integrity tests passed.
- This is only two **disclosed and repeatedly encountered** world layouts; most improvements come from learning the SAME causal mechanism again and again. It is **NOT** a fresh unseen-family zero-shot transfer experiment, AGI, symbolic object grounding, arbitrary POMDP inference or a self-invented new architecture. The scoring and probe algorithms were authored by developers, not discovered autonomously. The technical result is *self-directed data gathering + acquired causal strategy*, not generalized recursive self-improvement.

## Next required scientific test

Freeze a new protocol BEFORE defining an independent disjoint virtual world generator. Give the same cold source/U1 and equal protected action budget in every source-disclosed but unfamiliar causal family; vary hidden temporal prerequisites and role permutations after source freeze via an unselectable CI run seed. Compare A vs C on family-wise task success and real action economy. Keep an explicit no-hidden-effect control. A failure is scientific NEGATIVE and must not be silently tuned or rerun as fresh. A positive one-use synthetic result still requires a third-party external environment to support broad generalization.

Research PR #9 remains DRAFT; HumanProtection and original RSP2 unchanged, main untouched.
