# G6 — LEARNED CROSS-FAMILY EXPLORATION STRATEGY

Status: **PRE-REGISTERED BEFORE G6 IMPLEMENTATION**

Date: 2026-10-06

## Question

Can EvoPhase acquire a reusable policy for choosing informative physical experiments from factual experience, then transfer that policy to a new world-model family where probe identities and rival-count structure differ?

G6 specifically removes the G2 hard-coded `4.0 * disagreement` probe rule from the tested path.

## Inherited substrate

Allowed generic probe features:

- prediction disagreement among currently active carrier hypotheses;
- prediction coverage (fraction of active hypotheses that predict a probe);
- probe novelty / local use count;
- factual reduction in surviving rivals after a physical POST.

These are carrier-native quantities, not evaluator law labels.

Allowed generic learning machinery:

- bounded scalar/vector weights;
- local supervised/credit update from factual information gain;
- argmax over the **learned** score.

Not allowed:

- a fixed positive coefficient on disagreement in the G6 tested path;
- evaluator-provided informative-probe labels;
- probe-index-specific task tables;
- copying the G2 hard-coded disagreement argmax as the strategy.

## Strategy tuition family

Tuition worlds contain:

- 2 rival acquired hypotheses;
- 4 opaque probes;
- exactly one strongly discriminating probe per world;
- informative probe index permuted across worlds;
- other probes provide zero or weak information.

World models are acquired first through a fixed full factual curriculum. Then a fixed generic probe curriculum physically tries every probe from a reset unknown episode.

For every tried probe, EvoPhase observes:

```text
carrier probe features before ACT
-> physical ACT
-> factual POST
-> rivals_before -> rivals_after
```

The strategy learner receives only the feature vector and factual information-gain target derived from rival reduction.

## Learned strategy state

The strategy begins with zero/uninformative weights.

It must learn a reusable mapping:

```text
probe features -> expected factual information gain
```

The learned strategy state is persistent EvoPhase-owned adaptive state.

## Held-out family

Held-out worlds differ structurally:

- 3 rival acquired hypotheses instead of 2;
- 4 opaque probes;
- one probe separates all three rivals in one factual action;
- another may only partially discriminate;
- remaining probes are redundant;
- probe IDs are permuted across held-out worlds.

The held-out family world models are acquired via a fixed full factual curriculum, but **no strategy credit update is allowed on held-out worlds before probe selection**.

Thus G6 tests transfer of the exploration policy, not transfer of the world model.

## Matched controls

LEARNED_STRATEGY:
- same held-out hypotheses;
- learned exploration weights from tuition family;
- strategy learning frozen.

ZERO_STRATEGY:
- same world models and resources;
- strategy weights reset/disabled;
- generic novelty/tie-break only.

RANDOM_ORDER:
- same world models;
- all 24 permutations of four probe orders evaluated as a baseline;
- reports mean physical probes required to identify one rival.

ORACLE_DISAGREEMENT:
- diagnostic ceiling using direct maximum disagreement;
- not counted as a learned baseline.

## Acceptance

G6 mechanism PASS requires:

1. Strategy weights are zero/uninformative before tuition.
2. Tuition uses at least 8 worlds with informative probe identity permuted across all four opaque IDs.
3. Learned strategy weights change only from factual rival reduction.
4. No probe-index-specific weights/table are stored.
5. On at least 8 held-out 3-rival worlds, with informative probe IDs permuted:
   - LEARNED_STRATEGY selects a maximally informative probe first in >= 7/8;
   - one factual probe reduces active rivals to one in those successful worlds.
6. ZERO_STRATEGY is strictly worse on first-probe identification.
7. LEARNED_STRATEGY mean physical probes is lower than RANDOM_ORDER mean over all 24 probe orders.
8. Learned strategy performance approaches the ORACLE_DISAGREEMENT diagnostic ceiling without calling the oracle path.
9. Strategy learning is frozen during held-out evaluation.
10. G0-G5 regressions and ownership audits remain PASS.
11. Rust tests and release build PASS.
12. Tuition physical probe cost is reported.

## Interpretation boundary

A PASS establishes a bounded learned exploration strategy over inherited epistemic features.

It does **not** establish:
- self-invention of the disagreement feature itself;
- arbitrary scientific reasoning;
- autonomous curriculum generation;
- open-world AGI.

## Next gate after PASS

G7 — open-world qualification with fresh one-use world packs, stronger baselines, and no benchmark-specific solver.
