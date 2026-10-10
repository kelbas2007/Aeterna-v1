// INNATE-SCAFFOLD-1: functional, testable "at birth" learning biases.
// Not a dictionary of true facts about a world, and not a claim that
// newborn humans possess these eight adult concepts.
//
// These circuits prepare learning, attention, organism regulation, and
// causal sensorimotor contingency. Social and speech readiness require
// actual trusted audiovisual inputs: never hallucinate them from grid bits.
const INNATE_PRIORS:usize=8;
const INNATE_CONTINUITY:usize=0;
const INNATE_ORIENTING:usize=1;
const INNATE_AGENCY:usize=2;
const INNATE_HABITUATION:usize=3;
const INNATE_MAGNITUDE:usize=4;
const INNATE_REGULATION:usize=5;
const INNATE_SOCIAL_READY:usize=6;
const INNATE_SPEECH_READY:usize=7;

#[derive(Debug,Clone)]
struct PhaseInnateScaffold {
    connections:[usize;INNATE_PRIORS],
    // These are preconfigured *functional predispositions*, not labels.
    sensory_change_mean:f32,
    motor_contingency:Vec<f32>,
    motor_observations:Vec<u32>,
    habituation:f32,
    intrinsic_energy:f32,
    observed_events:u64,
    acquired_associations:u64,
    social_modality_ready:bool,
    speech_modality_ready:bool,
    // Episode-only subjective recent perception, cleared on reset/restore.
    last_sense:Option<Vec<f32>>,
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct PhaseInnateReadout {
    pub continuity_prior:f32,
    pub orienting_prior:f32,
    pub agency_prior:f32,
    pub habituation_prior:f32,
    pub approximate_magnitude_prior:f32,
    pub regulation_prior:f32,
    pub social_readiness:f32,
    pub speech_readiness:f32,
    pub observed_events:u64,
    pub acquired_associations:u64,
    pub intrinsic_energy:f32,
}

impl EvoPhase {
    pub fn enable_phase_native_innate_scaffold(&mut self)->bool {
        if self.phase_native.as_ref()
            .is_none_or(|n|n.innate_scaffold.is_some())
            || self.config.dormant_cells<INNATE_PRIORS {
            return false;
        }
        let mut links=Vec::new();
        for i in 0..INNATE_PRIORS {
            let Some(cell)=self.dormant_range()
                .find(|&j|!self.cells[j].recruited) else{return false;};
            self.cells[cell].recruited=true;
            let source=i%self.config.sensory_cells;
            let link=self.native_synapse(source,cell);
            let syn=&mut self.synapses[link];
            syn.weight=0.55;
            syn.confidence=1.0;
            syn.eligibility=1.0;
            syn.phase_offset=wrap_phase(
                self.cells[syn.to].phase-self.cells[syn.from].phase
            );
            links.push(link);
        }
        let Ok(links):Result<[usize;INNATE_PRIORS],_>=links.try_into()
            else{return false;};
        let n=self.config.motor_cells;
        self.phase_native.as_mut().expect("native")
            .innate_scaffold=Some(PhaseInnateScaffold{
                connections:links,
                sensory_change_mean:0.0,
                motor_contingency:vec![0.0;n],
                motor_observations:vec![0;n],
                habituation:0.0,
                intrinsic_energy:1.0,
                observed_events:0,
                acquired_associations:0,
                social_modality_ready:false,
                speech_modality_ready:false,
                last_sense:None,
            });
        true
    }

    pub fn phase_native_innate_enabled(&self)->bool{
        self.phase_native.as_ref()
            .is_some_and(|n|n.innate_scaffold.is_some())
    }

    pub fn phase_native_innate_synapse(&self,prior:usize)->Option<usize>{
        self.phase_native.as_ref()?.innate_scaffold.as_ref()?
            .connections.get(prior).copied()
    }

    pub fn is_phase_native_innate_synapse(&self,link:usize)->bool{
        self.phase_native.as_ref().and_then(|n|n.innate_scaffold.as_ref())
            .is_some_and(|s|s.connections.contains(&link))
    }

