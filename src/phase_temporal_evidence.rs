// TE1: phase-native, bounded temporal evidence from repeated noisy RAW sensing.
//
// The only externally supplied datum is an acquired raw sensory observation.
// No hidden-cause bit, correct response action or answer-program label enters
// these APIs. Evidence strength resides in the shared physical phase synapses.
// A small project manifest contains only addresses/observation budget, not
// a second source of truth for evidence or an answer table.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseTemporalEvidenceConfig {
    pub max_observations: u32,
    pub minimum_observations: u32,
    pub decisive_margin: f32,
}

impl Default for PhaseTemporalEvidenceConfig {
    fn default() -> Self {
        Self {
            max_observations: 8,
            minimum_observations: 3,
            decisive_margin: 0.125,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PhaseTemporalCue {
    source_cell: usize,
    evidence_synapse: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PhaseTemporalSampler {
    motor_action: usize,
    sensory_synapse: usize,
    trials: u32,
    distinctions: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseTemporalSensingDecision {
    pub action: usize,
    pub synapse: usize,
    pub learned_affordance: f32,
    pub missing_evidence: f32,
}

#[derive(Debug, Clone)]
pub(super) struct PhaseTemporalEvidenceState {
    config: PhaseTemporalEvidenceConfig,
    hub_cell: usize,
    cues: Vec<PhaseTemporalCue>,
    samplers: Vec<PhaseTemporalSampler>,
    observations: u32,
    episodes: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhaseTemporalEvidenceReadout {
    pub source_cells: [Option<usize>; 2],
    pub synapses: [Option<usize>; 2],
    pub physical_evidence: [f32; 2],
    pub observations: u32,
    pub evidence_margin: f32,
    pub winner_cell: Option<usize>,
    pub needs_more: bool,
}

impl EvoPhase {
    /// Opt in before target sensory evidence arrives. The project hub is a
    /// recruited physical cell, not a host Bayes table or action policy.
    pub fn enable_phase_native_temporal_evidence(
        &mut self,
        config: PhaseTemporalEvidenceConfig,
    ) -> bool {
        if !(2..=32).contains(&config.max_observations)
            || config.minimum_observations < 2
            || config.minimum_observations > config.max_observations
            || !config.decisive_margin.is_finite()
            || config.decisive_margin <= 0.0
            || config.decisive_margin >= 1.0
            || !self.config.structural_growth_enabled
        {
            return false;
        }
        let Some(mut state) = self.phase_native.take() else {
            return false;
        };
        if state.temporal_evidence.is_some() {
            self.phase_native = Some(state);
            return false;
        }
        let Some(hub_cell) = self.dormant_range()
            .find(|&i| !self.cells[i].recruited)
        else {
            self.phase_native = Some(state);
            return false;
        };
        self.cells[hub_cell].recruited = true;
        state.temporal_evidence = Some(PhaseTemporalEvidenceState {
            config,
            hub_cell,
            cues: Vec::new(),
            samplers: Vec::new(),
            observations: 0,
            episodes: 0,
        });
        self.phase_native = Some(state);
        true
    }

    pub(super) fn is_native_temporal_evidence_synapse(
        &self,
        synapse_index: usize,
    ) -> bool {
        self.phase_native.as_ref()
            .and_then(|state| state.temporal_evidence.as_ref())
            .map(|p|p.cues.iter().any(|c|c.evidence_synapse==synapse_index)
                || p.samplers.iter().any(|m|m.sensory_synapse==synapse_index))
            .unwrap_or(false)
    }

    /// Sensing is an actual external factual observation, not a guessed side.
    /// First two distinct raw abstract carrier cells acquire the source paths.
    /// Third unknown cue is refused rather than forced into one explanation.
    ///
    /// Transient evidence can accumulate while model learning is frozen;
    /// structural source birth, however, cannot occur during frozen learning.
    pub fn observe_phase_native_temporal_signal(
        &mut self,
        sensory: &[f32],
    ) -> bool {
        let Some(source) = self.phase_native_abstract_state(sensory) else {
            return false;
        };
        let Some(mut native) = self.phase_native.take() else {
            return false;
        };
        let Some(mut evidence) = native.temporal_evidence.take() else {
            self.phase_native = Some(native);
            return false;
        };
        if evidence.observations >= evidence.config.max_observations {
            native.temporal_evidence = Some(evidence);
            self.phase_native = Some(native);
            return false;
        }
        let chosen = evidence.cues.iter()
            .position(|cue|cue.source_cell==source.cell);
        let index = match chosen {
            Some(i) => i,
            None => {
                if evidence.cues.len()>=2 || !native.config.learning_enabled {
                    native.temporal_evidence = Some(evidence);
                    self.phase_native = Some(native);
                    return false;
                }
                let link = self.native_synapse(source.cell,evidence.hub_cell);
                let syn = &mut self.synapses[link];
                syn.phase_offset = wrap_phase(
                    self.cells[syn.to].phase-self.cells[syn.from].phase
                );
                syn.confidence = 1.0;
                evidence.cues.push(PhaseTemporalCue {
                    source_cell:source.cell,
                    evidence_synapse:link,
                });
                evidence.cues.len()-1
            }
        };
        let link = evidence.cues[index].evidence_synapse;
        let inc = 1.0/evidence.config.max_observations as f32;
        let syn = &mut self.synapses[link];
        // Only the actual synapse carries the episode's signal strength.
        // The metadata 'observations' enforces an absolute resource ceiling.
        syn.weight=(syn.weight+inc).min(1.0);
        syn.eligibility=1.0;
        evidence.observations+=1;
        native.temporal_evidence = Some(evidence);
        self.phase_native = Some(native);
        true
    }

    /// Learn an observation-producing motor only from factual PRE/action/POST.
    /// An observable contrast between two previously acquired cue cells is
    /// positive sensor evidence; a no-op/terminal/unrelated successor is not.
    /// No evaluator reward, action semantics or hidden variable is passed.
    pub fn observe_phase_native_sensing_affordance(
        &mut self,
        action: usize,
        pre_sensory: &[f32],
        post_sensory: &[f32],
    ) -> bool {
        if action>=self.config.motor_cells { return false; }
        let Some(pre)=self.phase_native_abstract_state(pre_sensory) else {
            return false;
        };
        let Some(post)=self.phase_native_abstract_state(post_sensory) else {
            return false;
        };
        let Some(mut native)=self.phase_native.take() else {
            return false;
        };
        let Some(mut evidence)=native.temporal_evidence.take() else {
            self.phase_native=Some(native);
            return false;
        };
        if !native.config.learning_enabled
            || evidence.cues.len()!=2
            || !evidence.cues.iter().any(|c|c.source_cell==pre.cell)
        {
            native.temporal_evidence=Some(evidence);
            self.phase_native=Some(native);
            return false;
        }
        let sampler_index = if let Some(i)=evidence.samplers.iter()
            .position(|m|m.motor_action==action) {
            i
        }else{
            let link=self.native_synapse(
                self.motor_cell(action),evidence.hub_cell
            );
            let syn=&mut self.synapses[link];
            syn.phase_offset=wrap_phase(
                self.cells[syn.to].phase-self.cells[syn.from].phase
            );
            syn.confidence=1.0;
            evidence.samplers.push(PhaseTemporalSampler{
                motor_action:action,
                sensory_synapse:link,
                trials:0,
                distinctions:0,
            });
            evidence.samplers.len()-1
        };
        let changed=pre.cell!=post.cell
            && evidence.cues.iter().any(|c|c.source_cell==post.cell);
        let sampler=&mut evidence.samplers[sampler_index];
        sampler.trials=sampler.trials.saturating_add(1);
        if changed {
            sampler.distinctions=sampler.distinctions.saturating_add(1);
            let syn=&mut self.synapses[sampler.sensory_synapse];
            // A repeatable sensor is an ACQUIRED physical motor->hub path.
            // The count only audits factual experience; the action winner
            // depends on the phase-sensitive synaptic conductance itself.
            syn.weight=(syn.weight
                +1.0/evidence.config.max_observations as f32).min(1.0);
            syn.eligibility=1.0;
        }
        native.temporal_evidence=Some(evidence);
        self.phase_native=Some(native);
        true
    }

    /// When the physical cue evidence is ambiguous, propose ONLY a sensor
    /// action whose own positive sensory affordance was acquired from real
    /// transitions. An action with no factual distinctions cannot win.
    /// The proposal is NOT an actuator permit or an action forced on U1.
    pub fn choose_phase_native_temporal_sensing_action(
        &self,
    ) -> Option<PhaseTemporalSensingDecision> {
        let belief=self.phase_native_temporal_evidence()?;
        if !belief.needs_more{return None;}
        let native=self.phase_native.as_ref()?;
        let state=native.temporal_evidence.as_ref()?;
        let floor=native.config.coherence_floor;
        let mut best:Option<(usize,usize,f32)>=None;
        let mut tied=false;
        for model in &state.samplers {
            if model.distinctions==0 {continue;}
            let confidence=conductance(
                &self.cells,&self.synapses[model.sensory_synapse],floor
            );
            if confidence<=1.0e-8 {continue;}
            match best {
                None=>{
                    best=Some((model.motor_action,model.sensory_synapse,confidence));
                    tied=false;
                }
                Some((_,_,score)) if confidence>score+1.0e-6=>{
                    best=Some((model.motor_action,model.sensory_synapse,confidence));
                    tied=false;
                }
                Some((_,_,score)) if (confidence-score).abs()<=1.0e-6=>{
                    tied=true;
                }
                _=>{}
            }
        }
        if tied {return None;}
        let (action,synapse,learned_affordance)=best?;
        Some(PhaseTemporalSensingDecision{
            action,synapse,learned_affordance,
            missing_evidence:(1.0-belief.evidence_margin).clamp(0.0,1.0),
        })
    }

    pub fn phase_native_temporal_evidence_enabled(&self)->bool{
        self.phase_native.as_ref()
            .and_then(|n|n.temporal_evidence.as_ref()).is_some()
    }

    pub fn phase_native_temporal_source_count(&self)->usize{
        self.phase_native.as_ref()
            .and_then(|n|n.temporal_evidence.as_ref())
            .map(|e|e.cues.len()).unwrap_or(0)
    }

    /// Phase-native sensing affordance for a given actually executed opaque
    /// motor. Even an unchanged cue can be a fresh independent observation
    /// when a motor has acquired a verified sensory-difference pathway.
    pub fn phase_native_temporal_action_affordance(
        &self,action:usize
    )->Option<f32>{
        let n=self.phase_native.as_ref()?;
        let p=n.temporal_evidence.as_ref()?;
        let motor=p.samplers.iter().find(|m|m.motor_action==action
            && m.distinctions>0)?;
        Some(conductance(
            &self.cells,&self.synapses[motor.sensory_synapse],
            n.config.coherence_floor
        ))
    }

    /// Explicit generic NEW EPISODE boundary resets the transient physical
    /// evidence without destroying the acquired receptor identities/synapses.
    /// It does not alter ordinary native transition learning or the goal.
    pub fn begin_phase_native_temporal_episode(&mut self) -> bool {
        let Some(native) = self.phase_native.as_mut() else {
            return false;
        };
        let Some(evidence) = native.temporal_evidence.as_mut() else {
            return false;
        };
        for cue in &evidence.cues {
            let syn = &mut self.synapses[cue.evidence_synapse];
            syn.weight=0.0;
            syn.eligibility=0.0;
        }
        evidence.observations=0;
        evidence.episodes=evidence.episodes.saturating_add(1);
        true
    }

    /// Carrier-native readout: changing or π-shifting a necessary physical
    /// synapse removes its influence; no metadata count can rescue the answer.
    pub fn phase_native_temporal_evidence(
        &self,
    ) -> Option<PhaseTemporalEvidenceReadout> {
        let native=self.phase_native.as_ref()?;
        let state=native.temporal_evidence.as_ref()?;
        let floor=native.config.coherence_floor;
        let mut signal=[0.0f32;2];
        let mut sources=[None;2];
        let mut synapses=[None;2];
        for (i,cue) in state.cues.iter().enumerate() {
            sources[i]=Some(cue.source_cell);
            synapses[i]=Some(cue.evidence_synapse);
            signal[i]=conductance(
                &self.cells,&self.synapses[cue.evidence_synapse],floor
            ).clamp(0.0,1.0);
        }
        let margin=(signal[0]-signal[1]).abs();
        let decisive = state.cues.len()==2
            && state.observations>=state.config.minimum_observations
            && margin+1.0e-7>=state.config.decisive_margin;
        let winner_cell = if decisive {
            if signal[0]>signal[1] {sources[0]} else {sources[1]}
        }else{
            None
        };
        Some(PhaseTemporalEvidenceReadout {
            source_cells:sources,
            synapses,
            physical_evidence:signal,
            observations:state.observations,
            evidence_margin:margin,
            winner_cell,
            needs_more:winner_cell.is_none()
                &&state.observations<state.config.max_observations,
        })
    }
}
