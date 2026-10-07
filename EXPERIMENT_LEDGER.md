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
| G8 | CI-1 | FAIL_ROBUST_TRACE_DEPTH_COLLAPSE | First completed G8 mechanism run reached the planning witness but the robust ShapeTrace containment channel collapsed distinct route states enough that the selected plan did not require depth >=3. Planner unit test and prior regressions passed. Development witness only; no fresh pack consumed. |
| G8 | CI-2 | TECHNICAL_FAIL_ESCAPED_NEWLINE | Follow-up run failed in rustfmt because a literal escaped newline was inserted into the evaluator test while adding diagnostics. No cognitive verdict. |
| G8 | CI-3 | PASS_IMAGINED_DELAYED_REWARD_PLANNING | Workflow 37513785835 at source `371f9579be97ec6a3a5816c31b01d53260924b72`: FULL_IMAGINATION 8/8, DEPTH1 0/8, SHUFFLED_MODEL 0/8, rollout_nodes=64. Selected plans were depth 3 with Authority::Imagined, opaque motor permutations were followed, REAL state remained unchanged during imagination, regressions/ownership/R1/release build PASS. Exact relational FHRR channel used for the bounded planning witness. |
| G7 | FRESH-2 | PASS_WHOLE_ORGANISM_OPEN_WORLD | One-use fresh run 37512861361 at source `80be115ce28b53d4668b75c9eab7105ada6780b6`, spec `a36fef13b97e0a83141c25dac5b7850f2281f582`, pack digest `982ea2a9d8512fa9`: FULL 79/80=98.75%, Wilson95 [0.9325,0.9978], per-seed [8,8,8,8,8,8,8,8,8,7]. Nuisance: distractor 20/20, dropout 19/20, rotation 20/20, scale 19/20. ZERO_EXPLORATION 78/80 but matched probes 2.3590 vs FULL 1.1667; NO_HIERARCHY 80/80 but matched parent candidates 2.5823 vs FULL 1.0000; UNREVISED 22/80 and 0/58 on revision-required worlds vs FULL 57/58. Release build PASS. Pack permanently burned. |
| P3 | PREFLIGHT-1 | TECHNICAL_FAIL_TRANSIENT_FINGERPRINT | Workflow 37527093837 at source `9f2ab1b6c98544d6003ea4c959c372cbd0d04739`: autonomy test reached checkpoint comparison, but the historical learned fingerprint included transient REAL charge although persistence was intended to exclude the current observation. No cognitive verdict. Fingerprint/checkpoint semantics were corrected without changing P3 world, action policy, budgets or thresholds. |
| P3 | PREFLIGHT-2 | TECHNICAL_FAIL_OVERSTRICT_RETENTION_OBSERVER | Workflow 37527565396 at source `bc955c4db07fd2ba6cb1fbd7cfe2b52648d8d933`: autonomous acquisition, restart and changed-law repair executed; final observer rejected decoder drift 3.4711426e-5 against a bit-identity <1e-6 criterion not required by the preregistered capability contract. Observer was changed to test the correct factual successor; production cognition, budgets and PASS thresholds were unchanged. |
| P3 | PREFLIGHT-3 | PASS_AUTONOMOUS_PHASE_NATIVE_ACQUISITION | Workflow 37528040155 at source `b2dade26db1a7d83a5c16f05fe2c70f39d27a33c`: 12/12 cold hidden reset-chain worlds acquired autonomously, DIRECT_ONLY 0/12, seeded RANDOM 4/12, mean acquisition 24.667 actions, mean changed-law repair 9.083. Checkpoint into a newly constructed EvoPhase, frozen held-out exploitation, detour repair, source guard, regressions and Release build PASS. |
| P3 | FRESH-1 | PASS_FRESH_AUTONOMOUS_ACQUISITION_AND_REVISION | First one-use authority run 37528857872 at source `52bbbc2654894bbaf8501e6834757dff25f1e242`, spec `b561a3af878c89531ed8df27b338959fd4d868f5`, pack `c9a3d25d6f64c483`: FULL 80/80, Wilson95 [0.954182,1.000000], every seed 8/8; frozen exploitation 80/80; checkpoint-restored exploitation 80/80; DIRECT_ONLY 9/80; RANDOM 35/80; controls [0,0,9,2]/80. Changed-law repair 78/80, per-seed [8,7,8,8,8,7,8,8,8,8], revised exploitation/restore 78/78, frozen stale 0/80. Mean acquisition 20.075, mean repair 16.462, legacy graph table 0. All frozen thresholds PASS; pack permanently burned. |
| P4 | PREFLIGHT-1 | PASS_LEARNED_PHASE_NATIVE_EXPLORATION_DRIVE | Workflow 37530869041 at source `9aa725a443ac7f975896c4e20f689cef90dae334`: learned drive weights [0.99999994,1.0], LEARNED_DRIVE 12/12 mean 25.000, ZERO_DRIVE 0/12, FRONTIER_WEIGHT_LESION 0/12 mean 60.000, ZERO_PHASE_DRIVE 0/12 mean 60.000, RANDOM 6/12 mean 47.917, P3 ceiling 12/12. Source guard, regressions and Release PASS. |
| P4 | FRESH-TECH-1 | TECHNICAL_FAIL_WORKFLOW_PARSE | Run 37531623310 created zero jobs because the first trigger expression was invalid. The evaluator never executed and no authority-derived world pack was generated/printed; no cognitive verdict. |
| P4 | FRESH-1 | PASS_FRESH_LEARNED_DRIVE_TRANSFER | First valid one-use authority run 37531676452 at source `3795f3865f90dbff1942ffbc7d450aba24b2b66d`, spec `2069b8269944cb65ff5967e9580bd3c152640fdb`, pack `47b2a86ebe50dc8f`: FULL 80/80, Wilson95 [0.954182,1.000000], all seeds 8/8; ZERO_DRIVE 0/80; FRONTIER_LESION 7/80; ZERO_PHASE_DRIVE 0/80; RANDOM 18/80; P3 ceiling 80/80. Mean FULL cost 24.087 vs lesion 55.700, zero-phase 60.000, random 52.062. Source meta-tuition 646 interactions. Target drive frozen; target carriers started with zero receptors/circuits/legacy graph transitions. Full regressions and Release PASS; pack permanently burned. |
| P5 | PREFLIGHT-1 | FAIL_CONTINUAL_FORWARD_READOUT | Workflow 37532474106: after learning B, persistent organism could no longer execute previously learned A. A test-only source-guard self-match was also found and fixed separately. No fresh pack consumed. |
| P5 | DIAG-REPRESENTATION | FAIL_NOT_ALIASING | Workflow 37533268179: max A/B carrier-trace similarity 0.077763 at threshold 0.97; A still failed after B. Representation aliasing ruled out. |
| P5 | DIAG-FORWARD | FAIL_FORWARD_ONLY | Workflow 37533755925: retained A P1 plan still selected correct motor 1 at value 0.9025, but ordinary P2 action+prediction path abstained. |
| P5 | DIAG-SYNAPSE | FAIL_NO_OLD_PARAMETER_OVERWRITE | Workflow 37534252495: 0/606 tracked pre-existing A/drive/decoder synapses changed after B; A forward readout still failed. |
| P5 | DIAG-COMPONENT | FAIL_SHARED_CELL_REFERENCE | Workflow 37534474528: zeroing newly added B synapses did not recover A forward readout, localizing failure to shared cell substrate state. |
| P5 | PREFLIGHT-2 | PASS_PERSISTENT_CONTINUAL_RETENTION_AND_REVISION | Workflow 37534699260 at source `de2f931b5a6b03995f62f8723011cfca89581be0` after production fix `797e366d64a186bc7c36d283f84fff3dfe57e8f7`: persistent acquisition costs [9,24,12,20], retention actions 34, changed-B repair 9, receptors [4,9,13,18], circuits [7,19,27,38], ZERO_DRIVE 0/4, NO_GROWTH 0/4, whole-lifetime checkpoint restore PASS, full regressions and Release PASS. |
| P5 | FRESH-1 | PASS_FRESH_CONTINUAL_RETENTION | Run 37569003124 at source `d7e33bb53cf63b1e21a96463bc1eea5b728f45e7`, spec `a2bc2717f94e3840576015265116a4c7b646234b`, pack `f7d409553cf1de82`: primary 80/80, Wilson95 [0.954182,1.000000], every sub-seed 8/8; acquisition 40/40, intermediate revisits 100/100, pre/post retention 40/40 + 40/40, repair 10/10, checkpoint 40/40; FROZEN_CHANGED 0/10, RESET_BETWEEN earlier retention 0/30, ZERO_DRIVE 1/40, NO_GROWTH 0/40. Mean acquisition 14.875, repair 9.400. Full regressions and Release PASS; pack burned. |


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


