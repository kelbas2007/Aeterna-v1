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
    /// Factual terminal/motor coverage conditioned on the physical belief.
    belief_trials: [u32;2],
}

#[derive(Debug, Clone)]
struct PhaseTemporalPendingChain {
    first_action: usize,
    marker_cell: usize,
    trials: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PhaseTemporalChain {
    first_action: usize,
    second_action: usize,
    marker_cell: usize,
    entry_synapse: usize,
    read_synapse: usize,
    support: u32,
}

/// A factual state-conditioned motor transition in the EXISTING carrier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PhaseTemporalTransition {
    from_cell:usize,
    motor_action:usize,
    to_cell:usize,
    entry_synapse:usize,
    exit_synapse:usize,
    observations:u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PhaseTemporalActionTrial {
    from_cell:usize,
    motor_action:usize,
    observations:u32,
}

/// Factual positive goal outcome. The physical motor->observed-goal
/// synapse, not a fixed motor-role label, gates future frontier inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PhaseTemporalGoalWitness {
    motor_action:usize,
    post_cell:usize,
    outcome_synapse:usize,
    observations:u32,
}

/// Carrier-owned, bounded route from a factual current state to the raw
/// goal, read through physically conducting state->motor->state links.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseTemporalGoalPlan {
    pub action:usize,
    pub synapse:usize,
    pub steps:usize,
    pub strength:f32,
    pub target_cell:usize,
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
    chain_learning_enabled: bool,
    pending_chain: Option<PhaseTemporalPendingChain>,
    chains: Vec<PhaseTemporalChain>,
    /// Open development: the same evidence carrier handles variable depth.
    multistep_enabled:bool,
    goal_replan_enabled:bool,
    cold_goal_acquisition:bool,
    executive_goal_cell:Option<usize>,
    transitions:Vec<PhaseTemporalTransition>,
    action_trials:Vec<PhaseTemporalActionTrial>,
    goal_witnesses:Vec<PhaseTemporalGoalWitness>,
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
            chain_learning_enabled: false,
            pending_chain: None,
            chains: Vec::new(),
            multistep_enabled:false,
            goal_replan_enabled:false,
            cold_goal_acquisition:false,
            executive_goal_cell:None,
            transitions:Vec::new(),
            action_trials:Vec::new(),
            goal_witnesses:Vec::new(),
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
                || p.outcomes.iter().any(|m|m.value_synapse==synapse_index)
                || p.chains.iter().any(|m|m.entry_synapse==synapse_index
                    || m.read_synapse==synapse_index)
                || p.transitions.iter().any(|e|e.entry_synapse==synapse_index
                    || e.exit_synapse==synapse_index)
                || p.goal_witnesses.iter().any(|g|g.outcome_synapse==synapse_index))
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
        // Snapshot the factual pre-action belief BEFORE moving native state
        // out for mutation; otherwise the readout would always be None.
        let belief_source=self.phase_native_temporal_evidence()
            .and_then(|readout|readout.winner_cell);
        let Some(mut native)=self.phase_native.take() else {
            return false;
        };
        let Some(mut evidence)=native.temporal_evidence.take() else {
            self.phase_native=Some(native);
            return false;
        };
        // Multistep extension: acquire a general factual state/motor/state
        // relation. No source, goal, correct motor, future trace or evaluator
        // phase is imported. All learned value resides in conducting physical
        // source->motor->destination synapses, not in the address record.
        if evidence.multistep_enabled {
            // The old one-step TE5 reward explorer reads belief-conditioned
            // factual action counts from sampler links. Never lose that
            // information merely because the same action also participates
            // in a new variable-depth transition graph. Without this, UCB
            // permanently treats a repeatedly chosen terminal motor as
            // untried and collapses to an opaque index tie.
            if let Some(source)=belief_source {
                if evidence.cues.iter().any(|c|c.source_cell==pre.cell) {
                    if let Some(cue_index)=evidence.cues.iter()
                        .position(|c|c.source_cell==source)
                    {
                        let i=if let Some(i)=evidence.samplers.iter()
                            .position(|m|m.motor_action==action){i}else{
                            let link=self.native_synapse(
                                self.motor_cell(action),evidence.hub_cell
                            );
                            evidence.samplers.push(PhaseTemporalSampler{
                                motor_action:action,sensory_synapse:link,
                                trials:0,distinctions:0,belief_trials:[0;2]
                            });
                            evidence.samplers.len()-1
                        };
                        evidence.samplers[i].trials=
                            evidence.samplers[i].trials.saturating_add(1);
                        evidence.samplers[i].belief_trials[cue_index]=
                            evidence.samplers[i].belief_trials[cue_index]
                                .saturating_add(1);
                    }
                }
            }
            let trial=if let Some(t)=evidence.action_trials.iter_mut()
                .find(|t|t.from_cell==pre.cell&&t.motor_action==action)
            {t}else{
                evidence.action_trials.push(PhaseTemporalActionTrial{
                    from_cell:pre.cell,motor_action:action,observations:0
                });
                evidence.action_trials.last_mut().expect("factual trial")
            };
            trial.observations=trial.observations.saturating_add(1);
            if evidence.goal_replan_enabled && native.config.learning_enabled {
                // Factual contradiction to an expected state/action outcome:
                // retire every stale competing PHYSICAL path from that same
                // PRE/motor pair. A no-op is also genuine counterevidence.
                // This rule is opt-in to deterministic causal planning and
                // does not rewrite noisy multi-outcome sensory models.
                for edge in evidence.transitions.iter().filter(|e|
                    e.from_cell==pre.cell && e.motor_action==action
                        && e.to_cell!=post.cell
                ){
                    self.synapses[edge.entry_synapse].weight=0.0;
                    self.synapses[edge.exit_synapse].weight=0.0;
                    self.synapses[edge.entry_synapse].eligibility=0.0;
                    self.synapses[edge.exit_synapse].eligibility=0.0;
                }
            }
            if native.config.learning_enabled && pre.cell!=post.cell {
                let index=if let Some(i)=evidence.transitions.iter().position(
                    |e|e.from_cell==pre.cell&&e.to_cell==post.cell
                        &&e.motor_action==action
                ){i}else{
                    let entry=self.native_synapse(
                        pre.cell,self.motor_cell(action)
                    );
                    let exit=self.native_synapse(
                        self.motor_cell(action),post.cell
                    );
                    if evidence.transitions.len()>=self.config.motor_cells*32 {
                        native.temporal_evidence=Some(evidence);
                        self.phase_native=Some(native);
                        return false;
                    }
                    evidence.transitions.push(PhaseTemporalTransition{
                        from_cell:pre.cell,motor_action:action,
                        to_cell:post.cell,entry_synapse:entry,
                        exit_synapse:exit,observations:0
                    });
                    evidence.transitions.len()-1
                };
                let edge=&mut evidence.transitions[index];
                edge.observations=edge.observations.saturating_add(1);
                for &syn_index in &[edge.entry_synapse,edge.exit_synapse] {
                    let syn=&mut self.synapses[syn_index];
                    syn.phase_offset=wrap_phase(
                        self.cells[syn.to].phase-self.cells[syn.from].phase
                    );
                    syn.confidence=1.0;
                    syn.weight=(syn.weight
                        +1.0/evidence.config.max_observations as f32)
                        .min(1.0);
                    syn.eligibility=1.0;
                }
            }
            native.temporal_evidence=Some(evidence);
            self.phase_native=Some(native);
            return true;
        }
        // Generic two-step causal affordance, opt-in. An opaque motor
        // producing a distinct factual non-cue state becomes a pending
        // hypothesis. A *different* motor that physically returns that state
        // to an acquired raw cue proves the full chain. Neither action is
        // named, taught or selected by the external environment.
        if evidence.chain_learning_enabled && native.config.learning_enabled
            && evidence.cues.len()==2
        {
            let pre_cue=evidence.cues.iter().any(|c|c.source_cell==pre.cell);
            let post_cue=evidence.cues.iter().any(|c|c.source_cell==post.cell);
            if let Some(pending)=evidence.pending_chain.clone() {
                if pre.cell==pending.marker_cell {
                    if let Some(ref mut active)=evidence.pending_chain {
                        active.trials[action]=active.trials[action].saturating_add(1);
                    }
                    if post_cue && pending.first_action!=action {
                        let index=if let Some(i)=evidence.chains.iter().position(|c|
                            c.first_action==pending.first_action
                                && c.second_action==action
                                && c.marker_cell==pending.marker_cell
                        ){ i } else {
                            let entry=self.native_synapse(
                                self.motor_cell(pending.first_action),
                                pending.marker_cell
                            );
                            let read=self.native_synapse(
                                self.motor_cell(action),evidence.hub_cell
                            );
                            evidence.chains.push(PhaseTemporalChain{
                                first_action:pending.first_action,
                                second_action:action,
                                marker_cell:pending.marker_cell,
                                entry_synapse:entry,
                                read_synapse:read,
                                support:0,
                            });
                            evidence.chains.len()-1
                        };
                        let chain=&mut evidence.chains[index];
                        chain.support=chain.support.saturating_add(1);
                        for syn_index in [chain.entry_synapse,chain.read_synapse] {
                            let syn=&mut self.synapses[syn_index];
                            syn.phase_offset=wrap_phase(
                                self.cells[syn.to].phase-self.cells[syn.from].phase
                            );
                            syn.confidence=1.0;
                            syn.weight=(syn.weight
                                +1.0/evidence.config.max_observations as f32)
                                .min(1.0);
                            syn.eligibility=1.0;
                        }
                        evidence.pending_chain=None;
                    }
                    native.temporal_evidence=Some(evidence);
                    self.phase_native=Some(native);
                    return true;
                }
            }
            if pre_cue && !post_cue && pre.cell!=post.cell {
                let index=if let Some(i)=evidence.samplers.iter()
                    .position(|m|m.motor_action==action){i}else{
                    let link=self.native_synapse(
                        self.motor_cell(action),evidence.hub_cell
                    );
                    evidence.samplers.push(PhaseTemporalSampler{
                        motor_action:action,sensory_synapse:link,
                        trials:0,distinctions:0,belief_trials:[0;2]
                    });
                    evidence.samplers.len()-1
                };
                evidence.samplers[index].trials=
                    evidence.samplers[index].trials.saturating_add(1);
                // A novel external state can represent an intermediate or
                // an eventual terminal outcome. Until later factual evidence
                // disambiguates the two, preserve BOTH learning pathways.
                // The chain hypothesis must not erase belief-conditioned
                // factual motor coverage used by terminal reward search.
                if let Some(source)=belief_source {
                    if let Some(i)=evidence.cues.iter()
                        .position(|cue|cue.source_cell==source) {
                        evidence.samplers[index].belief_trials[i]=
                            evidence.samplers[index].belief_trials[i]
                                .saturating_add(1);
                    }
                }
                evidence.pending_chain=Some(PhaseTemporalPendingChain{
                    first_action:action,
                    marker_cell:post.cell,
                    trials:vec![0;self.config.motor_cells],
                });
                native.temporal_evidence=Some(evidence);
                self.phase_native=Some(native);
                return true;
            }
        }
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
                belief_trials:[0;2],
            });
            evidence.samplers.len()-1
        };
        let changed=pre.cell!=post.cell
            && evidence.cues.iter().any(|c|c.source_cell==post.cell);
        let belief_index=belief_source.and_then(|source|
            evidence.cues.iter().position(|cue|cue.source_cell==source)
        );
        let sampler=&mut evidence.samplers[sampler_index];
        sampler.trials=sampler.trials.saturating_add(1);
        if let Some(i)=belief_index {
            sampler.belief_trials[i]=sampler.belief_trials[i].saturating_add(1);
        }
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

    /// Bounded graph traversal through physical, state-conditioned factual
    /// synapses. Any change to an essential source->motor or motor->state
    /// synapse invalidates the route. Each node is an actually acquired
    /// abstract sensory state; the graph carries no evaluator action labels.
    fn phase_native_temporal_multistep_route(
        &self, origin:usize, targets:&[usize], min_steps:usize
    )->Option<(usize,usize,f32,usize)>{
        let native=self.phase_native.as_ref()?;
        let evidence=native.temporal_evidence.as_ref()?;
        if !evidence.multistep_enabled{return None;}
        let floor=native.config.coherence_floor;
        let mut frontier=std::collections::VecDeque::new();
        frontier.push_back((origin,None,1.0f32,0usize,vec![origin]));
        let mut considered=0usize;
        let mut best:Option<(usize,usize,f32,usize)>=None;
        while let Some((cell,first,strength,depth,visited))=frontier.pop_front(){
            if considered>=1024 {break;}
            considered+=1;
            if depth>=8 {continue;}
            if best.is_some_and(|b|depth>=b.3){continue;}
            for edge in evidence.transitions.iter().filter(|e|
                e.from_cell==cell && e.observations>0
                && !self.phase_native_temporal_rewarded_motor(e.motor_action)
            ){
                let c0=conductance(
                    &self.cells,&self.synapses[edge.entry_synapse],floor
                );
                let c1=conductance(
                    &self.cells,&self.synapses[edge.exit_synapse],floor
                );
                let physical=strength.min(c0).min(c1);
                if physical<=1.0e-8 {continue;}
                let nextdepth=depth+1;
                let initial=first.unwrap_or((edge.motor_action,edge.entry_synapse));
                if nextdepth>=min_steps &&targets.contains(&edge.to_cell){
                    if best.is_none_or(|b|nextdepth<b.3
                        ||(nextdepth==b.3&&physical>b.2+1.0e-6)){
                        best=Some((initial.0,initial.1,physical,nextdepth));
                    }
                }else if !visited.contains(&edge.to_cell)
                    && nextdepth<8
                {
                    let mut seen=visited.clone();
                    seen.push(edge.to_cell);
                    frontier.push_back((
                        edge.to_cell,Some(initial),physical,nextdepth,seen
                    ));
                }
            }
        }
        best
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
        if state.multistep_enabled {
            if let Some(real)=self.current_real.as_ref() {
                if let Some(current)=self.phase_native_abstract_state(&real.sensory) {
                    let sources=state.cues.iter()
                        .map(|cue|cue.source_cell).collect::<Vec<_>>();
                    if let Some((action,synapse,strength,_steps))=
                        self.phase_native_temporal_multistep_route(
                            current.cell,&sources,1
                        )
                    {
                        return Some(PhaseTemporalSensingDecision{
                            action,synapse,learned_affordance:strength,
                            missing_evidence:(1.0-belief.evidence_margin)
                                .clamp(0.0,1.0),
                        });
                    }
                }
            }
        }
        if state.chain_learning_enabled {
            if let Some(real)=self.current_real.as_ref() {
                if let Some(entry)=self.phase_native_abstract_state(&real.sensory) {
                    let mut acquired:Option<PhaseTemporalSensingDecision>=None;
                    for chain in &state.chains {
                        let first=conductance(&self.cells,
                            &self.synapses[chain.entry_synapse],floor);
                        let second=conductance(&self.cells,
                            &self.synapses[chain.read_synapse],floor);
                        let strength=first.min(second);
                        if strength<=1.0e-8 {continue;}
                        let (motor,syn)=if entry.cell==chain.marker_cell {
                            (chain.second_action,chain.read_synapse)
                        } else if state.cues.iter().any(|c|c.source_cell==entry.cell) {
                            (chain.first_action,chain.entry_synapse)
                        } else {continue;};
                        if acquired.as_ref().map(|a|
                            strength>a.learned_affordance+1.0e-6
                        ).unwrap_or(true) {
                            acquired=Some(PhaseTemporalSensingDecision{
                                action:motor,synapse:syn,
                                learned_affordance:strength,
                                missing_evidence:(1.0-belief.evidence_margin)
                                    .clamp(0.0,1.0),
                            });
                        }
                    }
                    if acquired.is_some(){return acquired;}
                }
            }
        }
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

    /// Opt-in generalization of a single-step sensory affordance to a
    /// physically learned two-motor chain with an observed intermediate.
    /// This does not expose the required motor identities to cognition.
    pub fn set_phase_native_temporal_chain_learning(&mut self, enabled:bool)->bool{
        let Some(state)=self.phase_native.as_mut()
            .and_then(|n|n.temporal_evidence.as_mut()) else {return false;};
        state.chain_learning_enabled=enabled;
        if !enabled {state.pending_chain=None;}
        true
    }

    /// A permitted, actually achieved full task reward demonstrates that the
    /// corresponding opaque motor can execute a GOAL, not gather more clues.
    /// Only bounded factual outcome 1.0 is accepted; a zero-reward action is
    /// not silently classified as a failure or terminal command.
    pub fn observe_phase_native_temporal_full_goal_outcome(
        &mut self, action:usize, post_sensory:&[f32], utility:f32
    )->bool{
        if !utility.is_finite()||utility<1.0-1.0e-6||utility>1.0
            || action>=self.config.motor_cells {return false;}
        let Some(post)=self.phase_native_abstract_state(post_sensory)
            else{return false;};
        let Some(mut native)=self.phase_native.take() else{return false;};
        let Some(mut t)=native.temporal_evidence.take() else{
            self.phase_native=Some(native);return false;
        };
        if !t.multistep_enabled||!native.config.learning_enabled {
            native.temporal_evidence=Some(t);
            self.phase_native=Some(native);return false;
        }
        let index=if let Some(i)=t.goal_witnesses.iter()
            .position(|g|g.motor_action==action&&g.post_cell==post.cell)
        {i}else{
            let physical=self.native_synapse(
                self.motor_cell(action),post.cell
            );
            t.goal_witnesses.push(PhaseTemporalGoalWitness{
                motor_action:action,post_cell:post.cell,
                outcome_synapse:physical,observations:0,
            });
            t.goal_witnesses.len()-1
        };
        let witness=&mut t.goal_witnesses[index];
        witness.observations=witness.observations.saturating_add(1);
        let syn=&mut self.synapses[witness.outcome_synapse];
        syn.phase_offset=wrap_phase(
            self.cells[syn.to].phase-self.cells[syn.from].phase
        );
        syn.confidence=1.0;
        syn.weight=(syn.weight+0.25).clamp(0.0,1.0);
        syn.eligibility=1.0;
        native.temporal_evidence=Some(t);
        self.phase_native=Some(native);
        true
    }

    fn phase_native_temporal_rewarded_motor(&self,action:usize)->bool{
        let Some(native)=self.phase_native.as_ref() else{return false;};
        let Some(t)=native.temporal_evidence.as_ref() else{return false;};
        t.multistep_enabled&&t.goal_witnesses.iter().any(|w|
            w.motor_action==action && w.observations>0
                && conductance(&self.cells,
                    &self.synapses[w.outcome_synapse],
                    native.config.coherence_floor)>1.0e-8
        )
    }

    /// Causal lesion address for factual full-goal reward evidence only.
    pub fn phase_native_temporal_reward_synapse(&self,action:usize)
        ->Option<usize>{
        let t=self.phase_native.as_ref()?.temporal_evidence.as_ref()?;
        t.goal_witnesses.iter().find(|w|w.motor_action==action)
            .map(|w|w.outcome_synapse)
    }

    pub fn phase_native_temporal_rewarded_action_count(&self)->usize{
        let Some(t)=self.phase_native.as_ref()
            .and_then(|n|n.temporal_evidence.as_ref()) else{return 0;};
        (0..self.config.motor_cells).filter(|a|
            t.multistep_enabled&&self.phase_native_temporal_rewarded_motor(*a)
        ).count()
    }

    /// Generalized, bounded multistep transitions: never a motor-role table.
    /// Requires the existing temporal evidence state and acquired raw cues.
    pub fn set_phase_native_temporal_multistep(&mut self,enabled:bool)->bool{
        let Some(state)=self.phase_native.as_mut()
            .and_then(|n|n.temporal_evidence.as_mut()) else{return false;};
        state.multistep_enabled=enabled;
        true
    }

    /// Opt-in goal reasoning over the same acquired physical transition
    /// graph. No external plan, motor-role semantics, or goal oracle supplied.
    pub fn set_phase_native_temporal_goal_replanning(&mut self,enabled:bool)->bool{
        let Some(t)=self.phase_native.as_mut()
            .and_then(|n|n.temporal_evidence.as_mut()) else{return false;};
        if enabled && !t.multistep_enabled{return false;}
        t.goal_replan_enabled=enabled;
        true
    }

    /// Experimental autonomous acquisition for a goal the user already
    /// supplied as a RAW sensory target. No state/action route is provided.
    /// The same carrier stores its live objective, per-state motor trials
    /// and physically acquired PRE/motor/POST links.
    pub fn set_phase_native_temporal_cold_goal_acquisition(
        &mut self,enabled:bool
    )->bool{
        let Some(t)=self.phase_native.as_mut()
            .and_then(|n|n.temporal_evidence.as_mut()) else{return false;};
        if enabled&&!t.goal_replan_enabled{return false;}
        t.cold_goal_acquisition=enabled;
        if !enabled{t.executive_goal_cell=None;}
        true
    }

    pub fn set_phase_native_temporal_executive_goal(
        &mut self,goal:&[f32]
    )->bool{
        let Some(cell)=self.phase_native_abstract_state(goal)
            .map(|state|state.cell) else{return false;};
        let Some(t)=self.phase_native.as_mut()
            .and_then(|n|n.temporal_evidence.as_mut()) else{return false;};
        if !t.cold_goal_acquisition{return false;}
        t.executive_goal_cell=Some(cell);
        true
    }

    /// An optimistic state-conditioned motor coverage policy. It searches
    /// only PHYSICALLY CONDUCTING actual transition links; no hidden world
    /// depth, correct motor identity, taught order or synthetic plan.
    /// Every action is still protected and its observed POST is learnt by
    /// the existing phase-native transition code.
    pub fn choose_phase_native_temporal_goal_frontier(
        &self
    )->Option<(usize,f32,usize)>{
        let native=self.phase_native.as_ref()?;
        let t=native.temporal_evidence.as_ref()?;
        if !t.cold_goal_acquisition || !t.goal_replan_enabled
            || !native.config.learning_enabled{return None;}
        let goal=t.executive_goal_cell?;
        let sensory=&self.current_real.as_ref()?.sensory;
        let origin=self.phase_native_abstract_state(sensory)?.cell;
        if origin==goal{return None;}
        let floor=native.config.coherence_floor;
        let mut queue=std::collections::VecDeque::new();
        queue.push_back((origin,None,0usize,vec![origin]));
        let mut considered=0usize;
        let mut best:Option<(usize,f32,usize)>=None;
        while let Some((cell,first,depth,visited))=queue.pop_front(){
            if considered>=1024 {break;}
            considered+=1;
            if depth>7{continue;}
            for action in 0..self.config.motor_cells {
                let trials=t.action_trials.iter()
                    .find(|tr|tr.from_cell==cell&&tr.motor_action==action)
                    .map(|tr|tr.observations).unwrap_or(0);
                let novelty=1.0/(1.0+trials as f32)
                    /(1.0+depth as f32*0.15);
                let first_action=first.unwrap_or(action);
                if best.is_none_or(|b|novelty>b.1+1.0e-6){
                    best=Some((first_action,novelty,depth+1));
                }
            }
            if depth>=7{continue;}
            for e in t.transitions.iter().filter(|e|
                e.from_cell==cell&&e.observations>0
                    &&e.to_cell!=goal&&!visited.contains(&e.to_cell)
            ){
                let entry=conductance(&self.cells,
                    &self.synapses[e.entry_synapse],floor);
                let exit=conductance(&self.cells,
                    &self.synapses[e.exit_synapse],floor);
                if entry.min(exit)<=1.0e-8{continue;}
                let mut seen=visited.clone();
                seen.push(e.to_cell);
                queue.push_back((
                    e.to_cell,Some(first.unwrap_or(e.motor_action)),
                    depth+1,seen
                ));
            }
        }
        // When novelty is exhausted, act on the acquired causal model
        // rather than continuously probing a known complete world.
        let (motor,value,steps)=best?;
        if value<0.14{return None;}
        Some((motor,value,steps))
    }

    pub fn phase_native_temporal_goal_replanning_enabled(&self)->bool{
        self.phase_native.as_ref()
            .and_then(|n|n.temporal_evidence.as_ref())
            .map(|t|t.goal_replan_enabled).unwrap_or(false)
    }

    /// Re-run bounded physical prediction from ACTUAL current observation on
    /// each query. No route is stored as the authoritative answer; a factual
    /// contradiction to any necessary synapse removes that route immediately.
    pub fn choose_phase_native_temporal_goal_plan(
        &self,goal_sensory:&[f32]
    )->Option<PhaseTemporalGoalPlan>{
        let native=self.phase_native.as_ref()?;
        let t=native.temporal_evidence.as_ref()?;
        if !t.multistep_enabled || !t.goal_replan_enabled {return None;}
        let real=self.current_real.as_ref()?;
        let origin=self.phase_native_abstract_state(&real.sensory)?.cell;
        let goal=self.phase_native_abstract_state(goal_sensory)?.cell;
        if origin==goal{return None;}
        let floor=native.config.coherence_floor;
        let mut queue=std::collections::VecDeque::new();
        queue.push_back((origin,None,1.0f32,0usize,vec![origin]));
        let mut considered=0usize;
        let mut best:Option<PhaseTemporalGoalPlan>=None;
        let mut tied=false;
        while let Some((cell,first,strength,depth,visited))=queue.pop_front(){
            if considered>=1024 {break;}
            considered+=1;
            if depth>=8 || best.is_some_and(|p|depth>=p.steps){continue;}
            for edge in t.transitions.iter().filter(|e|
                e.from_cell==cell && e.observations>0
            ){
                let a=conductance(&self.cells,
                    &self.synapses[edge.entry_synapse],floor);
                let b=conductance(&self.cells,
                    &self.synapses[edge.exit_synapse],floor);
                let value=strength.min(a).min(b);
                if value<=1.0e-8{continue;}
                let next_depth=depth+1;
                let first_step=first.unwrap_or((edge.motor_action,edge.entry_synapse));
                if edge.to_cell==goal {
                    let proposal=PhaseTemporalGoalPlan{
                        action:first_step.0,synapse:first_step.1,
                        steps:next_depth,strength:value,target_cell:goal,
                    };
                    match best {
                        None=>{best=Some(proposal);tied=false;},
                        Some(prev) if proposal.steps<prev.steps
                            ||(proposal.steps==prev.steps
                                && proposal.strength>prev.strength+1.0e-6)=>{
                            best=Some(proposal);tied=false;
                        }
                        Some(prev) if proposal.steps==prev.steps
                            && (proposal.strength-prev.strength).abs()<=1.0e-6
                            && proposal.action!=prev.action => {tied=true;}
                        _=>{}
                    }
                } else if !visited.contains(&edge.to_cell)
                    && next_depth<8
                {
                    let mut seen=visited.clone();
                    seen.push(edge.to_cell);
                    queue.push_back((
                        edge.to_cell,Some(first_step),
                        value,next_depth,seen
                    ));
                }
            }
        }
        if tied {None}else{best}
    }

    pub fn phase_native_temporal_multistep_enabled(&self)->bool{
        self.phase_native.as_ref()
            .and_then(|n|n.temporal_evidence.as_ref())
            .map(|t|t.multistep_enabled).unwrap_or(false)
    }

    pub fn phase_native_temporal_transition_count(&self)->usize{
        self.phase_native.as_ref()
            .and_then(|n|n.temporal_evidence.as_ref())
            .map(|t|t.transitions.len()).unwrap_or(0)
    }

    pub fn phase_native_temporal_chain_learning(&self)->bool{
        self.phase_native.as_ref()
            .and_then(|n|n.temporal_evidence.as_ref())
            .map(|t|t.chain_learning_enabled).unwrap_or(false)
    }

    pub fn phase_native_temporal_chain_count(&self)->usize{
        self.phase_native.as_ref()
            .and_then(|n|n.temporal_evidence.as_ref())
            .map(|t|t.chains.len()).unwrap_or(0)
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
        if temporal.multistep_enabled {
            let current=self.current_real.as_ref()
                .and_then(|r|self.phase_native_abstract_state(&r.sensory))?
                .cell;
            // Once a complete physical path to a cue has been acquired, TE3
            // supplies it, not open-ended novelty. When incomplete, follow
            // the bounded factual graph to the nearest under-tested frontier.
            let cues=temporal.cues.iter().map(|c|c.source_cell)
                .collect::<Vec<_>>();
            if self.phase_native_temporal_multistep_route(current,&cues,1)
                .is_some(){return None;}
            let floor=native.config.coherence_floor;
            let mut queue=std::collections::VecDeque::new();
            queue.push_back((current,None,1.0f32,0usize,vec![current]));
            let mut best:Option<(usize,f32)>=None;
            let mut expanded=0usize;
            while let Some((cell,first,confidence,depth,visited))=
                queue.pop_front()
            {
                if expanded>=512{break;}
                expanded+=1;
                let (trial_action,trials)=(0..self.config.motor_cells)
                    .filter(|action|!self.phase_native_temporal_rewarded_motor(*action))
                    .map(|action|{
                        let seen=temporal.action_trials.iter().find(|t|
                            t.from_cell==cell && t.motor_action==action
                        ).map(|t|t.observations).unwrap_or(0);
                        (action,seen)
                    }).min_by_key(|(action,seen)|(*seen,*action))?;
                let candidate=first.unwrap_or(trial_action);
                // A *conducting* route certifies reachability. Once it
                // clears that causal gate, its partial synaptic maturation
                // must not suppress all work at a deeper untried frontier:
                // otherwise a one-trial link (weight ~0.125) can never
                // compete with shallow no-op trials. The learning target is
                // novelty conditional on physical reachability, discounted
                // for action cost, not raw absolute synaptic confidence.
                let novelty=1.0/(1.0+trials as f32)
                    /(1.0+depth as f32*0.12);
                if best.is_none_or(|b|novelty>b.1+1.0e-6){
                    best=Some((candidate,novelty));
                }
                if depth>=7{continue;}
                for edge in temporal.transitions.iter().filter(|e|
                    e.from_cell==cell && !visited.contains(&e.to_cell)
                    &&!self.phase_native_temporal_rewarded_motor(e.motor_action)
                ){
                    let physical=confidence.min(conductance(&self.cells,
                        &self.synapses[edge.entry_synapse],floor
                    )).min(conductance(&self.cells,
                        &self.synapses[edge.exit_synapse],floor
                    ));
                    if physical<=1.0e-8{continue;}
                    let mut seen=visited.clone();
                    seen.push(edge.to_cell);
                    queue.push_back((
                        edge.to_cell,Some(first.unwrap_or(edge.motor_action)),
                        physical,depth+1,seen
                    ));
                }
            }
            return best;
        }
        if temporal.chain_learning_enabled {
            if let (Some(pending),Some(real))=(
                temporal.pending_chain.as_ref(),self.current_real.as_ref()
            ) {
                if self.phase_native_abstract_state(&real.sensory)
                    .is_some_and(|state|state.cell==pending.marker_cell)
                {
                    let (motor,trials)=(0..self.config.motor_cells)
                        .filter(|action|*action!=pending.first_action)
                        .map(|action|(action,pending.trials[action]))
                        .min_by_key(|(action,trials)|(*trials,*action))?;
                    return Some((motor,
                        (1.0/(1.0+trials as f32)).clamp(0.0,1.0)));
                }
            }
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

    /// Whether an action is physically part of a *complete* acquired
    /// information path of arbitrary depth. Every necessary edge must
    /// conduct; metadata cannot independently declare a sensory action.
    fn phase_native_temporal_multistep_information_action(
        &self,action:usize
    )->bool{
        let Some(native)=self.phase_native.as_ref() else{return false;};
        let Some(t)=native.temporal_evidence.as_ref() else{return false;};
        if !t.multistep_enabled{return false;}
        let cues=t.cues.iter().map(|cue|cue.source_cell).collect::<Vec<_>>();
        let floor=native.config.coherence_floor;
        t.transitions.iter().any(|edge|{
            if edge.motor_action!=action
                ||self.phase_native_temporal_rewarded_motor(action)
            {return false;}
            if conductance(&self.cells,
                &self.synapses[edge.entry_synapse],floor)<=1.0e-8
                ||conductance(&self.cells,
                    &self.synapses[edge.exit_synapse],floor)<=1.0e-8
            {return false;}
            let prefix=cues.contains(&edge.from_cell)||
                cues.iter().any(|source|
                    self.phase_native_temporal_multistep_route(
                        *source,&[edge.from_cell],1
                    ).is_some()
                );
            let suffix=cues.contains(&edge.to_cell)||
                self.phase_native_temporal_multistep_route(
                    edge.to_cell,&cues,1
                ).is_some();
            prefix&&suffix
        })
    }

    /// Optimistic but evidence-based search of opaque outcome motors for a
    /// specific physically acquired belief. The reward estimate is read from
    /// the EXISTING phase-sensitive cue->motor synapse; the uncertainty bonus
    /// uses only self-executed factual action coverage. There is no known
    /// correct action, world ID, response table or external training schedule.
    /// Disabled during frozen evaluation: use the acquired policy then.
    pub fn choose_phase_native_temporal_outcome_probe(&self)
        ->Option<(usize,f32,f32)>{
        let belief=self.phase_native_temporal_evidence()?;
        let cue=belief.winner_cell?;
        let native=self.phase_native.as_ref()?;
        let temporal=native.temporal_evidence.as_ref()?;
        if !temporal.autonomous_probe_enabled || !native.config.learning_enabled {
            return None;
        }
        let cue_index=temporal.cues.iter()
            .position(|c|c.source_cell==cue)?;
        let sensor=self.choose_phase_native_temporal_sensing_action();
        // SensingAction readout requires needs_more, which is false after
        // decisiveness. Exclude physical sensor paths directly instead.
        let _=sensor;
        let total:u32=temporal.samplers.iter().map(|m|m.belief_trials[cue_index])
            .fold(0u32,|a,b|a.saturating_add(b));
        let log_term=(total as f32+2.0).ln();
        let floor=native.config.coherence_floor;
        let mut best:Option<(usize,f32,f32)>=None;
        let mut best_score=f32::NEG_INFINITY;
        for action in 0..self.config.motor_cells {
            let single_step_sensor=temporal.samplers.iter().any(|m|
                m.motor_action==action && m.distinctions>0
                && conductance(&self.cells,
                    &self.synapses[m.sensory_synapse],floor)>1.0e-8
            );
            let chain_sensor=temporal.chain_learning_enabled
                && temporal.chains.iter().any(|chain|
                    (chain.first_action==action || chain.second_action==action)
                        && conductance(&self.cells,
                            &self.synapses[chain.entry_synapse],floor)>1.0e-8
                        && conductance(&self.cells,
                            &self.synapses[chain.read_synapse],floor)>1.0e-8
                );
            // Both causally acquired INFORMATION actions are not candidate
            // terminal responses while their two-link physical path conducts.
            // No motor role or action ID is supplied by the evaluator.
            if single_step_sensor || chain_sensor
                ||self.phase_native_temporal_multistep_information_action(action)
            {continue;}
            let trials=temporal.samplers.iter()
                .find(|m|m.motor_action==action)
                .map(|m|m.belief_trials[cue_index]).unwrap_or(0);
            let value=temporal.outcomes.iter()
                .filter(|o|o.source_cell==cue && o.motor_action==action)
                .map(|o|conductance(
                    &self.cells,&self.synapses[o.value_synapse],floor
                )).fold(0.0f32,f32::max).clamp(0.0,1.0);
            // Standard upper confidence bound, computed only from factual
            // sample counts and physically represented outcome values.
            let uncertainty=(2.0*log_term/(1.0+trials as f32)).sqrt();
            let score=value+uncertainty;
            if score>best_score+1.0e-6 {
                best_score=score;
                best=Some((action,value,uncertainty.clamp(0.0,1.0)));
            }
        }
        best
    }

    /// In an uncertain episode a physically verified information-producing
    /// operation is the only currently supported way of changing the belief.
    /// Do not treat an unsupported optimistic goal path as an informed action.
    /// When belief becomes decisive, all normal motor proposals return.
    /// No source class, hidden cause, reward motor or host phase is inspected.
    pub fn phase_native_temporal_action_evidence_admissible(
        &self,action:usize
    )->bool{
        // During explicitly opted-in cold acquisition the physically
        // reachable, least-tested causal hypothesis receives an executable
        // investigation opportunity. No evaluator action identity enters it.
        // After the model is frozen the ordinary acquired goal plan wins.
        if let Some((probe,_,_))=self.choose_phase_native_temporal_goal_frontier(){
            return action==probe;
        }
        if !self.phase_native_temporal_autonomous_probe(){return true;}
        let Some(belief)=self.phase_native_temporal_evidence()
            else{return true;};
        if !belief.needs_more {
            // In a plastic lifetime evaluate unverified terminal alternatives
            // using physical reward + uncertainty; in frozen evaluation
            // execute the acquired phase-conditioned best outcome.
            if let Some((probe,_,_))=self.choose_phase_native_temporal_outcome_probe(){
                return probe==action;
            }
            return self.choose_phase_native_temporal_outcome_action()
                .map(|policy|policy.action==action).unwrap_or(true);
        }
        let Some(sensor)=self.choose_phase_native_temporal_sensing_action()
            else{
                if self.phase_native_temporal_multistep_enabled(){
                    if let Some((probe,_))=
                        self.choose_phase_native_temporal_unknown_probe(){
                        return probe==action;
                    }
                }
                if self.phase_native_temporal_chain_learning() {
                    if let Some((probe,_))=
                        self.choose_phase_native_temporal_unknown_probe()
                    {
                        if self.phase_native.as_ref()
                            .and_then(|n|n.temporal_evidence.as_ref())
                            .and_then(|t|t.pending_chain.as_ref()).is_some()
                        {
                            return probe==action;
                        }
                    }
                }
                return true;
            };
        // If the necessary physical chain is broken, do not use metadata
        // alone to force an action that is no longer supported.
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
        let direct=p.samplers.iter()
            .filter(|m|m.motor_action==action && m.distinctions>0)
            .map(|m|conductance(
                &self.cells,&self.synapses[m.sensory_synapse],
                n.config.coherence_floor
            )).fold(0.0_f32,f32::max);
        let chain=if p.chain_learning_enabled {
            p.chains.iter().filter(|c|c.second_action==action)
                .map(|c|conductance(
                    &self.cells,&self.synapses[c.entry_synapse],
                    n.config.coherence_floor
                ).min(conductance(
                    &self.cells,&self.synapses[c.read_synapse],
                    n.config.coherence_floor
                ))).fold(0.0_f32,f32::max)
        }else{0.0};
        let mut multi=0.0f32;
        if p.multistep_enabled {
            let cues=p.cues.iter().map(|c|c.source_cell)
                .collect::<Vec<_>>();
            for edge in p.transitions.iter().filter(|e|
                e.motor_action==action && cues.contains(&e.to_cell)
            ){
                let before=cues.contains(&edge.from_cell)
                    ||cues.iter().any(|source|
                        self.phase_native_temporal_multistep_route(
                            *source,&[edge.from_cell],1
                        ).is_some()
                    );
                if !before {continue;}
                let strength=conductance(&self.cells,
                    &self.synapses[edge.entry_synapse],n.config.coherence_floor
                ).min(conductance(&self.cells,
                    &self.synapses[edge.exit_synapse],n.config.coherence_floor
                ));
                multi=multi.max(strength);
            }
        }
        let value=direct.max(chain).max(multi);
        if value>1.0e-8 {Some(value)}else{None}
    }

    /// A learned physical path only counts an actual new sensory observation
    /// at a supported causal PRE/action/POST context, never merely because a
    /// motor was useful elsewhere. This prevents a read call from a non-ready
    /// state being falsely credited as a fresh independent cue.
    pub fn phase_native_temporal_factual_sample_from(
        &self, action:usize, pre:&[f32],post:&[f32]
    )->bool{
        let Some(before)=self.phase_native_abstract_state(pre) else{return false;};
        let Some(after)=self.phase_native_abstract_state(post) else{return false;};
        let Some(native)=self.phase_native.as_ref() else{return false;};
        let Some(t)=native.temporal_evidence.as_ref() else{return false;};
        if !t.cues.iter().any(|c|c.source_cell==after.cell){return false;}
        let floor=native.config.coherence_floor;
        let immediate=t.cues.iter().any(|c|c.source_cell==before.cell)
            &&t.samplers.iter().any(|m|
                m.motor_action==action && m.distinctions>0
                    && conductance(&self.cells,
                        &self.synapses[m.sensory_synapse],floor)>1.0e-8
            );
        let variable_depth=t.multistep_enabled &&t.transitions.iter().any(|e|
            e.from_cell==before.cell && e.motor_action==action
                &&e.to_cell==after.cell
                &&conductance(&self.cells,
                    &self.synapses[e.entry_synapse],floor)>1.0e-8
                &&conductance(&self.cells,
                    &self.synapses[e.exit_synapse],floor)>1.0e-8
        ) && (
            t.cues.iter().any(|cue|cue.source_cell==before.cell)
            ||t.cues.iter().any(|cue|
                self.phase_native_temporal_multistep_route(
                    cue.source_cell,&[before.cell],1
                ).is_some()
            )
        );
        let two_step=t.chain_learning_enabled && t.chains.iter().any(|c|
            c.second_action==action && c.marker_cell==before.cell
                && conductance(&self.cells,
                    &self.synapses[c.entry_synapse],floor)>1.0e-8
                && conductance(&self.cells,
                    &self.synapses[c.read_synapse],floor)>1.0e-8
        );
        immediate || two_step || variable_depth
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
        // Capture the *pre-action* physical belief before mutating native
        // state; never infer the latent cause from POST/reward.
        let factual_winner=self.phase_native_temporal_evidence()
            .and_then(|readout|readout.winner_cell);
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
            && (!evidence.autonomous_probe_enabled || factual_winner.is_some())
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
            // In explicit autonomous inference, an already decisive physical
            // hypothesis owns the observed consequence. Crediting its rejected
            // rival would invent an unsupported cue->motor value and mix the
            // two alternative policies. Legacy prepared TE5 soft-credit mode
            // remains unchanged when this integration is not enabled.
            let credit=if evidence.autonomous_probe_enabled {
                if Some(source_cell)==factual_winner {1.0} else {0.0}
            } else {
                (observations[cue_index]/mass).clamp(0.0,1.0)
            };
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
        evidence.pending_chain=None;
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
        // A belief should carry greater factual support before selecting
        // between *different* physically learned goal actions. When both
        // hypotheses recommend the same motor, extra observations cannot
        // change the choice and the old minimum margin remains sufficient.
        // This is a generic, action-consequence-dependent information demand,
        // not a fixed number of observations or a world/role lookup.
        let mut required_margin=state.config.decisive_margin;
        if state.autonomous_probe_enabled && state.cues.len()==2 {
            let best_for=|source:usize| {
                state.outcomes.iter()
                    .filter(|o|o.source_cell==source && o.observations>0)
                    .map(|o|(o.motor_action,conductance(
                        &self.cells,&self.synapses[o.value_synapse],floor
                    )))
                    .filter(|(_,value)|*value>0.05)
                    .max_by(|a,b|a.1.total_cmp(&b.1))
            };
            if let (Some(a),Some(b))=(
                best_for(state.cues[0].source_cell),
                best_for(state.cues[1].source_cell)
            ){
                if a.0!=b.0 {
                    // A consequential disagreement normally needs two
                    // net physical observations. But another observation
                    // may require a *learned multi-action causal path*.
                    // Discount the marginal confidence demand by its
                    // physically conducting action cost; otherwise the
                    // organism can exhaust its external budget investigating
                    // rather than acting. This uses acquired phase links,
                    // not an external time or motor-role schedule.
                    let floor=native.config.coherence_floor;
                    let extra_costly_chain=state.chain_learning_enabled
                        &&state.chains.iter().any(|c|
                            conductance(&self.cells,
                                &self.synapses[c.entry_synapse],floor)>1.0e-8
                            &&conductance(&self.cells,
                                &self.synapses[c.read_synapse],floor)>1.0e-8
                        );
                    let raw_cues=state.cues.iter()
                        .map(|c|c.source_cell).collect::<Vec<_>>();
                    let physical_depth=if state.multistep_enabled{
                        raw_cues.iter().filter_map(|source|
                            self.phase_native_temporal_multistep_route(
                                *source,&raw_cues,1
                            ).map(|(_,_,_,steps)|steps as f32)
                        ).min_by(|a,b|a.total_cmp(b))
                    }else{None};
                    // This is NOT an evaluator-supplied sequence length:
                    // the organism measures action cost from its own
                    // conducting acquired source->motor->state graph.
                    let predicted_information_cost=physical_depth.unwrap_or(
                        if extra_costly_chain {2.0}else{1.0}
                    );
                    let extra=2.0/(state.config.max_observations as f32
                        *predicted_information_cost);
                    required_margin=required_margin.max(extra);
                }
            }
        }
        let decisive = state.cues.len()==2
            && state.observations>=state.config.minimum_observations
            && margin+1.0e-7>=required_margin;
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
