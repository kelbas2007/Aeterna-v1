# Development integration validation — 2026-10-09

The latest section covers the seventh combined uncertainty mode and recorded
Iris measurements. The earlier integration record is retained below. Both are
open development validation, not new sealed intelligence qualifications.

## Combined uncertainty validation — completed locally

| Check | Outcome |
|---|---|
| `cargo check --locked --all-targets` | PASS |
| `bash scripts/check.sh` | **137 passed, 0 failed**, 8 ignored, 93 filtered in 33 test-binary runs; statistical crosscheck, dataset checksum and release builds PASS |
| New interval/continual tests | 9 passed; two-equation modular inverse, noise enclosure, cap/budget widening, factual-only tuition, causal cut/π/restore, sensing/no-loop and atomic checkpoint checks |
| Recorded Iris comparison | 108/120 phase, 111/120 matched stump; phase sensing first 120/120, one wrong commitment and 11 abstentions; [full task contract](IRIS_BENCHMARK.md) |
| One mixed noisy partial lifetime | A → B → A, 768 actual actions, sparse every 17th POST, acquired archives/reactivation; additional 2048 actions kept 29 cells / 420 synapses and bounded evidence |
| `bash scripts/demo.sh` | All **seven** examples learned and resumed in separate processes |
| New `uncertain_learning` demonstration | Restored hidden-context sensing then conditional commitment, actions `[2, 0]`, factual goal, unchanged frozen knowledge |
| Actual previously saved v5 files | Original inverse/adaptive checkpoints resumed successfully using the v6 reader |
| `cargo clippy --locked --lib -- -D clippy::correctness` | PASS; nonfatal style/legacy warnings remain |
| Documentation, shell and diff checks | No missing Markdown targets; `bash -n` and `git diff --check` PASS |

The new writer is v6; numerical tuition still requires actual full PRE/POST.
Examples saved new files in a fresh temporary directory. The common regression
runner now includes `uncertain_learning` and `iris_grounded`; 137 invocations
include shared fixtures and do not represent independent scientific trials.
Consumed authority packs and negative cold scorers were not rerun. Remote checks
for this new publication are separate from the historical run IDs listed below.

## Earlier six-mode integration

This earlier record covers the six opt-in development modes integrated with TE5 upstream
`3e6fe616` and the repository navigation/CI update. The original development
commit is `1c89e96`. It is an open regression record, not a sealed authority
qualification. Scientific verdicts retain their original scope.

## Completed locally

| Check | Outcome |
|---|---|
| `cargo check --locked --all-targets` on research | PASS |
| `bash scripts/check.sh` on research | 127 passed, 0 failed, 8 ignored, 93 filtered in 31 test-binary runs; statistical crosscheck and release bins/examples PASS |
| `bash scripts/demo.sh` on research | All six examples learned and resumed in separate processes |
| Main cognitive-source comparison | Exact match with `c7b5455ba006b297288fa8d16ef6300c8a19ceca` for `src`, `Cargo.toml`, `Cargo.lock` |
| Markdown file links | No missing local targets on research or main |
| New workflow YAML and embedded shell | Parsed and passed shell syntax checks |
| `git diff --check` | PASS in both working trees |

The test runner includes both upstream TE5 physical mechanism tests and the
inverse/adaptive rule tests. Shared fixtures are invoked in several binaries;
127 test invocations are not 127 independent scientific experiments. Existing
compiler warnings remain. Consumed qualification packs and explicitly negative
cold-development scorers were not executed by this runner.

## Separate-process demonstrations

| Example | Observed resumed behavior |
|---|---|
| `online_learning` | 11 stored states, 18 circuits; factual goal reached in 4 actions with frozen knowledge |
| `learned_rules` | 2 acquired rules; new goal in 4 actions, no restored acquisition |
| `partial_observation` | 3 rules and 3 visibility masks; 1 informative action, 3 goal actions, factual goal |
| `expanded_rules` | 4 rules spanning 5 families; frozen goal in 3 actions |
| `inverse_inference` | 2 modular roots retained; separating measurement and factual goal within 2 actions |
| `adaptive_learning` | 2 conditional actions, 2 archived models and 2 reactivations; factual goal in 1 action after restore |

Every resumed example reported unchanged frozen knowledge. New checkpoints were
written to a fresh temporary directory; previously saved knowledge was preserved.
The adaptive learning process also completed its uninterrupted 768-action life.

## Verified on GitHub — 2026-10-09

After adding `api.github.com` to the cloud network policy and applying the
environment configuration, API access works. The published integration commits
were verified against remote branch tips; four maintained branches remain and
the frozen/archive refs retain their original SHAs. No pull requests were open
at the time of the check.

| Published source | GitHub check | Outcome |
|---|---|---|
| Research `3871613cce1bd393451c33dbf3c712ff680637aa` | [Development regressions 37898432849](https://github.com/kelbas2007/Aeterna-v1/actions/runs/37898432849) | SUCCESS |
| Main `6239918043157c06b8257601d43180d38c2cd221` | [Qualified baseline regressions 37898432679](https://github.com/kelbas2007/Aeterna-v1/actions/runs/37898432679) | SUCCESS |
| Main and research | [Main documentation 37898432728](https://github.com/kelbas2007/Aeterna-v1/actions/runs/37898432728), [research documentation 37898432706](https://github.com/kelbas2007/Aeterna-v1/actions/runs/37898432706) | SUCCESS |
| Research `3871613` | [TE3 37898432741](https://github.com/kelbas2007/Aeterna-v1/actions/runs/37898432741), [TE5 physical 37898432731](https://github.com/kelbas2007/Aeterna-v1/actions/runs/37898432731), [TE5 unified 37898432723](https://github.com/kelbas2007/Aeterna-v1/actions/runs/37898432723) | SUCCESS |
| Research `3871613` | [Historical TE1 37898432696](https://github.com/kelbas2007/Aeterna-v1/actions/runs/37898432696) | Physical job SUCCESS; noisy-evidence job FAILURE |

The TE1 noisy test still enforces its original criterion of at least 70/80. Its
original recorded score was 63/80 and remains a scientific FAIL. Historical TE1
qualification is now a manual diagnostic workflow; ordinary regressions continue
to exercise its physical causal mechanism. The tests and thresholds are unchanged.

API run/job outcomes establish remote check completion. Downloading full Actions
logs additionally requires access to `results-receiver.actions.githubusercontent.com`,
which this environment's current network policy does not provide. The repository
metadata-update and Actions-permissions endpoints return `Resource not accessible
by integration` (HTTP 403): the integration lacks the administrative authorization
needed to change those settings. These limits do not prevent reading the checks
above or publishing source/documentation updates through Git.

Historical FRONTIER-1, TE1 noisy and TE4 cold failures remain failures. These
development checks do not establish open-world AGI or real-world safety.
