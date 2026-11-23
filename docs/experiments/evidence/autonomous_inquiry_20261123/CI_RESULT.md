# Autonomous inquiry evaluation result

**Verdict: SUPPORTED.** Valid worlds: 16/16.

Seed: `45259082705`. Source commit: `df9257ac2c15be9a843409ad28b25adc5ab3ab86`.

This evaluates the integrated bounded inquiry mechanism in an opaque two-rule sensor/actuator family. It does not establish general intelligence.

## Observed behavior

| Condition | Arm | Goals | Success | Actions | Diagnostic reads | Noise reads | Selected inquiries |
|---|---|---:|---:|---:|---:|---:|---:|
| switching | ordinary | 13049/16384 | 79.64% | 40664 | 8324 | 3247 | 0 |
| switching | active | 13535/16384 | 82.61% | 40725 | 6953 | 2847 | 8351 |
| switching | observe_only | 13049/16384 | 79.64% | 40664 | 8324 | 3247 | 0 |
| stationary | ordinary | 3670/4096 | 89.60% | 9289 | 822 | 542 | 0 |
| stationary | active | 3717/4096 | 90.75% | 9333 | 664 | 622 | 1211 |
| stationary | observe_only | 3670/4096 | 89.60% | 9289 | 822 | 542 | 0 |

## Fixed-gate conclusions

Directed-inquiry benefit: **True**. Integrated factual chain: **True** (16/16 worlds, 434 chains).

Active versus observe-only mean switching gain: 2.966 percentage points; positive worlds: 15/16; stationary gain: 1.147 points; action ratio: 1.0015.

A valid negative result means one or more preregistered scientific gates failed. It does not turn evidence-integrity checks into an ability claim.

## Per-world comparison

| World | Ordinary | Active | Observe only | Active gain (pp) | Complete chains | Passive behavior identical |
|---|---:|---:|---:|---:|---:|---|
| 0 | 78.42% | 82.03% | 78.42% | 3.613 | 25 | True |
| 1 | 78.22% | 86.62% | 78.22% | 8.398 | 3 | True |
| 2 | 76.86% | 83.89% | 76.86% | 7.031 | 63 | True |
| 3 | 78.71% | 72.66% | 78.71% | -6.055 | 32 | True |
| 4 | 78.12% | 80.08% | 78.12% | 1.953 | 25 | True |
| 5 | 80.18% | 81.05% | 80.18% | 0.879 | 44 | True |
| 6 | 68.55% | 70.51% | 68.55% | 1.953 | 19 | True |
| 7 | 81.64% | 85.55% | 81.64% | 3.906 | 2 | True |
| 8 | 83.98% | 87.11% | 83.98% | 3.125 | 24 | True |
| 9 | 83.30% | 85.06% | 83.30% | 1.758 | 28 | True |
| 10 | 72.75% | 77.25% | 72.75% | 4.492 | 51 | True |
| 11 | 88.57% | 92.58% | 88.57% | 4.004 | 28 | True |
| 12 | 78.42% | 83.89% | 78.42% | 5.469 | 10 | True |
| 13 | 78.42% | 79.20% | 78.42% | 0.781 | 38 | True |
| 14 | 84.08% | 86.52% | 84.08% | 2.441 | 31 | True |
| 15 | 84.08% | 87.79% | 84.08% | 3.711 | 11 | True |

Every arm starts from the same acquired organism. Ordinary/observe-only equality is checked on physical choices and outcomes, excluding inquiry diagnostics and computational accounting. If it differs, equal memory ceilings and the added inquiry reservation may change eviction pressure; the exact traces retain the evidence.

## Reproduction

`python3 scripts/run_autonomous_inquiry.py --score-only --output <evidence-directory>`
