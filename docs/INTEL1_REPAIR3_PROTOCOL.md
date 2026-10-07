# INTEL-1 Repair-3 — factual evidence fan-out between enabled representation learners

Status: **PRE-REGISTERED BEFORE REPAIR-3 CODE CHANGE**

Date: 2026-10-07

## Preserved INTEL verdicts

- Original INTEL-1 remains FAIL: disconnected-goal bootstrap arbitration gap.
- INTEL-1R1 remains FAIL: frozen exploitation arbitration gap.
- INTEL-1R2 run `37661614264` remains FAIL: W1 and W2 both acquired and reused successfully, then W3 stopped with `NoSupportedAction`.

No prior burned pack is reused or relabeled.

## Earliest R2 causal bottleneck

At the R2 frozen core, context, perceptual and compositional refinement are enabled once before target worlds, as required by INTEL-1.

Production action arbitration checks module applicability. However factual POST routing is not symmetric: `ScientificRuntime::step` dispatches to the first **enabled** representation updater, currently compositional. When compositional refinement is enabled, `observe_phase_native_compositional_result` receives every fact; G21 `observe_phase_native_context_result` receives none.

Therefore the context learner cannot maintain its factual `previous_base`, discovery record, candidate evidence or contextual transition supports during W3. The separate G21 capability remains qualified; the unified lifetime fails to feed it evidence.

## Allowed repair

Repair-3 may ONLY make a single factual transition available to all enabled representation learners that need it.

Requirements:

1. One external action callback and one actual POST only.
2. No evaluator world/task/context IDs enter cognition.
3. No hidden W3 flag, action answer, predecessor answer or state ID.
4. A factual transition may be consumed by:
   - contextual refinement;
   - current-sensory perceptual refinement;
   - compositional refinement;
   as generic observers of the SAME pre/action/post event.
5. The base phase-native transition/rival update must happen at most once per external fact.
6. `current_real` advances exactly once per external fact.
7. Learned evidence counters for each enabled sidecar advance at most once per external fact.
8. Human Protection authorization remains before execution and cannot be bypassed.
9. Existing G20-G23 behavior/regressions remain unchanged.

## Implementation constraint

Preferred implementation: split each representation learner's factual processing into a sidecar phase that consumes explicit pre/action/post abstractions/raw features without independently advancing `current_real` or re-learning the shared parent transition. Then a single coordinator performs:

```text
one factual pre/action/post
 -> context sidecar
 -> perceptual sidecar
 -> compositional sidecar
 -> one shared parent/rival transition update
 -> advance current_real once
```

An equivalent implementation is acceptable only if tests prove no duplicated base transition support/fingerprint side effects.

## Repair witness

Before another INTEL verdict:

- recreate an unknown G21-style history world with all three refinement modules enabled simultaneously;
- no pre-tuition of contextual split;
- FULL must promote a context witness and reach >=60/64 junction decisions;
- context-disabled matched control <=40/64;
- compositional/perceptual candidate counts may remain zero if no raw collision supports them;
- base transition support must equal the number implied by actual factual observations, not 2x/3x fan-out;
- W1/W2 acquisition+reuse repair witnesses PASS;
- G21/G22/G23 regressions PASS;
- Human Protection PASS;
- Release build PASS.

## Freeze/rerun rule

If Repair-3 passes, freeze a new cognitive SHA and run a new independently seeded INTEL verdict. Do not add G24.

Any later INTEL failure is recorded at its earliest criterion and only that bottleneck may be repaired.
