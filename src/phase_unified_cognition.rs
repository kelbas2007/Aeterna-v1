// Unified-cognition assembly for INTEL-2.
//
// Enumeration is allowed to know which qualified mechanisms can emit an
// operation. Authority is not: every emitted action receives the SAME generic
// action-level fields, and the winner is selected by U1/U2/U3.

include!("phase_unified_representation.rs");

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseUnifiedCognitiveProposal {
    pub persistent_candidate_id: Option<u64>,
    pub applicability: f32,
    pub proposal: PhaseCognitiveProposal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhaseUnifiedDecision {
    pub supporting_candidate_ids: Vec<u64>,
    pub proposal_id: u64,
    pub action: usize,
    pub score: f32,
}

/// Read-only evidence of the *actual* coalesced U1 action competition.
/// Action IDs are opaque; fields and scores come from physical readout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseUnifiedActionTrace {
    pub action: usize,
    pub fields: [f32; META_FIELD_COUNT],
    pub score: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseUnifiedKnowledgeSnapshot {
    pub circuits: u64,
    pub revisions: u64,
    pub representation_candidates: u64,
    pub promoted_representations: u64,
    pub evidence_observations: u64,
}

impl PhaseUnifiedKnowledgeSnapshot {
    pub fn gained_since(self, before: Self) -> bool {
        self.circuits > before.circuits
            || self.revisions > before.revisions
            || self.representation_candidates > before.representation_candidates
            || self.promoted_representations > before.promoted_representations
            || self.evidence_observations > before.evidence_observations
    }
}

fn unified_hash(parts: &[u64]) -> u64 {
    let mut h = 14_695_981_039_346_656_037u64;
    for value in parts {
        for byte in value.to_le_bytes() {
            h ^= u64::from(byte);
            h = h.wrapping_mul(1_099_511_628_211);
        }
    }
    if h == 0 { 1 } else { h }
}

impl EvoPhase {
    fn unified_action_fields(
        &self,
        sensory: &[f32],
        goal_sensory: &[f32],
        action: usize,
    ) -> Option<[f32; META_FIELD_COUNT]> {
        if action >= self.config.motor_cells { return None; }
        let entry = self.phase_native_abstract_state(sensory)?;
        let goal = self.phase_native_abstract_state(goal_sensory)?;
        if entry.level != goal.level { return None; }
        let state = self.phase_native.as_ref()?;
        // A acquired refinement is the operational state, not merely the name
        // of the source that suggested an action. Every competing action must
        // be evaluated in the same physically supported representation.
        let operational = self.unified_operational_entry(sensory, entry.cell)?;
        let mut state_cells = self.phase_native_abstract_cells_at_level(entry.level);
        if !state_cells.contains(&operational) { state_cells.push(operational); }

        let goal_value = {
            let mut probe = self.clone();
            let decision = if operational == entry.cell {
                probe.plan_phase_native_abstract_goal(sensory, goal_sensory, None)
            } else {
                probe.phase_native_goal_decision_from_cells(operational, goal.cell, None)
            };
            match decision {
                Some(decision) if decision.first_action == action =>
                    decision.predicted_value.clamp(0.0,1.0),
                _ => 0.0,
            }
        };

        // A native physical belief can lower the certainty of an optimistic
        // goal path. Once evidence becomes decisive the full learned goal
        // value is restored. No motor/task identity is inspected here.
        let goal_value=goal_value
            *self.phase_native_temporal_goal_evidence_coverage().unwrap_or(1.0);

        let frontier = self.phase_drive_frontier_activity_for_cells(state, &state_cells);
        let drive_features = self.phase_drive_features(state, operational, action, &frontier);
        let epistemic_value = drive_features[0].max(drive_features[1]).clamp(0.0,1.0);

        let relevance =
            self.phase_goal_relevance_for_cells(state, goal.cell, &state_cells);
        let contradiction = self.phase_goal_rival_disagreement_score(
            state, operational, action, &relevance
        ).clamp(0.0,1.0);

        let support = self.phase_action_global_support(state, action) as f32;
        let confidence = (support / (support + 4.0)).clamp(0.0,1.0);

        // Every external action has the same one-step execution economy at
        // this layer. No proposal source gets a class-specific cost bonus.
        let economy = 1.0;

        Some([
            goal_value,
            epistemic_value,
            contradiction,
            confidence,
            economy,
        ])
    }

    // Model validity remains a live epistemic question even when every
    // local motor owns an old supported transition. If a fresh raw goal is
    // unreachable using the physical learned recurrence and all ordinary
    // cognition abstains, revisit the weakest *factual* local model edge.
    // This is a carrier-based pressure, not a world label or answer lookup.
    // When multiple genuinely goal-progressing physical successor paths are
    // known, keep their evidential coverage under observation. An established
    // route with heavy support must not permanently exclude a weaker but
    // physically supported competing route from factual verification.
    // All values come from the same phase-native circuits and goal relevance.
    fn unified_competing_goal_path_coverage(
        &self,
        sensory: &[f32],
        goal_sensory: &[f32],
    ) -> Option<(usize, f32, f32)> {
        let base = self.phase_native_abstract_state(sensory)?;
        let goal = self.phase_native_abstract_state(goal_sensory)?;
        if base.level != goal.level || base.cell == goal.cell {
            return None;
        }
        let native = self.phase_native.as_ref()?;
        if !native.config.learning_enabled { return None; }
        let entry = self.unified_operational_entry(sensory, base.cell)?;
        let mut state_cells = self.phase_native_abstract_cells_at_level(base.level);
        if !state_cells.contains(&entry) { state_cells.push(entry); }
        let relevance = self.phase_goal_relevance_for_cells(
            native,goal.cell,&state_cells
        );
        if relevance[entry] <= 1.0e-8 { return None; }

        let floor = native.config.coherence_floor;
        let min_support = u64::from(self.config.min_recruit_support);
        let mut pathways: Vec<(usize, u64, f32)> = Vec::new();
        for action in 0..self.config.motor_cells {
            let motor = self.config.sensory_cells + action;
            let mut total_support = 0u64;
            let mut best_progress = 0.0f32;
            for circuit in &native.circuits {
                if circuit.support < min_support { continue; }
                let aff = &self.synapses[circuit.afferent_synapse];
                let succ = &self.synapses[circuit.successor_synapse];
                let output = &self.synapses[circuit.motor_synapse];
                if aff.from != entry || output.to != motor
                    || succ.to == entry || !state_cells.contains(&succ.to)
                {
                    continue;
                }
                let conducting = conductance(&self.cells,aff,floor)
                    .min(conductance(&self.cells,succ,floor));
                if conducting <= 1.0e-8 { continue; }
                let progress = relevance[succ.to]-relevance[entry];
                if progress <= 1.0e-7 { continue; }
                total_support = total_support.saturating_add(circuit.support);
                best_progress = best_progress.max(
                    conducting * relevance[succ.to]
                );
            }
            if total_support > 0 && best_progress > 1.0e-8 {
                pathways.push((action,total_support,best_progress));
            }
        }
        if pathways.len() < 2 { return None; }
        let max_support=pathways.iter().map(|(_,n,_)|*n).max()?;
        let mut under=pathways.into_iter().filter(|(_,n,_)|*n<max_support)
            .collect::<Vec<_>>();
        if under.is_empty() { return None; }
        // Least-verified physical model first. The code receives no
        // task/action labels; ties use factual progress then opaque motor id.
        under.sort_by(|a,b|{
            a.1.cmp(&b.1)
                .then_with(||b.2.total_cmp(&a.2))
                .then_with(||a.0.cmp(&b.0))
        });
        let (action,support,progress)=under[0];
        let evidence_debt = ((max_support-support) as f32
            /(max_support as f32 + 1.0)).clamp(0.0,1.0);
        Some((action,progress.clamp(0.0,1.0),evidence_debt))
    }

    fn unified_exhausted_model_revalidation(
        &self,
        sensory: &[f32],
        goal_sensory: &[f32],
    ) -> Option<(usize, f32)> {
        let entry = self.phase_native_abstract_state(sensory)?;
        let goal = self.phase_native_abstract_state(goal_sensory)?;
        if entry.level != goal.level || entry.cell == goal.cell {
            return None;
        }
        let native = self.phase_native.as_ref()?;
        if !native.config.learning_enabled
            || native.drive.as_ref().map(|d|d.config.readout_enabled) != Some(true)
        {
            return None;
        }
        let mut known_path_probe = self.clone();
        if known_path_probe.phase_native_goal_decision_from_cells(
            entry.cell, goal.cell, None
        ).is_some() {
            return None;
        }
        let motor_count = self.config.motor_cells;
        let min_support = u64::from(self.config.min_recruit_support);
        let mut weakest: Option<(usize, u64)> = None;
        for action in 0..motor_count {
            if !self.phase_drive_action_known_at(native, entry.cell, action) {
                // Ordinary frontier learning, not model revalidation, owns
                // genuinely unmodelled actions at this state.
                return None;
            }
            let target_motor = self.config.sensory_cells + action;
            let support = native.circuits.iter()
                .filter(|c| {
                    self.synapses[c.afferent_synapse].from == entry.cell
                        && self.synapses[c.motor_synapse].to == target_motor
                        && c.support >= min_support
                })
                .map(|c| c.support)
                .sum::<u64>();
            if support == 0 {
                return None;
            }
            if weakest.map(|(_, old)| support < old).unwrap_or(true) {
                weakest = Some((action, support));
            }
        }
        let (action, support) = weakest?;
        let epistemic_debt = 1.0 / (1.0 + support as f32);
        Some((action, epistemic_debt))
    }

    fn unified_context_candidate(
        &self,
        action: usize,
    ) -> Option<(u64,f32)> {
        let sensory = self.current_real.as_ref()?.sensory.clone();
        let base = self.phase_native_abstract_state(&sensory)?;
        let ctx = self.phase_native.as_ref()?.contextual.as_ref()?;
        let previous = ctx.previous_base?;

        let mut parts = vec![base.cell as u64, action as u64];
        let mut applicability = 0.0f32;
        let mut found = false;

        for witness in &ctx.candidates {
            if witness.base_cell != base.cell
                || witness.retired
                || !witness.predecessor_cells.contains(&previous)
            {
                continue;
            }
            let relevant = if witness.promoted {
                true
            } else {
                witness.anchor_action == action
            };
            if !relevant { continue; }
            found = true;
            parts.extend(witness.input_synapses.iter().flatten().map(|x|*x as u64));
            let evidence = (witness.eligible_observations as f32 / 32.0).clamp(0.0,1.0);
            applicability = applicability.max(if witness.promoted {1.0} else {evidence});
        }

        found.then(||(unified_hash(&parts),applicability))
    }

    fn unified_perceptual_candidate(
        &self,
        action: usize,
    ) -> Option<(u64,f32)> {
        let sensory = self.current_real.as_ref()?.sensory.clone();
        let base = self.phase_native_abstract_state(&sensory)?;
        let raw = self.phase_raw_features(&sensory)?;
        let perceptual = self.phase_native.as_ref()?.perceptual.as_ref()?;

        let mut parts = vec![base.cell as u64, action as u64];
        let mut applicability = 0.0f32;
        let mut found = false;
        for witness in &perceptual.candidates {
            if witness.base_cell != base.cell || witness.retired { continue; }
            if Self::percept_side(&raw,witness).is_none() { continue; }
            found = true;
            parts.extend(witness.input_synapses.iter().flatten().map(|x|*x as u64));
            let evidence=(witness.eligible_observations as f32/32.0).clamp(0.0,1.0);
            applicability=applicability.max(if witness.promoted{1.0}else{evidence});
        }
        found.then(||(unified_hash(&parts),applicability))
    }

    fn unified_composition_candidate(
        &self,
        action: usize,
    ) -> Option<(u64,f32)> {
        let sensory=self.current_real.as_ref()?.sensory.clone();
        let base=self.phase_native_abstract_state(&sensory)?;
        let raw=self.phase_raw_features(&sensory)?;
        let comp=self.phase_native.as_ref()?.compositional.as_ref()?;

        let mut parts=vec![base.cell as u64,action as u64];
        let mut applicability=0.0f32;
        let mut found=false;
        for witness in &comp.candidates {
            if witness.base_cell!=base.cell||witness.retired {continue;}
            let _side=witness.program.eval(&raw);
            found=true;
            parts.extend(witness.input_synapses.iter().flatten().map(|x|*x as u64));
            let evidence=(witness.eligible_observations as f32/32.0).clamp(0.0,1.0);
            applicability=applicability.max(if witness.promoted{1.0}else{evidence});
        }
        found.then(||(unified_hash(&parts),applicability))
    }

    fn unified_rival_candidate(
        &self,
        action: usize,
        goal_sensory: &[f32],
    ) -> Option<(u64,f32)> {
        let sensory=self.current_real.as_ref()?.sensory.clone();
        let entry=self.phase_native_abstract_state(&sensory)?;
        let goal=self.phase_native_abstract_state(goal_sensory)?;
        if entry.level!=goal.level{return None;}
        let state_cells=self.phase_native_abstract_cells_at_level(entry.level);
        let state=self.phase_native.as_ref()?;
        let relevance=self.phase_goal_relevance_for_cells(state,goal.cell,&state_cells);
        let disagreement=self.phase_goal_rival_disagreement_score(
            state,entry.cell,action,&relevance
        ).clamp(0.0,1.0);
        if disagreement<=1.0e-8{return None;}

        let motor=self.config.sensory_cells+action;
        let min_support=u64::from(self.config.min_recruit_support);
        let floor=state.config.coherence_floor;
        let mut links=state.circuits.iter().filter_map(|circuit|{
            if circuit.support<min_support{return None;}
            let aff=&self.synapses[circuit.afferent_synapse];
            let succ=&self.synapses[circuit.successor_synapse];
            let out=&self.synapses[circuit.motor_synapse];
            if aff.from==entry.cell
                && out.to==motor
                && conductance(&self.cells,aff,floor)>1.0e-8
                && conductance(&self.cells,succ,floor)>1.0e-8
            {
                Some(circuit.successor_synapse as u64)
            }else{None}
        }).collect::<Vec<_>>();
        links.sort_unstable();
        links.dedup();
        if links.len()<2{return None;}
        let mut parts=vec![entry.cell as u64,action as u64];
        parts.extend(links);
        Some((unified_hash(&parts),disagreement))
    }

    fn push_unified_proposal(
        &self,
        output:&mut Vec<PhaseUnifiedCognitiveProposal>,
        sensory:&[f32],
        goal:&[f32],
        action:usize,
        candidate:Option<(u64,f32)>,
        opaque_salt:u64,
    ){
        let Some(fields)=self.unified_action_fields(sensory,goal,action) else{return;};
        let (persistent_candidate_id,applicability)=
            candidate.map(|(id,a)|(Some(id),a)).unwrap_or((None,1.0));
        let proposal_id=unified_hash(&[
            opaque_salt,
            action as u64,
            persistent_candidate_id.unwrap_or(0),
        ]);
        if output.iter().any(|item|
            item.proposal.action==action
                && item.persistent_candidate_id==persistent_candidate_id
        ){return;}
        output.push(PhaseUnifiedCognitiveProposal{
            persistent_candidate_id,
            applicability,
            proposal:PhaseCognitiveProposal{
                proposal_id,
                action,
                fields,
            },
        });
    }

    /// Enumerate qualified operations without ranking them by source class.
    /// The only authority decision happens later in U1/U2/U3.
    pub fn collect_phase_native_unified_proposals(
        &self,
        goal_sensory:&[f32],
    )->Vec<PhaseUnifiedCognitiveProposal>{
        if self.phase_native_online_enabled() {
            return self.phase_native_online_proposal(goal_sensory).into_iter().collect();
        }
        let Some(sensory)=self.current_real.as_ref().map(|r|r.sensory.clone())
            else{return Vec::new();};
        // A learned word-action relation may control one native proposal.
        // No motor ID, interaction script or external permit is embedded in
        // the spoken word. Unknown/not-visible relation fails closed.
        if self.phase_native_word_intent_active() {
            return self.choose_phase_native_grounded_word_action(&sensory)
                .map(|witness| vec![PhaseUnifiedCognitiveProposal {
                    persistent_candidate_id: None,
                    applicability: 1.0,
                    proposal: PhaseCognitiveProposal {
                        proposal_id: unified_hash(&[
                            0xA660_u64, witness.action as u64
                        ]),
                        action: witness.action,
                        fields: [
                            witness.strength.clamp(0.0, 1.0), 0.0,
                            0.0, witness.strength.clamp(0.0, 1.0), 0.8
                        ],
                    }
                }]).unwrap_or_default();
        }
        // Autonomous local manipulation mode: a bodily relative target,
        // not an externally named object or demonstrated motor. The proposal
        // still goes through native U1 and Human Protection. If there is no
        // visible supported object, generic factor exploration may proceed.
        if self.phase_native_self_experiment_enabled() {
            if let Some(witness)=self.choose_phase_native_self_object_action(&sensory) {
                let strength=witness.strength.clamp(0.0,1.0);
                return vec![PhaseUnifiedCognitiveProposal {
                    persistent_candidate_id:None,
                    applicability:1.0,
                    proposal:PhaseCognitiveProposal {
                        proposal_id:unified_hash(&[
                            0xA5E1_u64,witness.action as u64,
                            u64::from(witness.learned)
                        ]),
                        action:witness.action,
                        fields:if witness.learned {
                            [strength,0.0,0.0,strength,0.7]
                        } else {
                            [0.0,strength,strength,0.5,0.7]
                        }
                    }
                }];
            }
        }
        // Opt-in factor mode recognizes a factual binary sensory vector
        // without requiring a previously visited whole-state class.
        // The single candidate still passes through U1 and Human Protection.
        if self.phase_native_factor_causality_enabled() {
            return self.choose_phase_native_factor_action(goal_sensory)
                .map(|choice| {
                    let value = choice.support.clamp(0.0, 1.0);
                    vec![PhaseUnifiedCognitiveProposal {
                        persistent_candidate_id: None,
                        applicability: 1.0,
                        proposal: PhaseCognitiveProposal {
                            proposal_id: unified_hash(&[
                                0xFAC7_u64, choice.action as u64,
                                u64::from(choice.planned),
                            ]),
                            action: choice.action,
                            fields: if choice.planned {
                                [value, 0.0, 0.0, value, 1.0 / choice.steps as f32]
                            } else {
                                [0.0, value, value, 0.4, 1.0 / choice.steps as f32]
                            },
                        },
                    }]
                }).unwrap_or_default();
        }
        if self.phase_native_abstract_state(goal_sensory).is_none(){
            return Vec::new();
        }
        let mut out=Vec::new();

        // A confirmed operational representation is learned knowledge, not
        // the investigation project that once assembled it. Dormancy in U2
        // can suppress a project probe, but must not erase a physically
        // supported native goal plan. This path is representation-class
        // agnostic: every uniquely applicable promoted refinement is resolved
        // by the same native operational-state function.
        if let (Some(base),Some(goal)) = (
            self.phase_native_abstract_state(&sensory),
            self.phase_native_abstract_state(goal_sensory),
        ) {
            if base.level == goal.level {
                if let Some(operational) =
                    self.unified_operational_entry(&sensory, base.cell)
                {
                    if operational != base.cell {
                        let mut planner = self.clone();
                        if let Some(decision) =
                            planner.phase_native_goal_decision_from_cells(
                                operational, goal.cell, None
                            )
                        {
                            self.push_unified_proposal(
                                &mut out, &sensory, goal_sensory,
                                decision.first_action, None, 0x0AE5,
                            );
                        }
                    }
                }
            }
        }

        let mut context=self.clone();
        if let (true,Some(action))=context.phase_native_context_action(goal_sensory){
            self.push_unified_proposal(
                &mut out,&sensory,goal_sensory,action,
                self.unified_context_candidate(action),0xC071,
            );
        }

        let mut percept=self.clone();
        if let (true,Some(action))=percept.phase_native_perceptual_action(goal_sensory){
            self.push_unified_proposal(
                &mut out,&sensory,goal_sensory,action,
                self.unified_perceptual_candidate(action),0xA221,
            );
        }

        let mut composition=self.clone();
        if let (true,Some(action))=composition.phase_native_compositional_action(goal_sensory){
            self.push_unified_proposal(
                &mut out,&sensory,goal_sensory,action,
                self.unified_composition_candidate(action),0xC023,
            );
        }

        // Before a useful sampling affordance has been discovered, give one
        // least-verified opaque motor a GENERAL epistemic proposal backed by
        // actual transition-coverage deficit. This is a carrier hypothesis
        // about information, not a sensor label, permission, or host schedule.
        if let Some((action,novelty))=
            self.choose_phase_native_temporal_unknown_probe()
        {
            if let Some(mut fields)=self.unified_action_fields(
                &sensory,goal_sensory,action
            ){
                fields[1]=fields[1].max(novelty).clamp(0.0,1.0);
                // The two acquired rival cue identities are not yet
                // discriminable in this episode. This pressure decays with
                // factual trials and does not invent a positive sensor link.
                let unresolved=1.0
                    -self.phase_native_temporal_goal_evidence_coverage()
                        .unwrap_or(1.0);
                fields[2]=fields[2].max(novelty*unresolved)
                    .clamp(0.0,1.0);
                out.push(PhaseUnifiedCognitiveProposal{
                    persistent_candidate_id:None,
                    applicability:1.0,
                    proposal:PhaseCognitiveProposal{
                        proposal_id:unified_hash(&[0x7E32u64,action as u64]),
                        action,fields
                    },
                });
            }
        }

        // Cold, goal-conditioned information-seeking over still untried
        // ACTUAL state-action links. The proposer sees only the raw supplied
        // goal and physically visited state/motor pairs. It neither imports
        // nor selects an evaluator-defined correct path.
        if let Some((action,novelty,steps))=
            self.choose_phase_native_temporal_goal_frontier()
        {
            if let Some(mut fields)=self.unified_action_fields(
                &sensory,goal_sensory,action
            ){
                fields[1]=fields[1].max(novelty).clamp(0.0,1.0);
                fields[2]=fields[2].max(novelty).clamp(0.0,1.0);
                fields[3]=fields[3].max(1.0/(1.0+steps as f32));
                out.push(PhaseUnifiedCognitiveProposal{
                    persistent_candidate_id:None,
                    applicability:1.0,
                    proposal:PhaseCognitiveProposal{
                        proposal_id:unified_hash(&[
                            0xC01D_u64,action as u64
                        ]),
                        action,fields,
                    },
                });
            }
        }

        // Goal-conditioned physical causal path. The route is inferred fresh
        // from the CURRENT observed state and the raw caller goal on every
        // step, including after factual drift. No evaluator-specified plan.
        // U1 still arbitrates and Human Protection still guards execution.
        if let Some(plan)=self.choose_phase_native_temporal_goal_plan(goal_sensory){
            if let Some(mut fields)=self.unified_action_fields(
                &sensory,goal_sensory,plan.action
            ){
                fields[0]=fields[0].max((0.95*plan.strength).clamp(0.0,1.0));
                fields[3]=fields[3].max(plan.strength);
                out.push(PhaseUnifiedCognitiveProposal{
                    persistent_candidate_id:None,
                    applicability:1.0,
                    proposal:PhaseCognitiveProposal{
                        proposal_id:unified_hash(&[
                            0x60A7_u64,plan.action as u64
                        ]),
                        action:plan.action,
                        fields,
                    },
                });
            }
        }

        // TE3: a genuinely acquired sensory motor can compete for authority
        // while two temporally integrated raw hypotheses remain unresolved.
        // Physical uncertainty and phase-conducting affordance determine the
        // evidence value. This does NOT authorize actuation; U1 and Human
        // Protection remain unchanged.
        if let Some(sensing)=self.choose_phase_native_temporal_sensing_action(){
            if let Some(mut fields)=self.unified_action_fields(
                &sensory,goal_sensory,sensing.action
            ){
                let physical_information_debt=(
                    sensing.learned_affordance*sensing.missing_evidence
                ).clamp(0.0,1.0);
                fields[1]=fields[1].max(
                    physical_information_debt
                ).clamp(0.0,1.0);
                // Contradictory physical cue paths constitute a distinct
                // epistemic gap. A learned sensory action can reduce it;
                // untested motors cannot claim that factual capability.
                fields[2]=fields[2].max(
                    physical_information_debt
                ).clamp(0.0,1.0);
                out.push(PhaseUnifiedCognitiveProposal{
                    persistent_candidate_id:None,
                    applicability:1.0,
                    proposal:PhaseCognitiveProposal{
                        proposal_id:unified_hash(&[
                            0x7E30u64,sensing.action as u64
                        ]),
                        action:sensing.action,
                        fields,
                    }
                });
            }
        }

        // TE5: when factual temporal evidence has become decisive, a
        // conducting acquired cue→motor reward synapse can compete as
        // ordinary goal value. No motor is privileged by its research role.
        if let Some(learned)=self.choose_phase_native_temporal_outcome_action(){
            if let Some(mut fields)=self.unified_action_fields(
                &sensory,goal_sensory,learned.action
            ){
                fields[0]=fields[0].max(
                    learned.learned_value
                ).clamp(0.0,1.0);
                fields[3]=fields[3].max(
                    learned.learned_value
                ).clamp(0.0,1.0);
                out.push(PhaseUnifiedCognitiveProposal{
                    persistent_candidate_id:None,
                    applicability:1.0,
                    proposal:PhaseCognitiveProposal{
                        proposal_id:unified_hash(&[
                            0x7E51u64,learned.action as u64
                        ]),
                        action:learned.action,
                        fields,
                    }
                });
            }
        }

        // After a decisive physical belief, terminal alternatives must
        // still be investigated when their observed reward coverage is weak.
        // The motor and its value come from the SAME causal phase evidence,
        // not from a host-provided correct-action or fixed motor schedule.
        if let Some((action,value,uncertainty))=
            self.choose_phase_native_temporal_outcome_probe()
        {
            if let Some(mut fields)=self.unified_action_fields(
                &sensory,goal_sensory,action
            ){
                fields[0]=fields[0].max(value).clamp(0.0,1.0);
                fields[1]=fields[1].max(uncertainty).clamp(0.0,1.0);
                out.push(PhaseUnifiedCognitiveProposal{
                    persistent_candidate_id:None,applicability:1.0,
                    proposal:PhaseCognitiveProposal{
                        proposal_id:unified_hash(&[0x7E57u64,action as u64]),
                        action,fields
                    },
                });
            }
        }

        let mut rival=self.clone();
        if let Some(action)=rival.choose_phase_native_goal_rival_probe(goal_sensory){
            self.push_unified_proposal(
                &mut out,&sensory,goal_sensory,action,
                self.unified_rival_candidate(action,goal_sensory),0xBEEF,
            );
        }

        let mut goal_active=self.clone();
        if let Some(action)=goal_active.choose_phase_native_goal_active_action(goal_sensory){
            self.push_unified_proposal(
                &mut out,&sensory,goal_sensory,action,None,0x60A1,
            );
        }

        let mut general=self.clone();
        if let Some(action)=general.choose_phase_native_abstract_learned_drive_action(){
            self.push_unified_proposal(
                &mut out,&sensory,goal_sensory,action,None,0xE915,
            );
        }

        // A distinct supported goal-reaching route can also be a valuable
        // scientific experiment. Preserve its goal value and relative
        // evidential deficit as a native proposal; common learned U1 weights
        // decide whether it wins against current exploitation.
        if let Some((action,progress,debt)) =
            self.unified_competing_goal_path_coverage(&sensory,goal_sensory)
        {
            if let Some(mut fields) =
                self.unified_action_fields(&sensory,goal_sensory,action)
            {
                fields[0] = fields[0].max(progress).clamp(0.0,1.0);
                fields[1] = fields[1].max(debt).clamp(0.0,1.0);
                out.push(PhaseUnifiedCognitiveProposal {
                    persistent_candidate_id: None,
                    applicability: 1.0,
                    proposal: PhaseCognitiveProposal {
                        proposal_id: unified_hash(&[
                            0xE71Du64, action as u64
                        ]),
                        action, fields,
                    },
                });
            }
        }

        // Only when no qualified cognitive proposal survives: the physical
        // model may be complete but no longer useful for the current raw goal.
        // A bounded uncertainty-bearing revalidation operation then enters
        // the SAME U1 competition as every other external motor proposal.
        if out.is_empty() {
            if let Some((action, debt)) =
                self.unified_exhausted_model_revalidation(
                    &sensory, goal_sensory
                )
            {
                if let Some(mut fields) =
                    self.unified_action_fields(&sensory, goal_sensory, action)
                {
                    fields[1] = fields[1].max(debt).clamp(0.0, 1.0);
                    out.push(PhaseUnifiedCognitiveProposal {
                        persistent_candidate_id: None,
                        applicability: 1.0,
                        proposal: PhaseCognitiveProposal {
                            proposal_id: unified_hash(&[
                                0xC0DEu64, action as u64
                            ]),
                            action,
                            fields,
                        },
                    });
                }
            }
        }
        out
    }

    /// Coalesce proposals by physical motor. This is shared by authority and
    /// read-only diagnostics so traces cannot silently score another policy.
    fn coalesced_phase_native_unified_actions(
        &self,
        proposals:&[PhaseUnifiedCognitiveProposal],
    )->Option<Vec<(usize,[f32;META_FIELD_COUNT],Vec<u64>)>>{
        let native=self.phase_native.as_ref()?;
        let ecology=native.meta_control.as_ref()?.ecology.as_ref()?;

        // Distinct explanations that request the SAME external motor are not
        // competing operations. Coalesce them into one action-level proposal.
        // This rule is source/class agnostic and invariant to enumeration/IDs.
        let mut groups:Vec<(usize,[f32;META_FIELD_COUNT],Vec<u64>)>=Vec::new();

        for item in proposals {
            if !item.applicability.is_finite()
                || !(0.0..=1.0).contains(&item.applicability)
                || !Self::valid_meta_fields(item.proposal.fields)
            {return None;}

            let mut fields=item.proposal.fields;
            let mut supporter=None;
            if let Some(id)=item.persistent_candidate_id {
                if self.phase_native_hypothesis_registered(id) {
                    let authority=self.phase_hypothesis_authority_with_state(
                        native,id,item.applicability
                    )?;
                    if authority<=ecology.config.dormancy_threshold{continue;}
                    fields[3]=(fields[3]*authority).clamp(0.0,1.0);
                }else{
                    // Probation before first factual usefulness update.
                    fields[3]=(fields[3]*item.applicability).clamp(0.0,1.0);
                }
                supporter=Some(id);
            }

            if let Some((_,group_fields,supporters))=groups.iter_mut()
                .find(|(action,_,_)|*action==item.proposal.action)
            {
                for index in 0..META_FIELD_COUNT {
                    group_fields[index]=group_fields[index].max(fields[index]);
                }
                if let Some(id)=supporter {
                    if !supporters.contains(&id){supporters.push(id);}
                }
            }else{
                groups.push((
                    item.proposal.action,
                    fields,
                    supporter.into_iter().collect(),
                ));
            }
        }

        // Evidence-ownership contract: do not let an optimistic goal route
        // masquerade as informed while a physically acquired sensor can
        // still resolve the current competing hypotheses. When sufficient
        // raw facts make the belief decisive, ordinary U1 resumes unaltered.
        groups.retain(|(action,_,_)|
            self.phase_native_temporal_action_evidence_admissible(*action)
        );
        if groups.is_empty(){return None;}
        for (_,_,supporters) in &mut groups {supporters.sort_unstable();}

        Some(groups)
    }

    /// The audit is read-only; it never reveals evaluator labels or selects an
    /// action, and uses exactly the same U1/U2 filtering as the actuator.
    pub fn trace_phase_native_unified_competition(
        &self,
        proposals:&[PhaseUnifiedCognitiveProposal],
    )->Option<Vec<PhaseUnifiedActionTrace>>{
        if self.phase_native_online_enabled(){return None;}
        let groups=self.coalesced_phase_native_unified_actions(proposals)?;
        let mut trace=Vec::with_capacity(groups.len());
        for (action,fields,_) in groups {
            trace.push(PhaseUnifiedActionTrace{
                action,fields,score:self.phase_native_meta_score(fields)?,
            });
        }
        Some(trace)
    }

    pub fn choose_phase_native_unified_proposal(
        &self,
        proposals:&[PhaseUnifiedCognitiveProposal],
    )->Option<PhaseUnifiedDecision>{
        if proposals.is_empty(){return None;}
        if self.phase_native_online_enabled() {
            let [item] = proposals else { return None; };
            if item.proposal.action >= self.config.motor_cells
                || !Self::valid_meta_fields(item.proposal.fields)
            {
                return None;
            }
            return Some(PhaseUnifiedDecision {
                supporting_candidate_ids: Vec::new(),
                proposal_id: item.proposal.proposal_id,
                action: item.proposal.action,
                score: item.proposal.fields[0].max(item.proposal.fields[1]),
            });
        }
        let groups=self.coalesced_phase_native_unified_actions(proposals)?;

        let adjusted=groups.iter().map(|(action,fields,_)|
            PhaseCognitiveProposal{
                proposal_id:unified_hash(&[0xAC710_u64,*action as u64]),
                action:*action,
                fields:*fields,
            }
        ).collect::<Vec<_>>();

        let meta=self.choose_phase_native_meta_proposal(&adjusted)?;
        let (_,_,supporters)=groups.iter()
            .find(|(action,_,_)|*action==meta.action)?;

        Some(PhaseUnifiedDecision{
            supporting_candidate_ids:supporters.clone(),
            proposal_id:meta.proposal_id,
            action:meta.action,
            score:meta.score,
        })
    }

    pub fn phase_native_unified_knowledge_snapshot(
        &self,
    )->PhaseUnifiedKnowledgeSnapshot{
        let circuits=self.phase_native_circuits();
        let revisions=circuits.iter().map(|c|c.revision).sum::<u64>();

        let contexts=self.phase_native_context_witnesses();
        let percepts=self.phase_native_perceptual_witnesses();
        let comps=self.phase_native_composition_witnesses();

        let representation_candidates=
            (contexts.len()+percepts.len()+comps.len()) as u64;
        let promoted_representations=
            contexts.iter().filter(|w|w.promoted).count() as u64
            +percepts.iter().filter(|w|w.promoted).count() as u64
            +comps.iter().filter(|w|w.promoted).count() as u64;
        let evidence_observations=
            contexts.iter().map(|w|w.eligible_observations).sum::<u64>()
            +percepts.iter().map(|w|w.eligible_observations).sum::<u64>()
            +comps.iter().map(|w|w.eligible_observations).sum::<u64>();

        PhaseUnifiedKnowledgeSnapshot{
            circuits:circuits.len() as u64,
            revisions,
            representation_candidates,
            promoted_representations,
            evidence_observations,
        }
    }
}
