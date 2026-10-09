# Development integration validation — 2026-10-09

This record covers the six opt-in development modes integrated with TE5 upstream
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

## GitHub verification boundary

Git transport is available. The environment's GitHub API requests return 403,
so this record cannot certify remote Actions results, current PR states or
repository settings. Active workflows are configured for ordinary development
regressions, exact-source baseline regressions and documentation links. Their
remote results must be read through a working GitHub API or the GitHub UI.

Historical FRONTIER-1, TE1 noisy and TE4 cold failures remain failures. These
development checks do not establish open-world AGI or real-world safety.
