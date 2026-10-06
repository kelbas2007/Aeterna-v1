# Validation hardening after external critique

Date: 2026-10-06

This document separates **mechanism witnesses** from **statistical qualification**.

G0-G4 remain valid as causal mechanism witnesses under their preregistered matched controls. They are not yet claims of broad generalization.

## 1. Statistical qualification

A 2/2 held-out result is not a reliable estimate of generalization. For 2 successes out of 2, the exact two-sided 95% Clopper-Pearson lower bound is only about 0.158.

New qualification policy:

- each capability that claims transfer must be evaluated on at least 64 fresh held-out worlds per qualification run;
- use at least 10 deterministic sub-seeds derived from one fresh authority seed;
- report success rate, mean physical interaction cost, standard deviation, and a 95% confidence interval;
- report matched-control results on the same worlds;
- preserve every failed qualification run.

Small 2/2 witnesses remain useful for causal debugging but are labeled MECHANISM_PASS, not GENERALIZATION_PASS.

## 2. Fresh authority

A held-out set that influenced a design change is burned for future generalization claims.

New fresh-run rule:

1. freeze source commit and qualification spec;
2. commit the spec hash before execution;
3. obtain a fresh seed only after source freeze;
4. generate qualification worlds from that fresh seed;
5. log the seed, source SHA, spec SHA, generated-world digest and result;
6. any design/code change invalidates the run and requires a new seed.

GitHub Actions qualification uses the workflow run ID as an external post-freeze seed source, expanded into deterministic sub-seeds. Re-running after a code change therefore produces a new world pack.

## 3. G1-B claim correction

The G1-B carrier has a built-in translation-equivariant/invariant inductive bias:

- retinotopic position roles are generated compositionally;
- pair relation uses FHRR/HDC unbinding;
- common translation factors cancel algebraically.

Therefore G1-B should not be described as "learning translation invariance".

What was learned:
- which invariant relational trace should recruit/reuse a carrier unit;
- its factual motor/outcome association.

What was inherited:
- the algebra that makes translation cancellation possible.

New stress tests must include transformations not guaranteed by that algebra:
- distractors;
- input noise;
- partial occlusion;
- rotation;
- scale change.

## 4. Stronger baselines

Internal ablations answer causal ownership questions but do not establish competitiveness.

Add external/simple baselines:

G1:
- raw-pixel nearest-neighbor/template memory;
- linear classifier on flattened raster;
- small convolutional baseline when practical.

G2:
- random probe order without replacement;
- generic novelty;
- oracle information-gain ceiling for diagnostic context only.

G3/G4:
- flat tabular trajectory memory;
- non-compositional sequence cache;
- no-consolidation/no-revision controls retained.

Report both interaction cost and tuition cost.

## 5. Mechanical ownership audit

"Full EvoPhase ownership" must not rely only on prose.

Immediate checks:
- production `src/` must not contain evaluator-only world labels or task-class names;
- production code must not import test/evaluator modules;
- task generators live under `tests/` or dedicated evaluator-only crates;
- production APIs receive raw sensors, opaque action IDs and factual outcomes only;
- no production table may map evaluator hidden-state labels to actions.

Stronger planned check:
- randomized action-label permutation and world-label permutation must leave capability intact;
- source-level ownership manifest lists every production function allowed to see REAL, MODEL and IMAGINED state;
- static CI audit fails on forbidden dependencies or evaluator symbol leakage.

## Advancement rule

G5 development may proceed only in parallel with this hardening. No claim of broad intelligence, benchmark competence or scientific superiority may be based on the 2/2 mechanism witnesses alone.
