# FRESH-G1 — Statistical qualification of translation transfer

Status: **PRE-REGISTERED BEFORE IMPLEMENTATION/RUN**

Date: 2026-10-06

This is the first statistical qualification under FRESH-1. It does not replace the G1-B mechanism witness.

## Frozen claim

Given the inherited FHRR/HDC translation-cancelling positional algebra, does the learned EvoPhase relational carrier reliably associate two raw-raster relation classes with opaque motor outcomes across many unseen absolute translations?

The claim is deliberately limited to **translation transfer**. Rotation, scale, distractor and noise robustness are reported separately as stress diagnostics and are not part of this PASS threshold.

## Fresh authority

The world seed is unavailable until after source/spec freeze.

At CI runtime:
- authority seed = GitHub Actions `github.run_id`;
- source SHA = `github.sha`;
- spec blob SHA is logged;
- 10 deterministic sub-seeds are derived from the authority seed;
- each sub-seed receives an independently randomized opaque 2-motor permutation.

Any source or protocol edit burns the pack and requires a new CI run ID.

## Tuition

For each sub-seed:
- a new carrier begins cold;
- both private relation classes are shown only at a fixed preregistered set of training translations;
- both opaque motors are factually tried for each class;
- factual Need identifies which motor succeeds;
- relation formation is enabled, readout disabled during tuition;
- no held-out translation is used in tuition.

## Fresh held-out pack

10 sub-seeds × 8 worlds = **80 held-out worlds**.

For every world:
- class is sampled from the authority seed;
- translation is sampled from valid 12x12 positions not present in tuition for that class;
- motor IDs are opaque and may be swapped by sub-seed;
- learning is frozen before held-out action selection.

## Baselines

Evaluated on the exact same tuition and fresh worlds:

1. **Raw template NN** — nearest raw-pixel training raster by dot similarity.
2. **Linear perceptron** — two-score linear classifier over flattened 144-pixel input.
3. **Chance reference** — 0.5 for two opaque motors; diagnostic only.

These baselines are deliberately simple, but unlike NO_FORMATION they are external representational alternatives.

## Statistics

Report:
- N;
- success count and rate;
- two-sided 95% Wilson interval;
- per-sub-seed success rates;
- template NN success;
- linear success;
- world-pack digest.

## PASS threshold

FRESH-G1 translation qualification PASS requires all:

- N >= 80;
- EvoPhase success >= 76/80 (95%);
- Wilson 95% lower bound > 0.88;
- EvoPhase exceeds raw-template success by >= 0.20 absolute;
- EvoPhase exceeds linear success by >= 0.15 absolute;
- every sub-seed is at least 6/8;
- previous regression and ownership tests remain PASS.

Thresholds are frozen before seeing the authority seed.

## Stress diagnostics

Using a separate deterministic derivative of each sub-seed, report but do not tune on:

- one random distractor pixel;
- one randomly dropped task pixel when at least two remain;
- 90-degree rotation around the local 3x3 pattern;
- doubled spacing where raster bounds permit.

These are **diagnostic** because the current translation-invariant algebra does not promise invariance to them.

Any future architecture change motivated by these results must use a new fresh authority pack for qualification.
