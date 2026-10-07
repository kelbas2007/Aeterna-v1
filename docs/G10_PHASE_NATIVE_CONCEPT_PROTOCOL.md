# G10-PHYS — PHASE-NATIVE COMPOSITE CONCEPT EXECUTION

Status: **PRE-REGISTERED BEFORE PHYSICAL G10 IMPLEMENTATION**

Date: 2026-10-07

## Motivation

FRESH-G10 qualified bounded composite concept construction, but the physical-ownership audit showed that the dedicated `EvoConceptMemory` can execute the same composite task without requiring the EvoPhase phase-cell/synapse network.

G10-PHYS does not erase that behavioral result. It adds the missing architectural requirement: the acquired concept must be instantiated and read out through the same physical EvoPhase cells and synapses used by P1-P5.

## Physical representation

The existing generic raw-raster atom extractor and factual promotion rule remain the structural learning substrate.

For every acquired lower-level atom:
- recruit one dormant physical EvoPhase cell;
- bind only the opaque acquired atom ID to that cell as structural address metadata.

For every promoted composite:
- recruit one dormant physical composite cell;
- create two acquired child-atom -> composite synapses;
- create composite -> opaque motor synapses;
- learn synaptic weights and phase offsets from the same factual evidence that promoted/revised the composite.

The dedicated concept memory may retain provenance/statistics and structural IDs, but **physical readout must not call its action-selection methods**.

## Physical readout

Input:
- raw 12x12 sensory raster only.

Execution:
1. generic atom recognition determines which already-acquired atom IDs are active;
2. the corresponding physical atom cells are activated;
3. a composite cell receives current only when both acquired child pathways conduct coherently;
4. composite current propagates through acquired physical motor synapses;
5. local motor competition selects the opaque action.

No table lookup from composite ID to answer is allowed in the readout path.

## Causal interventions

The physical mechanism test must identify actual synapse indices and show:

1. intact PHYSICAL_COMPOSITE solves 8/8 held-out development decisions;
2. lesioning one necessary child->composite synapse causes a material loss;
3. shifting that same learned synapse phase by pi causes a material loss;
4. restoring the exact saved synapse without retraining restores the lost decisions;
5. lesioning an unrelated concept synapse preserves the target decision;
6. zero phase learning is materially worse;
7. zero weight learning is materially worse;
8. no dormant capacity / structural growth cannot acquire the physical concept layer;
9. old dedicated-memory action readout is disabled during physical qualification;
10. both opaque motor permutations are covered.

## Source guard

The physical G10 action selector must not call:
- `choose_composite_action`;
- `choose_atom_only_action`;
- evaluator class/pair helpers;
- legacy planner/search;
- a mapping from composite IDs to correct motor IDs.

It may use:
- active acquired atom IDs from generic perception;
- physical cell/synapse structural addresses;
- physical conductance/coherence;
- local motor competition.

## Mechanism PASS thresholds

PASS requires:

- intact physical readout: **8/8**;
- old table readout disabled/unused in the physical path;
- necessary weight lesion: <= **4/8** on the targeted causal cases or at least 2 decisions lost;
- necessary pi phase shift: same loss criterion;
- exact restoration: recovers the intact score without concept retraining;
- unrelated lesion: preserves the corresponding target decision;
- ZERO_PHASE acquisition <= **4/8**;
- ZERO_WEIGHT acquisition <= **4/8**;
- NO_CAPACITY / NO_GROWTH cannot reach intact capability;
- full previous regressions and Release build PASS.

A later one-use fresh qualification is required before extending the fresh G10 claim to physical execution.

## Boundary

G10-PHYS qualifies physical execution of bounded acquired binary composites. It does not establish arbitrary-arity/recursive concepts, invention of primitive feature vocabularies, language semantics, AGI or consciousness.
