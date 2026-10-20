# T-SR1 — First one-use protected-action trial: negative scientific verdict

Date: 2026-10-19. **Recorded after execution, with no policy edits or post-hoc qualification changes.**

## Reproducible provenance

- Pre-registered protocol-only commit: `cf71c52fd609ff648d424a1008bdc2d806ccafae`, [protocol](TRAJECTORY_SELF_REVISION_TRANSFER_PROTOCOL.md).
- Frozen tested pull-request head: `ff3331de01c4674e8ec1ad17470fdd87dbc99ca6` (PR #9, draft).
- GitHub pull-request test/merge source SHA used by Actions: `b7b52182d58be368aa7361bf2d01db3ff3f9f86e`.
- One-use Actions run/seed: `39259810410`.
- [Exact qualification run](https://github.com/kelbas2007/Aeterna-v1/actions/runs/39259810410) — status **FAILURE**, because the PRE-REGISTERED scientific criteria failed, not because the VM did not compile.
- [Machine output and complete action-trace artifact](https://github.com/kelbas2007/Aeterna-v1/actions/runs/39259810410/artifacts/12189043452).
- Factual journal checkpoint fingerprint (noncryptographic diagnostic): `13499986757948477000`.

## Acquired program — genuine bounded self-change

AETERNA collected 24 completed episodes through the same ScientificRuntime and physically authorized PRE/action/POST callback. It invoked its own bounded synthesis and **admitted one VM evaluator** after prospectively observing later outcomes. The learned code was not written into the test by its author. Exact debug form:

```text
If(
  Bin(Lt, Bin(Add, Input(3), Lit(Int(-1))), Input(0)),
  Input(4),
  Lit(Int(-1))
)
```

The learned numeric valuation reads two of the existing five U1 fields and computes a replacement goal-value field. This is restricted executable evaluator revision, not self-modifying Rust, not native SNN invention of new primitives, and not a free-form AGI capability.

Both policy arms restored from the **exact same frozen joint checkpoint**. In the control arm, only the newly admitted evaluator's readout was disabled. All action execution remained under the original HumanProtection permits, real POST observations and fixed budgets. The qualification evaluated **48 new generator-produced virtual worlds**: four rule families, 12 motor-role cases each; exactly 24 allowed external actions per case. The generating code and family distribution are synthetic and source-disclosed; the GitHub run ID determines the actual role permutations after the candidate source was frozen.

## Outcomes (all 48 paired cases, failed cases cost their full budget)

| World family | Old U1 successes / 12 | Self-revised U1 successes / 12 | Old actions | Self-revised actions |
| --- | ---: | ---: | ---: | ---: |
| 0 — linear increments | 8 | 8 | 136 | 162 |
| 1 — conditional last-step motor | 2 | 5 | 262 | 265 |
| 2 — two-unit jumps | 11 | 8 | 121 | 140 |
| 3 — stronger harmful decrement | 2 | 6 | 254 | 207 |
| **Total** | **23 / 48** | **27 / 48** | **773** | **774** |

- The self-revised evaluator changed the **first action** in 48/48 cases; the selected first-action goal-score field also differed in 48/48. Thus the acquired program really changed *action selection*, not just a hypothetical model score.
- Aggregate goal successes rose **23 → 27**, but family 2 regressed **11 → 8**. Aggregate action cost rose **773 → 774**.
- Consequently, preregistered no-family-regression and >=10% action-economy improvement requirements both **failed**. Scientific verdict: **NEGATIVE_ACTION_ADVANTAGE / NOT QUALIFIED**.

## Interpretation and next scientifically honest barrier

This is the first mechanism-level evidence that **self-synthesized, prospectively admitted VM valuation affects protected U1 decisions** in new virtual environments. It is **not** evidence that the resulting decision policy has become better overall. Predicting observed delayed outcomes more accurately is insufficient for generalizing *counterfactual control* to new tasks. In particular, improvement in families 1 and 3 did not prevent negative transfer in family 2.

Do not retune the algorithm on the now-exposed 48 scenarios and call them held-out. Preserve this negative result. A next disjoint protocol must distinguish (1) conditional prediction calibration, (2) actual action advantage versus the exact-score ablation, and (3) per-environment non-regression under independent randomized protected execution. Prospective policy-level canaries and trustworthy rollback of *behavioral*, not only predictive, regressions would be a meaningful research direction, but are **not** demonstrated here.

PR #9 remains DRAFT; `main` stays unchanged until real qualifying evidence exists.
