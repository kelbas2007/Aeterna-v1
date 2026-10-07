// U2: shared carrier-owned hypothesis lifecycle inside the U1 meta-control state.

pub const HYPOTHESIS_CAPACITY: usize = 64;

#[derive(Debug, Clone)]
pub struct PhaseHypothesisEcologyConfig {
    pub learning_rate: f32,
    pub dormancy_threshold: f32,
    pub learning_enabled: bool,
    pub phase_learning_enabled: bool,
}

impl Default for PhaseHypothesisEcologyConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.30,
            dormancy_threshold: 0.05,
            learning_enabled: true,
            phase_learning_enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
struct PhaseHypothesisRecord {
    candidate_id: u64,
    candidate_cell: usize,
    utility_synapse: usize,
    observations: u64,
}

#[derive(Debug, Clone)]
pub(super) struct PhaseHypothesisEcologyState {
    config: PhaseHypothesisEcologyConfig,
    ecology_cell: usize,
    records: Vec<PhaseHypothesisRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseHypothesisProposal {
    pub candidate_id: u64,
    pub applicability: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseHypothesisDecision {
    pub candidate_id: u64,
    pub authority: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhaseHypothesisRecordInfo {
    pub candidate_id: u64,
    pub candidate_cell: usize,
    pub utility_synapse: usize,
    pub observations: u64,
    pub weight: f32,
    pub authority_at_full_applicability: f32,
    pub dormant: bool,
}

impl EvoPhase {
    pub fn enable_phase_native_hypothesis_ecology(
        &mut self,
        config: PhaseHypothesisEcologyConfig,
    ) -> bool {
        assert!(config.learning_rate > 0.0 && config.learning_rate <= 1.0);
        assert!(config.dormancy_threshold >= 0.0 && config.dormancy_threshold < 1.0);

        let Some(mut native) = self.phase_native.take() else { return false; };
        let Some(meta) = native.meta_control.as_ref() else {
            self.phase_native = Some(native);
            return false;
        };
        if meta.ecology.is_some() {
            self.phase_native = Some(native);
            return false;
        }

        let Some(ecology_cell) = self.dormant_range()
            .find(|index| !self.cells[*index].recruited)
        else {
            self.phase_native = Some(native);
            return false;
        };
        self.cells[ecology_cell].recruited = true;

        native.meta_control.as_mut().expect("meta").ecology =
            Some(PhaseHypothesisEcologyState {
                config,
                ecology_cell,
                records: Vec::new(),
            });
        self.phase_native = Some(native);
        true
    }

    pub fn phase_native_hypothesis_ecology_enabled(&self) -> bool {
        self.phase_native.as_ref()
            .and_then(|n|n.meta_control.as_ref())
            .and_then(|m|m.ecology.as_ref())
            .is_some()
    }

    pub fn register_phase_native_hypothesis(&mut self, candidate_id: u64) -> bool {
        let Some(mut native) = self.phase_native.take() else { return false; };
        let Some(ecology) = native.meta_control.as_ref()
            .and_then(|m|m.ecology.as_ref())
        else {
            self.phase_native = Some(native);
            return false;
        };
        if ecology.records.len() >= HYPOTHESIS_CAPACITY
            || ecology.records.iter().any(|r|r.candidate_id == candidate_id)
        {
            self.phase_native = Some(native);
            return false;
        }
        let ecology_cell = ecology.ecology_cell;
        let Some(candidate_cell) = self.dormant_range()
            .find(|index| !self.cells[*index].recruited)
        else {
            self.phase_native = Some(native);
            return false;
        };
        self.cells[candidate_cell].recruited = true;
        let utility_synapse = self.native_synapse(candidate_cell, ecology_cell);

        native.meta_control.as_mut().expect("meta")
            .ecology.as_mut().expect("ecology")
            .records.push(PhaseHypothesisRecord {
                candidate_id,
                candidate_cell,
                utility_synapse,
                observations: 0,
            });
        self.phase_native = Some(native);
        true
    }

    fn phase_hypothesis_authority_with_state(
        &self,
        native: &PhaseNativeState,
        candidate_id: u64,
        applicability: f32,
    ) -> Option<f32> {
        if !applicability.is_finite() || !(0.0..=1.0).contains(&applicability) {
            return None;
        }
        let ecology = native.meta_control.as_ref()?.ecology.as_ref()?;
        let record = ecology.records.iter()
            .find(|r|r.candidate_id == candidate_id)?;
        let floor = native.config.coherence_floor;
        Some((applicability
            * conductance(
                &self.cells,
                &self.synapses[record.utility_synapse],
                floor,
            )).clamp(0.0,1.0))
    }

    pub fn phase_native_hypothesis_authority(
        &self,
        candidate_id: u64,
        applicability: f32,
    ) -> Option<f32> {
        let native = self.phase_native.as_ref()?;
        self.phase_hypothesis_authority_with_state(
            native,candidate_id,applicability
        )
    }

    pub fn phase_native_hypothesis_records(
        &self,
    ) -> Vec<PhaseHypothesisRecordInfo> {
        let Some(native)=self.phase_native.as_ref() else{return Vec::new();};
        let Some(ecology)=native.meta_control.as_ref()
            .and_then(|m|m.ecology.as_ref())
        else{return Vec::new();};
        ecology.records.iter().map(|record|{
            let authority=self.phase_hypothesis_authority_with_state(
                native,record.candidate_id,1.0
            ).unwrap_or(0.0);
            PhaseHypothesisRecordInfo {
                candidate_id:record.candidate_id,
                candidate_cell:record.candidate_cell,
                utility_synapse:record.utility_synapse,
                observations:record.observations,
                weight:self.synapses[record.utility_synapse].weight,
                authority_at_full_applicability:authority,
                dormant:authority <= ecology.config.dormancy_threshold,
            }
        }).collect()
    }

    pub fn phase_native_hypothesis_dormant(
        &self,
        candidate_id:u64,
    )->Option<bool>{
        self.phase_native_hypothesis_records().into_iter()
            .find(|r|r.candidate_id==candidate_id)
            .map(|r|r.dormant)
    }

    pub(super) fn is_native_hypothesis_synapse(&self,index:usize)->bool{
        self.phase_native.as_ref()
            .and_then(|n|n.meta_control.as_ref())
            .and_then(|m|m.ecology.as_ref())
            .map(|e|e.records.iter().any(|r|r.utility_synapse==index))
            .unwrap_or(false)
    }

    pub fn observe_phase_native_hypothesis_utility(
        &mut self,
        candidate_id:u64,
        factual_usefulness:f32,
    )->bool{
        if !factual_usefulness.is_finite()
            || !(0.0..=1.0).contains(&factual_usefulness)
        {return false;}

        let Some(mut native)=self.phase_native.take() else{return false;};
        let Some(ecology)=native.meta_control.as_ref()
            .and_then(|m|m.ecology.as_ref())
        else{
            self.phase_native=Some(native);
            return false;
        };
        let Some(index)=ecology.records.iter()
            .position(|r|r.candidate_id==candidate_id)
        else{
            self.phase_native=Some(native);
            return false;
        };
        let config=ecology.config.clone();
        let synapse_index=ecology.records[index].utility_synapse;
        let floor=native.config.coherence_floor;
        let predicted=conductance(
            &self.cells,&self.synapses[synapse_index],floor
        );
        let error=factual_usefulness-predicted;

        if config.learning_enabled{
            let syn=&mut self.synapses[synapse_index];
            if syn.plastic{
                syn.eligibility=1.0;
                syn.weight=(syn.weight+config.learning_rate*error)
                    .clamp(0.0,1.0);
                if config.phase_learning_enabled{
                    let target=wrap_phase(
                        self.cells[syn.to].phase-self.cells[syn.from].phase
                    );
                    syn.phase_offset=wrap_phase(
                        syn.phase_offset
                            +self.config.phase_learning_rate
                                *signed_phase_error(target,syn.phase_offset)
                    );
                }
                syn.confidence=(syn.confidence
                    +config.learning_rate*(1.0-syn.confidence))
                    .clamp(0.0,1.0);
            }
            native.meta_control.as_mut().expect("meta")
                .ecology.as_mut().expect("ecology")
                .records[index].observations=
                native.meta_control.as_ref().expect("meta")
                    .ecology.as_ref().expect("ecology")
                    .records[index].observations.saturating_add(1);
        }
        self.phase_native=Some(native);
        true
    }

    pub fn choose_phase_native_hypothesis(
        &self,
        proposals:&[PhaseHypothesisProposal],
    )->Option<PhaseHypothesisDecision>{
        if proposals.is_empty(){return None;}
        let native=self.phase_native.as_ref()?;
        let ecology=native.meta_control.as_ref()?.ecology.as_ref()?;

        let mut scored=Vec::with_capacity(proposals.len());
        for proposal in proposals{
            let authority=self.phase_hypothesis_authority_with_state(
                native,proposal.candidate_id,proposal.applicability
            )?;
            if authority > ecology.config.dormancy_threshold {
                scored.push((*proposal,authority));
            }
        }
        if scored.is_empty(){return None;}
        let max=scored.iter().map(|(_,a)|*a)
            .fold(f32::NEG_INFINITY,f32::max);
        let winners=scored.iter()
            .filter(|(_,a)|(*a-max).abs()<=1.0e-6)
            .collect::<Vec<_>>();
        if winners.len()!=1{return None;}
        let (proposal,authority)=winners[0];
        Some(PhaseHypothesisDecision{
            candidate_id:proposal.candidate_id,
            authority:*authority,
        })
    }
}
