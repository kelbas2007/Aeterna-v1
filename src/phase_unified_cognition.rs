// Unified-cognition assembly for INTEL-2.
//
// Enumeration is allowed to know which qualified mechanisms can emit an
// operation. Authority is not: every emitted action receives the SAME generic
// action-level fields, and the winner is selected by U1/U2/U3.

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
        let state_cells = self.phase_native_abstract_cells_at_level(entry.level);

        let goal_value = {
            let mut probe = self.clone();
            match probe.plan_phase_native_abstract_goal(sensory, goal_sensory, None) {
                Some(decision) if decision.first_action == action =>
                    decision.predicted_value.clamp(0.0,1.0),
                _ => 0.0,
            }
        };

        let frontier = self.phase_drive_frontier_activity_for_cells(state, &state_cells);
        let drive_features = self.phase_drive_features(state, entry.cell, action, &frontier);
        let epistemic_value = drive_features[0].max(drive_features[1]).clamp(0.0,1.0);

        let relevance =
            self.phase_goal_relevance_for_cells(state, goal.cell, &state_cells);
        let contradiction = self.phase_goal_rival_disagreement_score(
            state, entry.cell, action, &relevance
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

    fn unified_state_action_candidate(
        &self,
        action: usize,
    ) -> Option<(u64,f32)> {
        if action >= self.config.motor_cells { return None; }
        let sensory = self.current_real.as_ref()?.sensory.clone();
        let entry = self.phase_native_abstract_state(&sensory)?;
        Some((
            unified_hash(&[
                0xA6710_u64,
                entry.level as u64,
                entry.cell as u64,
                action as u64,
            ]),
            1.0,
        ))
    }

    fn unified_state_action_available(&self, action: usize) -> bool {
        let Some((candidate_id,_)) = self.unified_state_action_candidate(action)
            else { return false; };
        if !self.phase_native_hypothesis_registered(candidate_id) {
            return true;
        }
        !self.phase_native_hypothesis_dormant(candidate_id).unwrap_or(true)
    }

    /// Unified-only G16 selector with the same learned drive score and
    /// evidence-order tie-break as the qualified selector. The only additional
    /// constraint is carrier-owned U2 dormancy for the current state-action
    /// candidate. No source/world/task identity enters the ranking.
    fn unified_general_action(&self) -> Option<usize> {
        let sensory = self.current_real.as_ref()?.sensory.clone();
        let entry = self.phase_native_abstract_state(&sensory)?;
        let state_cells = self.phase_native_abstract_cells_at_level(entry.level);
        if !state_cells.contains(&entry.cell) {
            return None;
        }

        let state = self.phase_native.as_ref()?;
        if state
            .drive
            .as_ref()
            .map(|drive| drive.config.readout_enabled)
            != Some(true)
        {
            return None;
        }

        let frontier =
            self.phase_drive_frontier_activity_for_cells(state, &state_cells);
        let min_support = u64::from(self.config.min_recruit_support);
        let mut best: Option<(usize, f32, u64)> = None;

        for action in 0..self.config.motor_cells {
            if !self.unified_state_action_available(action) {
                continue;
            }

            let features =
                self.phase_drive_features(state, entry.cell, action, &frontier);
            let score = self.phase_drive_score(state, features)?;
            let global_support = state
                .circuits
                .iter()
                .filter(|circuit| {
                    circuit.support >= min_support
                        && self.synapses[circuit.motor_synapse].to
                            == self.config.sensory_cells + action
                })
                .map(|circuit| circuit.support)
                .sum::<u64>();

            match best {
                None => best = Some((action, score, global_support)),
                Some((best_action, best_score, best_support)) => {
                    if score > best_score + 1.0e-6
                        || ((score - best_score).abs() <= 1.0e-6
                            && (global_support > best_support
                                || (global_support == best_support
                                    && action < best_action)))
                    {
                        best = Some((action, score, global_support));
                    }
                }
            }
        }

        let (action, score, _) = best?;
        if score <= 1.0e-8 {
            let mut probe = self.clone();
            let decision = probe.phase_native_decision_from_cell(entry.cell, None)?;
            return self.unified_state_action_available(decision.first_action)
                .then_some(decision.first_action);
        }
        Some(action)
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
        let Some(sensory)=self.current_real.as_ref().map(|r|r.sensory.clone())
            else{return Vec::new();};
        if self.phase_native_abstract_state(goal_sensory).is_none(){
            return Vec::new();
        }
        let mut out=Vec::new();

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
                &mut out,&sensory,goal_sensory,action,
                self.unified_state_action_candidate(action),0x60A1,
            );
        }

        if let Some(action)=self.unified_general_action(){
            self.push_unified_proposal(
                &mut out,&sensory,goal_sensory,action,
                self.unified_state_action_candidate(action),0xE915,
            );
        }

        out
    }

    pub fn choose_phase_native_unified_proposal(
        &self,
        proposals:&[PhaseUnifiedCognitiveProposal],
    )->Option<PhaseUnifiedDecision>{
        if proposals.is_empty(){return None;}
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

        if groups.is_empty(){return None;}
        for (_,_,supporters) in &mut groups {supporters.sort_unstable();}

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
