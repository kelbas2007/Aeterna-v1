from pathlib import Path

def edit(path,replacements):
 p=Path(path);s=p.read_text()
 for old,new in replacements:
  if s.count(old)!=1:raise RuntimeError('Stale source: '+path+' '+old[:70])
  s=s.replace(old,new)
 p.write_text(s)
edit('src/construction/runtime.rs',[
 ('struct Commitment {','pub(crate) struct Commitment {'),
 ('    frozen_at: Option<u64>,','    frozen_at: Option<u64>,\n    #[serde(default,skip_serializing_if="carrier_unmanaged")]\n    carrier_managed: bool,'),
 ('frozen_at: None })','frozen_at: None,carrier_managed:false })'),
 ('        if self.frozen_at.is_none() && self.shadow.is_none() && self.records.len() >= self.config.minimum_training','        if !self.carrier_managed && self.frozen_at.is_none() && self.shadow.is_none() && self.records.len() >= self.config.minimum_training')])
p=Path('src/construction/runtime.rs');p.write_text(p.read_text()+'\nfn carrier_unmanaged(v:&bool)->bool{!*v}\ninclude!("unified_port.rs");\n')
edit('src/scientific_runtime.rs',[
 ('    VectorModeRequired,','    VectorModeRequired,\n    ConstructionModeRequired,'),
 ('    unified_trace_enabled: bool,','    unified_trace_enabled: bool,\n    constructing: Option<UnifiedConstruction>,'),
 ('            unified_trace_enabled: false,','            unified_trace_enabled: false,\n            constructing: None,')])
p=Path('src/scientific_runtime.rs');s=p.read_text()
for signature in [
 'pub fn observe_external(&mut self, raster: &[f32]) -> Result<(), RuntimeError> {',
 'pub fn propose(&mut self) -> Result<Option<ActionProposal>, RuntimeError> {',
 'pub fn propose_unified(&mut self) -> Result<Option<ActionProposal>, RuntimeError> {',
 'pub fn propose_vector(&mut self) -> Result<Option<ActionProposal>,RuntimeError> {',
 'pub fn propose_partial(&mut self) -> Result<Option<ActionProposal>, RuntimeError> {',
 'pub fn observe_external_partial(&mut self, values: &[Option<f32>]) -> Result<(), RuntimeError> {']:
 if s.count(signature)!=1:raise RuntimeError('Missing boundary: '+signature)
 s=s.replace(signature,signature+'\n        if self.constructing.is_some(){return Err(RuntimeError::ConstructionModeRequired);}')
p.write_text(s+'\ninclude!("unified_runtime.rs");\n')
edit('src/phase_meta_control.rs',[
 ('#[derive(Debug, Clone)]\npub struct PhaseMetaControlConfig','#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]\npub struct PhaseMetaControlConfig')])
p=Path('src/phase_native.rs');p.write_text(p.read_text()+'\ninclude!("phase_constructing_control.rs");\n')
print('Unified ownership and exact-source ports installed')
