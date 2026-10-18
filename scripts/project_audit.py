#!/usr/bin/env python3
"""Verify that consolidation did not alter cognitive Rust code from merged RSP2."""
import hashlib,json
from pathlib import Path
r=Path(__file__).resolve().parents[1]
binding=json.loads((r/'docs/evidence/PROJECT_SOURCE_BINDING.json').read_text())
files=sorted(p for p in (r/'src').rglob('*.rs') if p.relative_to(r).as_posix() not in binding['exclusions'])
h=hashlib.sha256()
for p in files:h.update(p.relative_to(r).as_posix().encode()+b'\0'+hashlib.sha256(p.read_bytes()).digest())
assert len(files)==binding['unchanged_cognitive_source_files']
assert h.hexdigest()==binding['aggregate_sha256'],'cognitive source differs; update the binding only with an explicit reviewed cognitive change'
assert hashlib.sha256((r/'src/construction/search_policy.rs').read_bytes()).hexdigest()==binding['search_policy_sha256']
print(json.dumps({'status':'PASS',**binding}))
