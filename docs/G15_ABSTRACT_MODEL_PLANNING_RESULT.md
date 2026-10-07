# G15 — abstract phase-native model-based planning

Date: 2026-10-07

## Verdict

**MECHANISM PASS — acquired physical abstractions serve as states of a learned multi-step future model.**

Workflow: `37608846717`  
Scored source: `156421b208b59591d6addffed3f8ce913471b7b6`  
Protocol: `docs/G15_ABSTRACT_MODEL_PLANNING_PROTOCOL.md`

## Preserved technical trigger failure

Workflow `37608815206` failed at workflow parsing before any job or cognitive test ran. The G15 code/protocol/test were unchanged; no scientific pack was involved.

## Mechanism result

Across both opaque planning-motor permutations and four held-out raw bindings per permutation:

- FULL_ABSTRACT_PLANNING: **8/8**;
- successful FULL decisions with selected physical depth >=3: **8/8**;
- DEPTH1 delayed-branch choice: **0/8**;
- NO_ABSTRACT_MODEL delayed-branch choice: **0/8**;
- BROKEN_ABSTRACT_STATE delayed choice: **0/8**;
- BROKEN_TRANSITION delayed choice: **0/8**;
- PI_PHASE_TRANSITION delayed choice: **0/8**;
- exact RESTORE without retraining: **8/8**;
- IRRELEVANT_LESION preserved target: **2/2**;
- source guard PASS;
- full optimized regressions PASS;
- Release build PASS.

The abstract transition model was learned on one raw binding and evaluated on different raw bindings without transition relearning.

Planning left the factual REAL frame and phase-native learned fingerprint unchanged.

## Architectural meaning

The causal chain is now:

```text
raw observation
 -> acquired physical L2 concept cell
 -> learned phase-native abstract transition
 -> future acquired abstract cell
 -> further learned abstract transition(s)
 -> delayed factual value
 -> backward physical value propagation
 -> opaque motor choice
```

The abstract planner does not create a host graph or task-level transition table.

The existing P1 physical transition/value circuit was generalized to accept already-acquired abstraction cells as its endpoints. Abstract-state recognition is itself physically causal through acquired lower concept/deep synapses.

The development world deliberately makes the immediate action worth 0.55 while the delayed route begins with value 0. Depth-1 therefore prefers the immediate option, while adequate physical propagation chooses the delayed route.

## Boundary

This establishes a bounded abstract model-based planning mechanism.

It does not establish:
- stochastic planning;
- open-ended goal invention;
- arbitrary program search;
- natural-language reasoning;
- AGI or consciousness.

The active evidence gate is one-use FRESH-G15 across randomized abstract hierarchies, raw bindings and delayed route lengths.
