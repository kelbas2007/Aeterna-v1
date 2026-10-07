# INTEL-1 Repair-2 result

Date: 2026-10-07
Verdict: **PASS — frozen exploitation now obeys the retained physical goal plan.**

Workflow: `37661137884`
Exact tested source: `c89f0ea9ad4d953c538ff3914994f3ad37dd85a4`.

Single cognitive change from Repair-1 baseline:
- when phase-native learning is disabled, `choose_phase_native_goal_active_action` bypasses epistemic novelty scoring and uses the existing physical goal recurrence directly;
- learning-enabled G18 behavior is unchanged.

Evidence:
- burned W1-style route is acquired through protected factual interaction;
- after learning freeze, runtime actions exactly match the retained direct physical plan and reach the goal;
- unknown-goal learning-enabled bootstrap still uses `GeneralEpistemic`;
- Repair-1 witness PASS;
- G16–G23 and Human Protection regressions PASS;
- Release build PASS;
- source unchanged during verification.

INTEL-1 and INTEL-1R1 remain failed burned verdicts. Repair-2 does not relabel them.
