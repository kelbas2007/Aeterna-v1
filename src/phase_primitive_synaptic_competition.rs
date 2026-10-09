// Optional PHYSICAL candidate competition for acquired operation arguments.
//
// Rust simulates the neurons and their plasticity, but no Rust-side matcher
// scans historical facts to choose a binding in this mode. Instead parallel
// candidates are represented by phase synapses; the same immutable acquired
// operation predicts each factual outcome, and physical weights accumulate
// agreement. Later decisions read the winning live synapse only. The finite
// set of candidate channels and plasticity rule remain engineered substrate;
// it is NOT spontaneous invention of the algorithm or an AGI claim.

fn synaptic_argument_winner(
    action:usize,
    p:&PhasePrimitiveState,
    links:&[PhaseSynapse],
)->Option<PrimitiveArgumentMatch>{
    if !p.native_argument_competition{return None;}
    let candidate=p.argument_candidates.iter()
        .filter(|c|c.action==action && c.observations>=16)
        .filter(|c|links.get(c.candidate_synapse)
            .is_some_and(|l|l.confidence>=0.5 && l.weight>=0.83))
        .max_by(|a,b|{
            links[a.candidate_synapse].weight
                .total_cmp(&links[b.candidate_synapse].weight)
                .then_with(||b.operation_index.cmp(&a.operation_index))
                .then_with(||b.candidate_synapse.cmp(&a.candidate_synapse))
        })?;
    let syn=&links[candidate.candidate_synapse];
    Some(PrimitiveArgumentMatch {
        operation_index:candidate.operation_index,
        source_inputs:candidate.source_inputs.to_vec(),
        arguments:vec![syn.from,syn.to],
        fit_facts:candidate.observations as usize,
        future_checks:0,
    })
}

impl EvoPhase {
    /// Activate direct factual learning in a bounded physical population of
    /// argument associations. This is only available for two-source acquired
    /// operations on 2-4 input channels; larger cases remain unsupported.
    /// The host supplies neither the target pair nor the chosen output.
    pub fn enable_phase_primitive_synaptic_argument_competition(&mut self)->bool {
        let width=self.config.sensory_cells;
        if !(2..=4).contains(&width){return false;}
        let Some(n)=self.phase_native.as_ref() else{return false;};
        let Some(i)=n.vector.as_ref().and_then(|v|v.induction.as_ref())
            else{return false;};
        let Some(p)=i.primitives.as_ref() else{return false;};
        if !n.config.learning_enabled || !p.argument_transfer_enabled
            || p.native_argument_competition || !p.bound_calls.is_empty(){
            return false;
        }
        let mut available=Vec::new();
        for (op_index,_) in p.operations.iter().enumerate(){
            if let Some(inputs)=primitive_argument_sources(
                op_index,p,width,&self.cells,&self.synapses
            ) {
                if inputs.len()==2 {
                    available.push((op_index,[inputs[0],inputs[1]]));
                }
            }
        }
        if available.is_empty(){return false;}
        let required=i.programs.len()*available.len()*width*(width-1);
        if self.synapses.len().saturating_add(required)>MAX_VECTOR_SYNAPSES{
            return false;
        }
        let mut hypotheses=Vec::with_capacity(required);
        for action in 0..i.programs.len(){
            for &(op_index,sources) in &available {
                for from in 0..width {
                    for to in 0..width {
                        if from==to{continue;}
                        let synapse=self.native_synapse(from,to);
                        let s=&mut self.synapses[synapse];
                        s.phase_offset=wrap_phase(
                            self.cells[to].phase-self.cells[from].phase
                        );
                        s.weight=0.5;
                        s.confidence=1.0;
                        hypotheses.push(PhaseSynapticArgumentCandidate{
                            action,operation_index:op_index,
                            source_inputs:sources,
                            candidate_synapse:synapse,observations:0,
                        });
                    }
                }
            }
        }
        let p=self.phase_native.as_mut().unwrap().vector.as_mut().unwrap()
            .induction.as_mut().unwrap().primitives.as_mut().unwrap();
        p.argument_candidates=hypotheses;
        p.native_argument_competition=true;
        true
    }

    /// Factual, local synapse update. Counterfactual calls reuse the original
    /// acquired computation; no correct argument mapping is ever accepted.
    pub(crate) fn observe_phase_primitive_synaptic_argument_credit(
        &mut self,action:usize
    )->usize {
        let Some(n)=self.phase_native.as_ref() else{return 0;};
        if !n.config.learning_enabled{return 0;}
        let Some(i)=n.vector.as_ref().and_then(|v|v.induction.as_ref())
            else{return 0;};
        let Some(p)=i.primitives.as_ref() else{return 0;};
        if !p.native_argument_competition{return 0;}
        let Some(fact)=i.programs.get(action).and_then(|x|x.facts.back())
            else{return 0;};
        let step=(p.config.policy_learning_rate*6.0).clamp(0.05,0.2);
        let mut reinforcement=Vec::new();
        for (index,c) in p.argument_candidates.iter().enumerate()
            .filter(|(_,c)|c.action==action){
            let Some(l)=self.synapses.get(c.candidate_synapse) else{continue;};
            let call=PrimitiveArgumentMatch{
                operation_index:c.operation_index,
                source_inputs:c.source_inputs.to_vec(),
                arguments:vec![l.from,l.to],
                fit_facts:0,future_checks:0,
            };
            if let Some((value,_))=primitive_argument_read(
                &call,p,&i.config,&fact.input,&self.cells,&self.synapses
            ){
                let agreement=f32::from(
                    (value-f32::from(fact.success)).abs()
                        <=1.0-i.config.minimum_outcome
                );
                reinforcement.push((index,c.candidate_synapse,agreement));
            }
        }
        for &(index,address,agreement) in &reinforcement {
            let syn=&mut self.synapses[address];
            syn.weight=(syn.weight*(1.0-step)+step*agreement).clamp(0.0,1.0);
            syn.eligibility=1.0;
            self.phase_native.as_mut().unwrap().vector.as_mut().unwrap()
                .induction.as_mut().unwrap().primitives.as_mut().unwrap()
                .argument_candidates[index].observations=self.phase_native
                    .as_ref().unwrap().vector.as_ref().unwrap()
                    .induction.as_ref().unwrap().primitives.as_ref().unwrap()
                    .argument_candidates[index].observations.saturating_add(1);
        }
        reinforcement.len()
    }

    /// Physical winner readout. Report candidate addresses and weights for
    /// falsifiable synapse-lesion controls.
    pub fn phase_primitive_synaptic_argument_winner(
        &self,action:usize
    )->Option<(usize,Vec<usize>,usize,f32)>{
        let p=self.phase_native.as_ref()?.vector.as_ref()?.induction.as_ref()?
            .primitives.as_ref()?;
        let winner=synaptic_argument_winner(action,p,&self.synapses)?;
        let found=p.argument_candidates.iter().find(|c|
            c.action==action && c.operation_index==winner.operation_index
                && self.synapses[c.candidate_synapse].from==winner.arguments[0]
                && self.synapses[c.candidate_synapse].to==winner.arguments[1]
        )?;
        Some((p.operations[winner.operation_index].program.nodes[0].cell,
            winner.arguments,found.candidate_synapse,
            self.synapses[found.candidate_synapse].weight))
    }

    pub fn phase_primitive_synaptic_argument_hypothesis_count(&self)->usize{
        self.phase_native.as_ref().and_then(|n|n.vector.as_ref())
            .and_then(|v|v.induction.as_ref())
            .and_then(|i|i.primitives.as_ref())
            .map(|p|p.argument_candidates.len()).unwrap_or(0)
    }
}
