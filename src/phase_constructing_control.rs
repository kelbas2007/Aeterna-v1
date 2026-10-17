// U1 bridge: bounded cold exploration in the same carrier readout.
impl EvoPhase {
    pub fn choose_phase_meta_exploratory(&self, proposals:&[PhaseCognitiveProposal], entropy:u64, explore:bool)->Option<PhaseMetaDecision> {
        if proposals.is_empty() || proposals.len()>256 {return None;}
        let mut ids=std::collections::BTreeSet::new();let mut scored=Vec::new();
        for p in proposals {
            if p.action>=self.config.motor_cells || !ids.insert(p.proposal_id){return None;}
            let score=self.phase_native_meta_score(p.fields)?;
            let mut h=entropy^p.proposal_id.wrapping_mul(0x9e3779b97f4a7c15);
            h=(h^(h>>30)).wrapping_mul(0xbf58476d1ce4e5b9);h=(h^(h>>27)).wrapping_mul(0x94d049bb133111eb);h^=h>>31;
            scored.push((p,score,h));
        }
        let exploratory=explore && entropy&7==0;
        let high=scored.iter().map(|(_,s,_)|*s).fold(f32::NEG_INFINITY,f32::max);
        let (p,score,_)=scored.into_iter().filter(|(_,s,_)|exploratory || (s-high).abs()<=1e-6).max_by_key(|(_,_,h)|*h)?;
        Some(PhaseMetaDecision{proposal_id:p.proposal_id,action:p.action,score})
    }
    // Exact data-only snapshot for cold-native + U1; other lanes are rejected,
    // never silently dropped. External authority is not cloned or serialized.
    pub(crate) fn constructing_carrier_checkpoint(&self)->Option<Vec<u8>> {
        let n=self.phase_native.as_ref()?;let m=n.meta_control.as_ref()?;
        if !n.receptors.is_empty() || !n.circuits.is_empty() || n.drive.is_some() || n.concepts.is_some()
            || n.deep.is_some() || n.contextual.is_some() || n.perceptual.is_some() || n.compositional.is_some()
            || n.temporal_evidence.is_some() || n.factor_causality.is_some() || n.grounded_objects.is_some()
            || n.general_policy.is_some() || n.innate_scaffold.is_some() || n.online.is_some()
            || n.rules.is_some() || n.partial.is_some() || n.vector.is_some() || m.ecology.is_some()
            || !self.branches.is_empty() || self.synapses.len()!=META_FIELD_COUNT {return None;}
        serde_json::to_vec(&ConstructingCarrierSnapshot{version:1,config:self.config.clone(),native:n.config.clone(),
            cells:self.cells.clone(),synapses:self.synapses.clone(),meta:m.config.clone(),observations:m.observations,
            monotone:m.monotone_evidence_score,tick:self.tick,current:self.current_real.as_ref().map(|r|(r.sensory.clone(),r.need,r.tick))}).ok()
    }
    pub(crate) fn from_constructing_carrier_checkpoint(bytes:&[u8])->Option<Self> {
        if bytes.len()>2*1024*1024{return None;}
        let s:ConstructingCarrierSnapshot=serde_json::from_slice(bytes).ok()?;
        let unit=|x:f32|x.is_finite()&&(0.0..=1.0).contains(&x);
        if s.version!=1 || !(1..=32).contains(&s.config.sensory_cells) || !(1..=16).contains(&s.config.motor_cells)
            || s.config.dormant_cells!=32 || s.config.hdc_dim!=32 || !unit(s.config.phase_learning_rate)
            || !unit(s.config.weight_learning_rate) || !unit(s.config.eligibility_decay)
            || !unit(s.config.residual_recruit_threshold) || s.config.min_recruit_support==0
            || !(1..=1024).contains(&s.native.horizon) || !unit(s.native.match_threshold)
            || !unit(s.native.coherence_floor) || s.native.coherence_floor>=1.0
            || !unit(s.native.discount) || s.native.discount>=1.0 || !unit(s.meta.learning_rate) || s.meta.learning_rate==0.0 {return None;}
        let mut evo=Self::new(s.config);evo.enable_phase_native_planning(s.native);
        if !evo.enable_phase_native_meta_control(s.meta){return None;}
        if s.cells.len()!=evo.cells.len() || s.synapses.len()!=evo.synapses.len() {return None;}
        for (a,b) in s.cells.iter().zip(&evo.cells) {
            if a.recruited!=b.recruited || !a.phase.is_finite() || !a.charge.is_finite() || !a.threshold.is_finite() || !a.utility.is_finite(){return None;}
        }
        for (a,b) in s.synapses.iter().zip(&evo.synapses) {
            if a.from!=b.from || a.to!=b.to || !unit(a.weight) || !unit(a.eligibility) || !unit(a.confidence) || !a.phase_offset.is_finite(){return None;}
        }
        evo.cells=s.cells;evo.synapses=s.synapses;evo.tick=s.tick;
        if let Some((sensory,need,tick))=s.current {
            if sensory.len()!=evo.config.sensory_cells || sensory.iter().any(|&x|!unit(x)) || tick!=s.tick {return None;}
            evo.current_real=Some(super::FactualFrame{sensory,need,tick});
        }
        let m=evo.phase_native.as_mut()?.meta_control.as_mut()?;
        m.observations=s.observations;m.monotone_evidence_score=s.monotone;Some(evo)
    }
}
#[derive(serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ConstructingCarrierSnapshot {
    version:u32,config:super::EvoConfig,native:PhaseNativeConfig,cells:Vec<super::PhaseCell>,synapses:Vec<super::PhaseSynapse>,
    meta:PhaseMetaControlConfig,observations:u64,monotone:bool,tick:u64,current:Option<(Vec<f32>,bool,u64)>,
}
