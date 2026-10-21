# CONSOLIDATED-MEMORY-1 evidence

Retention relative to the parent passes; improvement relative to FIFO fails.
Consolidation learned bounded sufficient statistics but was never selected by
its own first-feedback loss rule. All 1,536 first test answers match FIFO.
The selected continuing state remains the PR18 parent under the registered
combined rule. Both trial continuations remain available without losing data.
See the [result](../../CONSOLIDATED_MEMORY_RESULT.md),
[advance protocol](../../CONSOLIDATED_MEMORY_PROTOCOL.md) and
[all machine conditions](comparison.json).

Source, adapter, binary, criteria and all reserved indices were published as
commit `6212fb7bef9f0d0b4a7a1f0011df687fd0411ce1` before collection.
`protocol.json` binds 170 source files, the parent and official datasets.
`lives/` holds two complete actual lives; `probe/` holds three frozen evaluations.
Every variant remains one Learner/U1 through all its phases and exact restart.
There is no independently initialized learner per task. The two archived
executables are identical; uncompressed SHA256:
`909c2d71485029e48b29ede8de9d48d052580c01de1c80e0c6b591e2f8266dda`.

Reproduce collection from a clean checkout of the registered source:

```bash
python scripts/run_consolidated_memory_lifetime.py --binary BINARY --data DATA \
  --parent docs/evidence/first-readout/study/organism_first_readout/one-organism.json.gz \
  --protocol docs/evidence/consolidated-memory/protocol.json --output NEW_OUTPUT
```

Read-only corpus audit, exact native replay and actual checkpoint inspection:

```bash
python scripts/audit_consolidated_memory.py --results docs/evidence/consolidated-memory \
  --data DATA --output /tmp/consolidated-audit.json
python scripts/replay_organism_lifetime.py --results docs/evidence/consolidated-memory/lives \
  --variant organism_consolidated --output /tmp/NEW_REPLAY_DIRECTORY
python scripts/check_consolidated_organism.py --binary target/release/aeterna-v1
python docs/evidence/consolidated-memory/validation/check_agency.py \
  --results docs/evidence/consolidated-memory --output /tmp/consolidated-agency.json
```

Replay requires no dataset and adds no world actions; it unpacks the frozen
Linux binary automatically. For all five recordings use the two life variants
`organism_consolidated` / `organism_fifo` and the three probe variants with
`organism_entry` added. `validation/` includes the five proofs, the older PR18
replay on this binary, source audits, full local regression log, corpus audit,
actual checkpoint checks and the credit ledger check. The latter distinguishes
3,402 actual multiple-motor choices from 901 one-motor forced actions per life.

The calibration diagnostic is explicitly retrospective on consumed V1 inputs
and was made before this registration; it is not a new whole-agent score.
The [earlier negative coverage study](../coverage-memory/README.md) is preserved
in full with its original source registration and binary. It has not been
replaced by this trial. `manifest.json` hashes every other file in this folder.
