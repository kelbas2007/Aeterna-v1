# Iris measurement fixture

`iris.csv` is an unmodified copy of the 150-record Iris dataset distributed
with scikit-learn 1.5.2. Its four columns are measured sepal length, sepal width,
petal length and petal width in centimetres; the fifth is a species index.

Source: <https://raw.githubusercontent.com/scikit-learn/scikit-learn/1.5.2/sklearn/datasets/data/iris.csv>

SHA-256: `f13ffa8fdd56fd8e6c8d16d4081a3fbd3114bcd0aae4256c43205169cd9d1449`.

Original reference: R. A. Fisher (1936), *The use of multiple measurements in
taxonomic problems*, Annals of Eugenics 7(2), 179–188.
Scikit-learn is distributed under the [BSD 3-Clause license](SCIKIT_LEARN_LICENSE). The original Iris
data are also available from the [UCI repository](https://archive.ics.uci.edu/dataset/53/iris).

This fixture is a small recorded measurement task. Species labels remain in
the external evaluator; the runtime sees numeric measurements, goals and the
actual outcome of opaque actions. See the Iris example for the fixed split,
action budget and matched decision-stump control.

# Handwritten digit image fixture

`digits.csv` is the decompressed, unmodified 1797-record image dataset distributed
with scikit-learn 1.5.2. Each row contains 64 intensities in 0..16 for an 8×8
handwritten image, followed by a digit annotation 0..9; there is no header.

Source: <https://raw.githubusercontent.com/scikit-learn/scikit-learn/1.5.2/sklearn/datasets/data/digits.csv.gz>

Compressed SHA-256: `09f66e6debdee2cd2b5ae59e0d6abbb73fc2b0e0185d2e1957e9ebb51e23aa22`.
Decompressed SHA-256: `6ebb3d2fee246a4e99363262ddf8a00a3c41bee6014c373ed9d9216ba7f651b8`.
`scripts/check.sh` verifies the decompressed fixture against `digits.sha256`.

Original data: [UCI Optical Recognition of Handwritten Digits](https://archive.ics.uci.edu/dataset/80/optical+recognition+of+handwritten+digits),
E. Alpaydin and C. Kaynak (1998), DOI 10.24432/C50P49.
The scikit-learn distribution is covered by the included
[BSD 3-Clause license](SCIKIT_LEARN_LICENSE).

The fixed within-class split has 1433 training records and 364 held-out records.
This fixture does not expose writer identities; the evaluation does not claim
a writer-independent split. Annotations stay with the external evaluator.
See the [task protocol](../../docs/VECTOR_PROTOCOL.md) and
[measured result](../../docs/DIGITS_BENCHMARK.md).

# Recorded GunPoint motion signals

`GunPoint_TRAIN.ts` and `GunPoint_TEST.ts` are unmodified files mirrored by aeon:
<https://raw.githubusercontent.com/aeon-toolkit/aeon/main/aeon/datasets/data/GunPoint/GunPoint_TRAIN.ts>
and <https://raw.githubusercontent.com/aeon-toolkit/aeon/main/aeon/datasets/data/GunPoint/GunPoint_TEST.ts>.
The downloaded snapshot is pinned by `gunpoint.sha256`, checked before tests.

Original [UCR archive description](https://www.timeseriesclassification.com/description.php?Dataset=GunPoint):
50 training and 150 test series, 150 samples each, two motion classes recorded
from one female and one male actor. The archived signal tracks the X coordinate
of the right-hand centroid during the two gestures. Files retain the original
description and predefined split. These are recorded movements, not generated
Boolean worlds; no live video or actor-independent evaluation is claimed.
See [the protocol](../../docs/PRIMITIVE_PROTOCOL.md) and
[measured results](../../docs/ACQUIRED_OPERATIONS.md).
