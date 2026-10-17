from pathlib import Path
import runpy,subprocess
runpy.run_path('.ci/unified/bootstrap.py')
def edit(path,pairs):
 p=Path(path);s=p.read_text()
 for old,new in pairs:
  if s.count(old)!=1:raise RuntimeError('Stale hardening source: '+path+' '+old[:70])
  s=s.replace(old,new)
 p.write_text(s)
 subprocess.run(['git','add',path],check=True)
# Actual HDC bundle API takes a slice of references, not owned values.
edit('src/construction/unified_port.rs', [('PhaseVector::bundle(&roles)','PhaseVector::bundle(&roles.iter().collect::<Vec<_>>())')])
# Preserve a monotonically increasing audit counter, never issued permits.
p=Path('src/human_protection.rs')
p.write_text(p.read_text()+'''
impl HumanProtection {
    pub(crate) fn audit_position(&self)->(u64,bool) {(self.sequence,self.emergency_stop_latched)}
    pub(crate) fn from_audit_position(sequence:u64,latched:bool)->Self {
        Self {sequence,emergency_stop_latched:latched,last_record:None,last_issued_sequence:None,last_consumed_sequence:None}
    }
}
''')
subprocess.run(['git','add',str(p)],check=True)
edit('src/unified_runtime.rs',[
 ('sequence:u64,emergency:bool,hash:u64','sequence:u64,emergency:bool,protection_sequence:u64,hash:u64'),
 ('&(1u32,&carrier,u,self.sequence,self.emergency_latched())','&(1u32,&carrier,u,self.sequence,self.emergency_latched(),self.protection.audit_position().0)'),
 ('"emergency":self.emergency_latched(),"hash":hash','"emergency":self.emergency_latched(),"protection_sequence":self.protection.audit_position().0,"hash":hash'),
 ('&(s.version,&s.carrier,&s.cognition,s.sequence,s.emergency)','&(s.version,&s.carrier,&s.cognition,s.sequence,s.emergency,s.protection_sequence)'),
 ('if s.emergency{let mut e=Self::absent_evidence();e.emergency_stop=true;rt.protection.screen(0,e);}','rt.protection=HumanProtection::from_audit_position(s.protection_sequence,s.emergency);'),
 ('self.organism=candidate.organism;self.constructing=candidate.constructing.take();Ok(())','if candidate.emergency_latched(){let mut e=Self::absent_evidence();e.emergency_stop=true;self.protection.screen(0,e);}\n        self.organism=candidate.organism;self.constructing=candidate.constructing.take();Ok(())')])
print('Audit-only authority continuity installed; no permit restored')
