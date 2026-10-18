from pathlib import Path
import hashlib,json,shutil
r=Path.cwd();p=r/'src/scientific_runtime.rs';raw=p.read_bytes()
assert hashlib.sha256(raw).hexdigest()=='8e09809514055de307becc1e490fd2f8d8f33902df38a5e06d998247214b00e1'
signatures=['    pub fn restart_cognition(&mut self) -> Result<(), RuntimeError> {','    pub fn restore_native_checkpoint(&mut self, checkpoint:PhaseNativeCheckpoint) -> Result<(),RuntimeError> {','    pub fn restore_online_checkpoint(&mut self, bytes: &[u8]) -> Result<(), RuntimeError> {']
guard='\n        // This mode owns a joint Learner/U1 checkpoint, not a native-only one.\n        if self.constructing.is_some() { return Err(RuntimeError::ConstructionModeRequired); }'
s=raw.decode()
for signature in signatures:
    assert s.count(signature)==1
    s=s.replace(signature,signature+guard)
p.write_text(s)
assert hashlib.sha256(p.read_bytes()).hexdigest()=='6e78805d8e7930518b8dd04a73d175010ca4d97dc45590c40c283a8d8872b1df'
b=r/'docs/evidence/PROJECT_SOURCE_BINDING.json';d=json.loads(b.read_text());d['bound_cognitive_source_files']=81;d['unchanged_cognitive_source_files']=80
d['reviewed_boundary_changes']=[{'path':'src/scientific_runtime.rs','before_sha256':hashlib.sha256(raw).hexdigest(),'after_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'added_guard':guard,'function_signatures':signatures,'reason':'PR6 review 4306651478: forbid partial native restoration of a constructing runtime before any mutation'}]
b.write_text(json.dumps(d,indent=2)+'\n')
shutil.copyfile('.ci/review/guards.rs','tests/unified_restore_guards.rs')
shutil.copyfile('.ci/review/audit.py','scripts/project_audit.py')
p=r/'scripts/check_project.py';s=p.read_text();assert s.count("'unified_runtime','human_protection'")==1;p.write_text(s.replace("'unified_runtime','human_protection'","'unified_runtime','unified_restore_guards','human_protection'"))
p=r/'docs/PROJECT_STATE.json';d=json.loads(p.read_text());d['manual_specialized_workflows']+=['dc2-autonomy.yml','dc2-r1.yml','education-development.yml'];d['restoration_contract']='construction state must use joint checkpoint APIs; native-only restart/restore rejected before mutation';p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n')
p=r/'docs/PROJECT_OVERVIEW.md';p.write_text(p.read_text()+'''\n## Review correction before consolidation\n\nGeneric native-only restart/restore APIs reject construction mode before any\nmutation. Use the joint checkpoint API for a Learner/U1 state. Four regression\ntests fail on the former source and pass with these three entry guards.\nThe RSP2 search algorithm and its admission rules remain byte-identical.\nTwo already-consumed DC-2 installers now have read-only manual verification\nworkflows; no current workflow applies those historical patches again.\n''')
