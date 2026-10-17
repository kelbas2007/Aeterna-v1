# REL-1: quantity relations before complete program selection

Base implementation: 177f787d; branch baseline: 662b6222.
This document is committed before any REL-1 external outcome is examined.

## Change being attempted

Replace a fixed whole-program classifier with learned relevance of quantities
and learned ordered pair relations. A bounded structured decoder must construct
a mutually consistent expression from these factors. Primitive arithmetic,
feature extraction, normal-form restrictions and the decoder are authored.
No LLM, pretrained language weights, hand-coded word-to-operation dictionary,
test IDs, gold test equation, or expected test answer may reach the solver.

Unlike EWP-1, REL-1 may learn from the TRAINING equations in the official
MAWPS+ASDiv-A train.csv. This is explicit teacher derivation supervision, NOT
answer-only autonomous discovery. Compare with a whole-program classifier
trained on the same derivations, not only with the old weak-label baseline.
The first local prototype explored C={0.2,1,4} and relevance weight={0.5,1,2}
on a grouped split inside the original training data. This is development,
not an external result. Retain all explored development outcomes.

A factor describes an input's relevance or a pair's least-common-ancestor
operation (+, -, reversed -, *, /, reversed /, or absent). These are computational
relations, not a claim of identified physical causation. Numeric values are
excluded from the learned language features. Exact rational execution retains
arithmetic validity checks and must not be confused with verified semantics.

## Evaluation before publication claims

1. Keep all 1000 original SVAMP examples as an EXPOSED regression set. It was
   already evaluated in EWP-1 and examples have been discussed. An improved
   score is not a new independent qualification.
2. Additional external source reserved before downloading its content:
   chaochun/nlu-asdiv-dataset commit
   883f90a9a65bf00304ba8f37423910fe743abc47.
   Evaluate the complete complement after excluding training-overlapping
   questions by normalized text and number-masked normalized text. Determine
   membership without reading the answer or equation. Keep all remaining
   problems, including unsupported input types, in the denominator. Report
   overlaps and the residual count. This is a public-data complement, not a
   claim of zero semantic overlap or an official leaderboard submission.
3. Freeze source hashes, training recipe, learned weights and decoder settings
   before either external evaluation. Do not adjust the learner to those results.
4. Record full, question-removed, same-derivation whole-program, previous EWP-1
   and previous SG1 results. Count wrong answers and refusals explicitly.
5. Use separate numeric and symbolic diagnostics. Symbolic reference evaluation
   may read gold equations only AFTER solver predictions are saved. Do not
   report available reference programs as solved tasks.
6. Test number substitution invariance and normalized-body groups with differing
   questions/reference programs; report denominator zero rather than passing
   an absent control. These controls are distinct from independent accuracy.
7. Preserve old acquired state and protected runtime boundaries. Verify native
   Rust/Python feature and score parity on training fixtures, frozen-state
   immutability, separate-process restore, and Windows/Linux replay.

The requested general intelligence is a research objective, not a promised
outcome of this correction. No improvement may be invented or negative result
hidden. Structural decomposition has precedents, including Roy & Roth,
Solving General Arithmetic Word Problems (EMNLP 2015), D15-1202. This project
must not claim that using pair factors or constrained expression construction
is itself a new scientific discovery.
