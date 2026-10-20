# T-SR1: prospective paired physical-action qualification — preregistration

Date: 2026-10-19. Scope: AETERNA-v1 PR9, **one-use developmental scientific challenge**. This protocol is committed BEFORE the evaluation implementation is executed. It describes a generated external virtual-world test, NOT an independent AGI benchmark or real-world autonomy test.

## Claim and essential controls

Claim being tested: does the *organism-acquired executable trajectory-value expression* improve actual protected multi-step decisions, beyond existing U1 operating on the **identical previously acquired experience**?

- Same version, organism/carrier, learned U1 weights, factual journal, protected-action audit position, initial sensor state, offers, goals and per-trial budget.
- Shared exact joint checkpoint created from real executed PRE/action/POST acquisition episodes.
- Experimental arm: preserve the organism-acquired VM trajectory evaluator.
- Ablation arm: disable *only its readout*, after freeze; preserve the program, acquisition traces, memory, safety gate and checkpoint provenance. Both arms frozen during qualification. No teacher guidance, motor-role names or privileged world transition observations reach the runtime during evaluation.
- The human/evaluator may schedule identified experiences in the **disclosed acquisition phase**, to make effect identification possible; this is not claimed to be autonomous discovery from cold start. A successful outcome cannot be credited to unseen synthetic labels.
- Frozen-code/environment protocol generates paired **new evaluation worlds after source freeze** using the nonselectable GitHub Actions run ID for the seed. The generated suite is a new evaluation cohort but **only a synthetic, source-disclosed family**, not a genuinely independent external domain.

## Evaluation

The training environment has one progress sensor and four opaque motors: incremental progress, neutral, reset and harmful delay. Success/failure trajectories for a goal are observed only through HumanProtection-authorized callbacks. The terminal success label and eight-step timeout are authored *objective specifications*, not injected trajectory labels. Acquisition and the subsequent prospective gate must yield an **admitted, code-bearing VM evaluator**; otherwise report **NOT_ADMITTED** and do not claim policy improvement.

Evaluation: a deterministic held-out generator produces four transition families that vary reward dynamics and goal horizon, with shuffled opaque motor roles and multiple cases per family. The precise role permutations are seeded by the CI run ID; no source is modified after evaluating this one-use seed. Each paired arm has 24 external action opportunities per world; success is progress >= goal. Report successes, action counts, selected action traces, score-field changes, differences by family, and protected callback counts. Never invent or mask unavailable results.

## Non-negotiable qualification gate

1. A VM evaluation program is **autonomously proposed, prospectively validated and admitted** from actual acquisition POST events; the same exact checkpoint restores in both arms, and ablating it changes no other learned state.
2. With fixed frozen control and identical new worlds, the evaluator-enabled arm must achieve **at least as many successes overall**, **no family has fewer successes**, and achieve **at least 10% lower total action count** without a greater external-action cost when counted on all cases with failures charged their full 24-action budget.
3. At least one action-selection divergence attributable to the evaluator must be observed; a prediction-only gain is NOT evidence of better policy.
4. No unauthorized external action, no failed checkpoint verification, no hidden reset/motor-role input to the organism, no process-external change to source/safety/RSP2; source audit and Linux/Windows CI remain green.

If any requirement fails, scientific outcome is **NEGATIVE / NOT QUALIFIED** even if engineering tests pass. No tuning on outcomes and relabeling as held-out. No promotion to main based solely on these cases.

## Evidence format and limitations

Archive exact commit SHA, CI run ID, environment generator seed, learned evaluator code, checkpoint identifier, per-arm/per-family goals and successes/actions, decision divergence counts, and the conclusion in CI logs/artifact. Expected learning from observed trajectories remains heavily dependent on the authored five U1 feature fields. This experiment tests a narrow causal contribution to behavior, not a universal intelligence claim.

**Immutable interpretation:** absence of admitted evaluator or a tie counts as no self-improvement advantage. Design changes after reading outcomes require a new disjoint, preregistered experiment.
