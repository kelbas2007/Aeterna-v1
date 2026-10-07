# G23 — evidence-gated compositional perceptual function synthesis

Date: 2026-10-07

## Verdict

**MECHANISM / INTEGRATION PASS — AETERNA synthesized and operationalized a depth-2 perceptual program whose constituent raw atoms were individually insufficient.**

Workflow: `37652805915`  
Job: `112900237094`  
Exact tested source: `6425ce20f947f07d57dfbefbf30b59e6020828b3`  
Protocol: `docs/G23_COMPOSITIONAL_PERCEPTUAL_FUNCTION_PROTOCOL.md`  
Evidence artifact: `g23-evidence`, ID `11496733521`, SHA256 `8cad1b04cf5d4e420b2ac1bc74741216d7613b27a105229ccfbc54da4ff758e5`.

This is a deterministic mechanism result, not a fresh statistical qualification or a world-first claim.

## Preserved preflight failures

1. Workflow `37652359750`: evaluator-only failure. It asserted exactly three newborn candidates although the preregistered grammar legitimately produced additional nuisance-dependent candidates. No cognitive score/promotion threshold was reached.
2. Workflow `37652587787`: target XOR already promoted after 34 future-only observations with valid evidence, but the evaluator incorrectly required exactly 40 evidence observations rather than the preregistered >=32 lower bound.

Neither failure changed production logic, protocol thresholds or the deterministic world law.

## What was synthesized

The inherited abstraction and every single G22 descriptor were insufficient for the target XOR consequence.

From one factual successor collision, the frozen grammar generated competing candidates including:

- `Atom(WeakAmplitudeBin(6))`;
- `And(WeakAmplitudeBin(3), WeakAmplitudeBin(6))`;
- `Xor(WeakAmplitudeBin(3), WeakAmplitudeBin(6))`;
- additional nuisance-dependent AND/XOR candidates.

All candidates remained present. They were not outcome-filtered after birth.

The target XOR program promoted independently in both opaque motor permutations after **34 future-only anchor observations**:

- log evidence: **14.71629432**;
- frozen evidence threshold: log(16/0.01) ~= **7.37775891**;
- constituent atom effects: **0.233333 / 0.233333**, both <=0.25;
- discovery observations in validation evidence: 0.

The competing target Atom and target AND programs remained unpromoted:

- Atom(B): log evidence **-2.50308554**, atom effect 0.190476;
- AND(A,B): log evidence **-1.57453121**, constituent atom effects 0.190476 / 0.190476.

Additional nuisance-dependent candidates also remained unpromoted.

## Held-out score

For each motor permutation:

- FULL synthesized-program readout: **64/64**;
- inherited/memoryless abstract readout: **32/64**;
- learned best single-atom comparator: **32/64**;
- all four binary cue combinations: **16/16** each.

Combined descriptive score:

- FULL: **128/128**;
- inherited/memoryless: **64/128**;
- single-atom: **64/128**.

The AND(A,B) candidate did not pass the same evidence gate, so an AND-only learned refined readout was unavailable in the XOR mechanism witness.

## Native representation and causality

The target program AST is stored in native PhaseNativeState. It owns physical program-side cells, two refined state cells and base/program input synapses. Ordinary native transition circuits learn consequences from the refined states.

The completed controls verified:

- zeroing the necessary program-side input synapse makes the target refined readout unavailable;
- pi phase shift on that link also removes the readout;
- exact restoration restores the action and original learned fingerprint without retraining;
- damaging an unrelated composition on another base state preserves the target decision;
- native checkpoint/restart preserves the program/evidence/state and fresh current raw sensing restores discrimination.

Legacy graph transition count remained 0 and dedicated table composites remained unused.

## Regression result

The same exact-source workflow passed:

- G22 perceptual-variable regression;
- G21 contextual-representation regression;
- G20 persistent protected-lifetime regression;
- all Human Protection tests;
- full ordinary regression suite;
- Release build;
- tracked-source integrity check.

No fresh qualification pack was consumed.

## Scientific interpretation

G22 showed selection of one previously unused current-sensory variable from a bounded descriptor family.

G23 adds a narrower but qualitatively stronger capability: **candidate perceptual functions can be synthesized compositionally from observed raw descriptors, compete under future-only evidence, and become operational only when the composition explains consequences that the constituent atoms do not.**

This is still bounded:
- grammar depth=2;
- operators are authored AND/XOR;
- primitives come from the authored G22 descriptor extractor;
- deterministic consequences;
- initial abstraction hierarchy is inherited.

A fresh qualification must now show that the same production mechanism works when descriptor identities, operator target, nuisances, motor roles and state roles are authority-selected rather than fixed in the development witness.
