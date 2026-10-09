# EvoPhase cold autonomous cognitive cycle — first-attempt Fresh-2 result

Date: 2026-10-09  
Status: **FRESH-2 DEVELOPMENT PASS, bounded stochastic family; NOT general intelligence qualification**

## Reproducible authority for this claim

- [Precommitted fresh2 protocol](AUTONOMOUS_CYCLE_FRESH2_PROTOCOL.md), seed `0xDA7E_2026_CE42_9801`, threshold >=64/80 correct and strictly better than first-cue oracle **in each arm**.
- Fixed cognitive `src/` tree: `f8a4047ea22e0d96fd0b8dba0c218e6259324774:src`; [workflow](../.github/workflows/autonomous-cycle-fresh2.yml) checked the exact git tree and preregistered seed before evaluation.
- First-attempt complete [GitHub Actions run 37934001957](https://github.com/kelbas2007/Aeterna-v1/actions/runs/37934001957): SUCCESS; diagnostic `U1_FRESH2_SUMMARY all_arms_pass=true verdict=DEVELOPMENT_PASS`.
- Exact test: [`tests/u1_cold_fresh2.rs`](../tests/u1_cold_fresh2.rs); one unchanged A(64)→B(64)→A(64) train lifetime **per arm**, then restored checkpoint and frozen model-learning for 80 balanced holdouts. Four arms = four organisms, **not a single lifetime across four worlds**.
- Historical fresh1 failure is preserved as [Actions 37932975901](https://github.com/kelbas2007/Aeterna-v1/actions/runs/37932975901): 40/80, 73/80, 40/80, 40/80 (only 1/4 passed). The general phase-native reward credit correction followed that first failure; do not rewrite it as a PASS.

## First-attempt frozen heldout evidence

| Randomized world | Correct/80 | First cue oracle/80 | Three-cue oracle/80 | Executed extra samples | Episodes with 2+ samples | Discovered physical sensor |
|---|---:|---:|---:|---:|---:|---|
| 0 | 69 | 66 | 66 | 240 | 80/80 | Yes |
| 1 | 64 | 50 | 57 | 294 | 80/80 | Yes |
| 2 | 72 | 59 | 66 | 254 | 80/80 | Yes |
| 3 | 68 | 53 | 62 | 272 | 80/80 | Yes |
| **Total** | **273/320 (85.31%)** | **228/320 (71.25%)** | **251/320 (78.44%)** | **1060** | **320/320** | **4/4** |

All arms: two acquired raw cue identities, conducting physical motor-to-sensor link weight 1.0; each 80/80 actual terminal commits; no unavailable/protection-blocked action and no heldout U1 meta-weight change; checkpoint state restored. New role assignments and noisy episodes not previously used to repair this exact frozen cognitive source.

## What is established

Within the bounded partially observed, 70%-accurate-cue synthetic hidden-cause family, EvoPhase in four separate cold-target lifetimes self-selects active sensing (no host-selected motor schedule), accumulates physical evidence, obtains a decisive belief, uses acquired outcome value for final motor selection, and beats the preregistered one-cue control per arm after an A→B→A changed-reward learning history.

The prior diagnosis was causal: on fresh1, a winning physical belief incorrectly allocated some terminal reward to the losing cue. This was repaired at the generic physical credit path, then source was frozen before fresh2.

## What is NOT established

This is a **development**, not an independently broad scientific/general AGI gate. The test family remains the same type as prior development experiments (the new seed randomizes assignments, not the world structure). Each arm includes a transferred pre-trained generic visual/U1 foundation, not a from-zero cognitive carrier. Four worlds are not one universal lifelong organism. Observation reliability and reward classes are synthetic. Safety is a software protection callback. Language understanding, unrestricted perception, real-world human safety, transfer to new stochastic task structures, or general AGI remain unverified.

Any further qualification must preregister *structurally different* unknown environments, action/observation dynamics and robust controls before seeing outcomes; do not rerun consumed fresh1 or fresh2 evidence packs.
