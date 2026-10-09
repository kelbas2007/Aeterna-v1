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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PhaseTemporalOutcomeLink {
    source_cell: usize,
    motor_action: usize,
    value_synapse: usize,
    observations: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseTemporalOutcomeDecision {
    pub action: usize,
    pub synapse: usize,
    pub learned_value: f32,
    pub source_cell: usize,
}

#[derive(Debug, Clone)]
pub(super) struct PhaseTemporalEvidenceState {
    config: PhaseTemporalEvidenceConfig,
    hub_cell: usize,
    cues: Vec<PhaseTemporalCue>,
    samplers: Vec<PhaseTemporalSampler>,
    outcomes: Vec<PhaseTemporalOutcomeLink>,
    observations: u32,
    episodes: u64,
    /// Opt-in carrier-owned exploration from factual motor coverage.
    autonomous_probe_enabled: bool,
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
            outcomes: Vec::new(),
            observations: 0,
            episodes: 0,
            autonomous_probe_enabled: false,
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
                || p.samplers.iter().any(|m|m.sensory_synapse==synapse_index)
                || p.outcomes.iter().any(|m|m.value_synapse==synapse_index))
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

    /// Opt in to evidence-driven unknown-motor coverage, not a prescribed
    /// action schedule. The physical sampling affordance still must be learned
    /// from a real permitted PRE/action/POST difference.
    pub fn set_phase_native_temporal_autonomous_probe(&mut self, enabled: bool)->bool{
        let Some(temporal)=self.phase_native.as_mut()
            .and_then(|n|n.temporal_evidence.as_mut()) else {return false;};
        temporal.autonomous_probe_enabled=enabled;
        true
    }

    pub fn phase_native_temporal_autonomous_probe(&self)->bool{
        self.phase_native.as_ref().and_then(|n|n.temporal_evidence.as_ref())
            .map(|t|t.autonomous_probe_enabled).unwrap_or(false)
    }

    /// An intrinsically uncertain organism can investigate an as-yet
    /// unverified opaque actuator. Pick the motor with least factual local
    /// coverage; never use latent classes, correct motor labels or reward maps.
    /// A conducting acquired sensor makes this cold probe unnecessary.
    /// Candidate is NOT a permit and must still compete in U1.
    pub fn choose_phase_native_temporal_unknown_probe(&self)
        ->Option<(usize,f32)>{
        let belief=self.phase_native_temporal_evidence()?;
        let native=self.phase_native.as_ref()?;
        let temporal=native.temporal_evidence.as_ref()?;
        if !temporal.autonomous_probe_enabled
            || temporal.cues.len()!=2 || !belief.needs_more
            || !native.config.learning_enabled {
            return None;
        }
        // Already-acquired physically conducting information paths use TE3.
        if temporal.samplers.iter().any(|model|
            model.distinctions>0
                && conductance(&self.cells,
                    &self.synapses[model.sensory_synapse],
                    native.config.coherence_floor)>1.0e-8
        ){return None;}
        let (motor,trials)=(0..self.config.motor_cells).map(|action|{
            let n=temporal.samplers.iter()
                .find(|m|m.motor_action==action)
                .map(|m|m.trials).unwrap_or(0);
            (action,n)
        }).min_by_key(|(action,trials)|(*trials,*action))?;
        Some((motor,(1.0/(1.0+trials as f32)).clamp(0.0,1.0)))
    }

    /// Acquired physical cue evidence limits certainty of an otherwise
    /// optimistic goal route. A conclusive belief (or exhausted evidence)
    /// restores normal goal value. Only used in explicit autonomous mode.
    pub fn phase_native_temporal_goal_evidence_coverage(&self)->Option<f32>{
        let native=self.phase_native.as_ref()?;
        let temporal=native.temporal_evidence.as_ref()?;
        if !temporal.autonomous_probe_enabled || temporal.cues.len()!=2 {
            return None;
        }
        let belief=self.phase_native_temporal_evidence()?;
        if !belief.needs_more {return Some(1.0);}
        let observed=(belief.observations as f32
            /temporal.config.minimum_observations as f32).clamp(0.0,1.0);
        let margin=(belief.evidence_margin
            /temporal.config.decisive_margin).clamp(0.0,1.0);
        Some((observed*margin).clamp(0.0,1.0))
    }

    /// In an uncertain episode a physically verified information-producing
    /// operation is the only currently supported way of changing the belief.
    /// Do not treat an unsupported optimistic goal path as an informed action.
    /// When belief becomes decisive, all normal motor proposals return.
    /// No source class, hidden cause, reward motor or host phase is inspected.
    pub fn phase_native_temporal_action_evidence_admissible(
        &self,action:usize
    )->bool{
        if !self.phase_native_temporal_autonomous_probe(){return true;}
        let Some(belief)=self.phase_native_temporal_evidence()
            else{return true;};
        if !belief.needs_more {
            // Once a physically decisive belief exists, execute its learned
            // factual outcome association rather than an unrelated inherited
            // optimistic route. If the necessary phase link disappears, this
            // constraint disappears too: metadata never stores a motor label.
            return self.choose_phase_native_temporal_outcome_action()
                .map(|policy|policy.action==action).unwrap_or(true);
        }
        let Some(sensor)=self.choose_phase_native_temporal_sensing_action()
            else{return true;};
        // If the physical sensor loses its conducting path, no metadata
        // whitelist may continue to force the same motor.
        sensor.action==action
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

    /// TE5: learn the factual value of an actually executed opaque motor
    /// given the *pre-action* physical distribution over acquired raw cues.
    /// No hidden side, goal-correct motor or symbolic class is accepted.
    /// This method is for true observed POST and bounded task consequence;
    /// integration under Human Protection must enforce that boundary.
    pub fn observe_phase_native_temporal_outcome(
        &mut self,
        action:usize,
        post_sensory:&[f32],
        task_outcome:f32,
    )->bool{
        if action>=self.config.motor_cells || !task_outcome.is_finite()
            || !(0.0..=1.0).contains(&task_outcome)
        {
            return false;
        }
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
        let ready=native.config.learning_enabled
            && evidence.cues.len()==2
            && evidence.observations>=1
            // A cue-to-cue observation is a sample, not a terminal result.
            && !evidence.cues.iter().any(|cue|cue.source_cell==post.cell);
        if !ready {
            native.temporal_evidence=Some(evidence);
            self.phase_native=Some(native);
            return false;
        }
        let floor=native.config.coherence_floor;
        let observations=evidence.cues.iter().map(|cue|
            conductance(&self.cells,
                &self.synapses[cue.evidence_synapse],floor)
        ).collect::<Vec<_>>();
        let mass=observations.iter().sum::<f32>();
        if mass<=1.0e-8 {
            native.temporal_evidence=Some(evidence);
            self.phase_native=Some(native);
            return false;
        }
        let source_cells=evidence.cues.iter().map(|c|c.source_cell)
            .collect::<Vec<_>>();
        let mut updated=0usize;
        for (cue_index,source_cell) in source_cells.into_iter().enumerate(){
            let credit=(observations[cue_index]/mass).clamp(0.0,1.0);
            if credit<=1.0e-8 {continue;}
            let policy_index=match evidence.outcomes.iter()
                .position(|w|w.source_cell==source_cell
                    && w.motor_action==action)
            {
                Some(i)=>i,
                None=>{
                    let motor=self.motor_cell(action);
                    let link=self.native_synapse(source_cell,motor);
                    let syn=&mut self.synapses[link];
                    syn.phase_offset=wrap_phase(
                        self.cells[syn.to].phase-self.cells[syn.from].phase
                    );
                    syn.confidence=1.0;
                    evidence.outcomes.push(PhaseTemporalOutcomeLink {
                        source_cell,motor_action:action,
                        value_synapse:link,observations:0,
                    });
                    evidence.outcomes.len()-1
                }
            };
            let value=&mut evidence.outcomes[policy_index];
            value.observations=value.observations.saturating_add(1);
            let syn=&mut self.synapses[value.value_synapse];
            let learning_rate=(0.35*credit).clamp(0.0,0.35);
            syn.weight += learning_rate*(task_outcome-syn.weight);
            syn.weight=syn.weight.clamp(0.0,1.0);
            syn.eligibility=credit;
            updated+=1;
        }
        native.temporal_evidence=Some(evidence);
        self.phase_native=Some(native);
        updated>0
    }

    /// TE5: physically acquired belief->motor consequence readout. An
    /// undecided/tied native physical belief cannot silently become a label.
    /// The unique winner must have phase-conducting useful reward evidence.
    pub fn choose_phase_native_temporal_outcome_action(
        &self
    )->Option<PhaseTemporalOutcomeDecision>{
        let belief=self.phase_native_temporal_evidence()?;
        let source_cell=belief.winner_cell?;
        let native=self.phase_native.as_ref()?;
        let evidence=native.temporal_evidence.as_ref()?;
        let floor=native.config.coherence_floor;
        let mut best:Option<PhaseTemporalOutcomeDecision>=None;
        let mut tied=false;
        for candidate in evidence.outcomes.iter()
            .filter(|w|w.source_cell==source_cell
                && w.observations>0)
        {
            let credit=conductance(&self.cells,
                &self.synapses[candidate.value_synapse],floor);
            if credit<=0.05 {continue;}
            match best {
                None=>{
                    best=Some(PhaseTemporalOutcomeDecision{
                        action:candidate.motor_action,
                        synapse:candidate.value_synapse,
                        learned_value:credit,
                        source_cell,
                    });
                    tied=false;
                }
                Some(prior) if credit>prior.learned_value+1.0e-6=>{
                    best=Some(PhaseTemporalOutcomeDecision{
                        action:candidate.motor_action,
                        synapse:candidate.value_synapse,
                        learned_value:credit,
                        source_cell,
                    });
                    tied=false;
                }
                Some(prior) if (credit-prior.learned_value).abs()<=1.0e-6=>{
                    tied=true;
                }
                _=>{}
            }
        }
        if tied {None}else{best}
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
