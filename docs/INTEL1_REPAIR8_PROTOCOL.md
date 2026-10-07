# INTEL-1 Repair-8 — consensus exploitation across promoted contextual hypotheses

Status: **PRE-REGISTERED BEFORE REPAIR-8 CODE CHANGE**

Date: 2026-10-07

## Preserved history

INTEL-1 and R1-R4 remain failed burned verdicts.
Repair-5/6/7 remain partial verification results and are not relabeled PASS.

## Earliest remaining bottleneck

Repair-7 can now acquire multiple useful promoted contextual hypotheses for the same physical base and current predecessor. G21 currently treats more than one coherent promoted refined state as an automatic conflict and returns no action.

Different hypotheses may have been discovered through different anchor actions yet still imply the same goal-directed first action.

## Allowed repair

Change ONLY promoted-context readout arbitration.

For all physically coherent promoted context refined states applicable to the current factual base/predecessor:

1. compute the existing phase-native physical goal decision independently from each refined state;
2. if every active refined state with a decision yields the SAME first action, and none lacks a decision, return that consensus action;
3. if decisions disagree, or any active promoted refined state lacks a supported decision, remain fail-closed;
4. with exactly one active promoted refined state, preserve existing behavior;
5. do not choose by candidate age, anchor action, state ID, support count, world ID or evaluator answer.

No candidate is deleted or merged in host metadata.
No transition/evidence/promotion threshold changes.

## Repair witness

1. Persistent all-refiners stale-history world must promote useful history hypotheses.
2. When multiple promoted context states are simultaneously coherent and their physical goal plans agree, contextual readout returns the shared action.
3. A constructed disagreement control must return `(true,None)`.
4. System contextual score >=60/64 and >=28/32 each predecessor side.
5. No-context control <=40/64.
6. Repair-7 predecessor-conditioned coverage witness PASS after evaluator isolation fix.
7. Repair-6 direct rival deferral PASS.
8. Repair-5 balance, Repair-4 coexistence, Repair-3 fanout, Repair-1/2 PASS.
9. G21/G22/G23, G20, Human Protection and Release PASS.

## Freeze / rerun

If Repair-8 passes, freeze a new cognitive SHA and run a new independently seeded INTEL verdict.

No G24/G25. Any next failure is recorded at its earliest frozen criterion.
