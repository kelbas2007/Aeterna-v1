# GitHub branch cleanup — completed 2026-10-08

**RESULT: SUCCESS.** The one-use GitHub Actions cleanup performed exactly the twelve deletions listed in [BRANCH_CLEANUP_PROTOCOL.md](BRANCH_CLEANUP_PROTOCOL.md) and left four protected branches.

- [GitHub Actions run 37832467225](https://github.com/kelbas2007/Aeterna-v1/actions/runs/37832467225): **SUCCESS**.
- Logged result: `BRANCH_CLEANUP_RESULT deleted=12 skipped=0`.
- All twelve candidate refs matched their individually pinned SHA.
- All eleven pre-integration branch heads were proved reachable from the Git octopus archive `46c8e64562de7b9e300cb65ddcacc89b4e652d1f`. The integration branch was proved reachable from `main`.
- Closed historical draft PR [#3](https://github.com/kelbas2007/Aeterna-v1/pull/3) remains unmerged but accessible in repository history.
- Force-with-lease prevented deletion of any silently updated head.

## Remaining branches (verified after cleanup)
1. `main` — current qualified EvoPhase INTEL-4 baseline and ordinary docs.
2. `intel4-frozen-unified` — exact historical authority PASS evidence.
3. `archive/evidence-20261008` — retained full pre-cleanup branch genealogy.
4. `research/beyond-intel4` — new stochastic / out-of-family research.

No EvoPhase `src/**` or Cargo manifest changed during archive integration or branch deletion; the qualified frozen core remains `c7b5455ba006b297288fa8d16ef6300c8a19ceca`.

To recover an old experiment, locate its exact SHA in [the archive manifest](https://github.com/kelbas2007/Aeterna-v1/blob/archive/evidence-20261008/ARCHIVE_MANIFEST.md) and create a temporary branch at that SHA. Do not restart development from superseded experiments or rewrite earlier FAIL/INVALID results.
