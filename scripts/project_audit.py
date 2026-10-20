#!/usr/bin/env python3
"""Verify audited base cognitive source, RSP3 and explicit LC1 extension."""
import hashlib,json,gzip
from pathlib import Path
r=Path(__file__).resolve().parents[1]
binding=json.loads((r/'docs/evidence/PROJECT_SOURCE_BINDING.json').read_text())
changes={c['path']:c for c in binding.get('reviewed_boundary_changes',[])}
rsp3=binding.get('reviewed_rsp3_extension')
lc1=binding.get('reviewed_lc1_extension')
autonomous=binding.get('reviewed_autonomous_u1_extension')
sha=lambda b:hashlib.sha256(b).hexdigest()
causal=binding.get('implemented_causal_continual_extension')
base_sources={}
if causal:
 packed=(r/causal['base_archive']).read_bytes()
 assert sha(packed)==causal['base_archive_sha256'],'causal base archive changed'
 base_sources=json.loads(gzip.decompress(packed))
 assert set(base_sources)==set(causal['modified']),'incomplete historical source archive'
 for name,d in causal['modified'].items():
  assert sha((r/name).read_bytes())==d['after_sha256'],name+' undeclared causal source mutation'
  assert sha(base_sources[name].encode())==d['before_sha256'],name+' historical source mismatch'
 for name,digest in causal['added'].items():
  assert sha((r/name).read_bytes())==digest,name+' new causal source mismatch'
def source(path):
 name=path.relative_to(r).as_posix()
 return base_sources[name].encode() if name in base_sources else path.read_bytes()
def git_blob(raw):
 return hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()
added=set(causal['added']) if causal else set()
original_hashes={}
if lc1:
 for path,expected in lc1['added'].items():
  assert sha(source(r / path))==expected, path+' LC1 file changed'
  added.add(path)
 for path,d in lc1['modified'].items():
  raw=source(r / path)
  if autonomous and path in autonomous['modified']:
   assert autonomous['modified'][path]['before_sha256']==d['after_sha256'],path+' unexpected pre-autonomy LC1 revision'
  else:
   assert sha(raw)==d['after_sha256'],path+' unreviewed LC1 source change'
  original_hashes[path]=d['before_sha256']
 if not autonomous:
  assert sha(source(r / 'src/bin/aeterna-unified.rs'))==lc1['jsonl_source_after_sha256'],'JSONL LC1 transport changed'
if autonomous:
 assert autonomous['jsonl_before_sha256']==lc1['jsonl_source_after_sha256'],'unexpected pre-autonomy JSONL revision'
 # Current autonomous JSONL edits are bound by their exact Git blob hash.
 # Historical SHA256 binding remains accepted for older PR heads.
 if autonomous.get('jsonl_after_git_blob'):
  assert git_blob(source(r / 'src/bin/aeterna-unified.rs'))==autonomous['jsonl_after_git_blob'],'unreviewed autonomous JSONL mutation'
 else:
  assert sha(source(r / 'src/bin/aeterna-unified.rs'))==autonomous['jsonl_after_sha256'],'unreviewed autonomous JSONL mutation'
 for path,d in autonomous['modified'].items():
  raw=source(r / path)
  if 'after_git_blob' in d:
   assert git_blob(raw)==d['after_git_blob'],path+' unreviewed autonomous Git source change'
  else:
   assert sha(raw)==d['after_sha256'],path+' unreviewed autonomous source change'
  if path not in original_hashes: original_hashes[path]=d['before_sha256']
if rsp3:
 path=r/rsp3['added_file']
 assert sha(source(path))==rsp3['added_sha256'],'RSP3 source changed'
 added.add(rsp3['added_file'])
 p=r/rsp3['touched_module']
 raw=source(p)
 if lc1 and rsp3['touched_module'] in lc1['modified']:
  # Original pre-LC1 module is already hash-pinned against the exact
  # RSP3 post-install module by the LC1 binding.
  assert original_hashes[rsp3['touched_module']]==rsp3['module_after_sha256']
 else:
  assert sha(raw)==rsp3['module_after_sha256'],'RSP3 module changed'
  assert raw.count(rsp3['added_line'].encode())==1,'RSP3 module entry missing'
  original_hashes[rsp3['touched_module']]=sha(raw.replace(rsp3['added_line'].encode(),b''))
 assert sha(source(r / rsp3['touched_module']).replace(b'\npub mod requirements;\n',b''))==rsp3['module_after_sha256'] if lc1 else True
files=sorted(p for p in (r/'src').rglob('*.rs') if p.relative_to(r).as_posix() not in binding['exclusions'] and p.relative_to(r).as_posix() not in added)
h=hashlib.sha256();count_untouched=0
for p in files:
 name=p.relative_to(r).as_posix();raw=source(p)
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
assert sha(source(r / 'src/construction/search_policy.rs'))==binding['search_policy_sha256'],'RSP2 engine changed'
print(json.dumps({'status':'PASS','original_source_files':len(files),'historical_base_untouched':count_untouched,'untouched':count_untouched-sum(p not in original_hashes and p not in changes and p not in binding['exclusions'] for p in (causal['modified'] if causal else {})), 'causal_extension_verified':bool(causal),'lc1_added_files':len(lc1['added']) if lc1 else 0,'original_aggregate':h.hexdigest(),'rsp2_search_policy_unchanged':True}))
