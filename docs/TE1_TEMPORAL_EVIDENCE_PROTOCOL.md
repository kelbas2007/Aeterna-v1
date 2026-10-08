# TE1 — phase-native temporal evidence accumulation

Status: PREREGISTERED BEFORE IMPLEMENTATION / BEFORE ANY TARGET TRIAL
Date: 2026-10-08
Branch: `research/beyond-intel4`
Frozen qualified INTEL-4 source: `c7b5455ba006b297288fa8d16ef6300c8a19ceca`
Previously burned FRONTIER-1: [FAIL report](FRONTIER1_RESULT_FAIL1.md). Its world/motor/cue map and seed are **never used** as a training or scoring target for TE1.

## Architecture hypothesis

The missing competence is at least two independent components:

1. physical persistence of evidence across several factual noisy observations, even when successive cues disagree;
2. a carrier-owned policy that weighs the value of acquiring further observations against choosing an action.

TE1 qualifies **only the first component** and a read-only epistemic `needs_more` signal. No claim of autonomous sampling or FRONTIER-1 resolution is allowed before an independent TE2 actuation-selection qualification.

## Contract

- An opted-in `PhaseTemporalEvidenceState` belongs to the *same* persistent EvoPhase native carrier and checkpoint, not an external memory table.
- It recruits one carrier hub; the first two distinct recognized *raw* observations are acquired as source cells on demand via the existing previously acquired abstract physical representation.
- Native phase-sensitive cue→hub synapses carry the episode's accumulated evidence strength. Candidate/source identities are physical cell addresses. The host supplies no hidden side, probability, class label, useful motor, or correct answer.
- Each true observation may increment the corresponding coherent native synaptic evidence at most once. Max eight samples per episode. An explicit generic **new episode** command clears transient synaptic evidence but preserves source-cell/synapse identities; checkpoint/restart preserves the ongoing physical belief.
- Readout: compare the two conducting physical evidence paths, abstain on ties/insufficient margin or first unseen rival, expose generic uncertainty as `needs_more`. A third incompatible cue abstains rather than being relabelled.
- External observation/factual POST are the only admissible inputs. Read-only readout must not mutate persistent synapses or observation counts.
- Diagnostic lesions and π-phase shifts of the necessary cue→hub synapse must eliminate/alter a previously chosen result. Restoring the original physical synapse must restore the result without relearning.
- Learning the cue/source *identity* is structural. Carrying transient evidence during frozen held-out sensing is not model-tuition; the test freezes structural growth and validates no new cue IDs are recruited.

## Independent evidence tests

Create fresh synthetic signal sequences from **two arbitrary abstract visual categories** drawn from the generic unmodified 24-class learned representation. Randomize which category is the more frequent underlying signal, cue pixel/layout rendering, sample order, and corruption for each new target series. The physical TE1 accumulator receives only the raw cue sequence and may not see the hidden majority/oracle label.

Compare:
- FULL carrier temporal evidence readout;
- matched LAST_ONLY single current cue;
- zero/π-shift necessary synapse controls;
- checkpoint/restart;
- episode reset that preserves learned cue identity but not the previous episode's evidence.

Score ≥70/80 on a preregistered balanced generated family with 8 independent cues and expected 0.7 per-sample reliability, and FULL better than LAST_ONLY by ≥10/80. This remains a **bounded physical temporal evidence gate** even on PASS, not proof of full noisy latent inference or autonomous choice of sampling motor.

Stop/report without threshold changes if noisy sampling produces fewer than the required margins; no rewrites after observing output. Never rerun burned FRONTIER-1 as if it were independent.
