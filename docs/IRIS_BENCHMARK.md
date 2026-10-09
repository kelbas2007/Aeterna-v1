# Iris: recorded real measurements — open development benchmark

This broadens development validation beyond the repository's synthetic world
generators. It is a small recorded classification experiment, not a live device
or a sealed intelligence qualification. Prior scientific failures remain intact.

## Data and fixed task contract

The unmodified [fixture and provenance](https://github.com/kelbas2007/Aeterna-v1/blob/research/beyond-intel4/tests/data/README.md) contain 150 Iris
plants with four measured lengths/widths. CI verifies the upstream CSV's SHA-256.
Normalization is the fixed mapping `0.15 + centimetres / 12`, with no parameters
fitted on held-out data. The evaluator adds deterministic sensor error ±0.00015;
the learner declares measurement radius 0.00025.

There are three binary tasks: setosa/versicolor, setosa/virginica and
versicolor/virginica. For each species, within-species row index modulo 5 in
{0,1} is held out: 60 training and 40 held-out records per pair. Across pairs,
120 binary episodes reuse 60 distinct held-out plants. No independent random
authority seed or unseen dataset is claimed.

Each cold organism gets five permuted numeric channels and three opaque,
permuted motors. Four channels contain raw measurements; the fifth is a
factual outcome. Motor roles are evaluator-private: measure, choose class A,
choose class B. Correct/incorrect terminal choices produce outcomes 0.75/0.25;
the organism receives goal 0.75 but never receives a species label or motor role.
The neutral outcome varies independently, avoiding aliases between a constant
law and copying a fixed neutral input.

Training presents 180 records in one persistent lifetime per pair, with at most
three actions per record. The organism chooses the actions; the evaluator ends
a record at its first terminal choice. The initial 32 factual exposures per
motor use the documented generic exploration quota. No hand-labelled curriculum
or feature selector is installed in the production runtime.

Frozen tests hide all four measured features initially. Each task permits at
most **two actual actions and one terminal choice**. Trying both class answers
until one succeeds is impossible. A sensor action can reveal features before
the commitment. Every claimed success requires the actual measured goal.

## Matched control and observed results

A non-phase decision stump receives exactly the same acquired PRE/POST tuples
and opaque action IDs, with the same 64-example bound per action. It identifies
the identity measurement action from factual preservation and fits one feature
threshold for terminal success. Its evaluation uses the same records, hidden
initial frame, sensor noise, outcome goal and two-action budget. It chooses its
own evaluation actions. Acquisition facts originate from the phase learner's
policy; this is a matched-experience comparison, not independent acquisition
policies with identical exploration algorithms. The two fitters also retain
examples differently.

| Pair | Actual tuition actions | Learned models | Frozen phase successes | Stump successes | Phase / stump actions | Phase abstentions |
|---|---:|---:|---:|---:|---:|---:|
| setosa / versicolor | 231 | 3 | 40/40 | 34/40 | 80 / 80 | 0 |
| setosa / virginica | 238 | 3 | 35/40 | 40/40 | 75 / 80 | 5 |
| versicolor / virginica | 238 | 3 | 33/40 | 37/40 | 74 / 80 | 6 |
| Total | 707 | 9 | **108/120 (90%)** | **111/120 (92.5%)** | 229 / 240 | 11 |

The phase learner selected the acquired measurement action first in **120/120**
tasks. It made one incorrect terminal choice and abstained in eleven tasks.
The stump made nine incorrect choices. Overall, the phase learner does **not**
beat the simple comparator. Gaps between observed condition branches contribute
to abstention; the harder species overlap also exposes limits of a single
threshold. No held-out-feedback adaptation occurred, and the learned fingerprint
remained unchanged through every frozen task.

Bounded memory: 167/171/172 retained examples, including pending change evidence;
checkpoint sizes 1,361,127 / 1,361,522 / 1,361,790 bytes with the fixed physical
template allocation. These are measured fixture sizes, not an estimate for
arbitrary lifelong knowledge. Restoring the serialized model precedes evaluation.

## Reproduce

```bash
cargo test --locked --release --test iris_grounded -- --nocapture
cargo run --locked --release --example iris_grounded
```

The regression checks the execution/data/budget contract and useful behavior on
the separable species pairs. It reports the overlapping pair's errors and
abstentions instead of imposing an unsupported perfect-score claim. This is
ordinary open development evidence, with a fixed familiar corpus and repeated
training presentations. It neither supersedes TE4/TE5 nor changes INTEL-4's
original five-world scope.
