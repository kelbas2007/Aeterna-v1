# Autonomous inquiry evaluation result

**Verdict: DEVELOPMENT.** Valid worlds: 2/2.

Seed: `7001`. Source commit: `48197411f2dc581b3eeba1bec2f47a11ec2ee8aa`.

This evaluates the integrated bounded inquiry mechanism in an opaque two-rule sensor/actuator family. It does not establish general intelligence.

## Observed behavior

| Condition | Arm | Goals | Success | Actions | Diagnostic reads | Noise reads | Selected inquiries |
|---|---|---:|---:|---:|---:|---:|---:|
| switching | ordinary | 1597/2048 | 77.98% | 5379 | 1300 | 530 | 0 |
| switching | active | 1650/2048 | 80.57% | 4975 | 448 | 448 | 1930 |
| switching | observe_only | 1597/2048 | 77.98% | 5379 | 1300 | 530 | 0 |
| stationary | ordinary | 464/512 | 90.62% | 1095 | 37 | 63 | 0 |
| stationary | active | 477/512 | 93.16% | 1126 | 29 | 97 | 204 |
| stationary | observe_only | 464/512 | 90.62% | 1095 | 37 | 63 | 0 |

## Fixed-gate conclusions

Directed-inquiry benefit: **False**. Integrated factual chain: **False** (2/2 worlds, 7 chains).

Active versus observe-only mean switching gain: 2.588 percentage points; positive worlds: 2/2; stationary gain: 2.539 points; action ratio: 0.9249.

A valid negative result means one or more preregistered scientific gates failed. It does not turn evidence-integrity checks into an ability claim.

## Per-world comparison

| World | Ordinary | Active | Observe only | Active gain (pp) | Complete chains | Passive behavior identical |
|---|---:|---:|---:|---:|---:|---|
| 0 | 81.64% | 82.42% | 81.64% | 0.781 | 1 | True |
| 1 | 74.32% | 78.71% | 74.32% | 4.395 | 6 | True |

Every arm starts from the same acquired organism. Ordinary/observe-only equality is checked on physical choices and outcomes, excluding inquiry diagnostics and computational accounting. If it differs, equal memory ceilings and the added inquiry reservation may change eviction pressure; the exact traces retain the evidence.

## Reproduction

`python3 scripts/run_autonomous_inquiry.py --score-only --output <evidence-directory>`
