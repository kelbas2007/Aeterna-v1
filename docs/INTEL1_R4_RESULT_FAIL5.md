# INTEL-1R4 — fifth frozen-core verdict

Date: 2026-10-07

## Verdict

**FAIL — earliest remaining bottleneck: stale one-sided unpromoted context hypotheses monopolize experiment choice.**

Run: `37669439298`
Frozen cognitive core: `b811b59a5777b59b4ab640ef4ac2ebf75a53c45d`
Authority seed / burned pack: `37669439298`.

Pre-verdict freeze and Repair-1/2/3/4, G21/G22/G23, G20 and Human Protection regressions passed.

Observed before the earliest failure:

- W1 unknown navigation: PASS, 11 actions;
- W1 translated frozen reuse: PASS, 3 actions;
- W2 causal machine: PASS, 10 actions;
- W2 translated frozen reuse: PASS, 3 actions;
- W3: FAIL, no promoted useful context hypothesis and 0/32 scored correct junction decisions.

W3 diagnostics showed multiple earlier-lifetime context candidates that matched only one of the current predecessors. Examples accumulated 53 or 39 observations on one side while the other side remained 0, with `context_switches=0`. Their anchor actions were unrelated to the authority-selected useful history actions.

Current `phase_native_context_action` gives the oldest applicable unpromoted candidate unconditional experiment priority. Therefore a hypothesis that cannot currently obtain its missing second side can repeatedly request the same non-discriminating anchor and starve ordinary epistemic exploration that would discover the useful history-dependent actions.

W4 was reached later but is not project-scored because W3 is already the earliest failed frozen criterion. Later NoSupportedAction is therefore not the repair target.

No G24 is opened.