## What G8 mechanism PASS establishes

Within the preregistered delayed-reward development witness:

- local factual transitions are acquired before planning;
- alternative future states are expanded inside EvoPhase-owned IMAGINED state;
- the planner rejects an immediate reward of 0.30 in favor of a three-step route whose discounted predicted value is 0.9025;
- the selected first action follows opaque motor permutation rather than a fixed action ID;
- depth-1 planning chooses the immediate trap and fails all 8 worlds;
- consistently shuffled learned successor bindings fail all 8 worlds;
- pure imagination does not mutate REAL factual state;
- factual execution of the selected route succeeds 8/8.

This is a bounded mechanism PASS, not yet fresh statistical qualification or unrestricted planning.


## G8-FRESH-1 one-use result — FAIL_DEPTH_WITNESS

Run `37515610684` opened and permanently burned the first G8 fresh pack.

- source SHA: `750a09cdca5afb7927a99e954498262131ab8985`;
- authority seed: `37515610684`;
- protocol fingerprint (FNV64): `392f3182f21e02f1`;
- pack digest: `5537015d542c2d4b`;
- sealed size: N=80 / 10 sub-seeds;
- world 0 (depth 2, clean): FULL succeeded; DEPTH1/IMMEDIATE/SHUFFLED failed;
- world 1 (depth 3, clean): FULL succeeded; DEPTH1/IMMEDIATE/SHUFFLED failed;
- world 2 (depth 4, clean): the qualification stopped because FULL selected the delayed-route motor without a first-decision depth witness reaching the evaluator's true depth 4.

The pack is a FAIL, not a PASS and not reusable for qualification.

Interpretation: the acquired transition model exposed a shorter high-value imagined path than the evaluator's physical route. The immediate candidate cause is state/transition aliasing under the robust planning trace or an equivalent depth-accounting/model-binding defect. The next design must not tune on this pack; it must define a stronger observability/model-integrity condition and use a new external authority seed.


## G8-FRESH-2 one-use result — PASS_OBSERVABLE_STATE_PLANNING

Run `37516415100` completed PASS and permanently burned the second G8 fresh pack.

