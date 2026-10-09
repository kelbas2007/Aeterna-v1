// U1: carrier-owned competition over task-agnostic cognitive proposal fields.
//
// This layer deliberately does not know which cognitive module emitted a
// proposal. It learns one physical utility projection and selects a unique
// proposal by that projection.

pub const META_FIELD_COUNT: usize = 5;

#[derive(Debug, Clone)]
pub struct PhaseMetaControlConfig {
    pub learning_rate: f32,
    pub learning_enabled: bool,
    pub readout_enabled: bool,
    pub phase_learning_enabled: bool,
}

impl Default for PhaseMetaControlConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.30,
            learning_enabled: true,
            readout_enabled: true,
            phase_learning_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseCognitiveProposal {
    pub proposal_id: u64,
    pub action: usize,
    /// [goal_value, epistemic_value, contradiction_pressure,
    ///  confidence, economy]
    pub fields: [f32; META_FIELD_COUNT],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseMetaDecision {
    pub proposal_id: u64,
    pub action: usize,
    pub score: f32,
}

#[derive(Debug, Clone)]
pub(super) struct PhaseMetaControlState {
    config: PhaseMetaControlConfig,
    feature_cells: [usize; META_FIELD_COUNT],
    utility_cell: usize,
    weight_synapses: [usize; META_FIELD_COUNT],
    observations: u64,
    /// Opt-in, non-diluting score for evidence bearing feature vectors.
    monotone_evidence_score: bool,
    ecology: Option<PhaseHypothesisEcologyState>,
}

#[derive(Debug, Clone)]
pub struct PhaseMetaControlCheckpoint {
    config: PhaseMetaControlConfig,
    learned_synapses: [PhaseSynapse; META_FIELD_COUNT],
    observations: u64,
    monotone_evidence_score: bool,
}

impl EvoPhase {
    /// Allocate a generic physical utility projection. Every weight begins at
    /// zero; no proposal source/module identity is represented.
    pub fn enable_phase_native_meta_control(
        &mut self,
        config: PhaseMetaControlConfig,
    ) -> bool {
        assert!(config.learning_rate > 0.0 && config.learning_rate <= 1.0);
        let Some(mut native) = self.phase_native.take() else { return false; };
        if native.meta_control.is_some() {
            self.phase_native = Some(native);
            return false;
        }

        let free = self.dormant_range()
            .filter(|index| !self.cells[*index].recruited)
            .take(META_FIELD_COUNT + 1)
            .collect::<Vec<_>>();
        if free.len() != META_FIELD_COUNT + 1 {
            self.phase_native = Some(native);
            return false;
        }

        let feature_cells:[usize;META_FIELD_COUNT] =
            free[..META_FIELD_COUNT].try_into().expect("fixed meta fields");
        let utility_cell=free[META_FIELD_COUNT];
        for &cell in feature_cells.iter().chain(std::iter::once(&utility_cell)) {
            self.cells[cell].recruited=true;
        }
        let weight_synapses=std::array::from_fn(|i|
            self.native_synapse(feature_cells[i],utility_cell)
        );

        native.meta_control=Some(PhaseMetaControlState{
            config,feature_cells,utility_cell,weight_synapses,observations:0,
            monotone_evidence_score:false,ecology:None,
        });
        self.phase_native=Some(native);
        true
    }

    pub fn phase_native_meta_control_enabled(&self)->bool{
        self.phase_native.as_ref()
            .and_then(|n|n.meta_control.as_ref()).is_some()
    }

    pub fn phase_native_meta_weights(&self)
        ->Option<[f32;META_FIELD_COUNT]>
    {
        let meta=self.phase_native.as_ref()?.meta_control.as_ref()?;
        Some(std::array::from_fn(|i|
            self.synapses[meta.weight_synapses[i]].weight
        ))
    }

    pub fn phase_native_meta_synapses(&self)
        ->Option<[usize;META_FIELD_COUNT]>
    {
        Some(self.phase_native.as_ref()?.meta_control.as_ref()?.weight_synapses)
    }

    pub fn phase_native_meta_observations(&self)->u64{
        self.phase_native.as_ref()
            .and_then(|n|n.meta_control.as_ref())
            .map(|m|m.observations).unwrap_or(0)
    }

    pub(super) fn is_native_meta_synapse(&self,index:usize)->bool{
        self.phase_native.as_ref()
            .and_then(|n|n.meta_control.as_ref())
            .map(|m|m.weight_synapses.contains(&index))
            .unwrap_or(false)
    }

    fn valid_meta_fields(fields:[f32;META_FIELD_COUNT])->bool{
        fields.into_iter().all(|x|x.is_finite()&&(0.0..=1.0).contains(&x))
    }

    fn phase_meta_score_with_state(
        &self,
        native:&PhaseNativeState,
        fields:[f32;META_FIELD_COUNT],
    )->Option<f32>{
        if !Self::valid_meta_fields(fields){return None;}
        let meta=native.meta_control.as_ref()?;
        if !meta.config.readout_enabled{return None;}
        let mass=fields.into_iter().sum::<f32>();
        if mass<=1.0e-8{return Some(0.0);}
        let floor=native.config.coherence_floor;
        let mut score=0.0f32;
        let mut available_weight=0.0f32;
        for i in 0..META_FIELD_COUNT{
            let physical=conductance(
                &self.cells,&self.synapses[meta.weight_synapses[i]],floor
            );
            score+=fields[i]*physical;
            available_weight+=physical;
        }
        // A normalized feature *mass* can make stronger positive evidence
        // decrease a candidate's value. During opt-in cold online learning
        // divide by the total PHYSICAL U1 capacity instead, which is shared
        // across rival actions and independent of their number of features.
        // A stronger positive field cannot dilute the existing evidence.
        let divisor=if meta.monotone_evidence_score {
            available_weight.max(1.0)
        } else {
            mass
        };
        Some((score/divisor).clamp(0.0,1.0))
    }

    pub fn phase_native_meta_score(
        &self,
        fields:[f32;META_FIELD_COUNT],
    )->Option<f32>{
        let native=self.phase_native.as_ref()?;
        self.phase_meta_score_with_state(native,fields)
    }

    /// Factual utility learning for a previously evaluated cognitive operation.
    /// The caller supplies only task-agnostic proposal fields and bounded
    /// observed utility, never a module/correct-operation label.
    pub fn observe_phase_native_meta_utility(
        &mut self,
        fields:[f32;META_FIELD_COUNT],
        factual_utility:f32,
    )->bool{
        if !Self::valid_meta_fields(fields)
            || !factual_utility.is_finite()
            || !(0.0..=1.0).contains(&factual_utility)
        {return false;}

        let Some(mut native)=self.phase_native.take() else{return false;};
        let Some(meta)=native.meta_control.as_ref() else{
            self.phase_native=Some(native);return false;
        };
        let config=meta.config.clone();
        let synapses=meta.weight_synapses;
        let mass=if meta.monotone_evidence_score {
            meta.weight_synapses.iter().map(|&index|
                conductance(&self.cells,&self.synapses[index],
                    native.config.coherence_floor)
            ).sum::<f32>().max(1.0)
        } else {
            fields.into_iter().sum::<f32>().max(1.0e-8)
        };
        let predicted=self.phase_meta_score_with_state(&native,fields).unwrap_or(0.0);
        let error=factual_utility-predicted;

        if config.learning_enabled{
            for i in 0..META_FIELD_COUNT{
                if fields[i]<=0.0{continue;}
                let syn=&mut self.synapses[synapses[i]];
                if !syn.plastic{continue;}
                let credit=fields[i]/mass;
                syn.eligibility=credit;
                syn.weight=(syn.weight
                    +config.learning_rate*error*credit).clamp(0.0,1.0);
                if config.phase_learning_enabled{
                    let target=wrap_phase(
                        self.cells[syn.to].phase-self.cells[syn.from].phase
                    );
                    syn.phase_offset=wrap_phase(
                        syn.phase_offset
                            +self.config.phase_learning_rate
                                *credit
                                *signed_phase_error(target,syn.phase_offset)
                    );
                }
                syn.confidence=(syn.confidence
                    +config.learning_rate*(1.0-syn.confidence))
                    .clamp(0.0,1.0);
            }
            let observations=native.meta_control.as_ref().expect("meta").observations;
            native.meta_control.as_mut().expect("meta").observations=
                observations.saturating_add(1);
        }
        self.phase_native=Some(native);
        true
    }

    /// Select the unique highest-utility opaque proposal. Enumeration order and
    /// proposal ID never contribute to the score. Near-ties fail closed.
    pub fn choose_phase_native_meta_proposal(
        &self,
        proposals:&[PhaseCognitiveProposal],
    )->Option<PhaseMetaDecision>{
        if proposals.is_empty(){return None;}
        let native=self.phase_native.as_ref()?;
        let mut scored=Vec::with_capacity(proposals.len());
        for proposal in proposals{
            if proposal.action>=self.config.motor_cells{return None;}
            let score=self.phase_meta_score_with_state(native,proposal.fields)?;
            scored.push((*proposal,score));
        }
        let max=scored.iter().map(|(_,s)|*s).fold(f32::NEG_INFINITY,f32::max);
        let winners=scored.iter()
            .filter(|(_,s)|(*s-max).abs()<=1.0e-6)
            .collect::<Vec<_>>();
        if winners.len()!=1{return None;}
        let (proposal,score)=winners[0];
        Some(PhaseMetaDecision{
            proposal_id:proposal.proposal_id,
            action:proposal.action,
            score:*score,
        })
    }

    pub fn phase_native_meta_checkpoint(&self)
        ->Option<PhaseMetaControlCheckpoint>
    {
        let meta=self.phase_native.as_ref()?.meta_control.as_ref()?;
        Some(PhaseMetaControlCheckpoint{
            config:meta.config.clone(),
            learned_synapses:std::array::from_fn(|i|
                self.synapses[meta.weight_synapses[i]].clone()
            ),
            observations:meta.observations,
            monotone_evidence_score:meta.monotone_evidence_score,
        })
    }

    pub fn restore_phase_native_meta_checkpoint(
        &mut self,
        checkpoint:PhaseMetaControlCheckpoint,
    )->bool{
        if self.phase_native.as_ref()
            .and_then(|n|n.meta_control.as_ref()).is_some()
        {return false;}
        if !self.enable_phase_native_meta_control(checkpoint.config.clone()){
            return false;
        }
        let mut native=self.phase_native.take().expect("native");
        let meta=native.meta_control.as_mut().expect("meta");
        for i in 0..META_FIELD_COUNT{
            let index=meta.weight_synapses[i];
            let learned=&checkpoint.learned_synapses[i];
            let syn=&mut self.synapses[index];
            syn.weight=learned.weight;
            syn.phase_offset=learned.phase_offset;
            syn.eligibility=0.0;
            syn.confidence=learned.confidence;
            syn.plastic=learned.plastic;
        }
        meta.observations=checkpoint.observations;
        meta.monotone_evidence_score=checkpoint.monotone_evidence_score;
        self.phase_native=Some(native);
        true
    }

    /// Preserve the frozen historical U1 policy unless explicitly enabled.
    /// This is an alternate readout of the SAME physical synapses, not a
    /// privileged source class, motor name, or external exploration schedule.
    pub fn set_phase_native_meta_monotone_evidence(&mut self,enabled:bool)->bool{
        let Some(meta)=self.phase_native.as_mut()
            .and_then(|n|n.meta_control.as_mut()) else{return false;};
        meta.monotone_evidence_score=enabled;
        true
    }

    pub fn phase_native_meta_monotone_evidence(&self)->bool{
        self.phase_native.as_ref()
            .and_then(|n|n.meta_control.as_ref())
            .map(|m|m.monotone_evidence_score).unwrap_or(false)
    }

    pub fn set_phase_native_meta_learning_enabled(&mut self,enabled:bool){
        if let Some(meta)=self.phase_native.as_mut()
            .and_then(|n|n.meta_control.as_mut())
        {
            meta.config.learning_enabled=enabled;
        }
    }
}
