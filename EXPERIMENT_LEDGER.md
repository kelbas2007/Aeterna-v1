# Experiment ledger

This ledger is append-only in meaning: failures are never silently rewritten into successes.

| Gate | Attempt | Status | What it means |
|---|---|---|---|
| G0 | design | READY_FOR_COMPILE | Carrier kernel and matched tests written; no Rust execution yet |
| G0 | CI-1 | TECHNICAL_FAIL_FORMAT | `cargo fmt --check` failed before tests; no scientific verdict |
| G0 | CI-2 | TECHNICAL_FAIL_COMPILE | Rust borrow-check error E0502 in factual state load; fixed without changing gate semantics |
| G0 | CI-3 | FAIL_HELDOUT_CONTEXT_TRANSFER | Code compiled; 3/4 ownership tests passed, but GENUINE solved only 1/2 held-out surfaces |
| G0 | REVISION-1 | PASS_MINIMAL_EVO_OWNERSHIP_PRECURSOR | Generic epistemic pressure was raised so unsupported motor hypotheses remain testable. Final CI: 7/7 tests PASS and release build PASS at commit `1002ae2310ce84cfac847ad941f641a659ac292f`; workflow run 37479430355 |
| G1-A | local-shared-field | FAIL_TRANSLATION_REUSE | CI run 37493255258: raw-raster code compiled, but the learned local motif set grew beyond the preregistered reuse bound on an unseen translation. The local-window representation was still too tied to patch alignment; no G1 capability PASS. |
| G1-B | relational-phase | PASS_RAW_RASTER_RELATIONAL_TRANSFER | CI run 37493664700 at commit `0dba8c2acac189712a9aebb1f7c9e58d4e135e8f`: all 12 tests PASS and release build PASS. GENUINE acquired relation units from raw 12x12 rasters, reused the same unit identity on an unseen translation, chose the correct opaque motor on 2/2 held-out translations, and matched formation/readout ablations lost the full advantage. |
| G2 | CI-1 | TECHNICAL_FAIL_OBSERVER_METRIC | First compiled G2 run reached the held-out epistemic test, but the observer incorrectly mixed withheld-prediction similarity with probe-2 disagreement via `min()`. No cognitive verdict; algorithm/world unchanged. |
| G2 | CI-2 | PASS_ACTIVE_EPISTEMIC_SELECTION | CI run 37495559913 at commit `4bbf08ead05d85c5a30363971e756e7e01615abb`: all 14 Rust tests PASS and release build PASS. Two carrier-owned rival world hypotheses coexist; GENUINE selects the most-discriminating opaque probe first, collapses to one rival after one factual POST on both held-out laws, and predicts a withheld consequence. Matched generic exploration starts with non-discriminating probe 0 and requires more physical probes. Tuition cost: 18 factual probes. |
| G3 | CI-1 | PASS_FIRST_ACQUIRED_MACRO | CI run 37497610519 at commit `da5711f7c2d9491ba38336f0b1154d6e8781ee34`: all 16 Rust tests PASS and release build PASS. From 108 matched factual tuition actions, GENUINE consolidates one two-step branching EvoPhase macro; NO_CONSOLIDATION acquires none. The macro learns opaque first motor 1 and two relational post-trace branches leading to different learned terminal motors, transfers to two unseen spatial bindings, changes held-out action, and reaches factual Need on 2/2 held-out contexts within the fixed two-action budget. |
| G4 | CI-1 | PASS_SAME_IDENTITY_MACRO_REVISION | CI run 37499873532 at commit `9dcf13c67fd44a52dac9a825b93c908b7b535bb8`: all 18 Rust tests PASS and release build PASS. After a world-law change, the same acquired macro ID is retained, contradiction evidence is preserved, obsolete branch action accumulates failures, a newly supported action takes over for the changed branch, the unchanged branch remains correct, and NO_REVISION fails the changed held-out context. Revision cost: 24 physical actions. |
| G5 | CI-1 | TECHNICAL_FAIL_EVALUATOR_BOUNDS | First G5 CI run reached the hierarchy test but one evaluator-only held-out translation exceeded the 12x12 raster boundary. No cognitive verdict; runtime mechanism unchanged. |
| G5 | CI-2 | PASS_HIERARCHICAL_REUSE | CI run 37503517616 at commit `c1292adc5c230c0616ce8c18a3829bd1f53947db`: all 22 Rust tests PASS and release build PASS. Two independently acquired child macros are reused by newly acquired parent macros that store child carrier IDs, not flattened primitive scripts. Parent readout transfers to unseen outer-cue translations and reduces held-out candidate-sequence evaluations from 5 to 2 and primitive search actions from 16 to 8 across two tasks. Reversing child acquisition order permutes opaque child IDs while preserving the hierarchy advantage. Parent tuition cost: 72 primitive actions per arm. |
| G6 | CI-1 | PASS_LEARNED_EXPLORATION_STRATEGY | CI run 37505474193 at commit `cd8330aed18d5d375aa6998420108faefe5042ef`: regressions and release build PASS. Strategy starts with zero weights, receives 32 factual tuition probes across eight 2-rival families with informative opaque probe identity rotated over all four IDs, and learns weights [0.6917086, 0.109836645, 0.109836645]. With strategy learning frozen, transfer to eight 3-rival worlds selects the maximally informative probe first in 7/8 versus ZERO_STRATEGY 2/8; direct disagreement oracle is also 7/8. Learned mean identification cost is 1.0000 probe versus 2.3958 for the all-24-random-orders baseline. |
| G7 | FRESH-1 | FAIL_ROBUST_PERCEPTION_BOUNDARY | One-use fresh run 37508620132 at source `7f6f0ed70cd808d7d4fe64c3c28d0b2c2245d65f`, spec `a36fef13b97e0a83141c25dac5b7850f2281f582`, pack digest `8aee63cd272b8536`: FULL 20/80 = 25.0%, Wilson95 [0.1681,0.3548]. Every sub-seed was 2/8. Distractor/dropout/rotation/scale were each 0/20. ZERO_EXPLORATION and NO_HIERARCHY were also 20/80; UNREVISED was 6/80. Completed cognitive FAIL; pack permanently burned for qualification. |
| R1 / G1-C | MECHANISM-1 | PASS_ROBUST_RELATIONAL_PERCEPTION | Workflow 37512561691 on `main`: preregistered robust-perception mechanism gate PASS. Robust carrier meets clean/nuisance thresholds, strictly exceeds directed-only nuisance performance, formation/readout ablations lose material advantage, whole-organism single-nuisance preflight passes 8/8, and G0-G6/ownership/release regressions remain PASS. Deterministic mechanism witness only; not fresh statistical qualification. |
| G7 | FRESH-2 | PASS_WHOLE_ORGANISM_OPEN_WORLD | One-use fresh run 37512861361 at source `80be115ce28b53d4668b75c9eab7105ada6780b6`, spec `a36fef13b97e0a83141c25dac5b7850f2281f582`, pack digest `982ea2a9d8512fa9`: FULL 79/80=98.75%, Wilson95 [0.9325,0.9978], per-seed [8,8,8,8,8,8,8,8,8,7]. Nuisance: distractor 20/20, dropout 19/20, rotation 20/20, scale 19/20. ZERO_EXPLORATION 78/80 but matched probes 2.3590 vs FULL 1.1667; NO_HIERARCHY 80/80 but matched parent candidates 2.5823 vs FULL 1.0000; UNREVISED 22/80 and 0/58 on revision-required worlds vs FULL 57/58. Release build PASS. Pack permanently burned. |

