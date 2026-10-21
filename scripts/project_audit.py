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
learning=binding.get('implemented_learning_world_extension')
learning_base={}
if learning:
 packed=(r/learning['base_archive']).read_bytes()
 assert sha(packed)==learning['base_archive_sha256'],'learning-world base archive changed'
 learning_base=json.loads(gzip.decompress(packed))
 assert set(learning_base)==set(learning['modified']),'incomplete learning-world historical archive'
 for name,d in learning['modified'].items():
  assert sha((r/name).read_bytes())==d['after_sha256'],name+' undeclared learning-world mutation'
  assert sha(learning_base[name].encode())==d['before_sha256'],name+' learning-world base mismatch'
 for name,digest in learning['added'].items():
  assert sha((r/name).read_bytes())==digest,name+' learning-world source mismatch'
def before_learning(path):
 name=path.relative_to(r).as_posix()
 return learning_base[name].encode() if name in learning_base else path.read_bytes()
consolidated=binding.get('implemented_consolidated_memory_extension')
consolidated_base={}
if consolidated:
 packed=(r/consolidated['base_archive']).read_bytes()
 assert sha(packed)==consolidated['base_archive_sha256'],'consolidated memory base archive changed'
 consolidated_base=json.loads(gzip.decompress(packed))
 assert set(consolidated_base)==set(consolidated['modified']),'incomplete consolidated memory archive'
 for name,d in consolidated['modified'].items():
  assert sha(before_learning(r/name))==d['after_sha256'],name+' undeclared consolidation mutation'
  assert sha(consolidated_base[name].encode())==d['before_sha256'],name+' consolidation base mismatch'
 for name,digest in consolidated['added'].items():
  assert sha(before_learning(r/name))==digest,name+' consolidation source mismatch'
def before_consolidated(path):
 name=path.relative_to(r).as_posix()
 return consolidated_base[name].encode() if name in consolidated_base else before_learning(path)
coverage=binding.get('implemented_coverage_memory_extension')
coverage_base={}
if coverage:
 packed=(r/coverage['base_archive']).read_bytes()
 assert sha(packed)==coverage['base_archive_sha256'],'coverage memory base archive changed'
 coverage_base=json.loads(gzip.decompress(packed))
 assert set(coverage_base)==set(coverage['modified']),'incomplete coverage memory historical archive'
 for name,d in coverage['modified'].items():
  assert sha(before_consolidated(r/name))==d['after_sha256'],name+' undeclared coverage memory mutation'
  assert sha(coverage_base[name].encode())==d['before_sha256'],name+' coverage memory historical mismatch'
 for name,digest in coverage['added'].items():
  assert sha(before_consolidated(r/name))==digest,name+' coverage memory source mismatch'
def before_coverage(path):
 name=path.relative_to(r).as_posix()
 return coverage_base[name].encode() if name in coverage_base else before_consolidated(path)
first=binding.get('implemented_first_readout_extension')
first_base={}
if first:
 packed=(r/first['base_archive']).read_bytes()
 assert sha(packed)==first['base_archive_sha256'],'first-readout base archive changed'
 first_base=json.loads(gzip.decompress(packed))
 assert set(first_base)==set(first['modified']),'incomplete first-readout historical archive'
 for name,d in first['modified'].items():
  assert sha(before_coverage(r/name))==d['after_sha256'],name+' undeclared first-readout mutation'
  assert sha(first_base[name].encode())==d['before_sha256'],name+' first-readout historical mismatch'
 for name,digest in first['added'].items():
  assert sha(before_coverage(r/name))==digest,name+' first-readout source mismatch'
def before_first(path):
 name=path.relative_to(r).as_posix()
 return first_base[name].encode() if name in first_base else before_coverage(path)
rules=binding.get('implemented_acquired_rules_extension')
rules_base={}
if rules:
 packed=(r/rules['base_archive']).read_bytes()
 assert sha(packed)==rules['base_archive_sha256'],'acquired rules base archive changed'
 rules_base=json.loads(gzip.decompress(packed))
 assert set(rules_base)==set(rules['modified']),'incomplete acquired rules historical archive'
 for name,d in rules['modified'].items():
  assert sha(before_first(r/name))==d['after_sha256'],name+' undeclared acquired rules mutation'
  assert sha(rules_base[name].encode())==d['before_sha256'],name+' acquired rules historical mismatch'
 for name,digest in rules['added'].items():
  assert sha(before_first(r/name))==digest,name+' acquired rules source mismatch'
