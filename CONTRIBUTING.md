# Contributing to Aeterna-v1

Use [the project map](PROJECT_MAP.md) to choose the branch and
[the documentation index](docs/README.md) to find the current APIs and evidence.

## Branches and scope

- `main` publishes the integrated current development source, examples and guides.
- `research/beyond-intel4` receives active development, opt-in modes and ordinary regressions.
- `intel4-frozen-unified` preserves the independent INTEL-4 snapshot.
- `archive/evidence-20261008` preserves earlier branch histories.

Prepare development against the latest main or research branch. Integrate tested
research into `main` by fast-forward when possible, otherwise preserve both
histories with a normal merge. After integration, bring the research branch
forward to that same integration commit before starting new work. Avoid
automatic merges of historical experiments. Keep PASS, FAIL and INVALID records
pinned to their original sources and runs.

## Development checks

Both working branches pin Rust 1.99.0. It requires Python 3 for documentation and
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
`scripts/demo.sh` exercises nine examples and the perception utility through separate learning/restoration
processes and writes checkpoints to a new temporary directory.

Active CI checks the current source on both main and research, including the
two-action and variable-depth physical sensing controls, and checks local
documentation links. The separate archived-baseline workflow explicitly checks
out previous main `6c6bb4d`, verifies its source against INTEL-4's exact cognitive
commit, and runs its ordinary regressions. A green
archived-baseline job says nothing about qualification of the current source.
Neither runner executes consumed authority packs. See the
[integration record](docs/BRANCH_INTEGRATION_20261009.md).

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

Recorded-data pipeline integrity and metric qualification are separate: the acquired-operation engine currently misses both declared usefulness targets. Preserve these FAIL reports; do not infer scientific PASS from ordinary CI success. See [the protocol](docs/PRIMITIVE_PROTOCOL.md).

Completed external-world diagnostic workflows are manual. Ordinary pushes
exercise the native components and Python causal-scorer contracts through
`scripts/check.sh`; they do not rescore consumed external samples. Optional
MiniGrid runs need Python 3.10+ and `minigrid==3.1.0`. The external agent accepts
JSONL frames up to 64 KiB and terminates on oversize input. See the
[latest integration review](docs/GITHUB_REVIEW_20261010.md).

The opt-in context-value head is documented in
[its guide](docs/CONTEXT_VALUE_LEARNING.md). Native regression controls are
part of `scripts/check.sh`; the fixed-source external result is in
[CONTEXT-VALUE-1](docs/CONTEXT_VALUE1_RESULT.md). Its heldout samples are used
and are not automatic regressions. Preserve both the MemoryS7 PASS and
the MemoryS9 transfer FAIL; generic learning math remains authored software.
