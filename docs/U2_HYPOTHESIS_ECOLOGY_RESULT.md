# U2 — carrier-owned lifetime hypothesis ecology

Date: 2026-10-07

## Verdict

**PASS — bounded carrier-owned hypothesis lifecycle, dormancy, reactivation and U1 coupling.**

Workflow: `37678466130`
Exact tested source: `7830187f1febf279a137359423d77f1693f73bf9`
Branch: `unified-cognition`
Evidence artifact: `u2-evidence`, ID `11507711850`, SHA256 `c7b8ac6af927c4ed5a004d5a07408fd1cb238031edac5e32366581d17beb4e1a`.

## Lifetime

Four opaque candidates were registered once and kept structurally present.

Observed authority sequence:

```text
A -> D -> A
```

No world ID, task ID, change flag, candidate type or host priority was supplied.

Final physical record snapshot:

- A: weight/authority **0.99967223**, observations 36, active;
- stale B: **0.00011476**, observations 42, dormant;
- irrelevant C: **0.0**, observations 20, dormant;
- later-useful D: **0.9717525**, observations 14, active.

A retained its original candidate cell/synapse address through the entire lifetime and reactivated without re-registration.

## U1 coupling

The two compared U1 proposals were identical except for confidence populated from U2 physical authority.

The winner switched:
- regime A -> A proposal;
- regime B -> D proposal;
- regime C -> A proposal.

U1 meta weights remained exactly unchanged throughout U2 learning:
`[0.0,0.0,0.0,0.99980843,0.099980846]`.

## Physical causality

Across four matched decisions:

- necessary hypothesis-utility synapse lesion lost original winner: **4/4**;
- pi phase shift lost original winner: **4/4**;
- exact restore recovered original winner: **4/4**;
- unrelated hypothesis-synapse lesion preserved winner: **4/4**.

## Persistence

Native checkpoint/restart preserved:
- all four provenance records;
- candidate physical addresses;
- learned utility weights;
- observation counts;
- dormancy state implied by physical authority;
- U1 meta weights;
- current winner.

## Regression gate

Same exact-source workflow also passed:
- U1 meta-control;
- G20 persistent runtime;
- G21;
- G22;
- G23;
- Human Protection;
- Release build;
- tracked-source unchanged check.

## Meaning

U2 closes the second post-INTEL architecture gate: stale hypotheses can lose action authority without deletion, later-useful hypotheses can gain it, and old useful hypotheses can reactivate from factual usefulness through shared physical utility.

U2 does not yet prove that heterogeneous real cognitive structures automatically compete for residual/explanation authority. That is U3.
