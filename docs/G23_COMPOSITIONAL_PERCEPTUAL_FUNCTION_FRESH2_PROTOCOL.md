# FRESH-G23-2 — REVISED INDEPENDENT COMPOSITIONAL QUALIFICATION

Status: **PRE-REGISTERED AFTER FRESH-G23-1 FAILURE, BEFORE FRESH-G23-2 EVALUATOR/RUN**

Date: 2026-10-07

## Why a second fresh qualification exists

FRESH-G23-1 permanently failed and burned pack `c81d41f6a27db876` because the preregistered total best-single-atom threshold <=44/80 was mathematically incompatible with the same protocol's balanced AND scoring.

This protocol does not relabel or reuse FRESH-1. It creates a new independently seeded experiment.

The production G23 mechanism, program grammar, evidence gate, target-world generation, intervention thresholds and FULL thresholds remain unchanged.

Only the single-atom comparator is corrected prospectively.

## Correct no-composition comparator

Held-out scoring presents all four binary input combinations equally.

For an AND target:
- target truth values are [0,0,0,1];
- a predictor using only A or only B can achieve at most 3/4;
- a no-feature majority-negative predictor also achieves 3/4;
- with eight held-out decisions per seed, the no-composition ceiling is **6/8**.

For an XOR target:
- target truth values are [0,1,1,0];
- a predictor using only A or only B can achieve at most 2/4;
- the majority baseline also achieves 2/4;
- with eight held-out decisions per seed, the no-composition ceiling is **4/8**.

With exactly five AND and five XOR sub-seeds:
- AND best-single ceiling = **30/40**;
- XOR best-single ceiling = **20/40**;
- total = **50/80**.

The scientific question is whether the composed program exceeds these operator-specific ceilings while its constituent atoms also remain below the frozen effect-size gate at promotion.

## Authority and pack

First valid G23-2 GitHub Actions attempt only:

- authority seed = `github.run_id`;
- source SHA = `github.sha`;
- spec SHA = blob SHA of this file;
- run attempt = 1;
- exactly 10 deterministic sub-seeds;
- new authority seed must differ from FRESH-1.

Before scoring, print/hash the same sealed fields as FRESH-G23-1:
- target operator;
- five distinct subthreshold bins A/B/N/C/D;
- base/goal/dead/irrelevant state roles;
- six motor roles;
- raw marker-position triplets;
- raster-layout order;
- future training samples;
- held-out score samples.

The first observed G23-2 pack is permanently burned.

## Frozen world generation

Use the same already preregistered construction as FRESH-G23-1:

- exactly 5 AND and 5 XOR seeds;
- same bounded G22 descriptor family;
- same inherited abstraction family;
- same authority-randomized descriptor/state/motor/position/nuisance assignments;
- same AND future distribution: 00=24, 01=24, 10=24, 11=8 over 80 anchor facts;
- same XOR future distribution: 00=48, 01=8, 10=8, 11=16 over 80 anchor facts;
- complementary Y action;
- outcome-independent irrelevant candidate pool;
- 8 held-out decisions per seed with all four combinations twice.

No development constants A/B or fixed target operator may enter production cognition.

## Frozen evidence and structure criteria

Unchanged from FRESH-G23-1:

1. 10 sub-seeds, 5 AND and 5 XOR.
2. Target composition promotes 10/10.
3. AND promotes 5/5 and XOR 5/5.
4. Every target is the authority-selected depth-2 operator.
5. Discovery-evidence leakage = 0.
6. Every constituent atom effect <=0.25 at promotion.
7. Wrong-operator target promotions = 0.
8. Irrelevant candidate promotions = 0.
9. Legacy graph/table-composite violations = 0.
10. All six motor IDs participate as X or Y.
11. Production source guard PASS.

## Frozen behavioral criteria

Unchanged except comparator correction:

- FULL >= **76/80**;
- Wilson95 lower bound >= **0.87**;
- every sub-seed FULL >= **7/8**;
- every A/B combination >= **19/20** overall;
- inherited/memoryless <= **44/80**;
- **AND best-single <=30/40**;
- **XOR best-single <=20/40**;
- combined best-single <= **50/80**.

The single-atom comparator is learned only from the same authority-selected target training facts and receives no operator label.

## Frozen interventions

Unchanged:

- target weight-lesion correct decisions <=1/10;
- pi-phase correct decisions <=1/10;
- exact restore >=9/10;
- unrelated lesion >=9/10;
- checkpoint/restart >=9/10;
- frozen-scoring learned-fingerprint violations =0.

## Regression gate

Before the new pack is exposed:

- deterministic G23 mechanism PASS;
- G22 PASS;
- G21 PASS;
- G20 PASS;
- Human Protection PASS;
- full ordinary regressions PASS;
- Release build PASS.

## Verdict rule

A workflow/compile failure before `FRESH_G23_2_SEAL` is technical and does not expose the new pack.

Once `FRESH_G23_2_SEAL` is printed, the pack is burned. Missing any frozen G23-2 threshold is a scientific FAIL.

FRESH-G23-1 remains a failed experiment regardless of G23-2 outcome.

## Interpretation boundary

A G23-2 PASS would qualify fresh bounded composition within a depth-2 authored AND/XOR grammar and the authored G22 primitive descriptor family.

It would still not establish arbitrary program synthesis, learned operator invention, unbounded grammar growth, language semantics, broad causal discovery or AGI.
