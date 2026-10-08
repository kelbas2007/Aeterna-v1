# INTEL-3 preflight-1 — evaluator source-guard self-match

Date: 2026-10-08

Run: `37725997937`

Verdict: **PRE-SEAL TECHNICAL FAILURE — authority pack not consumed.**

The run passed the frozen cognitive source check and failed before
`INTEL3_SEAL` in the evaluator source guard.

Cause: the guard searched for forbidden literal strings that were themselves
present inside the guard's own assertions, so it matched its own source text.

No cognitive source changed. The evaluator guard is corrected prospectively by
constructing forbidden tokens from fragments at runtime.

This run is not an INTEL-3 scientific verdict and does not consume an authority
pack.
