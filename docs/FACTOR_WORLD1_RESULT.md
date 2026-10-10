# FACTOR-WORLD-1 — FIRST composition diagnostic, negative result

Date: 2026-10-10. Branch: \`research/beyond-intel4\`.
Code commit at first run: \`b0c89798829f0c297135c19662f085e82b41e2f3\`.
[GitHub Actions run 38029493198](https://github.com/kelbas2007/Aeterna-v1/actions/runs/38029493198)
completed with CI SUCCESS, but its **actual cognitive verdict is DEVELOPMENT_FAIL (0/4)**.
Do not equate CI SUCCESS with an ability PASS.

## Observed first run

| Arm | Training: key subworld /160 | Training: supply subworld /160 | Factual acquired links | Heldout plan | Heldout goal | Steps used | Verdict |
|---|---:|---:|---:|---|---|---:|---|
| 0 | 144 | 157 | 19 | None | No | 14 | FAIL |
| 1 | 142 | 156 | 19 | None | No | 14 | FAIL |
| 2 | 148 | 152 | 19 | None | No | 14 | FAIL |
| 3 | 145 | 156 | 19 | None | No | 14 | FAIL |

Overall exact log:
\`FACTOR_WORLD1_SUMMARY causal_start_plans=0/4 composed_goals=0/4 split_valid=4/4 verdict=DEVELOPMENT_FAIL\`.

All arms: zero observed joint key-and-supply feature tuples during
training; evaluator-only oracle shortest heldout solution five actions;
frozen U1 meta weights; zero protection blocks and unsupported actions.
The first frozen decision never produced a supported learned causal
plan. Four organisms failed the goal within their 14-action budget.

## What this actually demonstrates

The current acquisition/search mechanism accumulates factual **complete
state -> motor -> complete state** links. In 2 familiar restricted
contexts it learns/reuses links, but it did not recombine them to invent
connections through the withheld joint states.

The present visual foundation recognizes **24 separately preprepared
complete-state classes**, arbitrarily permuted against factor tuples.
This deliberately isolates the current whole-state graph; it does NOT
provide an independently identifiable composition of object features
in the raw images. A powerful factor learner with no structure in those
codes would also lack evidence for unseen visual combinations.
Therefore this first test is a **negative control for whole-state
graph transfer**, not proof that true grounded factorization is
impossible, or a fair gate for a future factorized perceptual model.

The result must remain immutable with its original seed and exact
source commit. Any follow-up feature model requires a separately
named test and seed, an explicitly identifiable factorized sensory
interface with raw PRE/action/POST (no symbolic tutor), then withheld
cross-product examples, checkpoint and lesion controls. Battery
depletion surprise and dynamic replanning were **not yet tested** here.
