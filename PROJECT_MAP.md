# AETERNA-v1 — WHERE EVERYTHING IS (2026-10-08)

## Single source of truth

**Main:** `main` — promoted qualified EvoPhase v1 research runtime, based on frozen successful INTEL-4. Never confuse with the separate mature Codex AETERNA repository.

**Last independently sealed system verdict:** **INTEL-4 PASS**, bounded five-world deterministic synthetic family only. Actions: https://github.com/kelbas2007/Aeterna-v1/actions/runs/37821849040. Frozen production cognition: `c7b5455ba006b297288fa8d16ef6300c8a19ceca`. Source and evaluator unchanged through seal.

**Read these first:**
- [INTEL-4 verified result](docs/INTEL4_RESULT_PASS.md)
- [INTEL-4 preregistered contract](docs/INTEL4_PROTOCOL.md)
- [Current research status](STATUS.md)
- [Historical experimental ledger](EXPERIMENT_LEDGER.md)
- [Frozen source guarantee](docs/INTEL4_CORE_FREEZE.md)

## Git branches

| Category | Branch | Purpose |
|---|---|---|
| Default | `main` | Current qualified INTEL-4 baseline; prefer this for ordinary reading |
| New work | `research/beyond-intel4` | Tests of structurally novel environments, without changing the frozen INTEL-4 verdict |
| Historical anchor | `archive/evidence-20261008` | Git octopus anchor holding **all 13 pre-cleanup branch heads**, including negative/burned experiments and former main |
| Qualified immutable snapshot | `intel4-frozen-unified` | Original INTEL-4 frozen authority run and its result |
| Temporary integration | `integration/intel4-main` | Source-compatible merge of former main ancestry with qualified INTEL-4; may be deleted after promotion |
| Existing PR head | `g1/raw-raster-distinction` | Open PR #3: keep until reviewed |
| Superseded experimental heads | remaining `post-intel2-*`, `intel2-r1-*`, `unified-*`, `developmental-*`, `genesis/*` | Archive anchored; not current production and should not be used as starting points |

All legacy heads are recorded with their exact SHA and preserved as reachable Git commit ancestors in [the archival manifest](https://github.com/kelbas2007/Aeterna-v1/blob/archive/evidence-20261008/ARCHIVE_MANIFEST.md). Do not delete negative reports or reclassify burned seeds as success. Previous main at `d13585da3af57ca71a4d863636e0b14cb7d83471` remains reachable in qualified integration history and the archive.

## Reconciliation method

The previous `main` and `intel4-frozen-unified` had diverged. The integration merge uses **two parents**, one from each history, while explicitly choosing the complete **qualified INTEL-4 tree** as the working content. This is deliberate; blindly merging conflicting Rust logic from failed experiments would compromise the frozen qualification. Historical unique files from the former main can be reviewed on the archival branch at the pinned original main SHA.

No genetic/source changes were made during consolidation. Documentation changes after qualification are not new scientific evidence.

## Workflow safety

- Never edit the historical frozen `INTEL4_RESULT_PASS.md` as if rerunning a result.
- New tests must have a preregistered contract, genuinely different laws, fresh seed, and original outcomes.
- A test on the old A–E generators is **diagnostic**, not proof of structural transfer.
- No automatic PR merges from archival experiments.
- Human Protection software gate is not equivalent to real-world physical safety engineering.

## GitHub GUI cleanup when branch deletion is authorized

The connected GitHub API currently permits creating/updating refs, but not deleting them. Once `main` is confirmed and archive branch reviewed, go to repository → **Branches**, and delete only the branches marked superseded above. Do not delete `main`, `archive/evidence-20261008`, `research/beyond-intel4`, or PR #3 head before review. The branch list may remain long until that last GUI-only step.
