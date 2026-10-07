# INTEL-1R3 — fourth frozen-core verdict

Date: 2026-10-07

## Verdict

**FAIL — earliest remaining bottleneck: continual contextual candidate interference across worlds.**

Run: `37666196895`  
Frozen cognitive core: `5509564e8deb67c418a4c52ac67cbf360517dd6d`  
Authority seed / burned pack: `37666196895`.

Pre-verdict freeze and all Repair-1/2/3, G21/G22/G23 and Human Protection regressions passed.

Observed:

- W1 unknown navigation: PASS, 9 actions;
- W1 frozen translated reuse: PASS, 3 actions;
- W2 causal machine: PASS, 11 actions;
- W2 frozen translated reuse: PASS, 3 actions;
- W3 factual fanout active, context candidates present, but no correct context promotion;
- W3 scored 0/32 and therefore failed the frozen criterion;
- W4 was reached later in the evaluator but is not a valid project verdict because W3 is already the earliest failure.

W3 candidate diagnostics showed candidates inherited from earlier lifetime experience on overlapping acquired base cells. Each candidate received only one predecessor side (context_switches=0) and no useful promotion.

Source diagnosis: G21 currently admits a new candidate only when **no candidate exists for the base cell at all**, and action readout selects the first non-retired candidate on that base. Therefore an unrelated earlier-world candidate can prevent a later, factually different predecessor/successor hypothesis from being born or used.

This is a continual-learning interference problem, not a failure of factual fanout and not a failure of isolated G21.
