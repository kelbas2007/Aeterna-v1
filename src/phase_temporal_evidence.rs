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

#[derive(Debug, Clone)]
pub(super) struct PhaseTemporalEvidenceState {
    config: PhaseTemporalEvidenceConfig,
    hub_cell: usize,
    cues: Vec<PhaseTemporalCue>,
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
            .map(|p|p.cues.iter().any(|c|c.evidence_synapse==synapse_index))
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
