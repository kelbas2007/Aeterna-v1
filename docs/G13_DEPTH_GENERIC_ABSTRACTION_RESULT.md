# G13 — depth-generic phase-native abstraction

Date: 2026-10-07

## Verdict

**MECHANISM PASS — one generic higher-abstraction rule reused through physical level 3 on the frozen 20x20 substrate.**

Qualifying workflow: `37603203990`  
Qualifying source: `6f0fb657309e16230f0fcc0ac659af57d579b61a`

Protocol: `docs/G13_DEPTH_GENERIC_ABSTRACTION_PROTOCOL.md`

## Preserved protocol-drift run

Workflow `37593455572` was functionally green but used a 40x40 evaluator while the preregistered protocol specified 20x20. It is preserved in the ledger as `NONQUALIFYING_PROTOCOL_DRIFT` and is not counted as G13 PASS.

Production G13 code was not changed to fix that discrepancy. Only evaluator spatial bindings were replaced with valid 20x20 layouts that avoid unintended cross-motif primitive relations.

## Qualifying 20x20 mechanism result

On the exact frozen 20x20 substrate:

- FULL_DEPTH3: **8/8**;
- MAX_LEVEL2: **0/8**;
- NO_HIGHER_ENGINE: **0/8**;
- ZERO_PHASE_L3: **0/8**;
- ZERO_WEIGHT_L3: **0/8**;
- NO_GROWTH_L3: **0/8**;
- necessary L2->L3 lesion: **4/8**;
- pi phase shift on that same physical synapse: **4/8**;
- exact restoration without retraining: **8/8**;
- unrelated L3 lesion preserved target: **2/2**;
- lower L1->L2 lesion left dependent L3 target correct: **0/2**;
- acquired physical hierarchy: **16 atoms -> 8 L1 -> 4 L2 -> 2 L3**;
- source guard PASS;
- full optimized regressions PASS;
- Release build PASS.

## Architectural meaning

G13 replaces the dedicated G11/G12 L2 structure with one generic higher-abstraction engine.

The same node representation and the same factual learning/readout rule are used for:
- L1 -> L2;
- L2 -> L3.

The production engine has no separate task algorithm for L2 and L3. Each higher node records its own level and physical child references. L3 children are promoted L2 nodes rather than flattened primitive atoms or L1 references.

The causal chain in the qualifying witness is:

```text
raw raster
 -> acquired relation atoms
 -> physical L1 concepts
 -> generic physical L2 nodes
 -> generic physical L3 nodes
 -> opaque motor
```

## Boundary

This qualifies one depth-generic mechanism through level 3. It does not establish unbounded recursion, autonomous unlimited depth selection, autonomous primitive feature invention, language, AGI or consciousness.

The next evidence gate is the preregistered one-use FRESH-G13 qualification.