Fresh authority:
- source SHA: `9770b6d4e7b1b6f6721f8b33290d81120fd61678`;
- authority seed: `37516415100`;
- protocol fingerprint (FNV64): `735d8c35a5af44d2`;
- pack digest: `270c9476d128b82a`;
- N=80 / 10 sub-seeds;
- 46 candidate state sets were rejected by the preregistered representation-only observability seal before the pack was sealed.

Observed:
- FULL_IMAGINATION: 80/80 = 100%;
- Wilson 95% CI: [0.9542, 1.0000];
- DEPTH1: 0/80;
- NO_IMAGINATION / immediate MODEL: 0/80;
- SHUFFLED_MODEL: 0/80;
- model-integrity/depth violations: 0;
- authority / REAL-firewall violations: 0;
- per-sub-seed FULL: [8,8,8,8,8,8,8,8,8,8];
- route depth 2: 30/30;
- route depth 3: 29/29;
- route depth 4: 21/21;
- clean: 40/40;
- distractor: 10/10;
- dropout: 10/10;
- rotation: 10/10;
- scale: 10/10;
- successful delayed-route first motors covered opaque IDs {0,2,3};
- mean FULL physical actions: 2.888;
- mean FULL rollout nodes: 23.700.

This statistically qualifies the bounded claim of delayed-reward carrier-owned planning **conditional on an identifiable carrier observation state and an acquired local transition model**. It does not qualify planning under perceptual aliasing/POMDP uncertainty. G8-FRESH-1 remains the preserved evidence that this observability boundary matters.


## What G9 mechanism PASS establishes

Workflow `37517251841` completed PASS on source `20c2599b6a5c0a68b44eed9537df443aa9c80e9c`.

Development witness:
- four opaque motor permutations;
- two hidden contexts per permutation;
- held-out absolute translations;
- the current corridor raw observation is context-independent;
- FULL_BELIEF: 8/8;
- OBSERVATION_ONLY: 4/8;
- RESET_HISTORY: 1/8;
- successful terminal motor IDs covered {0,1,2,3};
- the recurrent belief traces for the two hidden histories remain below the carrier match threshold despite the identical current observation;
- G0-G8 regressions, ownership audits and optimized release build remained PASS.

Mechanism:
```text
belief_0 = encode(current observation)
belief_t+1 = permute(belief_t) ⊗ action_role ⊗ encode(next observation)
```

The dimension permutation makes the recurrent HDC/FHRR state order-sensitive. The evaluator does not pass latent-context IDs to production cognition.

This is a bounded mechanism witness for history-conditioned carrier state under deliberate observation aliasing. Fresh statistical qualification remains required.


## G9-FRESH one-use result — PASS_HISTORY_CONDITIONED_BELIEF

Run `37517982423` completed PASS and permanently burned the G9 fresh pack.

Fresh authority:
- source SHA: `b218668436b5baf29f2e3a479991c233a255bc4a`;
- authority seed: `37517982423`;
- protocol fingerprint (FNV64): `cd76f953e5a49b2c`;
- pack digest: `bc2608fd9a60e0df`;
- N=80 / 10 sub-seeds;
- representation-only pre-seal rejects: 3.

Observed:
- FULL_BELIEF: 80/80 = 100%;
- Wilson 95% CI: [0.9542, 1.0000];
- OBSERVATION_ONLY: 20/80 = 25%;
- RESET_HISTORY: 2/80 = 2.5%;
- belief-separation violations: 0;
- authority/REAL-firewall violations: 0;
- per-sub-seed FULL: [8,8,8,8,8,8,8,8,8,8];
- history length 1: 30/30;
- history length 2: 30/30;
- history length 3: 20/20;
- successful terminal motor IDs: {0,1,2,3};
- mean physical actions: FULL 2.875, OBSERVATION_ONLY 2.000, RESET_HISTORY 1.200.

The lower physical-action counts of controls reflect early failure, not greater efficiency.

This statistically qualifies the bounded claim that recurrent carrier history resolves deliberate current-observation aliasing across fresh randomized episodes and history lengths up to three. It does not establish arbitrary POMDP solving or learned memory architecture.


## P4 learned exploration result

P4 changes the architectural interpretation of autonomous exploration in the tested reset-chain family.

P3 proved that a cold organism could gather its own transition experience, but the behavioral value of unknown/reachable-frontier states was still hard-coded. P4 begins the drive weights at zero and learns them only from factual structural gain in AETERNA's own P2 model.

The fresh P4 result shows that this learned drive can be transferred **without** source-world receptors or transition circuits into new cold carriers and still solve 80/80 longer target worlds. Removing the learned frontier synapse drops success to 7/80; disabling drive phase learning drops it to 0/80; a zero drive scores 0/80; seeded random scores 18/80.

This qualifies learned weighting of inherited epistemic features, not invention of the feature vocabulary. Continual learning under interference, stochastic/POMDP exploration, arbitrary concepts/domains and AGI remain open.


## P5 continual-retention diagnosis and mechanism result

The first P5 failure was not erased. The old A world model remained physically present, but long-lived P2 decoder coherence was lost because factual observations permanently drifted shared sensory-cell phases. The phase-native fix stabilizes that intrinsic reference while leaving adaptation in synapses.

After the fix, one persistent organism acquired four distinct worlds, retained earlier worlds after each later acquisition, selectively repaired a changed B law while preserving A/C/D, and survived one full checkpoint/restart. This remains a deterministic mechanism witness pending FRESH-P5.

