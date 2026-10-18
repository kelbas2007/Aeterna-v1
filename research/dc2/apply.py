#!/usr/bin/env python3
"""Apply only the reviewed, hash-pinned Rust patch; never execute payload code."""
import base64
import bz2
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]
FILES = [
    'src/construction/synthesis.rs', 'src/construction/recursive.rs',
    'src/construction/runtime.rs', 'src/construction/mod.rs',
    'src/construction/curriculum.rs',
    'examples/directed_construction_autonomy.rs',
    'tests/directed_construction_autonomy.rs',
]
EXPECTED = 'd062469f5eb5cf2dd3895d7a6d0ed3361d595ba7b0c4b1d6a795b16ad7a56675'

def git(*args, **kw):
    return subprocess.run(['git', *args], cwd=ROOT, check=True, **kw)

def main():
    raw = base64.b64decode((ROOT/'research/dc2/source.patch.b64').read_text().strip(), validate=True)
    patch = bz2.decompress(raw)
    if len(patch) > 100_000 or hashlib.sha256(patch).hexdigest() != EXPECTED:
        raise SystemExit('Payload checksum or size mismatch; no source changed.')
    paths = [line[6:] for line in patch.decode('utf-8').splitlines() if line.startswith('+++ b/')]
    if sorted(paths) != sorted(FILES):
        raise SystemExit('Unexpected patch destinations; no source changed.')
    git('diff', '--exit-code')
    check = subprocess.run(['git','apply','--check','-'],input=patch,cwd=ROOT,capture_output=True)
    if check.returncode == 0:
        git('apply','-', input=patch)
    else:
        reverse = subprocess.run(['git','apply','--reverse','--check','-'],input=patch,cwd=ROOT,capture_output=True)
        if reverse.returncode:
            raise SystemExit(check.stderr.decode('utf-8', errors='replace'))
        print('Exact DC-2 patch already installed.')
    manifest = {p: hashlib.sha256((ROOT/p).read_bytes()).hexdigest() for p in FILES}
    (ROOT/'dc2-source-manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
    print(json.dumps(manifest, indent=2))

if __name__ == '__main__':
    main()
