# TE4 — cold autonomous discovery diagnostic, four new worlds

Date: 2026-10-08
Branch: `research/beyond-intel4`
Verdict: **DEVELOPMENT_FAIL**. This is not a new frozen intelligence PASS or a replay of burned FRONTIER-1.

## Evidence
- [GitHub Actions 37841337318](https://github.com/kelbas2007/Aeterna-v1/actions/runs/37841337318), complete scientific run after a pre-execution duplicate test-import failure in run `37841224861`.
- Starting cognition exactly TE3 post-qualification, no target motor tuition and no target-specific representation/response injected.
- Each arm used randomized cue/goal/dead class roles and shuffled opaque motor IDs, 128 genuinely self-selected training episodes and 80 independent balanced held-out episodes.
- Normal `ScientificRuntime::step_unified` + Human Protection, preserved EvoPhase lifetime and physical checkpoint. No matched action schedule.

| Arm | Sensor action (evaluator only) | Training real samples | Physical sensor affordance | Heldout correct | Heldout real samples | ≥2 samples episodes | Last-cue oracle | Unsupported |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 0 | 1 | 75 | 1.000000 | 40/80 | 0 | 0/80 | 50/80 | 0 |
| 1 | 3 | 4 | 0.250000 | 55/80 | 0 | 0/80 | 55/80 | 0 |
| 2 | 2 | 2 | 0.000000 | 40/80 | 0 | 0/80 | 55/80 | 0 |
| 3 | 3 | 170 | 1.000000 | 38/80 | 124 | 28/80 | 42/80 | 0 |

All four arms acquired two physical cue-source identities. The final heldout control froze structural model learning but allowed transient evidence updates. No protected action was blocked, and all episodes except two in arm 3 did commit.

## Causal decomposition

1. **Cold sensor discovery is inconsistent:** arm 2 never acquired an active observation-producing physical motor path.
2. **Acquired sensing value does not consistently survive the goal-policy competition at frozen evaluation:** arm 0 learned the motor after 75 real samples but made zero heldout samples; arm 1 did likewise after 4.
3. **Belief does not yet drive the correct terminal choice:** arm 3 took 124 heldout additional samples, yet obtained only 38/80 correct. More sampling alone does not create a learned connection from temporally accumulated evidence to the appropriate opaque goal action.

Thus TE3 6/6 only establishes physically learned sensing after balanced motor-experience and successful protected U1 selection under its prepared context. It cannot be reinterpreted as autonomous noisy hidden-cause inference.

## Next architectural requirement

One physically learned *belief-conditioned outcome-policy* is needed: after only factual protected PRE/action/POST and bounded actual task outcome, the organism must associate physically accumulated cue evidence with expected utility of opaque terminal actions. It should express a confidence-dependent action-value proposal into U1 without host labels, and preserve the distinction between information-gathering and terminal choices. A coherent physical lesion/π-shift, negative evidence revision, restart and new motor assignment must qualify it before new independent cold sealed targets.

Do NOT patch this already exposed TE4 generator to pretend the four-arm verdict passed. INTEL-4 remains separate bounded PASS; FRONTIER-1 and TE1 strict score remain FAIL.
