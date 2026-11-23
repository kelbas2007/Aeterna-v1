# Complete autonomous inquiry evidence archive

These binary chunks preserve the **complete** `autonomous-inquiry-evidence.tar.xz`
from [GitHub Actions run 45259082705](https://github.com/kelbas2007/Aeterna-v1/actions/runs/45259082705).
They are ordinary Git blobs, so this copy does not depend on the workflow
artifact's retention period. Each chunk is at most 8 MiB.

The archive includes the factual action traces, acquisition and continuation
checkpoints, process records and scoring evidence for all sixteen worlds.
Splitting changes no archive bytes and performs no additional behavioral run.
The evaluated source commit is `df9257ac2c15be9a843409ad28b25adc5ab3ab86`.

| Preserved object | Bytes | SHA256 |
|---|---:|---|
| Complete tar.xz | 51,156,344 | `29391217a3bcbd48683ded61c99e39a246cc7f047503da2bb85487407918594d` |
| Original downloaded artifact ZIP | 52,069,993 | `d1ebec600fc0967f046e53476fad025c4dc1d14f45bc29749f64cba019db09f8` |

`manifest.json` records the order, byte length, SHA256 and Git blob SHA1 of
every chunk. The ZIP is provenance for the original download; it is not
included again here. The reconstructed tar.xz contains the complete archived
experimental evidence.

## Verify without writing

Python 3 and its standard library are sufficient:

```sh
python3 docs/experiments/evidence/autonomous_inquiry_20261123/archive/reassemble.py --verify-only
```

## Reconstruct

Choose a new output path whose parent directory already exists:

```sh
python3 docs/experiments/evidence/autonomous_inquiry_20261123/archive/reassemble.py --output /tmp/autonomous-inquiry-evidence.tar.xz
```

On Windows, use `python` and a suitable Windows output path. The helper checks
every chunk and the whole archive before creating the destination. It refuses
to overwrite an existing file. It does not extract or execute archive contents.

After successful verification, extract the archive into a chosen evidence
directory with an archive tool. See the experiment protocol and sibling result
documents for the scope and interpretation of the behavioral findings.
