# Unified runtime preflight-1 — same-action proposal tie

Date: 2026-10-07
Run: `37682181828`
Branch: `unified-cognition`

## Verdict

**ASSEMBLY FAIL — U1 correctly failed closed on two equal proposal records that requested the same external action.**

The collector exposed at least four real proposals. In the witness, a persistent rival/explanatory proposal and a nonpersistent goal-active proposal both requested action 0 and carried the same generic action-level fields:

`[0.95, 1.0, 1.0, 0.94285715, 1.0]`.

U1 proposal competition treats equal proposal records as a tie and returned no unique winner. This is correct for distinct operations, but at the runtime assembly boundary these records are not distinct external operations: both support the same motor action.

## Prospective assembly repair

No U1, U2 or U3 learning/scoring rule changes.

Before U1 action competition, the unified runtime layer will coalesce proposals that request the same external action:

- grouping key = opaque motor action only;
- no module/source/class key;
- generic action fields = element-wise maximum of already carrier-derived, ecology-modulated supporting fields;
- all persistent supporting hypothesis IDs are retained for later generic factual credit;
- different actions remain separate competitors;
- enumeration order and proposal IDs cannot affect grouping.

This is an assembly rule for equivalent external operations, not a new cognitive mechanism.

The next verification must retain the original same-action witness and show:
- a unique action winner exists;
- all supporting persistent hypotheses remain available for credit;
- U1/U2/U3 regressions remain PASS.
