# Contributing to Aeterna-v1

Use [the project map](PROJECT_MAP.md) to choose the branch and
[the documentation index](docs/README.md) to find the current APIs and evidence.

## Branches and scope

- `main` publishes the qualified INTEL-4 baseline and current project navigation.
- `research/beyond-intel4` receives active development, opt-in modes and ordinary regressions.
- `intel4-frozen-unified` preserves the independent INTEL-4 snapshot.
- `archive/evidence-20261008` preserves earlier branch histories.

Prepare development against the latest research branch. Submit documentation and
navigation changes to `main` without changing its frozen cognitive source. Avoid
automatic merges of historical experiments. Keep PASS, FAIL and INVALID records
pinned to their original sources and runs.

## Development checks

The research branch pins Rust 1.99.0. It requires Python 3 for documentation and
statistical checks and Bash for the runners.

```bash
cargo fetch --locked
cargo check --locked --all-targets
bash scripts/check.sh
bash scripts/demo.sh
git diff --check
```

`scripts/check.sh` selects ordinary and causal regression tests, including TE5.
It excludes one-use authority packs and explicitly negative development scorers.
`scripts/demo.sh` exercises seven prior examples and the perception utility through separate learning/restoration
processes and writes checkpoints to a new temporary directory.

Active CI has three purposes: development regressions on the research branch,
ordinary baseline regressions plus exact frozen-source verification on `main`,
and local documentation-link checks on both. The baseline workflow is pinned to
INTEL-4's cognitive source; manually invoking it on modified research code is
expected to reject that source. It does not execute an authority qualification.

Change only the relevant modules and format touched Rust files. Existing files
have differing historical formatting; a whole-repository reformat would obscure
the evidence changes. Use the locked dependencies for reproducible builds.

## Research evidence

Document the task generator, factual observations, controls and equal action
budgets before interpreting a new capability. Predicted or inferred values
remain imagined until the external environment measures them. Record acquisition,
revision, retention and restoration where the claim requires them.

Development test success is distinct from independent scientific qualification.
Consumed authority seeds remain consumed; a qualification needs its own frozen
source/protocol and fresh first-attempt run. Historical workflows stay available
for provenance and are not an instruction to rerun their qualification packs.

The new rule modes use physical phase parameters with software hypothesis fitting
and bounded search. Describe both parts accurately. Human Protection and factual
execution boundaries remain part of the runtime contract.
