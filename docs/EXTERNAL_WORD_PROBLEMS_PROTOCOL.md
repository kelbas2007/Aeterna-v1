# EWP-1: externally authored word problems, answer-supervised program grounding

## Frozen evaluation intention (before reading SVAMP outcomes)

Source repository: arkilpatel/SVAMP, revision
`78e727689e1c1bebfc4be39c446898e8e10b0518`.
Training: `data/mawps-asdiv-a_svamp/train.csv` (3138 records actually present).
External examination: **all 1000 SVAMP.json records**. Despite its filename,
`dev.csv` in that directory is NOT used for development or model selection.
The external test remains unavailable while constructing/selecting the model.
No existing solution code or pretrained weights is used.

Development selection used only a grouped 80/20 split of training text:
GroupShuffleSplit seed 2701, 650 held-out development records. Initial trials
compared C in {0.3, 1, 3} and bag versus question/quantity-role features. Chosen:
role C=1; bag C=3. Repeated weak-label selection did not improve the pilot;
the final learner uses one fit. Raw-text parity exposed unmasked quantities
(e.g. `1-year`) and lexical digits (`MP3`); the tokenizer was corrected before
external evaluation. These fixes are not post-SVAMP tuning.

## What is learned, what is authored

Only reconstructed text and numeric teacher answers enter learning. The
provided Equation, group_nums, Body/Ques segmentation and task IDs are not
learner inputs. The public raw tokenizer independently identifies quantities.
A bounded generic arithmetic search finds expressions matching each teacher
answer, preferring minimal expressions; ties use compatible corpus frequency.
This weak supervision can select spurious explanations. It is not a proof that
an expression is the intended meaning.

The language ranker is a small linear statistical model trained from scratch,
with learned word/ngram weights, question context and local quantity roles.
It selects an acquired executable Expr program. This is NOT a pretrained LLM,
NOT a newly self-invented learning algorithm, and NOT evidence of SNN reasoning.
The tokenizer, learner, feature families and arithmetic grammar are authored.
No fixed mapping such as `left -> subtraction` is supplied.

Grammar: at most two operations from Add/Sub/Mul/rational division, at most
three distinct input uses selected from at most six extracted quantities.
Programs contain no side effects, tools, conditionals, code imports or game IDs.
The existing integer VM contract is not relaxed: its Expr trees have a separate
exact rational interpretation for decimal amounts and fractions.

The original SG1 school state, facts, arithmetic functions, world Learner,
controller weights and pending evidence are retained. An explicit teacher
operation opens an educational extension; frozen inference alone cannot
replace or modify the learned interpretation model. The new probabilistic
path is explicitly requested (`/solve` or `--word-problems`); it does not
silently override an SG1 refusal with a guessed answer.

## Examination and controls

The native Rust ScientificRuntime receives only Body + Question as ordinary
text. The Python adapter never supplies answers, equations, a class label,
privileged quantity groups or a problem ID. It first saves every prediction
for every arm, checks unchanged checkpoints, then reads labels for scoring.
Freeze manifest records source revision, binary/model hashes, native parity
checks and retention checks before test download. Scores from SVAMP may not
change weights, preprocessing, thresholds or candidate choices in this run.

Arms: full learned question/role grounding; bag-of-words learner trained on the
same teacher cases; full learner with question removed; arithmetic corpus prior
without words; previous SG1 computation path with all prior knowledge retained.
All unsupported inputs and refusals count wrong in the primary denominator.
Primary metric: numeric answer match, all 1000 cases. Also report conservative
number-normalized text overlap with training, non-overlap results, same-body
pairs with different questions/answers, and a separate higher-margin subset.
Margin >= 1 was chosen on development data; it is NOT a probability or a
promised precision on a new domain. Every answer remains a hypothesis.

Independent verifier recomputes every numerical score and every emitted
arithmetic tree using Python Fraction rather than the Rust arithmetic code.
Where reference numeric literals uniquely identify source quantities, exact
polynomial cross-multiplication checks equivalence to the reference equation.
Ambiguous bindings are reported separately, never turned into successes.
A correct numerical answer is not automatically a correct interpretation.

## Resources and limits

Rust/Cargo 1.99.0; no new Rust dependencies. Python training uses numpy 2.3.5,
scipy 1.17.0, scikit-learn 1.8.0. Inference is CPU-only Rust, offline. Quantized
int16 coefficients are persisted with the existing cognitive checkpoint;
Python/Rust token, feature and score parity is tested using training inputs.

This is an external math-word-problem result, not an ARC-AGI-3 game score,
not a human-age assessment, not autonomous open-ended education, and not AGI.
SVAMP was designed specifically to expose shallow heuristics; the comparison
must retain failures and strong controls rather than claim that a positive
answer count closes general understanding.

References: Patel, Bhattamishra and Goyal, NAACL 2021,
https://aclanthology.org/2021.naacl-main.168/ ;
https://github.com/arkilpatel/SVAMP . Dataset copyright/license and attribution
are retained with the experiment. No large build-output directory is committed.
