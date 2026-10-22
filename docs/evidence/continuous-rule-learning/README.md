# Continuous rule learning — negative real-data qualification

The requested useful recursive improvement was **not obtained**. Read the
[result](../../CONTINUOUS_RULE_LEARNING_RESULT.md) and
[frozen protocol](../../CONTINUOUS_RULE_LEARNING_PROTOCOL.md) first.
The complete and fixed-updater organisms produced exactly the same 25,062
evaluation forecasts; the complete run used 0.73% more runtime work and
lost to AR(8). No update program or search-policy generation was admitted.

## Source and data

- [Frozen source/binary identity](frozen-identity.json): source commit
  `c333b0665d5724d5961bbf692895b7b5ad1428db`, Rust 1.99.0, build flags and
  SHA256 of native build inputs. Later commits publish documentation/evidence.
- [Pinned NAB sources](data-sources.json) and [upstream MIT notice](NAB-LICENSE.txt).
  Values and timestamps are from Numenta NAB; no anomaly labels were read.
- `nab-development.jsonl.gz`: 6258 permitted development transitions.
  [Development comparison](development-comparison.json) and
  [earlier development iterations](engineering-prefix-history.json).
- `nab-evaluation.jsonl.gz`: 25,062 subsequent transitions, generated after
  source freeze. [Stream manifest](evaluation-stream-manifest.json).
- [Final comparison](evaluation-comparison.json) and
  [exact equivalence of all complete/fixed forecasts](forecast-equivalence.json).

These are public historical recordings. The native learner receives only the
common physical `SensorFrame`, available historical action and observed reward.
Paths and timestamps are trace provenance, not predictive input. Recorded
playback supplies no choice of the original physical experiments.

## Saved lives

Each directory contains `starting-organism.json.gz`, `one-organism.json.gz`,
`life.jsonl.gz`, and `summary.json`. The trace commits the native forecast
before each actual POST. Parent state is preserved in full, not reconstructed
from a metric summary. [Lifetime manifest](lifetime.json) records hashes,
factual counts, whole-runtime work and the two control branches.

| Continuation | Parent → final factual events | Meaning |
|---|---:|---|
| `development-full` | 17,990 → 24,248 | Repeatedly usable development prefixes; final frozen implementation |
| `development-fixed` | 17,990 → 24,248 | Same observations and fusion; search disabled |
| `evaluation-full` | 24,248 → 49,310 | Single frozen-source qualification |
| `evaluation-fixed` | 24,248 → 49,310 | Matched fixed-updater control |
| `physics` | 49,310 → 49,438 | Same primary organism returns to image+signal simulated control |

The initial parent is the published
[PR21 continuing organism](../learning-world/physics/one-organism.json.gz),
uncompressed SHA256
`9f602b6fa673785fbe0dcf894c62ff964f7f5a29abee4a42cae4913db9db61b2`.
The physics continuation is Gymnasium Pendulum, not a physical-world test;
it supplies no controlled evidence of improved action quality.

`replay-proof.json` in the relevant directories records exact native commit
and final checkpoint equality. Replays read recorded actual results; they
perform zero new physical experiments. Validation logs are under `validation/`.

## Reproduce the scoring or replay

From the repository root, Python 3.12 and NumPy 2.3.5:

```bash
python - <<'PY'
import gzip
from pathlib import Path
out = Path('/tmp/continuous-rule-reproduce')
out.mkdir(exist_ok=True)
root = Path('docs/evidence/continuous-rule-learning')
(out / 'aeterna-v1').write_bytes(gzip.decompress(
    (root / 'native/aeterna-v1-linux-x86_64.gz').read_bytes()))
(out / 'aeterna-v1').chmod(0o755)
(out / 'evaluation.jsonl').write_bytes(gzip.decompress(
    (root / 'nab-evaluation.jsonl.gz').read_bytes()))
PY
python scripts/continuous_rule_analysis.py \
  --events /tmp/continuous-rule-reproduce/evaluation.jsonl \
  --full docs/evidence/continuous-rule-learning/evaluation-full \
  --fixed docs/evidence/continuous-rule-learning/evaluation-fixed \
  --output /tmp/continuous-rule-reproduce/comparison.json
python scripts/replay_learning_world.py \
  --binary /tmp/continuous-rule-reproduce/aeterna-v1 \
  --results docs/evidence/continuous-rule-learning/evaluation-full \
  --output /tmp/continuous-rule-reproduce/replay-full
```

The executable is the evaluated Linux x86_64 build. Cross-platform byte-exact
replay is not established. To repeat acquisition, use
`scripts/learning_world_stream.py` on the decompressed development stream,
starting with the published parent, then continue the corresponding saved
state on the evaluation stream. Enable `--temporal` in the complete arm;
use `--temporal-config` with `{"search":false,"recursive":false}` in the
control. Both arms need `--forecast-log`. This repeats consumed public data
and does not create another independent qualification.

The archive SHA256 list is `SHA256SUMS.txt`. It covers stored bytes;
checkpoint SHA256 in summaries and replay proofs covers uncompressed JSON.