| G10 | MECHANISM-1 | PASS_AUTONOMOUS_COMPOSITE_CONCEPT | Workflow 37569651485 at source `74b736986870419ea42989c387026a57e498d900`: FULL 8/8, NO_CONSTRUCTION 4/8, NO_READOUT 4/8 across both opaque motor permutations. Four acquired atom units were identical in matched arms; four composites referenced child atom IDs; child evidence stayed <=0.20 and composite evidence >=0.60. Regressions and Release PASS. |

| G10 | FRESH-1 | PASS_FRESH_COMPOSITE_CONCEPT | Run 37570124870 at source `4bc42dfc670689842619661d0e13ea54b3689048`, spec `abfc17b088cdd133eaa683c27ef8b9cfbac43180`, pack `7b41074ef2926446`: FULL 80/80, Wilson95 [0.954182,1.000000], every seed 8/8; NO_CONSTRUCTION 40/80, NO_READOUT 40/80; atom mismatches 0, child-reference violations 0, max child evidence 0.0, min composite evidence 1.0, motor mappings 5/5. Regressions and Release PASS; pack burned. |

| G10 | PHASE-AUDIT-1 | NEGATIVE_PHASE_DEPENDENCY_WITNESS | Workflow 37570662521 at source `05f47e75a75a9f1be38aa5a6563e461530698d52`: standalone EvoConceptMemory 8/8 and zero-physical EvoPhase 8/8. G10 behavior remains qualified, but physical phase-cell/synapse execution was not required. Full regressions and Release PASS. |

| G10-PHYS | MECHANISM-1 | PASS_PHASE_NATIVE_COMPOSITE_EXECUTION | Workflow 37571372722 at source `9accdaf95f7df67f1a32aa9c704e96a54a86dd03`: intact 8/8, necessary lesion 6/8, pi phase shift 6/8, exact restore 8/8, unrelated target 2/2; ZERO_PHASE/ZERO_WEIGHT/NO_CAPACITY/NO_GROWTH each 0/8. Dedicated table composites/readout unused in physical path. Full regressions and Release PASS. |

| G10-PHYS | FRESH-1 | PASS_FRESH_PHASE_NATIVE_COMPOSITE | Run 37571774016 at source `7dd0985588abced43e26905330e5d12c95ff3122`, spec `4a278e6245816b09b2468efe6c8bc2c3450cf537`, pack `1b97f883da445664`: FULL 80/80, Wilson95 [0.954182,1.000000], every seed 8/8; ZERO_PHASE/ZERO_WEIGHT/NO_CAPACITY/NO_GROWTH all 0/80; lesion 0/20, pi phase shift 0/20, exact restore 20/20, unrelated 10/10; table violations 0, physical count violations 0. Regressions and Release PASS; pack burned. |

| G11 | PREFLIGHT-1 | FAIL_ORDER_SENSITIVE_LEVEL1_PROMOTION | Workflow 37577194623: G11 source compiled and P5 fast regression passed, but Stage-1 expected 4 promoted physical concepts and observed 0. Diagnosis: supported pair candidates were evaluated only on their own observations; later balancing evidence made children non-predictive but did not re-evaluate old candidates. Source guard passed. No fresh pack consumed. |

| G11 | PREFLIGHT-2 | TECHNICAL_FAIL_NO_GROWTH_ASSERTION | Workflow 37577379726 after order-invariant promotion fix: Stage-1 promotion advanced beyond the first failure, but the shared Stage-2 test helper asserted that every factual observation must return true even for the preregistered NO_GROWTH control. NO_GROWTH is expected to reject new L2 structural allocation; this was an evaluator assertion error, not a cognitive verdict. No fresh pack consumed. |

| G11 | MECHANISM-1 | PASS_RECURSIVE_PHASE_NATIVE_ABSTRACTION | Workflow 37577562551 at source `5f0423ea7f98a74024e42570c00cbc00994b605e`: FULL 8/8, NO_RECURSION 0/8, LEVEL1_ONLY 4/8, lesion 6/8, pi shift 6/8, restore 8/8, unrelated 2/2, lower-level lesion success 0/2, ZERO_PHASE/ZERO_WEIGHT/NO_GROWTH all 0/8, max L1 evidence 0, min L2 evidence 1.0. Regressions and Release PASS. |

| G11 | FRESH-TECH-1 | TECHNICAL_FAIL_TYPE_INFERENCE | Workflow 37578180975 stopped while compiling the fresh evaluator before any `FRESH_G11_SEAL` or authority-derived pack was printed. Rust E0282 could not infer the fixed array type for `target_bindings`; no scientific pack was generated or consumed. |

| G11 | FRESH-1 | PASS_FRESH_RECURSIVE_PHASE_NATIVE_ABSTRACTION | Run 37578375117 at source `2b43c9fa44398c9031f444461345e912e2b8d14a`, spec `aa575097c44c5d56afd778a7aff3cf8bf7594db4`, pack `aa06228ae6032c44`: FULL 80/80, Wilson95 [0.954182,1.000000], every seed 8/8; NO_RECURSION 0/80, LEVEL1_ONLY 40/80, ZERO_PHASE/ZERO_WEIGHT/NO_GROWTH 0/80; lesion 0/20, pi phase 0/20, restore 20/20, unrelated 10/10, lower lesion 0/20; atom/L1/L2/reference violations 0. Regressions and Release PASS; pack burned. |

| G12 | PREFLIGHT-1 | FAIL_INCOMPLETE_SELF_TRIGGERED_PROMOTION | Workflow 37589809697 at source `633ee71a79eecbab0aa891bcd5b89c5c7ed82ff3`: source guard PASS and P5 fast regression PASS, but after the frozen residual regime FULL_AUTO promoted 3 recursive concepts instead of required 4. No G12 fresh pack consumed. Criteria/tuition unchanged; diagnostic follows. |

