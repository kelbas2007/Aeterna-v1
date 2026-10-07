# FRESH-G23-1 — burned qualification failure

Date: 2026-10-07

## Verdict

**FAIL — preregistered single-atom control threshold was missed. Pack permanently burned.**

Run: `37654097035`  
Job: `112904681620`  
Source: `86944a3814e63672a3152e6f57e3de09495865cb`  
Spec blob SHA: `22519515188e9c2a00933ddd0f638e0d28d21ec9`  
Authority seed: `37654097035`  
Burned pack: `c81d41f6a27db876`.

Observed frozen metrics:

- FULL **80/80**, Wilson95 [0.954182,1.000000];
- every sub-seed 8/8;
- target promoted 10/10;
- AND 5/5, XOR 5/5;
- all four cue combinations 20/20;
- memoryless 42/80;
- best single-atom comparator **50/80**;
- structure/leakage/atom-effect/wrong-operator/irrelevant-promotion violations all 0;
- lesion 0/10;
- pi phase 0/10;
- exact restore 10/10;
- unrelated lesion 10/10;
- restart 10/10;
- fingerprint/legacy violations 0;
- all six motor roles exercised;
- mechanism preflight, G22/G21/G20/Human Protection, full regressions and Release all PASS.

The preregistered PASS threshold required single-atom <=44/80, so the pack is a scientific qualification FAIL regardless of all other passing criteria.

## Post-failure diagnosis

This threshold was incompatible with the preregistered balanced AND scoring design.

For balanced binary AND:
- a composition solves 4/4 truth-table cases;
- any one constituent atom can achieve 3/4 by predicting the majority negative class when that atom is true or false;
- the no-feature majority classifier also achieves 3/4.

For balanced XOR:
- one atom and the majority baseline each achieve 2/4.

Therefore, with five AND and five XOR seeds and eight repeated scoring decisions per seed, the natural no-compositional-information ceiling is:

- AND single atom: 5 * 6/8 = **30/40**;
- XOR single atom: 5 * 4/8 = **20/40**;
- total = **50/80**.

The observed FRESH-1 single-atom score was exactly this ceiling: every AND seed 6/8 and every XOR seed 4/8. Thus FRESH-1 provides no evidence of a single-atom shortcut, but it cannot be retroactively relabeled PASS because its frozen threshold was <=44/80.

A new protocol may correct this control only prospectively using a new authority pack. FRESH-1 remains failed and burned.
