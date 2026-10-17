# School teaching and assessment: development protocol

This work targets the user's request to acquire knowledge and test it, not a
claim that an implemented agent is a 12-year-old human. No validated human-age
norm is used. Mathematics curricula also include explanations, fractions,
decimals, diagrams and non-routine reasoning; the integer subset here cannot
stand for that curriculum as a whole.

## Implemented input and knowledge paths

`aeterna-unified` accepts JSONL `init`, `teach`, `facts`, `ask`, `school_report`,
`freeze`, `checkpoint` and the prior environment protocol. `teach` accepts only
question/answer/source examples, not an expression, topic ID or solved program.
A generic tokenizer creates integer slots, induction searches the existing VM,
and the last quarter of the teacher lesson validates a frozen candidate.
Failure on that quarter rejects the candidate; those labels are not used to
search an alternative. Repeated teaching of that same lexical pattern is refused.

Accepted expressions become functions in a shared school library. Exact
algebraic identities deduplicate a limited set of commutative expressions;
observational signatures used during search are NOT semantic equivalence.
The new `DivExact` instruction rejects non-integral quotients instead of rounding.
No subject-specific formulas or word-to-operator dictionary are added to the core.

Known lexical bindings are exact. TF-IDF features fitted to teacher patterns can
propose a marked approximate binding for similar phrases, including bounded
numeric-slot alignment. This is not general natural-language understanding.
Facts are source-attributed statements; recall or lexical retrieval is not
reported as scientific inference. Contradictory exact statements cause abstention.
No text is executed as a command or used to grant external permissions.

The school state is stored inside the existing `ScientificRuntime` checkpoint,
not a second independent action controller. Answers use the same U1 readout over
eligible proposals. Agreement/validation gates are outside that choice. Most
questions have only one eligible proposal; this experiment DOES NOT establish
that U1 learning improves the school task. The VM, lexical algorithms and their
budgets remain authored software, not newly invented spike computations.

## Training and examination boundaries

`education_experiment.py` fixes a teaching seed, 26 numeric topics, two surface
forms per topic, 64 examples per form and a separate evaluation seed. Each
lesson is 48 fitting examples + 16 teacher-validation examples. Examples contain
ordinary numeric answers from the teacher; reference equations stay in the
Python teacher/evaluator and are never passed into Rust.

A second path imports only TRAIN questions and correct answer text from the
public `allenai/ai2_arc` dataset. These are remembered examples, not automatically
understood textbooks. This is the AI2 science-question dataset, NOT ARC-AGI-3.
The upstream revision, files, hashes and data licence are recorded. External TEST
files are first fetched after the trained checkpoint is frozen and saved.

The separate-process exam contains 3,328 changed-number questions; 208 new-wording
questions; both ARC science TEST splits without cherry-picking; and an explicitly
SEEN recall check. Questions alone go to the learner. Its free answer is mapped
to multiple-choice text by the grader; choices are not used by this implementation.
This is a stricter, nonstandard diagnostic, NOT an official ARC benchmark score.
Unanswered questions count as incorrect. Exact training-question overlaps in the
external split are counted and separated. No exam feedback reaches training.

Controls keep the same acquired checkpoint but disable expression evaluation,
leaving all teacher examples available to retrieval. A cold control has no teacher
experience. Neither control is a competent general-purpose calculator baseline;
the contrast tests dependence on acquired computation, not superiority over tools.

Examination does not mutate program, vocabulary, factual memory, risk, U1 weights
or checkpoint bytes. `education_verify.py` independently recounts saved answers.
Negative development runs are retained. The synthetic wording set was inspected
during development and is NOT an independent hidden qualification.

## Reproduction

```
cargo test --locked --test education_runtime
cargo build --locked --release --bin aeterna-unified
python -m pip install pyarrow==19.0.1
python scripts/education_experiment.py --binary target/release/aeterna-unified --output school-results --external
python scripts/education_verify.py school-results
```

Offline, with an acquired checkpoint:

```
aeterna-unified.exe --school learned-school.json
```

Example wording after the supplied teaching: `Периметр прямоугольника со сторонами
37 и 58.` Unrecognized questions produce an explicit abstention, not a fabricated
conversation. The trained file includes attributed CC-BY-SA-4.0 data; see its
`DATA_NOTICE.txt`. The code retains the repository licence.

Reference scope:
- https://www.gov.uk/government/publications/national-curriculum-in-england-mathematics-programmes-of-study/national-curriculum-in-england-mathematics-programmes-of-study
- https://www.gov.uk/government/publications/national-curriculum-in-england-science-programmes-of-study/national-curriculum-in-england-science-programmes-of-study
- https://huggingface.co/datasets/allenai/ai2_arc
- https://arxiv.org/abs/1803.05457
