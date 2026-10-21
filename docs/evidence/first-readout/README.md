# Reproduction and evidence index

See [the result](../../FIRST_READOUT_RESULT.md),
[architecture audit](../../ARCHITECTURE_AUDIT_20261021.md),
[first preregistration](../../FIRST_READOUT_PROTOCOL.md), and
[equal-policy probe registration](../../FROZEN_PROBE_PROTOCOL.md).

`study/` preserves all four full continuing lives, frames/actions/actual POSTs,
intermediate/live/frozen checkpoints, original provenance, registration,
environment audit and comparison. `frozen-probe/` preserves both later complete
frozen evaluations on reserved official test rows. The identical frozen Linux
executable is gzip-packed in each directory; Git stores identical blobs once.

The exact first runtime/adapter registration is commit `4293be9`; the separate
probe registration is `5c463b0`. Use those clean checkouts for exact collection:
registration binds source bytes, including validation scripts. Later validator
fixes are disclosed in [source-identity.json](source-identity.json). The cognitive
Rust sources, compiled manifest and executable did not change after collection.
No result is relabelled as performance of a retuned organism.

Rebuild with Rust 1.99.0, release opt-level=3, LTO=false, codegen-units=16, or
unpack the archived executable into a temporary path and make it executable.
Its SHA256 must be
`b5b8d70bcc708597811c2ee633f5bf3014402da6de069642841a06762b132df4`.

From the repository root, with original Fashion-MNIST / UCI HAR / MNIST data
at `DATA`, the current validators can check the published records directly:

```bash
python scripts/audit_organism_lifetime.py \
  --results docs/evidence/first-readout/study --data DATA --output /tmp/study-audit.json
python scripts/audit_organism_lifetime.py \
  --results docs/evidence/first-readout/frozen-probe --data DATA --output /tmp/probe-audit.json
python scripts/replay_organism_lifetime.py \
  --results docs/evidence/first-readout/study --variant organism_first_readout \
  --output /tmp/NEW-native-replay-directory
python scripts/check_first_readout_organism.py --binary target/release/aeterna-v1
```

Replay requires the frozen executable but no datasets. The last command loads
and exactly round-trips the actual published live learner and frozen reference
without executing an action or downloading anything. Native replays are software
verification of existing records, not new independent performance experiments.

`replay_with_capture.py` adds a read-only checkpoint export before return to
Fashion while retaining all action/commit/final-state assertions.
`diagnose_retention.py` counts coverage loss and approximate negative penalties
on already consumed rows; it does not learn or qualify a new policy.
`run_frozen_probe.py` is the registered subsequent evaluator. All probe models
are frozen and distinct from the one live state intended for continued learning:
[study/organism_first_readout/one-organism.json.gz](study/organism_first_readout/one-organism.json.gz).

The initial audit rejected the intentionally frozen reference because the old
validator expected every saved learner to be live; replay also needed to finish
its last HAR reward episode before the freeze marker. Their errors and fixes
are preserved in validation logs and Git history. Current audits and exact native
replays passed. Original actions, outcomes, success criteria and binary remain
unchanged.

File hashes and byte counts are enumerated in [manifest.json](manifest.json).
