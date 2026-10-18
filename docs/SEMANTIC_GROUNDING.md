# SG1: acquired computation composition and interpretation monitoring

This is an opt-in extension of the existing `ScientificRuntime` education path,
not a second action owner, a language model, or a claim of human understanding.
It changes how acquired computations are applied, not the external permission
boundary. Existing checkpoints remain loadable; the old mode remains a matched
control. No Rust dependencies are added.

## Mechanism

The old mode grounds the whole utterance against one learned surface pattern.
SG1 derives a grammar from each **validated** scalar Q/A lesson: lexical terminals
remain learned, numerical slots become holes that can themselves contain another
acquired computation. A bounded chart search constructs executable VM trees.
The Python teacher gives examples and scalar answers; it never passes a formula,
parse tree or full composite test sentence to Rust.

Every token must be consumed. Numeric order is retained and the induced function
binds argument roles. Grouping parentheses have built-in syntactic meaning.
Paraphrases, case variants and wrapper phrases are taught; their meanings are not
hardcoded in the Rust parser. This is a learned **restricted grammar**, not the
ability to infer arbitrary unseen words or all human syntax.

The interpreter compares symbolically expanded programs before substituting the
current numbers. Different programs that happen to give the same number remain
ambiguous. Only exact supported algebraic identities are canonicalized; no claim
of deciding arbitrary program equivalence is made. Arithmetic is still i64 with
checked overflow and exact division, not general rational arithmetic.

An incomplete bounded search cannot authorize an answer. Unknown words, missing
arguments, invalid grouping and arithmetic-domain failures are reported instead
of being silently discarded by nearest-pattern matching. In particular, refusing
an unknown negation does **not** mean understanding the semantics of negation.
Conflicting teacher sources retain priority over composition and U1 scoring.

For ambiguous programs the system searches for alternate numeric inputs on which
predictions differ. `understand` returns this distinguishing question. `clarify`
accepts a teacher's scalar answer to that **generated question**, filters the
interpretations and evaluates the original request only if one remains. This is
episodic information, not a mutation of the frozen library. It does not authorize
external actions or assert that the teacher is infallible. If no distinguishing
question is found within the budget, uncertainty remains.

Examples such as `12 минус 6 минус 2` are ambiguous **in this acquired grammar**:
the conventional left-associative school-mathematics rule has not been supplied.
Parentheses or a teacher reply disambiguate. This is not a claim that conventional
arithmetic notation is universally ambiguous.

## Interfaces

`enable_school_composition` is called before freeze. `answer_school` then uses
composed interpretation; `answer_school_surface` disables only this mechanism
while retaining identical lessons and library. `understand_school` exposes
candidate programs, provenance, missing terms, computation work and a proposed
question. `clarify_school` processes a reply without teaching.

JSONL operations: `enable_composition`, `ask`, `ask_surface`, `understand`,
`clarify`. The Windows console also accepts `/why <question>` and requests a
reply when it has generated a distinguishing question. Enter skips clarification.
Old `ask`, `teach`, `facts`, checkpoint and freeze contracts remain available.

The grammar search and arithmetic are programmatic VM computations, not newly
proven SNN reasoning. U1 selects the surviving evidence-bearing proposal; with
only one surviving proposal this experiment is not evidence for a new neural
control advantage. Recursive constructor improvement was not measured here.

## Experiment and controls

`scripts/semantic_experiment.py` replays the 52 original numerical lessons and
all 6,698 stored prior fact records. It adds 50 isolated language lessons (1,600
examples): the 26 formerly unseen wordings plus 24 fragments/wrappers. There are
102 validated bindings, 4,928 numerical teaching examples and 26 functions.
The 6,698 facts include the original numerical Q/A; they must not be double
counted as 6,698 additional independent capabilities or learning examples.

The prior checkpoint is not overwritten. A new state is trained by replay of its
lessons and facts, with additional language teaching; it is not a continuation
of every bit of the earlier controller state. Existing source statements retain
their attribution. `prior.json` identifies the exact imported checkpoint.

After freeze, 512 new composite questions combine known relations at depths 2–5,
128 per depth. Their complete utterances and computation trees were never
training examples. Grouping is explicit, some compositions mix the two taught
languages, and the grammar and operators are deliberately restricted. These are
**authored development tests**, not COGS, an independent benchmark, ARC-AGI-3,
or a representative 12-year-old assessment.

The paired surface control gets the exact same checkpoint, including the added
language lessons. Therefore the difference is not attributed merely to more
words or facts. Another 64 composed tests check argument order; 64 check explicit
grouping. 208 old wording failures, now explicitly taught, are labelled remedial
regression rather than unseen-language generalization. 128 rejection controls
are reported separately; a cold model that refuses everything must not be
credited with solving positive tasks. 32 teacher-assisted clarification cases
are also separate from unassisted solutions.

## Measured local result (Rust/Cargo 1.99.0)

| Test | SG1 | Same knowledge, surface only |
|---|---:|---:|
| Untrained full compositions, depths 2–5 | 512/512 | 0/512 |
| Composed argument-order problems | 64/64 | 0/64 |
| Explicit grouping | 64/64 | 0/64 |
| Previously failed wordings after additional teaching | 208/208 | 208/208 |
| Required-refusal controls | 128/128 | 96/128 |

The 32 surface-control mistakes were answers that discarded an **unlearned**
negation. SG1 refused rather than guessing a positive operation. All 848 solvable
questions are distinct from teaching strings. Refusals on solvable questions
count as incorrect. The 32 clarification cases were answered correctly after
one teacher answer to a generated counterexample, not independently.

`scripts/semantic_verify.py` independently replays VM expressions in Python and
recounts all 976 unassisted records (1,088 returned program answers across arms,
including erroneous surface-control answers). The teacher-assisted 32 are checked
separately. The state is unchanged after examination, counterexample generation,
clarification and separate-process restore.

Acquired checkpoint SHA256:
`7665578b6b843a807f6b960ab6bbfd0112c8264cec76515d21246adf9ad8003f`.

Completed local verification: all compilation targets checked; **197/197**
selected Rust tests passed, including **28 new SG1 tests**. Counts are selected
regressions, not every research qualification in the repository. Frozen external
qualifications were not reopened. GitHub/Windows reproduction is recorded in the
follow-up `SEMANTIC_GROUNDING_RESULTS.md`, not assumed from this local result.

Development negatives retained: a unit-teacher dataset initially correlated the
divisor with the answer, permitting an unintended simpler rule; it was replaced
with varied divisors, without weakening the check. The first experimental driver
stopped on a duplicate generated unary question; duplicate-free generation was
fixed without changing the learner. The independent evaluator initially expected
`Const` rather than the actual VM tag `Lit`; schema handling was corrected, not
scores or predictions. No external examination result was used to fit SG1.

## What remains

The learner still refuses the untaught scientific question about winter and
summer. Broad causal knowledge, open vocabulary, physical quantity typing,
spatial perception, general goals and recursive learning-to-learn are not solved
by this change. The last ARC-AGI-3 failure (0/7) is not replaced with an invented
new success. The established improvement is **compositional use and explicit
monitoring of acquired computations inside the declared language**.

Compositional generalization is an established distinction, not claimed here as
a new scientific invention: Kim & Linzen (2020), COGS,
https://aclanthology.org/2020.emnlp-main.731/ . We did not run or train on COGS.