def before_rules(path):
 name=path.relative_to(r).as_posix()
 return rules_base[name].encode() if name in rules_base else before_first(path)
organism=binding.get('implemented_organism_development_extension')
organism_base={}
if organism:
 packed=(r/organism['base_archive']).read_bytes()
 assert sha(packed)==organism['base_archive_sha256'],'organism base archive changed'
 organism_base=json.loads(gzip.decompress(packed))
 assert set(organism_base)==set(organism['modified']),'incomplete organism historical archive'
 for name,d in organism['modified'].items():
  assert sha(before_rules(r/name))==d['after_sha256'],name+' undeclared organism source mutation'
  assert sha(organism_base[name].encode())==d['before_sha256'],name+' organism historical source mismatch'
 for name,digest in organism['added'].items():
  assert sha(before_rules(r/name))==digest,name+' organism source mismatch'
def before_organism(path):
 name=path.relative_to(r).as_posix()
 return organism_base[name].encode() if name in organism_base else before_rules(path)
external=binding.get('implemented_external_reward_extension')
external_base={}
if external:
 packed=(r/external['base_archive']).read_bytes()
 assert sha(packed)==external['base_archive_sha256'],'external base archive changed'
 external_base=json.loads(gzip.decompress(packed))
 assert set(external_base)==set(external['modified']),'incomplete external historical archive'
 for name,d in external['modified'].items():
  assert sha(before_organism(r/name))==d['after_sha256'],name+' undeclared external source mutation'
  assert sha(external_base[name].encode())==d['before_sha256'],name+' external historical source mismatch'
 for name,digest in external['added'].items():
  assert sha(before_organism(r/name))==digest,name+' external source mismatch'
def before_external(path):
 name=path.relative_to(r).as_posix()
 return external_base[name].encode() if name in external_base else before_organism(path)
causal=binding.get('implemented_causal_continual_extension')
base_sources={}
if causal:
 packed=(r/causal['base_archive']).read_bytes()
 assert sha(packed)==causal['base_archive_sha256'],'causal base archive changed'
 base_sources=json.loads(gzip.decompress(packed))
 assert set(base_sources)==set(causal['modified']),'incomplete historical source archive'
 for name,d in causal['modified'].items():
  assert sha(before_external(r/name))==d['after_sha256'],name+' undeclared causal source mutation'
  assert sha(base_sources[name].encode())==d['before_sha256'],name+' historical source mismatch'
 for name,digest in causal['added'].items():
  assert sha(before_external(r/name))==digest,name+' new causal source mismatch'
def source(path):
 name=path.relative_to(r).as_posix()
 return base_sources[name].encode() if name in base_sources else before_external(path)
def git_blob(raw):
 return hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()
added=set(causal['added']) if causal else set()
if external:added.update(external['added'])
if organism:added.update(organism['added'])
if rules:added.update(rules['added'])
if first:added.update(first['added'])
if coverage:added.update(coverage['added'])
if consolidated:added.update(consolidated['added'])
if learning:added.update(learning['added'])
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
current_modified=set(causal['modified'] if causal else {})|set(external['modified'] if external else {})|set(organism['modified'] if organism else {})|set(rules['modified'] if rules else {})|set(first['modified'] if first else {})|set(coverage['modified'] if coverage else {})|set(consolidated['modified'] if consolidated else {})|set(learning['modified'] if learning else {})
untouched=count_untouched-sum(p.relative_to(r).as_posix() in current_modified
 and p.relative_to(r).as_posix() not in original_hashes and p.relative_to(r).as_posix() not in changes for p in files)
print(json.dumps({'status':'PASS','original_source_files':len(files),'historical_base_untouched':count_untouched,
 'untouched':untouched,'causal_extension_verified':bool(causal),'external_extension_verified':bool(external),
 'organism_extension_verified':bool(organism),
 'acquired_rules_extension_verified':bool(rules),
 'first_readout_extension_verified':bool(first),
 'coverage_memory_extension_verified':bool(coverage),
 'consolidated_memory_extension_verified':bool(consolidated),
 'learning_world_extension_verified':bool(learning),
 'lc1_added_files':len(lc1['added']) if lc1 else 0,'original_aggregate':h.hexdigest(),'rsp2_search_policy_unchanged':True}))
