//! Single-writer persistent software cognition and fail-closed action intent.
//! Host attestation is not a proof of hidden-world continuity. No live external
//! permission is serialized. An uncertain action is never automatically retried.
use super::autonomy::{ActionOffer,AutonomyPolicy};
use super::runtime::{Config,ExternalPost,Learner,ProtectedConstructionRuntime};
use super::vm::{fingerprint,Error,Result};
use crate::{EvoConfig,EvoPhase,HumanProtectionEvidence};
use serde::{Deserialize,Serialize};
use std::{fs::{self,File,OpenOptions},io::{Read,Write},path::{Path,PathBuf}};
fn io(e:std::io::Error)->Error {Error::Execution(format!("session storage: {e}"))}
#[derive(Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Stored {version:u32,checkpoint:String,emergency_latched:bool,hash:u64}
impl Stored {
    fn hash(&self)->u64 {fingerprint(&serde_json::to_vec(&(self.version,&self.checkpoint,self.emergency_latched)).expect("plain data"))}
    fn learner(&self)->Result<Learner> {
        if self.version!=1 || self.hash!=self.hash() {return Err(Error::InvalidData);}
        Learner::from_checkpoint(self.checkpoint.as_bytes())
    }
}
#[derive(Debug,Clone,Serialize,Deserialize,PartialEq,Eq)]
#[serde(deny_unknown_fields)]
pub struct Continuity {pub checkpoint:u64,pub next_event:u64,pub observation:Vec<i64>}
pub struct PersistentSession {path:PathBuf,_lock:File,stored:Option<Stored>,runtime:Option<ProtectedConstructionRuntime>,storage_failed:bool}
impl PersistentSession {
    pub fn open(path:impl AsRef<Path>)->Result<Self> {
        fs::create_dir_all(path.as_ref()).map_err(io)?;let path=path.as_ref().to_path_buf();
        let lock=OpenOptions::new().read(true).write(true).create(true).truncate(false).open(path.join("session.lock")).map_err(io)?;
        lock.try_lock().map_err(|e|Error::Execution(format!("session already in use: {e}")))?;
        if path.join("pending.json").exists() {return Err(Error::Execution("unresolved external action; host reconciliation required, no automatic retry".into()));}
        let stored=if path.join("state.json").exists() {
            let mut bytes=Vec::new();File::open(path.join("state.json")).map_err(io)?.take(8*1024*1024+1).read_to_end(&mut bytes).map_err(io)?;
            if bytes.len()>8*1024*1024 {return Err(Error::Capacity);}
            let s:Stored=serde_json::from_slice(&bytes).map_err(|_|Error::InvalidData)?;s.learner()?;Some(s)
        } else {None};
        Ok(Self{path,_lock:lock,stored,runtime:None,storage_failed:false})
    }
    pub fn continuity(&self)->Result<Option<Continuity>> {
        self.stored.as_ref().map(|s|{let l=s.learner()?;Ok(Continuity{checkpoint:s.hash,next_event:l.records().len() as u64,observation:l.current().to_vec()})}).transpose()
    }
    pub fn start(&mut self,config:Config,policy:AutonomyPolicy,initial:Vec<i64>)->Result<()> {
        if self.stored.is_some() || self.runtime.is_some() || self.storage_failed {return Err(Error::Busy);}
        let organism=EvoPhase::new(EvoConfig{sensory_cells:config.width,motor_cells:config.actions,..Default::default()});
        let mut runtime=ProtectedConstructionRuntime::new(organism,config,initial)?;runtime.enable_autonomy(policy)?;self.runtime=Some(runtime);self.save()
    }
    pub fn resume(&mut self,attestation:&Continuity)->Result<()> {
        if self.runtime.is_some() || self.storage_failed {return Err(Error::Busy);}
        if self.continuity()?.as_ref()!=Some(attestation) {return Err(Error::Stale);}
        let s=self.stored.as_ref().ok_or(Error::NoEvidence)?;
        self.runtime=Some(ProtectedConstructionRuntime::resume_session(s.checkpoint.as_bytes(),s.emergency_latched)?);Ok(())
    }
    pub fn runtime(&self)->Result<&ProtectedConstructionRuntime> {self.runtime.as_ref().ok_or(Error::NoEvidence)}
    pub fn freeze(&mut self)->Result<()> {
        if self.storage_failed {return Err(Error::Busy);}
        self.runtime.as_mut().ok_or(Error::NoEvidence)?.freeze_learning()?;self.save()
    }
    pub fn rollback(&mut self)->Result<()> {
        if self.storage_failed {return Err(Error::Busy);}
        self.runtime.as_mut().ok_or(Error::NoEvidence)?.rollback_cognition()?;self.save()
    }
    pub fn step<H,F>(&mut self,goal:Option<&[Option<i64>]>,offers:&[ActionOffer],hazard:H,execute:F)->Result<usize>
    where H:FnOnce(usize)->HumanProtectionEvidence,F:FnOnce(usize)->Result<ExternalPost> {
        if self.storage_failed || self.path.join("pending.json").exists() {return Err(Error::Busy);}
        let _=self.runtime()?;
        // Durable intent before possible external effect. A false stop after a
        // crash before execution is acceptable; an automatic duplicate is not.
        let bytes=serde_json::to_vec(&self.continuity()?).map_err(|_|Error::InvalidData)?;
        let mut intent=OpenOptions::new().write(true).create_new(true).open(self.path.join("pending.json")).map_err(io)?;
        intent.write_all(&bytes).and_then(|_|intent.sync_all()).map_err(io)?;Self::sync_dir(&self.path)?;
        let result=self.runtime.as_mut().unwrap().step_autonomous(goal,offers,hazard,execute);
        self.save()?;
        if let Err(e)=fs::remove_file(self.path.join("pending.json")).map_err(io).and_then(|_|Self::sync_dir(&self.path)) {self.storage_failed=true;return Err(e);}
        result
    }
    fn save(&mut self)->Result<()> {
        let result=(||{
            let r=self.runtime()?;let mut s=Stored{version:1,checkpoint:String::from_utf8(r.checkpoint()?).map_err(|_|Error::InvalidData)?,emergency_latched:r.organism().human_protection_emergency_latched(),hash:0};
            s.hash=s.hash();let bytes=serde_json::to_vec(&s).map_err(|_|Error::InvalidData)?;if bytes.len()>8*1024*1024 {return Err(Error::Capacity);}
            let mut file=File::create(self.path.join("state.tmp")).map_err(io)?;file.write_all(&bytes).and_then(|_|file.sync_all()).map_err(io)?;drop(file);
            fs::rename(self.path.join("state.tmp"),self.path.join("state.json")).map_err(io)?;Self::sync_dir(&self.path)?;self.stored=Some(s);Ok(())
        })();if result.is_err(){self.storage_failed=true;}result
    }
    fn sync_dir(path:&Path)->Result<()> {
        #[cfg(unix)] {File::open(path).and_then(|f|f.sync_all()).map_err(io)?;}
        #[cfg(not(unix))] {let _=path;}
        Ok(())
    }
}