## What G0 PASS establishes

Within the deliberately small 8-channel / 2-motor precursor world:

- MODEL and IMAGINED prediction do not overwrite REAL factual state;
- residual-driven structural recruitment occurs only when EvoPhase growth is enabled;
- the acquired carrier structure changes held-out motor choice on both tested surfaces;
- matched NO_STRUCTURAL_GROWTH loses the contextual advantage;
- a factual counterexample increments revision on the same recruited EvoPhase branch;
- the Rust crate compiles and an optimized release build succeeds.

This is a **minimal ownership precursor**, not a claim of raw-RGB representation learning, autonomous concept formation, program invention, planning, ARC competence or AGI.

## Preserved failed result

Before the epistemic-pressure revision, the compiled G0 actor collapsed too strongly onto an early globally successful motor. The GENUINE arm achieved only 1/2 held-out surfaces. That failure remains part of the evidence: structural growth alone was insufficient when unsupported motor hypotheses were not explored enough.

## Historical evidence imported as lessons, not PASSes

Earlier AETERNA work showed:
- acquired-model readout can change behavior;
- bounded learning and factual revision are possible;
- retained observations can affect inquiry;
- replacing a redundant representation bank reduced one acquisition sequence from 126 to 79 ACT while preserving 8/8 old missions;
- relation transfer and autonomous program formation were not established.

These facts motivate Aeterna-v1 but do not qualify any G1–G7 gate here.


## What G1-B PASS establishes

Within the preregistered 12x12 / 2-motor world:

