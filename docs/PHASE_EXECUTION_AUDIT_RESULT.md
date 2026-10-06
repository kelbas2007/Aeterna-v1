# Phase-execution audit — execution 1

Date: 2026-10-06
Architectural verdict: **NEGATIVE_PHASE_DEPENDENCY_WITNESS**
Software verdict: **DIAGNOSTICS AND REGRESSIONS PASS**
Full physical phase-ownership claim: **NOT ESTABLISHED**

## Provenance

- Original production source inspected: `8732be69db3fe64f03f5b65ab00212a7cdd6f3f2`.
- Preregistration commit: `efb4c6dbf8fa2e7f6d17cc0e288f69f721c5f947`.
- Diagnostic implementation commit: `be64efabad6b8c3e57d924a60d876e1435dee940`.
- Executed source: `199739b97685f351c7027aa6488c562bea7c2876`.
- Preregistered audit spec blob: `8b699abf7ab3581ab0aa6b135477be99d81dd71a`.
- GitHub Actions run: `37519821112`.
- Job: `112462059505`.
- Run URL: https://github.com/kelbas2007/Aeterna-v1/actions/runs/37519821112
- No production source file was changed for this diagnostic.
- No fresh qualification seed was supplied; no new fresh-pack result is claimed.

## Observed results

The three added tests compiled and passed.

1. The standalone `EvoImaginationPlanner`, with an acquired four-transition table and HDC traces, selected the delayed route at depth 3 after expanding 4 nodes. No `EvoPhase` instance was constructed in this component-level test.
2. Through `EvoPhase::plan_imagined`, all **18/18** paired decisions were identical across six opaque motor permutations and three translated bindings. Action, predicted value bits, selected depth and expanded-node count all matched. The second arm had zero dormant cells, structural growth disabled, zero phase/weight learning rates and zero eligibility decay. Both arms acquired the same planner transition table. Neither recruited a relay or dendritic branch. REAL remained unchanged by planning.
3. The same disabled-phase configuration without an acquired transition table returned no plan.

Relevant log excerpts:

```text
AUDIT_SOURCE_SHA=199739b97685f351c7027aa6488c562bea7c2876
AUDIT_SPEC_SHA=8b699abf7ab3581ab0aa6b135477be99d81dd71a
PHASE_EXECUTION_AUDIT standalone_table_planner=SUCCESS phase_network_instances=0 depth=3 expanded=4
PHASE_EXECUTION_AUDIT empty_table_control=NO_PLAN
PHASE_EXECUTION_AUDIT result=NEGATIVE_PHASE_DEPENDENCY_WITNESS matched_identical=18/18 permutations=6 bindings=3 recruited_relays=0 dendritic_branches=0 table_learning=PRESENT full_physical_ownership=NOT_ESTABLISHED
test result: ok. 3 passed; 0 failed; 0 ignored
```

The full `cargo test --all --all-targets` regression command and `cargo build --release` also passed. Historical one-use qualification entry points were not supplied a new qualification seed; their wrapper return status is not fresh evidence. Existing test-fixture compiler warnings remain; this is not a warning-free build claim.

## What this means

The tested planning computation depends on an acquired transition table, not on the phase plasticity/recruitment mechanisms disabled in this audit. Source inspection further shows that a bounded LIFO graph traversal computes action selection in `EvoImaginationPlanner::plan_with_depth`; the carrier planning entry points delegate to it.

This supports learned model-based graph planning over carrier traces. It does not demonstrate the stronger architectural promise in `docs/ARCHITECTURE_RU.md`: one physical phase-cell/synapse network itself performing imagination and planning.

Containing a planner field inside `EvoPhase` establishes data encapsulation. That alone is not causal evidence of phase-native execution.

## Limits

This is an implementation-level negative witness, not a statistical evaluation or proof about every EvoPhase capability. The disabled arm still contains fixed sensory/motor cells, HDC algebra, raster encoding and an acquired planner table. No claim is made that all computation, all memory or all learning was removed.

The test did not remove or scramble every existing phase-cell value after learning. A future stronger test needs such targeted interventions once planning actually consumes those learned physical pathways. The current independent call path explains why suppressing unused plasticity does not affect this planner.

Previous G0-G9 task outcomes, including their fresh successes and failures, are preserved. They must not be silently upgraded to full physical phase ownership. G9 belief-to-plan calls the same planner, but its recurrent belief mechanism was not separately ablated here.

## Consequence for the next step

G10 remains preregistered, not implemented or qualified by this audit. Before claiming a phase-native G10 or full physical G8/G9 ownership, establish a learned physical execution path and a matched suppression/restoration test. Keep the present graph planner available as a labeled baseline; do not replace evidence or lower the ownership requirement to make existing code pass.

Required follow-on witness:
- factual experience forms addressed phase-cell/synapse pathways;
- model transitions used by imagination have provenance in those pathways;
- generic carrier dynamics, not a separate frontier over a transition table, produces the candidate continuation and motor preference;
- selective suppression of the learned path removes its contribution while leaving factual records and control budgets intact;
- restoration recovers it and a factual counterexample revises the same physical structure.

This follow-on mechanism has NOT been implemented in execution 1.

## Execution controls

Automatic push-triggered CI was restored to manual-only after the sealed diagnostic checkpoint, in commit `0822643ef92136de8ab4f816e4bda613fcfd3d49`. No LLM dependency, paid browser task or external inference API was introduced.
