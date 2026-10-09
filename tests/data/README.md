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
