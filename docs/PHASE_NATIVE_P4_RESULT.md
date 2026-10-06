# P4 — learned phase-native exploration drive result

Date: 2026-10-06

## Verdict

**P4 mechanism PASS + FRESH-P4 PASS (bounded claim).**

P4 replaces the P3 target-world hand-written frontier selector with a learned exploration drive stored in physical EvoPhase synapses. The drive is meta-trained only from factual phase-native model gain on independently cold short source worlds, then transferred by itself into cold longer target worlds with no source world model.

No AGI claim follows.

## Preregistered sources

- Mechanism protocol: `docs/PHASE_NATIVE_P4_PROTOCOL.md`
- Fresh protocol: `docs/PHASE_NATIVE_P4_FRESH_PROTOCOL.md`
- Fresh protocol preregistration commit: `6dd36801172019e01921a0cdb3783aa76fcddc1c`

## Deterministic mechanism witness

Workflow: `37530869041`  
Source: `9aa725a443ac7f975896c4e20f689cef90dae334`

Observed:

- learned physical drive weights: `[0.99999994, 1.0]`;
- factual drive observations: 61;
- LEARNED_DRIVE: 12/12;
- mean target acquisition cost: 25.000 physical interactions;
- ZERO_DRIVE: 0/12;
- FRONTIER_WEIGHT_LESION: 0/12, mean 60.000;
- ZERO_PHASE_DRIVE_TUITION: 0/12, mean 60.000;
- seeded RANDOM_ACTION: 6/12, mean 47.917;
- P3 teacher ceiling: 12/12;
- exact learned frontier-synapse restoration recovered the deterministic lesion without drive retraining;
- source guard, previous regressions and Release build PASSed.

The target selector contains no P3 selector fallback or host graph-search fallback.

## Fresh qualification

A first workflow edit produced run `37531623310` with zero jobs because the workflow expression was invalid. The evaluator never started and no authority-derived world pack was generated or printed. This is preserved as **TECHNICAL_FAIL_WORKFLOW_PARSE**, not a cognitive fresh attempt.

The first valid authority execution was:

- run: `37531676452`;
- source SHA: `3795f3865f90dbff1942ffbc7d450aba24b2b66d`;
- spec blob SHA: `2069b8269944cb65ff5967e9580bd3c152640fdb`;
- authority seed: `37531676452`;
- run attempt: 1;
- pack digest: `47b2a86ebe50dc8f`;
- N = 80 target worlds across 10 sub-seeds.

All generated source and target laws were printed before acquisition/scoring.

### Fresh result

- FULL_LEARNED_DRIVE: **80/80**;
- Wilson 95% CI: **[0.954182, 1.000000]**;
- per-sub-seed FULL: **[8,8,8,8,8,8,8,8,8,8]**;
- ZERO_DRIVE: **0/80**;
- FRONTIER_LESION: **7/80**;
- ZERO_PHASE_DRIVE: **0/80**;
- RANDOM_ACTION: **18/80**;
- P3_TEACHER ceiling: **80/80**.

Interaction cost:

- FULL mean **24.087**, SD **8.243**;
- FRONTIER_LESION mean **55.700**, SD **14.031**;
- ZERO_PHASE_DRIVE mean **60.000**, SD **0.000**;
- RANDOM_ACTION mean **52.062**, SD **16.311**;
- total source meta-tuition cost across the 10 sub-seeds: **646 physical interactions**.

Learned drive weights by sub-seed:

```text
[0.99999994, 1.0]
[0.99999994, 1.0]
[0.99999994, 1.0]
[0.99999994, 1.0]
[0.9999993,  0.8200544]
[0.99999994, 1.0]
[0.99999994, 1.0]
[0.99999994, 1.0]
[0.99999994, 1.0]
[0.99999994, 1.0]
```

Drive observation counts:

`[60,51,78,76,41,65,66,66,71,72]`.

Every fresh target carrier began with zero world receptors, zero transition circuits and zero legacy graph transitions. Target drive learning was frozen, and assertions verified the restored drive weights did not change during target acquisition.

Full optimized regression suite and Release build PASSed before the fresh P4 step.

The pack is now permanently burned.

## What P4 establishes

Within the tested deterministic reset-chain family:

1. exploration value begins with zero learned weights;
2. its usefulness is learned from factual changes in AETERNA's own phase-native world model;
3. only the learned meta-drive, not source-world routes, transfers between cold organisms;
4. the transferred drive causally improves acquisition in longer unseen worlds;
5. the REACHABLE_FRONTIER physical synapse is causally necessary for the large advantage;
6. drive phase learning is causally necessary under the tested contract;
7. the P3 fixed target selector is no longer needed for the qualified target behavior.

This is materially stronger than P3: target exploration policy is no longer a fixed unknown/frontier valuation supplied by substrate code. The generic feature vocabulary itself remains inherited.

## Remaining limits

P4 does **not** establish:

- invention of new intrinsic-motivation features;
- stochastic/noisy exploration;
- general POMDP exploration;
- continual learning across interfering long-lived worlds;
- protection from catastrophic forgetting across domains;
- autonomous creation of arbitrary concepts/goals;
- general intelligence, AGI or consciousness.

## Next architectural bottleneck

The next useful step should not be another reset-chain score.

The stronger target is **continual self-directed learning**:

- one persistent organism, not a newly cold carrier per target;
- multiple changing worlds/domains encountered sequentially;
- learned exploration retained and adapted;
- useful old world models/skills remain recoverable;
- contradiction causes selective revision rather than global overwrite;
- no evaluator curriculum and no task identity label.

Separately, preregistered G10 remains the target for autonomous composite-concept construction.
