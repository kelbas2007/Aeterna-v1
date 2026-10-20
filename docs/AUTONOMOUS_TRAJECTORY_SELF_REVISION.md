# Self-triggered trajectory evaluator revision (research, 2026-10-19)

**Not qualified AGI or autonomous policy advantage.** This is a narrow development mechanism added after PR9 exposed that a hand-authored factual chain did not win U1 arbitration.

The organism's opt-in `enable_constructing_trajectory_self_revision` keeps bounded goal episodes from **only executed and human-protection-authorized real PRE/action/POST**. It retrospectively credits earlier actions when a later step reaches the same externally supplied goal; unfinished or changed-goal episodes are not falsely labelled. Failure to reach within 8 actions receives a bounded failure label. This objective is authored; **the evaluator program is not.**

When enough completed success/failure episodes show that the existing immediate U1 goal-value field poorly predicts realized delayed credit, the runtime **automatically** invokes its existing VM induction or RSP2-compatible expression search on original PRE-decision five-field observations. This yields an executable, typed, metered evaluator program. A candidate stays shadow and cannot affect action scores until *subsequent chronological real episodes* (at least four independent completed episodes, including success and failure) show at least 15% lower prediction loss with no regression in either category.

After qualification, the acquired VM expression replaces only the numeric goal-value field passed to the existing U1 arbitration for action proposals. It cannot change the permission boundary, actuators, hazard model, goal, codebase, or VM. Attempt counts, generations, source-program and future evidence are retained in the same joint checkpoint; admission receipts are checked by replay on restore. All legacy experiments are unchanged unless this feature is explicitly enabled. The generation bound is three and the episode/sample/computation budgets are finite.

**Research limits:** Predicted delayed utility on *observed actions* is not an unbiased counterfactual policy-value estimate. The tests' useful epistemic signal is a disclosed educational fixture, not a blind independent experiment. There is no qualified improvement of external actions due to the newly induced evaluator; no unconstrained novel primitive creation, self-generated goals or native SNN discovery is claimed. Further qualification needs randomized paired actual execution in disjoint environments and a check for policy-induced distribution shift.

Reproduce via project CI and `cargo test --lib autonomous_temporal_revision_tests`. Promotion of PR9 to main still prohibited absent independent prospective action advantage.

## Post-admission rejection / drift rollback

An accepted expression is not permanent. After at least four later completed episodes
with diverse success/failure outcomes, the organism compares the active expression's
delayed-outcome prediction error against its previous evaluator. A sufficiently
large prospective regression revokes the active expression, restores its previous
evaluator, cancels any shadow candidate, and saves the *actual later* contradiction
witnesses in a checkpoint-verifiable receipt. This is conservative predictive
rollback, not proof that an alternative policy would choose better actions.

A goal-less external action interrupts the open credit episode and cannot be
retrospectively mislabeled as though no interaction intervened.
