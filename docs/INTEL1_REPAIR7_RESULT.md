# INTEL-1 Repair-7 verification result

Date: 2026-10-07

Verdict: **FAIL / PARTIAL — predecessor-conditioned coverage discovers and promotes the useful history-dependent hypotheses, but multiple promoted contextual explanations cannot yet be jointly exploited.**

Workflow: `37672036604`.

## What improved

The persistent stale-history witness now created promoted candidates on the actual two factual history predecessors. Two useful candidates on the same ambiguous base promoted after 32 future observations:

- anchor action 2, predecessor pair [467,468], log evidence 18.187124;
- anchor action 3, predecessor pair [467,468], log evidence 18.247769.

This is qualitatively different from R4/Repair-5/6, where the useful history collision was not reliably acquired.

Thus predecessor-conditioned action coverage succeeded in making the same base actions observable under both histories.

## Remaining failure

The witness scored only 13/35 before `NoSupportedAction`.

Current G21 readout collects every coherent promoted refined state for the current base/predecessor. If more than one is active, it unconditionally returns `(true,None)`:

`active_states.len() > 1 -> fail closed`.

The two promoted candidates above are not necessarily contradictory. They were learned from different opaque anchor actions and can represent complementary evidence about the same latent history. The current implementation cannot test whether their independently learned physical goal plans agree.

The direct Repair-7 micro-test also hit an evaluator construction rejection before its intended assertion. That test-harness issue does not erase the system evidence above; it will be narrowed to context-only observation for future regression.

No INTEL authority pack was consumed by Repair-7 verification.
