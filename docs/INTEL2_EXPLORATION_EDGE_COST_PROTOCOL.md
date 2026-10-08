# Post-INTEL-2: exploration edge-cost diagnosis and correction contract

Date: 2026-10-08
Working branch: post-intel2-c-diagnosis
Pre-change source: 92b812b8ad54bac16f8e133bc8916076972dc2f1
Historical frozen cognitive source: 07b44fb8e00837568bc9655e760eb031d1944dff

## Preserved result

INTEL-2 remains FAIL. Its authority seed 37687243350 is burned. This work is development diagnosis, not a fresh qualification. Do not rewrite INTEL2_RESULT_FAIL1.md or modify main / unified-cognition / Codex AETERNA.

## Evidence and corrected attribution

The terminal-reset diagnostic ran 450 World-C trials. At each history side it chose motor 0 on all 225 visits; contexts and U2 registrations remained empty. Available capacity was not exhausted. Traces showed direct_unknown=Some(1), general=Some(0), and exactly one proposal reaching unified competition.

Thus the useful alternative is already lost inside the generic exploration selector, BEFORE U1 competition. The available trace does not support blaming U1 for choosing among multiple alternative actions.

Separately, the original evaluator helper maps StepOutcome::GoalReached to RuntimeError::NoSupportedAction. The episodic diagnostic resets the environment at terminal success AND failure, preserving the organism. This is an evaluator-semantic change and cannot relabel the original INTEL result.

## Source-level explanation to test

phase_drive_frontier_activity_for_cells discounts each traversed model edge. But phase_drive_features does not discount the final current-state-to-successor edge. A known action leading to a wholly unexplored terminal therefore receives reachable=1, the same nominal epistemic value as a directly unknown action. With transferred weights approximately [1,1], the general selector's global-support tie-break prefers the already familiar motor indefinitely.

The collector emits only the selected general-exploration action, so later unified competition cannot recover an alternative that was never proposed.

## Single permitted production change

In phase_drive_features, multiply the known-action successor frontier contribution by the existing PhaseNativeConfig.discount. Direct unknown features remain [1,0]. This charges exactly one model edge consistently with the existing frontier recurrence.

Do not change learned weights, source/module priorities, promotion gates, candidate capacity, motor IDs, world labels, Human Protection, terminal semantics in production, or any historical test.

## Checks fixed before the change

1. A generic six-motor witness: after one motor repeatedly reaches an unvisited successor, an immediately unknown motor must not be starved solely by the known motor's higher global support. Exercise each of the six choices of familiar motor, not only World C's motor 0.
2. Repeat the existing burned-pack terminal-reset diagnostic unchanged. Report coverage, promotion and held-out score even if it fails. Its PASS would be development evidence only.
3. Run historical G16, G17, G18, G19, G20, G21, G22, G23, U1/U2/U3 and runtime integration checks, plus Human Protection and Release. Preserve all regressions; do not alter their expected values to force success.
4. Compare committed production source against the frozen baseline: only src/phase_drive.rs may differ, and the intended semantic diff is one discounted edge.
5. Run tests on exact committed source; no CI rewriting of cognition.

## Decision rule

If the local starvation witness passes but the World-C diagnostic still fails, record the narrower improvement and the remaining failure. Do not call the whole issue fixed and do not launch a new INTEL qualification automatically. Any further change requires a separate observed cause and prospective contract.
