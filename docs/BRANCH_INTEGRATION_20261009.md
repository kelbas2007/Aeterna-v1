# Current-source integration — 2026-10-09

Main now publishes the current development code rather than documentation for
features available only on the research branch. The original qualified source
and complete pre-cleanup history retain their separate immutable references.

## Sources and history

- Previous main: `6c6bb4d597619060449cb0f63d4266b3b283271d`.
- Reviewed research tip: `3248fc09e8346731b5d680ae1a3fc9fb88b3607d` (same
  cognitive source as `28764868d9f22891d0e8e30ce335e8b69fef25cb`).
- Previously published research: `13e2c4677e5bfee68ea706a19ac2b9d74a05dfbb`;
  45 subsequent commits add physical two-action and variable-depth sensing,
  terminal trial accounting and completed-goal discrimination.
- Previous main is already an ancestor of reviewed research: 0 main-only,
  160 research-only commits. Integration can fast-forward main and preserve
  every commit without squashing, rewriting or selecting an older source tree.
- Frozen INTEL-4 branch remains at
  `8f70de4bf391b605be99c4476512de538a4644e2`; qualified cognitive source is
  `c7b5455ba006b297288fa8d16ef6300c8a19ceca`.
- Archive remains at `46c8e64562de7b9e300cb65ddcacc89b4e652d1f`.

After verification, publish the integration commit atomically to main and
research. Future research starts from this common point. Keep all four branch
roles; do not merge the archive or overwrite the scientific snapshot.

## Verification and CI

The current-source runner checks all compilation targets, ordinary and causal
regressions, dataset checksums, documentation links and nine learn/restore
examples plus the perception utility. It now includes both
`structure1_chain_mechanism` and `multistep_physical_path` mechanism suites.

The archived-baseline workflow explicitly checks out the exact INTEL-4 cognitive
commit before its ordinary regression run. Its success concerns that historical
source, not qualification of current main. Consumed authority tests, fresh seed
packs, protocols and trigger files are unchanged and are not rerun.

Local results and the final publication are recorded in
[development validation](DEVELOPMENT_VALIDATION.md). Open transfer diagnostics
and real-data quality criteria retain their original verdicts independently of
software pipeline checks. In particular, the acquired-operation engine's two
recorded-data usefulness gates remain FAIL.

## GitHub audit

The default branch is main. Four remote branches remain. There are no open pull
requests; the obsolete draft #3 is closed without merging and its history is
archived. Two open issues are preserved for separate triage.

The branches API reports no server-side branch protection on these four refs.
Git publication works; repository administration is a separate capability and
must not be reported as configured merely because checks and pushes succeed.

An authorized attempt to preserve merge history in repository merge settings
(`allow_merge_commit=true`, squash/rebase disabled) returned HTTP 403,
`Resource not accessible by integration`. No administrative settings were
changed. Branch protection is therefore not claimed as configured.
