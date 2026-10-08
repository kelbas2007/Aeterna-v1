# Remainder development preflight 1

Date: 2026-10-08
Run: 37785468007
Job: 113338970417
Workflow event source: 3768382e58df0b8bcc6cfc6b8f839e91a8c26358
Checked-out evaluator/source: 624aa1c835440244a32c407b621aa266b8435dd2

Verdict: TECHNICAL PREFLIGHT FAILURE. No remainder cognitive test ran.

The new workflow incorrectly passed --locked before Cargo.lock existed. The repository manifest has no dependencies, but Cargo still needs to generate its package lock file. Both the evaluator unit-test step and Release build stopped before compilation with 'cannot create the lock file ... because --locked was passed'. The actual one-organism diagnostic was skipped. Source and historical-test comparisons passed.

This is a workflow-authoring error, not an EvoPhase failure. It supplies no D/E/retention outcome.

The prospective workflow correction generates the dependency-free lock file offline before invoking the existing --locked commands, records the generated file and its hash as build evidence, and does not commit any Cargo.toml, Cargo.lock, src or evaluator changes. The same evaluator/source snapshot will be tested.

Artifact: 11553712381, ZIP SHA256 5cf0468d1e655db76b1dfd2670303f8c83f1f43bd32d8f5e1d085f5a84afe30b.
