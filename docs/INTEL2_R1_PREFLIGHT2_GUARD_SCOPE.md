# INTEL-2R1 preflight-2 — source-guard self-literal scope

Date: 2026-10-08
Run: `37728456049`

All frozen cognitive pre-gates passed again.

The R1 evaluator source guard failed before `INTEL2_R1_SEAL` because it scanned the
entire file, including its own string literals. The first newly exposed literal was
`ReasoningMode::`, present only inside the guard assertion itself. The remaining
forbidden-token assertions had the same latent self-match problem.

No target pack was exposed or consumed.

Prospective repair: scope the source guard to the evaluator text *before* the guard
function itself. No world law, threshold, cognitive source or scored logic changes.
