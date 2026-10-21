# Shared learning life v2: reproducible engineering evidence

The raw-sensory learning loop is implemented. Its behavioral advantage is
**not established**; the real-signal model loses to simple PRE-only forecasting.
See the [result](../../LEARNING_ORGANISM_RESULT.md) and
[computed comparison](comparison.json).

## Provenance and contents

- [identity.json](identity.json) binds the native source revision, all Rust
  source files, Cargo inputs, embedded project state, build flags and parent.
- [runner-source.json](runner-source.json) binds the Python transport and
  evidence tools; [environment.json](environment.json) records their environment.
- [native-aeterna-v1.gz](native-aeterna-v1.gz) is the exact Linux x86_64 executable
  used, compressed without changing its bytes. Rust 1.99.0, release,
  `CARGO_PROFILE_RELEASE_LTO=false`, `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16`.
- [real-signals.jsonl.gz](real-signals.jsonl.gz) contains 64 raw training windows
  of UCI HAR, without class or subject labels. The
  [preparer](../../../scripts/prepare_learning_world_signals.py) verifies the
  [previously pinned raw source](../external5/har-mirror-manifest.json).
  Original archive SHA256:
  `ba235c5da70bc5de792aab7b0207326cb29eaf0941c3a44f1589c2ad023c752e`.
  Dataset: Anguita, Ghio, Oneto, Parra, Reyes-Ortiz,
  *Human Activity Recognition Using Smartphones*, UCI Machine Learning Repository,
  [official source](https://archive.ics.uci.edu/static/public/240/human+activity+recognition+using+smartphones.zip).
  These are recorded measurements, not actions newly performed by AETERNA.
- Each of `real-first`, `real-resumed`, `real-frozen`, `physics` and
  `physics-frozen` contains its initial/final checkpoint, complete native life
  trace, original summary, run log, and byte-identical replay proof.
- [final-regression.log.gz](final-regression.log.gz) is the full local final
  regression. [github-runs.json](github-runs.json) records 13 successful
  source-revision CI workflows. [MANIFEST.json](MANIFEST.json) binds every
  file in this directory except itself.

The primary continuation is `real-first` → `real-resumed` → `physics`.
Both `*-frozen` lives are comparison branches. They do not contribute learned
state to the primary. Pendulum is a third-party simulator. Neither the physics
run nor its comparison is an independent or equal-compute intelligence exam.

## Verify, summarize, and replay

Run from the repository root. Check the complete artifact manifest:

```bash
python - <<'PY'
import hashlib, json
from pathlib import Path
p = Path('docs/evidence/learning-world')
for name, digest in json.loads((p / 'MANIFEST.json').read_text())['sha256'].items():
    assert hashlib.sha256((p / name).read_bytes()).hexdigest() == digest, name
print('Evidence SHA256: PASS')
PY
python scripts/summarize_learning_world_evidence.py \
  --evidence docs/evidence/learning-world --output /tmp/learning-world-comparison.json
```

The summarizer verifies checkpoint continuity and replay proofs and recomputes
the four simple signal baselines using PRE only. The native MSE is copied from
its recorded pre-update report; its checkpoint and full native trace are
independently replayed below. The summaries retain the original run paths
as historical metadata; these paths are not needed for replay.

To reproduce the exact native life on compatible Linux x86_64, unpack and hash
the archived executable, then replay a run into a new output directory:

```bash
python - <<'PY'
import gzip, hashlib, json
from pathlib import Path
p = Path('docs/evidence/learning-world')
b = gzip.decompress((p / 'native-aeterna-v1.gz').read_bytes())
assert hashlib.sha256(b).hexdigest() == json.loads((p / 'identity.json').read_text())['binary_sha256']
out = Path('/tmp/aeterna-learning-v2')
out.write_bytes(b)
out.chmod(0o755)
PY
python scripts/replay_learning_world.py --binary /tmp/aeterna-learning-v2 \
  --results docs/evidence/learning-world/physics --output /tmp/aeterna-physics-replay
```

Repeat with the other four run directories to check all 384 native commits.
Replay does not import Gymnasium, execute another environment, or generate
new evidence. Rebuilding source on another platform checks portability;
it does not promise the same floating-point bytes as this archived executable.

To obtain new experience, use the separate data-ingestion and simulator
commands in the [guide](../../LEARNING_ORGANISM.md). New experience is not a
reproduction of the frozen readout and should be reported separately.
