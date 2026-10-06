# Phase-execution ownership audit

Date: 2026-10-06
Status: PRE-REGISTERED DIAGNOSTIC; NOT A CAPABILITY PASS
Audited source: 8732be69db3fe64f03f5b65ab00212a7cdd6f3f2

## Question

The architecture document requires one physical EvoPhase network to support cognition. Owning an adaptive Rust data structure is a weaker condition than executing cognition through phase-cell/synapse dynamics. Does the current G8 production planning path require the latter?

## Source observations

- `src/planning.rs::EvoImaginationPlanner` stores `Vec<LearnedTransition>` and a rollout log.
- `plan_with_depth` uses a LIFO frontier, similarity-matched transition enumeration and discounted reward maximization. It does not take phase cells, synapses or dendritic branches as inputs.
- `src/carrier.rs::plan_imagined` encodes a trace and delegates to that planner. `plan_from_belief` delegates to the same planner.
- `src/raster.rs::robust_shape_trace` supplies inherited geometric features, including coordinate offsets, GCD normalization and normalized squared side lengths. R1 success is not evidence that these invariances were themselves acquired through phase plasticity.

These observations do not invalidate recorded task outcomes. They limit the architectural interpretation of those outcomes.

## Frozen diagnostic

Add `tests/phase_execution_dependency.rs`, using existing production APIs only; do not modify production behavior.

1. Show that the public planning component can acquire a four-edge delayed-reward model and select its depth-three action without constructing any `EvoPhase` instance.
2. Through the ordinary `EvoPhase::plan_imagined` API, compare matched carriers with identical raw transition tuition:
   - default phase-learning settings and 32 dormant cells;
   - zero dormant cells, structural growth disabled, weight/phase learning rates zero, eligibility decay zero.
3. Keep the acquired planner transition table present in both arms. This is an intervention on phase plasticity/capacity, NOT an all-learning ablation.
4. Evaluate six opaque motor permutations at three spatial bindings. Compare action, depth, value and expanded-node count. Check that no dendritic branch or dormant relay was recruited and that planning leaves REAL unchanged.
5. An empty-model control with the same disabled phase settings must return no plan.

No fresh seed, held-out statistical qualification or generalization claim is involved. These are deterministic implementation diagnostics, not an estimate of intelligence.

## Interpretation fixed before execution

If all matched decisions survive the phase-plasticity intervention, record `NEGATIVE_PHASE_DEPENDENCY_WITNESS`: the tested learned planning path depends on its acquired transition table but does not require the disabled phase plasticity/recruitment mechanisms. Combined with the production call path, this does not satisfy the strong claim that the phase network itself executes the planning computation.

This does NOT prove that all EvoPhase mechanisms are irrelevant, that no learning happened, that HDC computation vanished, or that every possible planning task is independent of phase dynamics. The fixed sensory/motor cells still exist in the disabled arm.

A passing diagnostic test means the negative architectural witness was reproduced; it MUST NOT be reported as full phase-ownership PASS.

## Advancement boundary

Preserve all previous numerical results and failed packs. Do not promote G10, or reinterpret G8/G9 as proof of full physical phase execution, merely by nesting more learned tables inside `EvoPhase`.

Before that stronger claim, require a phase-native execution path with an explicit learned-transition-to-cell/synapse provenance map; matched phase-path suppression and restoration must remove and recover the capability while preserving the factual evidence and comparison budget. Existing graph search can remain an explicitly labeled reference baseline.

## Result

NOT EXECUTED at preregistration. Append the exact tested source, CI run and observed counts after execution.
