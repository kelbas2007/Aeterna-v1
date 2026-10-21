# COVERAGE-MEMORY-1 evidence

The preregistered improvement and retention criteria both failed.
The selected continuing organism remains the PR18 parent, not either trial arm.
See [the result](../../COVERAGE_MEMORY_RESULT.md) and
[advance protocol](../../COVERAGE_MEMORY_PROTOCOL.md).

Source, adapter and criteria were published as commit `f329e74` before collection.
`protocol.json` binds their bytes, the parent and datasets. `lives/` contains two
complete actual lives, `probe/` three frozen evaluations on the same reserved
test rows. Every variant remains one Learner/U1 across all phases and exact
checkpoint restart. The two copies of the archived executable are identical.
Uncompressed executable SHA256:
`24d142caf1228eaeea995ec464dc431160ccc6dc1706d7cb018e724430c51316`.

Reproduce collection from a clean checkout of the registered source:

```bash
python scripts/run_coverage_memory_lifetime.py --binary BINARY --data DATA \
  --parent docs/evidence/first-readout/study/organism_first_readout/one-organism.json.gz \
  --protocol docs/evidence/coverage-memory/protocol.json --output NEW_OUTPUT
```

Read-only environment audit and exact native replay of existing records:

```bash
python scripts/audit_coverage_memory.py --results docs/evidence/coverage-memory \
  --data DATA --output /tmp/coverage-audit.json
python scripts/replay_organism_lifetime.py --results docs/evidence/coverage-memory/lives \
  --variant organism_coverage --output /tmp/NEW_REPLAY_DIRECTORY
python scripts/check_coverage_organism.py --binary target/release/aeterna-v1
```

Replay requires no dataset or new world action and unpacks the frozen Linux
binary automatically. `validation/` contains the six native replay proofs,
original audits, regression logs and actual checkpoint checks. `diagnose_consolidation.py`
is a subsequent read-only diagnostic on the consumed V1 rows, not new behavioral
evidence or a tuned successor qualification.
