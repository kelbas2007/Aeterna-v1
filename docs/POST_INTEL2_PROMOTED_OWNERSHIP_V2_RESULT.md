# POST-INTEL-2 promoted representation ownership V2 — result

Date: 2026-10-08

## Verdict

**PASS — local evidence coverage plus promoted representation ownership close the diagnosed INTEL-2 World C bootstrap/readout failure without regression.**

Workflow: `37725137951`
Exact cognitive source: `2c71da210357cb9452d5bd2071faab2f38d2f352`
Branch: `unified-operation-competition`

## Learned-drive regressions

- P4: 12/12 learned targets, mean cost 25.0;
- G16: 12/12 acquisition, 12/12 held-out planning, 12/12 restart;
- learned drive weights remained approximately `[0.99999994,1.0]`.

## Burned INTEL-2 C diagnostic

This is diagnostic only; INTEL-2 remains FAIL.

After the redesign:

- context promoted: true;
- 72 acquisition trials;
- no unsupported action;
- context evidence: 32 observations, 31 switches;
- held-out FULL: **32/32**;
- side 0: **16/16**;
- side 1: **16/16**;
- memoryless: **16/32**.

Promoted context readout alternated correctly:
- predecessor side 0 -> action 1;
- predecessor side 1 -> action 3.

The ambiguous parent goal remained visible diagnostically but was not allowed to
bypass the promoted refinement.

## Full regression gate

Same workflow passed:

- U1;
- U1 runtime integration;
- U2;
- U3;
- G20;
- G21;
- G22;
- G23;
- Human Protection;
- Release build.

## Scientific boundary

This result validates the two diagnosed architecture changes. It does not alter
the burned INTEL-2 verdict and is not a new intelligence verdict.

A new frozen, independently seeded system test is required.
