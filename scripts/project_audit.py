#!/usr/bin/env python3
"""Verify unchanged RSP2 source plus explicitly reviewed restoration guards."""
import hashlib,json
from pathlib import Path
r=Path(__file__).resolve().parents[1]
binding=json.loads((r/'docs/evidence/PROJECT_SOURCE_BINDING.json').read_text())
changes={c['path']:c for c in binding.get('reviewed_boundary_changes',[])}
extension=binding.get('reviewed_rsp3_extension')
if extension:
    new_path=r/extension['added_file']
    assert hashlib.sha256(new_path.read_bytes()).hexdigest()==extension['added_sha256'],'RSP3 proposed policy source changed'
    mod_path=r/extension['touched_module']
    current=mod_path.read_bytes()
    assert hashlib.sha256(current).hexdigest()==extension['module_after_sha256'],'module integration changed'
    assert current.count(extension['added_line'].encode())==1,'module import changed'
    unpatched=current.replace(extension['added_line'].encode(),b'')
    assert hashlib.sha256(unpatched).hexdigest()==extension['module_before_sha256'],'module contains unreviewed edits'
files=sorted(p for p in (r/'src').rglob('*.rs') if p.relative_to(r).as_posix() not in binding['exclusions'] and (not extension or p.relative_to(r).as_posix()!=extension['added_file']))
h=hashlib.sha256()
for p in files:
 name=p.relative_to(r).as_posix(); raw=p.read_bytes()
 if extension and name==extension['touched_module']:raw=unpatched
 if name in changes:
  c=changes[name]
  assert hashlib.sha256(raw).hexdigest()==c['after_sha256'],name+' reviewed result changed'
  text=raw.decode('utf-8')
  for signature in c['function_signatures']:
   guarded=signature+c['added_guard']
   assert text.count(guarded)==1,name+' reviewed guard missing'
   text=text.replace(guarded,signature)
  raw=text.encode('utf-8')
  assert hashlib.sha256(raw).hexdigest()==c['before_sha256'],name+' contains an unreviewed change'
 h.update(name.encode()+b'\0'+hashlib.sha256(raw).digest())
assert len(files)==binding['bound_cognitive_source_files']
assert len(files)-len(changes)==binding['unchanged_cognitive_source_files']
assert h.hexdigest()==binding['aggregate_sha256'],'original cognitive source differs beyond explicitly reviewed guards'
assert hashlib.sha256((r/'src/construction/search_policy.rs').read_bytes()).hexdigest()==binding['search_policy_sha256']
print(json.dumps({'status':'PASS',**binding}))
