# Multivariate perception: development contract fixed before scoring

This is an open development experiment, not an independent sealed gate. It
extends acquisition from a few scalar rules to real multi-feature observations.
Production cognition must live in EvoPhase; the host supplies raw measurements,
opaque actuator IDs and actual bounded outcomes. Digit labels stay in the
external evaluator. No pretrained weights or supplied class-to-motor map enter
the carrier.

## Fixed primary experiment

- Dataset: unmodified scikit-learn 1.5.2 `digits.csv.gz`, 1797 real handwritten
  8×8 digit images derived from UCI Optical Recognition of Handwritten Digits.
- Compressed SHA-256: `09f66e6debdee2cd2b5ae59e0d6abbb73fc2b0e0185d2e1957e9ebb51e23aa22`;
  decompressed SHA-256: `6ebb3d2fee246a4e99363262ddf8a00a3c41bee6014c373ed9d9216ba7f651b8`.
- Normalize each pixel by the supplied maximum 16; no normalization is learned
  using held-out values. Pixel intensity 0 and 1 must remain distinguishable.
- Within each digit, successive record index modulo 5 equal to 0 is held out;
  the other records train. The split is fixed before score interpretation.
- One cold persistent organism per each of three fixed opaque motor/pixel
  permutations. Training observes actual full image frames; the organism picks
  an action and the evaluator returns the same image plus actual success 1 or
  failure 0. At most 11 attempts per training record, stopping at success. This
  is supervised corrective feedback, not unlabelled autonomous discovery.
- Ten response actions and one image-measurement action are evaluator-private.
  At evaluation, all pixels initially are missing. At most two actions and one
  terminal response are allowed. Frozen models must acquire measurement choice
  from factual exposure; retrying ten digit answers is prohibited in evaluation.
- Generic defaults: 32 prototype slots per opaque motor, three nearest physical
  prototypes, merge RMSE 0.12, prototype learning rate 0.05, rejection RMSE 0.25,
  minimum vote margin 0.05, initial factual exposure quota 32 per motor.
- Comparators see exactly the same successful factual tuples: a running class
  centroid and bounded Euclidean 3-nearest-neighbour memory with 32 examples per
  response. All have the same evaluation images/action budget. A host classifier
  is a diagnostic comparator, not carrier intelligence.
- Report total accuracy including abstentions, actions, measurement choices,
  memory size and confusion matrix. Report comparator ties/losses. The primary
  capability target is at least 80% total held-out correctness per permutation;
  no predeclared superiority over a conventional classifier.

## Required engineering checks

Actual phase-synapse centers and motor links must be necessary: cut/phase-shift/
exact restore controls, no numerical prototype backup, frozen fingerprint
unchanged, no imagined tuition, feedback-based revision, bounded topology and
memory, malformed data/config/checkpoint rejection, restart requiring a new
frame, and the existing single-use actuator permit/emergency-stop boundary.
The old regression runner and prior scientific verdicts remain unchanged in
meaning. Code changes that alter this contract must be documented openly before
interpreting subsequent scores.

## User-file scope

Provide an executable that trains, restores and reads an external grayscale
image, reports an opaque response/abstention and accepts factual correction.
Document the supported file format and resolution. General photographic vision,
new-writer robustness, language understanding and universal intelligence are
not established by this small handwritten-image task.
