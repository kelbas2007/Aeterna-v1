#!/usr/bin/env python3
"""Verify audited base cognitive source, RSP3 and explicit LC1 extension."""
import hashlib,json
from pathlib import Path
r=Path(__file__).resolve().parents[1]
binding=json.loads((r/'docs/evidence/PROJECT_SOURCE_BINDING.json').read_text())
changes={c['path']:c for c in binding.get('reviewed_boundary_changes',[])}
rsp3=binding.get('reviewed_rsp3_extension')
lc1=binding.get('reviewed_lc1_extension')
autonomous=binding.get('reviewed_autonomous_u1_extension')
sha=lambda b:hashlib.sha256(b).hexdigest()
def git_blob(raw):
 return hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()
added=set()
original_hashes={}
if lc1:
 for path,expected in lc1['added'].items():
  assert sha((r/path).read_bytes())==expected, path+' LC1 file changed'
  added.add(path)
 for path,d in lc1['modified'].items():
  raw=(r/path).read_bytes()
  if autonomous and path in autonomous['modified']:
   assert autonomous['modified'][path]['before_sha256']==d['after_sha256'],path+' unexpected pre-autonomy LC1 revision'
  else:
   assert sha(raw)==d['after_sha256'],path+' unreviewed LC1 source change'
  original_hashes[path]=d['before_sha256']
 if not autonomous:
  assert sha((r/'src/bin/aeterna-unified.rs').read_bytes())==lc1['jsonl_source_after_sha256'],'JSONL LC1 transport changed'
if autonomous:
 assert autonomous['jsonl_before_sha256']==lc1['jsonl_source_after_sha256'],'unexpected pre-autonomy JSONL revision'
 assert sha((r/'src/bin/aeterna-unified.rs').read_bytes())==autonomous['jsonl_after_sha256'],'unreviewed autonomous JSONL mutation'
 for path,d in autonomous['modified'].items():
  raw=(r/path).read_bytes()
  if 'after_git_blob' in d:
   assert git_blob(raw)==d['after_git_blob'],path+' unreviewed autonomous Git source change'
  else:
   assert sha(raw)==d['after_sha256'],path+' unreviewed autonomous source change'
  if path not in original_hashes: original_hashes[path]=d['before_sha256']
if rsp3:
 path=r/rsp3['added_file']
 assert sha(path.read_bytes())==rsp3['added_sha256'],'RSP3 source changed'
 added.add(rsp3['added_file'])
 p=r/rsp3['touched_module']
 raw=p.read_bytes()
 if lc1 and rsp3['touched_module'] in lc1['modified']:
  # Original pre-LC1 module is already hash-pinned against the exact
  # RSP3 post-install module by the LC1 binding.
  assert original_hashes[rsp3['touched_module']]==rsp3['module_after_sha256']
 else:
  assert sha(raw)==rsp3['module_after_sha256'],'RSP3 module changed'
  assert raw.count(rsp3['added_line'].encode())==1,'RSP3 module entry missing'
  original_hashes[rsp3['touched_module']]=sha(raw.replace(rsp3['added_line'].encode(),b''))
 assert sha((r/rsp3['touched_module']).read_bytes().replace(b'\npub mod requirements;\n',b''))==rsp3['module_after_sha256'] if lc1 else True
files=sorted(p for p in (r/'src').rglob('*.rs') if p.relative_to(r).as_posix() not in binding['exclusions'] and p.relative_to(r).as_posix() not in added)
h=hashlib.sha256();count_untouched=0
for p in files:
 name=p.relative_to(r).as_posix();raw=p.read_bytes()
 if name in original_hashes:
  prior_digest=original_hashes[name]
  if rsp3 and name==rsp3['touched_module']:
   assert sha(raw.replace(b'\npub mod requirements;\n',b'').replace(rsp3['added_line'].encode(),b''))==rsp3['module_before_sha256']
   prior_digest=rsp3['module_before_sha256']
 elif name in changes:
  c=changes[name]
  assert sha(raw)==c['after_sha256'],name+' unreviewed boundary change'
  text=raw.decode('utf-8')
  for signature in c['function_signatures']:
   guarded=signature+c['added_guard']
   assert text.count(guarded)==1,name+' guard missing'
   text=text.replace(guarded,signature)
  assert sha(text.encode())==c['before_sha256'],name+' altered outside approved guard'
  prior_digest=c['before_sha256']
 else:
  prior_digest=sha(raw);count_untouched+=1
 h.update(name.encode()+b'\0'+bytes.fromhex(prior_digest))
assert len(files)==binding['bound_cognitive_source_files'],len(files)
assert count_untouched==binding['unchanged_cognitive_source_files'],(count_untouched,binding['unchanged_cognitive_source_files'])
assert h.hexdigest()==binding['aggregate_sha256'],'original core changed unexpectedly'
assert sha((r/'src/construction/search_policy.rs').read_bytes())==binding['search_policy_sha256'],'RSP2 engine changed'
print(json.dumps({'status':'PASS','original_source_files':len(files),'untouched':count_untouched,'lc1_added_files':len(lc1['added']) if lc1 else 0,'original_aggregate':h.hexdigest(),'rsp2_search_policy_unchanged':True}))
