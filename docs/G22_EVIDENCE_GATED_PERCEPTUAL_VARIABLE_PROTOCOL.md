# G22 — EVIDENCE-GATED PERCEPTUAL VARIABLE INVENTION

Status: **PREREGISTERED BEFORE IMPLEMENTATION AND OUTCOME OBSERVATION**

Date: 2026-10-07
Base source: `ea03ecbb2516792bbba726e56b675271bf360517`

## Scientific question

Can one continuing AETERNA detect that its current acquired perceptual abstraction is insufficient, discover a previously unused property of the **current raw observation**, validate that property only on later factual experience, recruit physical feature/state cells for it, and use the new variable for action — without a host-supplied feature label, split command or correct-action table?

G21 refines one state by one-step history. G22 deliberately forbids history as the useful discriminator. The useful distinction must be present in the current raw sensory frame and absent from the inherited abstraction vocabulary.

This is a bounded mechanism experiment, not a claim of arbitrary feature invention, world-first novelty, AGI or broad visual understanding.

## Candidate feature vocabulary

The generic search substrate may propose bounded raw descriptors that were not part of the inherited concept encoder:

1. **subthreshold intensity bins**: nonzero raw values below the inherited binary concept threshold 0.5, quantized into eight fixed bins;
2. **wide pair offsets**: translation-invariant absolute (dx,dy) relations between active >=0.5 pixels with Chebyshev radius greater than the inherited concept local radius and at most 6.

The evaluator does not choose which descriptor is useful. A candidate is selected only from a symmetric difference between two actually observed conflicting raw frames that resolve to the SAME acquired abstract base cell/action but produce different acquired successor cells.

The discovery collision selects a hypothesis; those selecting observations NEVER count as validation evidence.

This authored bounded descriptor family is a major limitation. Passing G22 does not mean AETERNA can invent arbitrary mathematical or semantic features.

## Native representation

For a selected binary perceptual hypothesis:

- store two learned raw feature descriptors in native PhaseNativeState;
- recruit two physical feature cells;
- recruit two physical refined-state cells;
- each refined state receives one learned base-cell input and one learned feature-cell input;
- current raw sensing activates the matching learned feature cell through the generic descriptor encoder;
- ordinary native transition circuits learn consequences FROM the refined state cells.

No host-side feature-to-action table is allowed.

A promoted refined state is usable only when:
- the current raw frame resolves to the inherited base abstraction;
- exactly one learned feature descriptor is present;
- both the base->state and feature->state physical synapses conduct coherently.

## Discovery and future-only validation

Maintain a bounded factual discovery record of:
- base abstract cell;
- opaque action;
- successor abstract cell;
- bounded raw descriptor set.

On the first eligible collision, select at most one candidate for that base state. Allocate at most 16 candidates per lifetime. Retired slots are not reused.

For subsequent actual facts:
- classify the current frame by the two selected descriptors;
- learn input synapses and ordinary native transition circuits from the matching refined state;
- on the selected anchor action, derive binary outcome counts from physical transition supports;
- use the SAME future-only evidence gate and alpha spending assumptions as G21:
  - >=32 future anchor observations;
  - >=8 observations per feature side;
  - >=4 side switches;
  - absolute empirical outcome-rate difference >=0.60;
  - log evidence >= log(16/0.01);
  - no third successor.

Retire at 128 anchor observations or on a third successor.

The statistical guarantee has exactly the same restricted stationary-binary-null assumptions as G21. It does not cover arbitrary drift, dependent sensor failures or unbounded descriptor search.

## Deterministic mechanism witness

Inherited acquisition creates one coarse abstract junction state plus goal/dead/staging states.

The inherited concept encoder uses its existing >=0.5 binary local-relation vocabulary. The two junction variants are therefore the SAME inherited abstract state.

The current raw junction additionally contains exactly one weak marker value:
- variant A: a value in one previously unused subthreshold bin;
- variant B: a value in a different previously unused subthreshold bin.

Marker absolute pixel location changes across learning and scoring, so an exact-position lookup cannot solve the test.

The weak markers are below 0.5 and therefore invisible to the inherited concept relation encoder. No predecessor/history cue predicts the variant.

Opaque action X:
- A -> useful goal successor;
- B -> dead successor.

Opaque action Y:
- A -> dead successor;
- B -> useful goal successor.

The first two conflicting X observations may select the perceptual candidate but do not count toward its evidence.

Future actual observations contain both X and Y facts under both raw variants. No feature label or current variant bit enters cognition.

## Scoring

After promotion and sufficient factual transition support:

- freeze learning;
- score 64 held-out junction decisions per opaque motor permutation;
- marker locations at scoring differ from candidate-discovery locations;
- raw goal is unchanged;
- context/history is cleared before every scored observation.

Total descriptive FULL decisions: 128 across two deterministic variants.

Matched comparator: the SAME acquired model queried through the old inherited/memoryless abstract state on the same raw observation and goal.

## Causal controls

For each learned feature side:
1. zero the necessary feature->refined-state physical synapse;
2. shift only that synapse phase by pi;
3. exact restore without retraining;
4. damage the other feature side;
5. checkpoint/restart and fresh sensing at the junction: unlike G21, the current raw feature must immediately recover the correct refined state without predecessor history.

## Mechanism acceptance

- No perceptual candidate exists before factual collision.
- Discovery facts do not increment validation evidence.
- Useful candidate promotes only after the frozen future-only evidence gate.
- FULL >=60/64 in each motor permutation.
- Old inherited/memoryless comparator <=40/64 in each permutation.
- Both feature sides >=30/32.
- Required feature descriptor is not an inherited active concept atom in the discovery/scoring frames.
- Marker position changes do not change the selected learned feature side.
- Weight lesion and pi phase shift each make the affected refined readout unavailable.
- Exact restoration recovers the correct action and original learned fingerprint.
- Unrelated feature-side lesion preserves the target decision.
- Checkpoint/restart retains the learned descriptor/state/circuits and fresh current sensing alone restores usable discrimination.
- An irrelevant candidate generated from a nonpredictive raw descriptor is rejected/retired or remains unpromoted.
- Legacy graph transition count remains zero; dedicated table composites remain unused.
- Source guard finds no evaluator feature labels, marker positions, outcome/action mapping, host graph/search or exact raw-frame lookup.
- Human Protection, G20, G21 and full ordinary regressions pass.
- Release build passes.

## Fresh qualification boundary

A later fresh protocol must independently randomize:
- inherited abstraction hierarchy;
- raw descriptor type (subthreshold intensity vs wide relation when feasible);
- descriptor identities;
- marker/geometry locations;
- successor/action assignments;
- nuisance raw descriptors;
- scoring translations.

No fresh qualification is consumed by the deterministic mechanism stage.

## Explicit limits

- Descriptor search is bounded and programmer-authored.
- The initial inherited abstraction vocabulary is still trained before the G22 lifetime.
- The test is deterministic and fully observed after raw sensing.
- Feature descriptors are simple low-level sensor properties, not language/semantic concepts.
- This does not establish arbitrary feature synthesis, causal sufficiency, stochastic robustness, open-ended memory reclamation or AGI.
- "Physical" means the shared simulated carrier cells/synapses, not fabricated neuromorphic hardware.
