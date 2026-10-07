# G11 — recursive phase-native abstraction

Date: 2026-10-07

## Verdict

**MECHANISM PASS after preserving two earlier preflight failures.**

Workflow: `37577562551`  
Scored source: `5f0423ea7f98a74024e42570c00cbc00994b605e`

Protocol: `docs/G11_RECURSIVE_ABSTRACTION_PROTOCOL.md`, frozen before implementation.

## Preserved failures

### PREFLIGHT-1 — order-sensitive promotion

Workflow `37577194623` reached Stage 1 but produced 0 promoted level-1 concepts instead of 4.

Diagnosis: a supported physical pair candidate was checked for promotion only when that pair itself was observed. Later factual evidence could make its children non-predictive, but the old supported candidate was not reconsidered.

Repair: after every factual observation, all sufficiently supported physical pair candidates are re-evaluated against current child evidence. The same order-invariant rule was applied to recursive candidates.

This is an architectural improvement: concept formation no longer requires a lucky curriculum ordering.

### PREFLIGHT-2 — evaluator control assertion

Workflow `37577379726` advanced beyond the first failure, then the shared Stage-2 helper asserted that `NO_GROWTH` must accept structural allocation.

That contradicted the preregistered negative control. The observer was fixed so FULL/learning controls must accept factual updates while NO_GROWTH may reject creation of a new physical level-2 cell.

No G11 fresh pack was consumed by either failure.

## Passing mechanism result

Observed in workflow `37577562551`:

- FULL_RECURSIVE: **8/8**;
- NO_RECURSION: **0/8**;
- LEVEL1_ONLY: **4/8**;
- necessary level1->level2 weight lesion: **6/8**;
- pi phase shift of the same necessary synapse: **6/8**;
- exact restoration without retraining: **8/8**;
- unrelated level-2 lesion preserves target: **2/2**;
- lower atom->level1 lesion leaves correct dependent top decision: **0/2**;
- ZERO_PHASE: **0/8**;
- ZERO_WEIGHT: **0/8**;
- NO_GROWTH: **0/8**;
- maximum individual level-1 top-action evidence: **0.000000**;
- minimum winning level-2 joint evidence: **1.000000**;
- source guard PASS;
- full optimized regressions PASS;
- Release build PASS.

## What is physically recursive

The successful action path is:

```text
raw relation atoms
 -> acquired atom cells
 -> acquired promoted level-1 concept cells
 -> newly recruited level-2 concept cell
 -> opaque motor
```

Every promoted level-2 circuit stores the IDs and physical cells of two previously acquired promoted level-1 concepts. It does not store primitive atom IDs as its children.

A lesion at either abstraction layer breaks the dependent top-level decision, while an unrelated lesion does not.

## Interpretation boundary

This establishes a bounded depth-2 recursive physical abstraction mechanism.

It does not establish arbitrary recursion depth, autonomous discovery that abstraction is needed, arbitrary arity, semantic language concepts, AGI or consciousness.

The next evidence gate is a one-use FRESH-G11 authority pack.