    pub fn phase_native_innate_readout(&self)->Option<PhaseInnateReadout>{
        let n=self.phase_native.as_ref()?;
        let state=n.innate_scaffold.as_ref()?;
        let gate=|i:usize|conductance(&self.cells,
            &self.synapses[state.connections[i]],n.config.coherence_floor);
        Some(PhaseInnateReadout{
            continuity_prior:gate(INNATE_CONTINUITY),
            orienting_prior:gate(INNATE_ORIENTING),
            agency_prior:gate(INNATE_AGENCY),
            habituation_prior:gate(INNATE_HABITUATION),
            approximate_magnitude_prior:gate(INNATE_MAGNITUDE),
            regulation_prior:gate(INNATE_REGULATION),
            // Readiness is present at birth; no social or speech *recognition*
            // is claimed or activated without appropriate input modalities.
            social_readiness:gate(INNATE_SOCIAL_READY),
            speech_readiness:gate(INNATE_SPEECH_READY),
            observed_events:state.observed_events,
            acquired_associations:state.acquired_associations,
            intrinsic_energy:state.intrinsic_energy,
        })
    }

    /// "Born" with orienting biases; not born having seen any external scene.
    pub fn begin_phase_native_innate_episode(&mut self,raw:&[f32])->bool{
        if raw.len()!=self.config.sensory_cells
            || raw.iter().any(|x|!x.is_finite()){return false;}
        let Some(n)=self.phase_native.as_mut() else{return false;};
        let Some(s)=n.innate_scaffold.as_mut() else{return false;};
        s.last_sense=Some(raw.to_vec());
        // Energy is a virtual regulation proxy and not a simulator's
        // hunger/oxygen/temperature sensor.
        s.intrinsic_energy=(s.intrinsic_energy+0.25).min(1.0);
        true
    }

    /// The only route to acquired sensorimotor knowledge is a factual
    /// protected action with real before/after sensations.
    pub fn observe_phase_native_innate_contingency(
        &mut self,action:usize,pre:&[f32],post:&[f32]
    )->bool{
        if action>=self.config.motor_cells
            || pre.len()!=self.config.sensory_cells
            || post.len()!=pre.len()
            || pre.iter().chain(post).any(|v|!v.is_finite()) {
            return false;
        }
        let Some(native)=self.phase_native.as_mut() else{return false;};
        let Some(state)=native.innate_scaffold.as_mut() else{return false;};
        let changed=pre.iter().zip(post).filter(|(a,b)|(*a-*b).abs()>0.001)
            .count() as f32/(pre.len() as f32);
        // Online subjective perception remains available under freeze.
        state.last_sense=Some(post.to_vec());
        if !native.config.learning_enabled {return true;}
        let old=state.sensory_change_mean;
        state.sensory_change_mean=0.90*old+0.10*changed;
        state.motor_contingency[action]=
            0.82*state.motor_contingency[action]+0.18*changed;
        state.motor_observations[action]=
            state.motor_observations[action].saturating_add(1);
        state.habituation=(0.86*state.habituation
            +0.14*(1.0-changed)).clamp(0.0,1.0);
        state.intrinsic_energy=(state.intrinsic_energy-0.0003).max(0.05);
        state.observed_events=state.observed_events.saturating_add(1);
        if changed>0.0 {
            state.acquired_associations=state.acquired_associations.saturating_add(1);
        }
        true
    }

    /// Directionless, world-independent biases to investigate an opaque
    /// action when its sensorimotor effects are poorly known. No action-role
    /// labels or ready-made concept facts. Physical lesions change bias.
    pub fn phase_native_innate_action_bias(&self,action:usize)->f32{
        let Some(n)=self.phase_native.as_ref() else{return 0.0;};
        let Some(s)=n.innate_scaffold.as_ref() else{return 0.0;};
        let Some(&trials)=s.motor_observations.get(action) else{return 0.0;};
        let novelty=conductance(&self.cells,
            &self.synapses[s.connections[INNATE_ORIENTING]],
            n.config.coherence_floor);
        let agency=conductance(&self.cells,
            &self.synapses[s.connections[INNATE_AGENCY]],
            n.config.coherence_floor);
        let regulation=conductance(&self.cells,
            &self.synapses[s.connections[INNATE_REGULATION]],
            n.config.coherence_floor);
        let curiosity=(1.0/(1.0+trials as f32).sqrt())
            *(1.0-0.6*s.habituation).max(0.05);
        let contingency=s.motor_contingency[action].clamp(0.0,1.0);
        let energy=s.intrinsic_energy;
        (0.15*novelty*curiosity+0.08*agency*contingency
            -0.03*regulation*(1.0-energy)).clamp(-0.03,0.23)
    }
}