- cognition receives only the raw numeric raster, opaque motor tokens and factual Need;
- no orientation label, object label, relevant-pixel list, host dx/dy histogram or correct-action table enters cognition;
- generic retinotopic phase roles are combined by HDC/FHRR binding and pairwise unbinding;
- absolute translation cancels in the carrier relation algebra;
- GENUINE and NO_FORMATION share the same acquisition policy/outcomes while relation readout is disabled;
- GENUINE acquires relation units while NO_FORMATION acquires none;
- an identical acquired unit ID is active on a training scene and an unseen translation;
- after learning is frozen, acquired relation readout gives 2/2 correct held-out opaque motor choices;
- removing relation readout and/or formation removes the complete held-out advantage;
- previous G0 ownership tests remain PASS;
- optimized release build succeeds.

This is a narrow raw-raster relational-transfer result. It does not establish arbitrary object discovery, unrestricted relation invention, acquired programs, planning, ARC competence or AGI.


## What G2 PASS establishes

Within the preregistered hidden-law probe world:

- two incompatible predictive episode models are acquired from factual raw-raster transitions without evaluator law labels entering cognition;
- translated tuition episodes of the same law consolidate into the same carrier hypothesis;
- both rivals remain active at the held-out start;
- epistemic value is computed from disagreement among acquired EvoPhase-owned predicted post-traces;
- GENUINE selects opaque probe 1 first because its carrier predictions disagree most;
- one factual probe reduces the rival set from two to one on each held-out law;
- matched generic exploration begins with non-discriminating probe 0 and requires more factual probes;
- after identification, the surviving model predicts an additional withheld relational consequence before it is revealed;
- tuition cost is 18 factual probes; held-out GENUINE cost is 1 probe per world;
- G0 and G1-B regression tests remain PASS.

This is active epistemic selection from an acquired hypothesis set. It does not yet prove that the organism has learned a reusable **strategy for how to explore** across new task families; that remains G6.


## What G3 PASS establishes

Within the preregistered bounded two-step macro world:

- the macro is absent before tuition;
- both GENUINE and NO_CONSOLIDATION receive identical factual episode ledgers;
- tuition cost is 108 physical actions;
- only factual Need=true trajectories contribute to generic structural consolidation;
- GENUINE promotes one EvoPhase macro from repeated successful carrier dynamics;
- NO_CONSOLIDATION retains the same facts and child representations but promotes zero macros;
- the learned macro contains an acquired opaque first motor and at least two learned relational branches with different terminal motors;
- the same macro transfers to two absolute raster translations absent from tuition;
- macro readout changes at least one held-out physical action;
- G_READOUT reaches factual Need in 2/2 held-out contexts within two actions;
- removing readout and/or consolidation removes the complete held-out advantage;
- G0, G1-B and G2 regressions remain PASS;
- optimized release build succeeds.

This is the first bounded acquired reusable control program in Aeterna-v1. It is a two-step branching macro-assembly, not arbitrary program induction, recursion, a universal VM or AGI.


## What G4 PASS establishes

Within the preregistered macro-revision world:

- both REVISION and NO_REVISION begin from the same single G3-acquired macro;
- both receive identical revision factual ledgers;
- revision cost is 24 physical actions;
- macro count remains 1 and the macro ID is unchanged;
- contradictory outcomes remain stored as explicit counterexamples;
- the obsolete branch action remains represented with accumulated failure evidence rather than being deleted;
- the same macro revision counter increases;
- a newly factually supported terminal action becomes selected for the changed branch;
- the unchanged branch keeps its previous action and remains successful;
- on two held-out translations, REVISION reaches factual Need in 2/2 contexts;
- NO_REVISION still invokes the obsolete changed-branch action and fails that context;
- G0-G3 regressions remain PASS;
- optimized release build succeeds.

This closes same-identity bounded macro repair. It does not yet establish hierarchical reuse of acquired macros inside newly acquired higher-level macros.


## What G5 PASS establishes

Within the preregistered bounded hierarchy world:

- both arms begin with the same two acquired child macros;
- parent tuition ledgers are identical;
- parent tuition cost is 72 primitive physical actions per arm;
- GENUINE acquires two parent macros; NO_HIERARCHY acquires none;
- parent bodies reference acquired child macro IDs and do not copy their primitive action sequences;
- held-out outer cues use translations absent from parent tuition;
- actual child macros execute their own learned bodies when invoked by the parent;
- across two held-out tasks, hierarchy reduces child-sequence candidate evaluations from 5 to 2;
- primitive physical actions spent searching fall from 16 to 8;
- reversing child acquisition order changes opaque carrier IDs, yet consistent parent formation preserves success and cost advantage;
- G0-G4 and ownership audits remain PASS;
- optimized release build succeeds.

This demonstrates bounded hierarchical reuse of acquired programs. It does not yet show that the organism learned a reusable exploration strategy or arbitrary-depth recursive program composition.