| G12 | PREFLIGHT-2 | FAIL_EXISTING_CANDIDATE_GATED_BY_CURRENT_CHILD_EVIDENCE | Workflow 37590095994 diagnostic: FULL_AUTO had 4 recursive candidates but only 3 promoted. Candidate supports were 13,9,9,4; the unpromoted candidate already had perfect joint motor evidence [1.0,0.0]. Root cause: after legal weak-evidence recruitment, future coactivations updated the candidate only when the children again passed the current-action weak gate. No fresh pack consumed. |

| G12 | PREFLIGHT-3 | FAIL_POST_UPDATE_RESIDUAL_MASKING | Workflow 37590289973 after persistent-candidate fix: still 3/4 promoted. Diagnostic candidates had supports 14,10,10,4; the late candidate again had perfect joint evidence. Cause: FULL revised child evidence with the current fact before testing sufficiency, so same-class observations could temporarily re-strengthen an already weak child and mask the pre-update residual. No fresh pack consumed. |

| G12 | PREFLIGHT-4 | TECHNICAL_FAIL_UNPREREGISTERED_LEVEL1_ONLY_PERMUTATION_ASSERT | Workflow 37590543635 after pre-update residual fix: FULL_AUTO reached 4/4 recursive promotion in both motor permutations, with first candidate observation >= first weak observation and max child evidence 0.157895. Test then failed only on a per-permutation LEVEL1_ONLY <=2/4 assertion that is not part of the frozen G12 protocol. This evaluator overconstraint is removed from PASS gating and retained as diagnostic. No fresh pack consumed. |

| G12 | MECHANISM-1 | PASS_SELF_TRIGGERED_ABSTRACTION | Workflow 37590743597 at source `62e26c5cbe4934545b8cbb19ab8e9ece04bad04b`: FULL_AUTO 8/8, NO_ESCALATION 0/8, FROZEN_SIMPLE 0/8, ZERO_PHASE/ZERO_WEIGHT 0/8; lesion 6/8, pi shift 6/8, exact restore 8/8, unrelated 2/2, lower lesion success 0/2; FULL pre-residual candidates 0 vs ALWAYS_ESCALATE 4; first candidate observation equals first supported-weak observation; 4/4 L2 promoted. Regressions and Release PASS. |

| G12 | FRESH-1 | PASS_FRESH_SELF_TRIGGERED_ABSTRACTION | Run 37591926021 at source `7452e31d3c04eaf35eb1b655f5ade1ca76c82572`, spec `27126f4b40ef3c612e63c2b1d8c869cf24ccf79c`, pack `a1a0025ed123f00c`: FULL 80/80, Wilson95 [0.954182,1.000000], every seed 8/8; NO_ESCALATION/FROZEN_SIMPLE/ZERO_PHASE/ZERO_WEIGHT 0/80; FULL pre-residual candidates 0 in all seeds vs ALWAYS_ESCALATE 2; final candidates/promoted 4/4; first candidate == first weak in every seed; lesion/phase 0/20, restore 20/20, unrelated 10/10, lower lesion 0/20. Regressions and Release PASS; pack burned. |

| G13 | PREFLIGHT-0 | NONQUALIFYING_PROTOCOL_DRIFT | Workflow 37593455572 was functionally green (FULL 8/8, controls causal) but the frozen protocol specified a 20x20 raw raster while the development evaluator used 40x40 spatial layouts. Production mechanism is unchanged; this run is preserved as non-qualifying evidence and must not count as G13 PASS. No fresh G13 pack consumed. |

| G13 | MECHANISM-1 | PASS_DEPTH_GENERIC_PHASE_NATIVE_ABSTRACTION | Workflow 37603203990 at source `6f0fb657309e16230f0fcc0ac659af57d579b61a`, conforming to frozen 20x20 substrate: FULL 8/8, MAX_LEVEL2/NO_HIGHER_ENGINE/ZERO_PHASE/ZERO_WEIGHT/NO_GROWTH 0/8; L2->L3 lesion 4/8, pi shift 4/8, exact restore 8/8, unrelated 2/2, lower L1->L2 lesion dependent success 0/2; hierarchy 16 atoms -> 8 L1 -> 4 L2 -> 2 L3. Regressions and Release PASS. |

| G13 | FRESH-1 | PASS_FRESH_DEPTH_GENERIC_PHASE_NATIVE_ABSTRACTION | Run 37603829184 at source `0ba1e6ef8081dd5de3151aa903b1dcc230f9a61b`, spec `43471572d08f6db4d18a712b98200affa9068257`, pack `e5d76dc63d2acca4`: FULL 80/80, Wilson95 [0.954182,1.000000], every seed 8/8; MAX_LEVEL2/NO_ENGINE/ZERO_PHASE/ZERO_WEIGHT/NO_GROWTH 0/80; structure/reference violations 0; lesion/phase 0/20, restore 20/20, unrelated 10/10, lower lesion 0/20; all motor roles exercised. Regressions and Release PASS; pack burned. |

| G14 | MECHANISM-1 | PASS_SELF_SELECTED_ABSTRACTION_DEPTH | Workflow 37604831999 at source `29f37f5b9d196533bbae205f8df91385a26d302d`: open safety ceiling 16; SIMPLE 4/4 stopped at L2 with 0 L3 candidates; DEEP 8/8 grew to L3 with 0 L4 candidates; CAP_LEVEL2/NO_ENGINE 0/8; lesion 4/8, pi shift 4/8, restore 8/8, lower lesion 0/2. Regressions and Release PASS. |

