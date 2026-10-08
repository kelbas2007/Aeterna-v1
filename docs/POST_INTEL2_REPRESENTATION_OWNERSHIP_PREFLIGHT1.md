# POST-INTEL-2 representation ownership preflight-1

Date: 2026-10-08

Verdict: **INVALID DESIGN SCOPE — unpromoted experiment request was treated as acquired representation ownership.**

The first ownership implementation used the selector's `applicable=true` bit
directly to suppress parent proposals.

Burned diagnostic stopped after only 8 World C trials:

- context candidate existed but was not promoted;
- eligible observations: 1;
- context switches: 0;
- `NoSupportedAction` occurred at the junction.

Diagnosis:

`applicable=true` has two meanings in current refinement APIs:

1. an unpromoted hypothesis requesting its anchor experiment;
2. an acquired/promoted refinement claiming the parent is insufficient.

Only (2) may own representation scope.

No claim from this preflight is promoted to a project result.