## Fresh statistical qualification — G1 translation

Run: `37504043614`  
Source SHA: `a73342a35dcf4b5c08162f3508ecaabbdc6ae261`  
Spec blob SHA: `e32fc861f94ad86bf93631c1ed671762e1fd3f50`  
Authority seed: `37504043614`  
World-pack digest: `8e09c44e887c2ba0`

One-use fresh pack:
- N = 80 held-out worlds;
- 10 independent sub-seeds;
- randomized opaque 2-motor permutation per sub-seed;
- EvoPhase = 80/80 = 1.0000;
- Wilson 95% CI = [0.9542, 1.0000];
- raw-template NN = 28/80 = 0.3500;
- linear perceptron = 36/80 = 0.4500;
- every sub-seed = 8/8.

Therefore the preregistered FRESH-G1 **translation-transfer qualification PASSed**.

Stress diagnostics on the same burned pack:
- +1 distractor pixel: 42/80;
- one task-pixel dropout: 42/80;
- 90-degree rotation with class identity preserved: 0/80;
- doubled spacing: 42/80.

These stress outcomes are diagnostic failures/limitations, not tuned-away results. This pack is burned. Any design change motivated by them must be evaluated on a new fresh authority seed.


## What G6 PASS establishes

Within the preregistered cross-family exploration world:

- exploration strategy weights begin at [0,0,0];
- 32 physical tuition probes provide factual rival-reduction credit;
- informative opaque probe identity rotates across all four action IDs;
- learned state has no probe-index-specific table;
- learned weights after tuition are [0.6917086, 0.109836645, 0.109836645] over [disagreement, coverage, novelty];
- strategy learning is frozen before held-out evaluation;
- held-out family changes from two rivals to three and adds a partial discriminator;
- learned strategy picks the maximally informative probe first in 7/8 held-out worlds;
- ZERO_STRATEGY with identical world models achieves 2/8;
- direct-disagreement oracle achieves 7/8;
- learned mean physical identification cost is 1.0000 probe;
- exhaustive random-order baseline over 24 permutations has mean 2.3958 probes;
- G0-G5 and ownership regressions remain PASS.

This is a bounded learned exploration strategy over inherited epistemic features. It does not show self-invention of the disagreement feature, autonomous curriculum generation, or open-world scientific reasoning.


## G7 FRESH-1 failure diagnosis

Fresh authority:
- source SHA: `7f6f0ed70cd808d7d4fe64c3c28d0b2c2245d65f`;
- protocol blob SHA: `a36fef13b97e0a83141c25dac5b7850f2281f582`;
- authority seed / workflow: `37508620132`;
- world-pack digest: `8aee63cd272b8536`;
- N=80 across 10 independently acquired organisms.

Observed:
- FULL_ORGANISM: 20/80 = 0.2500;
- Wilson 95%: [0.1681, 0.3548];
- per-sub-seed: [2,2,2,2,2,2,2,2,2,2];
- ZERO_EXPLORATION: 20/80;
- NO_HIERARCHY: 20/80;
- UNREVISED_CHILD: 6/80;
- distractor: 0/20;
- dropout: 0/20;
- rotation: 0/20;
- scale: 0/20.

The nuisance assignment contains exactly two clean worlds per 8-world sub-seed. The 2/8-per-seed result together with 0/20 in every nuisance category is therefore consistent with a sharp upstream representation boundary: the whole chain works on clean translated worlds and collapses whenever any preregistered nuisance is applied.

Downstream mechanisms still show causal value on cases that reach them:
- matched probe cost FULL 1.05 vs ZERO_EXPLORATION 2.70;
- matched parent-candidate cost FULL 1.00 vs NO_HIERARCHY 2.65;
- revision-required worlds: FULL 14/57 vs UNREVISED_CHILD 0/57.

No G7 PASS is claimed. This pack is burned and may be used only for diagnosis/regression, never as fresh evidence after redesign.


## What G7 FRESH-2 PASS establishes

Under the frozen G7 protocol and one-use authority pack:

- the ordinary organism combines robust raw-raster perception, acquired rival hypotheses, learned exploration, acquired/revised child macros and acquired hierarchy;
- 79/80 fresh worlds succeed across 10 sub-seeds;
- every nuisance class has high non-zero success;
- learned exploration retains a causal probe-cost advantage even though ZERO_EXPLORATION can often recover by extra probing;
- hierarchy retains a causal candidate-search advantage even though NO_HIERARCHY can often recover by enumeration;
- same-identity macro revision is strongly causal on changed-law worlds;
- all source/spec/seed/digest values were fixed/logged before scoring.

This is a bounded fresh whole-organism qualification, not AGI or unrestricted open-world competence.