| G14 | FRESH-1 | PASS_FRESH_SELF_SELECTED_ABSTRACTION_DEPTH | Run 37605704724 at source `6f015be57cbf81edc6b0b61ee30ebeec78c621c5`, spec `61b55d96b9f3e5de2abdf2f1fb886dfce4520b6e`, pack `dbfb32f48339bb0a`: SIMPLE 40/40 stopped at L2 in every seed with 0 L3 violations; DEEP 80/80, Wilson95 [0.954182,1.000000], grew to L3 in every seed with 0 L4 violations; ceiling 16; CAP_LEVEL2/NO_ENGINE 0/80; lesion/phase 0/20, restore 20/20, lower lesion 0/20. Regressions and Release PASS; pack burned. |

| G15 | TRIGGER-TECH-1 | TECHNICAL_FAIL_WORKFLOW_PARSE | Workflow 37608815206 created no valid cognitive job because the temporary YAML trigger was malformed. G15 source/protocol/test unchanged; no scientific pack consumed. |

| G15 | MECHANISM-1 | PASS_ABSTRACT_PHASE_NATIVE_MODEL_PLANNING | Workflow 37608846717 at source `156421b208b59591d6addffed3f8ce913471b7b6`: FULL 8/8, depth>=3 8/8, DEPTH1 0/8, NO_MODEL 0/8, BROKEN_STATE 0/8, BROKEN_TRANSITION 0/8, PI_PHASE 0/8, exact restore 8/8, irrelevant 2/2; REAL/fingerprint unchanged; regressions and Release PASS. |

| G15 | FRESH-1 | PASS_FRESH_ABSTRACT_MODEL_PLANNING | Run 37609743908 at source `bebd479389bad814093a459c59f60df691442181`, spec `89589201d28b078bb326c41be7f53c7d08ab59a1`, pack `84dffac8bcb36171`: FULL 80/80, Wilson95 [0.954182,1.000000], every seed 8/8; route lengths 2..4; depth1/no-model 0/80; broken state/transition/pi 0/20; restore 20/20; irrelevant 10/10; structure/endpoint/REAL/fingerprint/legacy violations 0; motor mask 0b111111. Regressions and Release PASS; pack burned. |

| G16 | PREFLIGHT-TECH-1 | TECHNICAL_FAIL_MISSING_IMPORT | Workflow 37611423730 stopped during compilation before G16 execution: `PhaseNativeCheckpoint` was extended with `Option<EvoConceptMemory>` but `phase_native.rs` lacked the type import (Rust E0425). No G16 cognitive test or fresh pack ran. |

| G16 | PREFLIGHT-1 | FAIL_EPISODIC_CONTINUATION_ALIAS | Workflow 37611533602: P5 fast regression PASS. FULL transferred drive autonomously reached delayed value 1.0 in 12/12 worlds (ZERO_DRIVE 0/12, FRONTIER_LESION 0/12, DIRECT_ONLY 0/12, RANDOM 1/12; endpoint errors 0; mean 45.667), but frozen/restarted delayed planning was 0/12. Diagnosis: the evaluator encoded terminal 0.55 reset as an ordinary S0->S0 transition, so the physical planner correctly treated 0.55 as repeatable future return and preferred it. This is a real episodic-continuation representation gap; no fresh G16 pack consumed. |

| G16 | PREFLIGHT-2 | FAIL_EFFICIENCY_THRESHOLD | Workflow 37612194226: source guard PASS; FULL autonomously reached delayed reward 12/12, frozen plan 12/12, restart plan 12/12; ZERO_DRIVE/FRONTIER_LESION/DIRECT_ONLY 0/12, RANDOM 1/12, NO_LEARNING/NO_GROWTH plans 0/12, endpoint errors 0. Mean first-delayed-reward cost was 45.667 against frozen <=45.0, so the run is preserved as scientific FAIL despite all capability criteria passing. No fresh G16 pack consumed. |

| G16 | PREFLIGHT-3 | TECHNICAL_FAIL_SOURCE_GUARD_COMMENT_TOKEN | Workflow 37614617047: Human Protection preflight PASS; G16 capability test reached FULL reward/plan/restart 12/12, controls satisfied, endpoint errors 0, and mean cost improved to 32.000 (frozen <=45). The only failure was source guard matching forbidden token `route` inside a non-executable explanatory comment in the selector. Logic/criteria unchanged; no fresh G16 pack consumed. |

| G16 | MECHANISM-1 | PASS_AUTONOMOUS_ABSTRACT_MODEL_ACQUISITION | Workflow 37614779191 at source `2f4bf80edf230778bac4c36080db66757e674a4d`: FULL reward/plan/restart 12/12; ZERO_DRIVE/FRONTIER_LESION/DIRECT_ONLY 0/12; RANDOM 1/12; NO_LEARNING/NO_GROWTH plans 0/12; endpoint errors 0; mean cost 32.000 vs frozen <=45. Source guard, regressions and Release PASS. |

| HUMAN-PROTECTION | MECHANISM-1 | PASS_NON_BYPASSABLE_ACTION_GATE | Workflow 37614779191 at source `2f4bf80edf230778bac4c36080db66757e674a4d`: 3/3 safety tests PASS; hard risk threshold 0.01, confidence minimum 0.90, fail-closed invalid evidence, emergency latch, explicit external reset, deterministic audit, cognitive checkpoint cannot clear latch; regressions and Release PASS. |

| G16 | FRESH-1 | PASS_FRESH_AUTONOMOUS_ABSTRACT_MODEL | Run 37615561023 at source `ba984adbcb8be19828513d045e4aa3fce61dc69c`, spec `adb3bf9259716faf5c3fe097489eca35a98adb22`, pack `ed19e91187055bea`: reward 40/40, held-out plan 80/80, Wilson95 [0.954182,1.000000], restart 40/40, every seed 4/4 acquisition + 8/8 plan, mean cost 31.575 (SD 16.958); ZERO_DRIVE 0/40, FRONTIER_LESION 1/40, DIRECT_ONLY 1/40, RANDOM 2/40; NO_LEARNING/NO_GROWTH plans 0/80; ownership/persistence violations 0; all six motors. Human Protection, regressions and Release PASS; pack burned. |

| HUMAN-PROTECTION | MECHANISM-2 | PASS_SINGLE_USE_ACTUATION_PERMIT | Workflow 37616386983 at source `435c1816c91c40acf53ffc8ea616862cf4d6352b`: 5/5 tests PASS; opaque non-Clone/non-Copy permit, current-sequence validation, one-time consumption, stale-token invalidation, emergency invalidation/latch, fail-closed low-confidence human absence; full regressions and Release PASS. |

| G17 | MECHANISM-1 | PASS_GOAL_CONDITIONED_ABSTRACT_PLANNING | Workflow 37617083481 at source `9f87bc44d71209942a9f35735f68ebad707fd3cb`: FULL 8/8, DEPTH1 0/8, NO_GOAL 0/8, wrong/opposite goal follows other goal 8/8, broken goal/route/pi 0/4 each, restore 8/8, irrelevant 2/2; zero-valued model; Human Protection v1.1, regressions and Release PASS. |

| G17 | FRESH-1 | PASS_FRESH_GOAL_CONDITIONED_PLANNING | Run 37617665614 at source `0fcb57f69eca1316c76fd77dcc55b89536a6e507`, spec `658495e27ca15aa5153d68fc53b8821e50a68308`, pack `f52c9427c40a5c44`: FULL 80/80, Wilson95 [0.954182,1.000000], every seed 8/8; DEPTH1/NO_GOAL 0/80; opposite goal 80/80; broken goal/route/pi 0/20; restore 20/20; irrelevant 10/10; all ownership/mutation violations 0; all six motors. Regressions and Release PASS; pack burned. |

| G18 | MECHANISM-1 | PASS_GOAL_CONDITIONED_ACTIVE_INFORMATION_ACQUISITION | Workflow 37627583024 at source `c60e518c233f7333330922c9e15a8c22a567a1d5`: FULL 8/8, first navigation 8/8, second probe 8/8, post-plan 8/8; GENERAL_FRONTIER 4/8, NO_GOAL 0/8, opposite goal 8/8; broken goal/route/pi 0/8, restore 8/8, irrelevant 8/8, no-learning post-plan 0/8. Source guard, Human Protection, regressions and Release PASS. |

| G18 | FRESH-1 | PASS_FRESH_GOAL_CONDITIONED_ACTIVE_INFORMATION_ACQUISITION | Run 37628961906 at source `dfeaf2d7b7dc93b84d871d5cfef95bdbb30dbb60`, spec `f989f7edf553bb2cef2345843ab9fca258a5e345`, pack `36b1d1f2340622c7`: FULL 80/80, Wilson95 [0.954182,1.000000], every seed 8/8; first 80/80, second 80/80, post-plan 80/80; GENERAL_FRONTIER 40/80, NO_GOAL 0/80, wrong goal 80/80; broken goal/route/pi 0/20, restore 20/20, irrelevant 10/10, no-learning post-plan 0/80; irrelevant unknown acquisition 0; drive/endpoint violations 0; all six motors. Human Protection, regressions and Release PASS; pack burned. |

| G19 | MECHANISM-1 | PASS_PHYSICAL_RIVAL_HYPOTHESIS_DISCRIMINATION | Workflow 37630711406 at source `ce402311f4c733957f8997e8edc23ea6d58569a8`: FULL probe 8/8, rival suppression 8/8, GOOD plan 4/4, DEAD fallback 4/4; WRONG_GOAL 8/8, NOVELTY 0/8, NO_GOAL 4/8; broken goal/relevance/pi 0/8, no-revision DEAD fallback 0/4, rival collapse 8/8, restore 8/8, irrelevant 8/8. Source guard, Human Protection, regressions and Release PASS. |

| G22 | PREFLIGHT-TECH-1 | TECHNICAL_FAIL_PRIVATE_TEST_FIXTURE | Workflow 37644131919 stopped while compiling the G22 evaluator before any G22 cognitive test ran: the new test called private imported helper `train_drive` directly (Rust E0603). Production G22 mechanism/protocol were not scored; no fresh pack existed or was consumed. |

| G22 | MECHANISM-1 | PASS_EVIDENCE_GATED_PERCEPTUAL_VARIABLE | Workflow 37644349129 at source `3f606fd2ee76b647105d4462beb07376cd2ec497`: both motor permutations FULL 64/64 vs inherited memoryless 32/64; useful candidate promoted after 32 future-only anchor facts with logE 18.247769; irrelevant candidate retired after 128 future facts with logE -4.618278; both feature sides 32/32; physical feature-link lesion/pi controls causal, restore/restart passed; legacy/table violations 0. G21/G20/Human Protection/full regressions and Release PASS. No fresh pack consumed. |

| G23 | PREFLIGHT-1 | TECHNICAL_FAIL_UNPREREGISTERED_CANDIDATE_COUNT_ASSERT | Workflow 37652359750 reached the G23 evaluator but stopped at an exact `born.len()==3` assertion that is not in the frozen protocol. The independently varying nuisance descriptor legitimately generated additional syntactically eligible candidates under the preregistered grammar. No G23 cognitive score or promotion threshold was reached. Production mechanism/protocol unchanged; evaluator now identifies required Atom/AND/XOR ASTs explicitly while retaining all nuisance candidates. |

| G23 | PREFLIGHT-2 | TECHNICAL_FAIL_UNPREREGISTERED_EXACT_PROMOTION_COUNT | Workflow 37652587787 produced a valid partial G23 witness: target XOR(A,B) promoted after 34 future-only anchor observations with logE 14.716294 and constituent atom effects 0.233333/0.233333; target Atom(B) and AND(A,B) remained unpromoted. The evaluator then failed only because it asserted exactly 40 evidence observations, while the frozen protocol requires >=32 and permits earlier promotion once all gates cross. Production/protocol unchanged; exact-count assertion removed. |

| G23 | MECHANISM-1 | PASS_EVIDENCE_GATED_COMPOSITIONAL_PERCEPTUAL_FUNCTION | Workflow 37652805915 at source `6425ce20f947f07d57dfbefbf30b59e6020828b3`: target XOR(A,B) promoted after 34 future-only anchor facts with logE 14.716294 and constituent atom effects 0.233333/0.233333; competing Atom(B) and AND(A,B) unpromoted; FULL 64/64 per motor permutation vs inherited 32/64 and single-atom 32/64, every cue combination 16/16. Program-link lesion/pi/restore/unrelated/restart causal controls passed; legacy/table violations 0. G22/G21/G20/Human Protection/full regressions and Release PASS. No fresh pack consumed. |

| G23 | FRESH-1 | FAIL_PREREGISTERED_SINGLE_ATOM_THRESHOLD | Run 37654097035, source `86944a3814e63672a3152e6f57e3de09495865cb`, spec `22519515188e9c2a00933ddd0f638e0d28d21ec9`, burned pack `c81d41f6a27db876`: FULL 80/80, target promotions 10/10 (AND 5/5, XOR 5/5), all structure/evidence/intervention controls PASS, but single-atom comparator 50/80 exceeded frozen <=44/80. Post-failure diagnosis: balanced AND necessarily permits 6/8 single-atom/majority accuracy and balanced XOR 4/8, so the mixed no-composition ceiling is exactly 50/80. Pack remains scientific FAIL; no retroactive threshold change. |

| G23 | FRESH3-TECH-1 | TECHNICAL_FAIL_TEST_NOT_REGISTERED | Workflow 37656843536 passed all pre-seal regressions, but the generated `g23_fresh3_compositional_perceptual_pack` function lacked the `#[test]`/`#[ignore]` attributes, so Cargo executed 0 matching fresh3 tests and printed no `FRESH_G23_3_SEAL`. No authority pack was exposed or consumed; G23 remains unqualified by Fresh3. |

| INTEL-1 | REPAIR1-TECH-1 | TECHNICAL_FAIL_TEST_WRAPPER_NAME_COLLISION | Workflow 37659211207 stopped compiling the Repair-1 witness because evaluator wrapper `base()` collided with the imported historical G19 fixture function `base(drive, swap)` (E0428/E0061/E0308). Repair-1 cognitive source was not scored; historical regressions were not reached. |

| INTEL-1 | REPAIR3-PREFLIGHT-1 | FAIL_HISTORY_WITNESS_BEFORE_PROMOTION_PLUS_EVALUATOR_FILTER_LEAK | Workflow 37664477188: one-fact fanout parent support was exactly 1 and runtime source guard passed, but the all-refiners history witness stopped before any scored junction (no context promotion). The test binary also unintentionally executed the nested historical INTEL verdict because the workflow lacked a name filter. Production fanout remains unchanged pending first-stop diagnosis; no new INTEL authority pack was used. |

| INTEL-1 | REPAIR3-PREFLIGHT-2 | TECHNICAL_FAIL_REPAIR_WITNESS_FIXED_GOAL_STALL | Workflow 37664898556 showed fanout parent_support=1 and no module candidates before failure. The history witness reached factual goal state 7 after three junction visits, then its constant goal caused `ScientificRuntime` to correctly return `GoalReached`, so the evaluator stopped generating episodes. Production fanout was not changed; witness now uses the same generic goal/staging cycling rule as G21. |

| INTEL-1 | REPAIR3-PREFLIGHT-3 | TECHNICAL_FAIL_WRONG_REPAIR2_TEST_TARGET | Workflow 37665078819: Repair-3 fanout witness PASS and Repair-1 PASS. CI then invoked nonexistent target `intel1_repair2_exploitation`; the actual target is `intel1_repair2_frozen_exploitation`. No Repair-2 cognitive regression was executed; production Repair-3 unchanged. |

| INTEL-1 | REPAIR3-PREFLIGHT-4 | TECHNICAL_FAIL_STALE_REFINEMENT_SOURCE_GUARD | Workflow 37665281422: Repair-3 fanout, Repair-1 and Repair-2 witnesses PASS. G21 behavioral regression also PASSed 64/64 in both motor permutations with unchanged promotion/restart evidence, but its source guard still required the old direct runtime call string `observe_phase_native_context_result`. Integration guard is updated to require the generic fanout coordinator plus context/perceptual/compositional sidecars; behavioral thresholds unchanged. |
