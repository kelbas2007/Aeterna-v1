// Experience-derived predicate programs for unseen observations. The substrate
// implements CART induction; learned predicates/actions are not written routes.
// Values used as targets are MODEL estimates from witnessed transitions, not
// additional REAL action labels or rewards. This is not primitive invention.
const VALUE_RULE_NODE_CAPACITY: usize = 255;
const VALUE_RULE_DEPTH: usize = 12;

#[derive(Debug, Clone)]
struct PhaseValueRuleNode {
    predicate: Option<usize>,
    zero: usize,
    one: usize,
    probabilities: Vec<f32>,
    support: usize,
}

#[derive(Debug, Clone)]
struct PhaseValueAbstraction {
    dimension: usize,
    link: usize,
    nodes: Vec<PhaseValueRuleNode>,
    completed: u32,
    fits: u32,
    trained_states: usize,
}

impl PhaseValueAbstraction {
    fn pack(raw: &[f32]) -> Vec<u64> {
        let mut out = vec![0; raw.len().div_ceil(64)];
        for (i, &v) in raw.iter().enumerate() {
            if v == 1.0 { out[i / 64] |= 1 << (i % 64); }
        }
        out
    }

    fn bit(words: &[u64], i: usize) -> bool {
        words.get(i / 64).is_some_and(|w| w & (1 << (i % 64)) != 0)
    }

    fn fit(&mut self, states: &[PhaseGeneralValueState]) {
        let mut examples = Vec::<(usize, usize, f32)>::new();
        for (i, s) in states.iter().enumerate() {
            if s.features.len() != self.dimension.div_ceil(64) { continue; }
            let Some((action, &best)) = s.values.iter().enumerate()
                .filter(|(a, _)| s.visits[*a] > 0)
                .max_by(|(a, x), (b, y)| x.total_cmp(y).then_with(|| b.cmp(a))) else { continue; };
            if best <= 0.04 || s.visits.iter().map(|&n|u64::from(n)).sum::<u64>() < 3 { continue; }
            // Evidence affects the weight, not the number of REAL transitions.
            let weight = (s.visits[action] as f32).sqrt().clamp(1.0, 8.0);
            examples.push((i, action, weight));
        }
        if examples.len() < 8 { return; }
        let motors = states[examples[0].0].values.len();
        let mut nodes = Vec::new();
        self.grow(states, &examples, motors, 0, VALUE_RULE_NODE_CAPACITY, &mut nodes);
        self.nodes = nodes;
        self.trained_states = examples.len();
        self.fits = self.fits.saturating_add(1);
    }

    fn grow(&self, states: &[PhaseGeneralValueState], rows: &[(usize, usize, f32)],
        motors: usize, depth: usize, budget: usize, nodes: &mut Vec<PhaseValueRuleNode>) -> usize {
        let mut votes = vec![0.0; motors];
        for &(_, action, weight) in rows { votes[action] += weight; }
        let total: f32 = votes.iter().sum();
        let probabilities = votes.iter().map(|v| v / total).collect::<Vec<_>>();
        let index = nodes.len();
        nodes.push(PhaseValueRuleNode { predicate: None, zero: 0, one: 0,
            probabilities, support: rows.len() });
        if budget < 3 || depth >= VALUE_RULE_DEPTH || rows.len() < 6
            || votes.iter().filter(|&&v| v > 0.0).count() == 1 { return index; }
        let impurity = |v: &[f32], n: f32| n - v.iter().map(|x| x * x).sum::<f32>() / n;
        let parent = impurity(&votes, total);
        let mut best = None;
        let mut best_gain = 0.001f32;
        for bit in 0..self.dimension {
            let mut one = vec![0.0; motors];
            let mut count = 0;
            for &(s, action, weight) in rows {
                if Self::bit(&states[s].features, bit) { one[action] += weight; count += 1; }
            }
            if count < 2 || rows.len() - count < 2 { continue; }
            let n: f32 = one.iter().sum();
            let zero = votes.iter().zip(&one).map(|(a,b)| a-b).collect::<Vec<_>>();
            let gain = parent - impurity(&one, n) - impurity(&zero, total-n);
            if gain > best_gain { best_gain = gain; best = Some(bit); }
        }
        let Some(bit) = best else { return index; };
        let (one, zero): (Vec<_>, Vec<_>) = rows.iter().copied()
            .partition(|&(s, _, _)| Self::bit(&states[s].features, bit));
        let pure=|part:&[(usize,usize,f32)]| part.iter().all(|r|r.1==part[0].1);
        let left_budget = if pure(&zero) {1} else if pure(&one) {budget-2}
            else {((budget-1)*zero.len()/rows.len()).clamp(1,budget-2)};
        let first_child=nodes.len();
        let zero_index = self.grow(states, &zero, motors, depth+1, left_budget, nodes);
        let used=nodes.len()-first_child;
        let one_index = self.grow(states, &one, motors, depth+1, budget-1-used, nodes);
        nodes[index].predicate = Some(bit);
        nodes[index].zero = zero_index;
        nodes[index].one = one_index;
        index
    }

    fn predict(&self, words: &[u64]) -> Option<Vec<f32>> {
        if words.len() != self.dimension.div_ceil(64) { return None; }
        let mut node = self.nodes.first()?;
        while let Some(bit) = node.predicate {
            node = self.nodes.get(if Self::bit(words, bit) { node.one } else { node.zero })?;
        }
        if node.support < 3 || node.probabilities.iter().copied().fold(0.0, f32::max) < 0.65 {
            return None;
        }
        Some(node.probabilities.clone())
    }
}

impl EvoPhase {
    /// Learn bounded predicates from acquired values. Must be enabled cold;
    /// legacy controllers and exact known-state decisions remain unchanged.
    pub fn enable_phase_native_value_abstraction(&mut self) -> bool {
        let Some(policy) = self.phase_native.as_ref().and_then(|n|n.general_policy.as_ref()) else {return false;};
        let Some(value) = policy.value_learning.as_ref() else {return false;};
        if value.abstraction.is_some() || !value.states.is_empty() {return false;}
        let identity = policy.relational_workspace.as_ref().map_or(0, |w|w.identity_channels);
        let dimension = self.config.sensory_cells * 3 + 12 * identity * 8 + 8;
        let Some(cell) = self.dormant_range().find(|&c|!self.cells[c].recruited) else {return false;};
        self.cells[cell].recruited = true;
        let link = self.native_synapse(0, cell);
        let syn = &mut self.synapses[link];
        syn.weight=0.6; syn.confidence=1.0; syn.eligibility=1.0;
        syn.phase_offset=wrap_phase(self.cells[syn.to].phase-self.cells[syn.from].phase);
        self.phase_native.as_mut().unwrap().general_policy.as_mut().unwrap()
            .value_learning.as_mut().unwrap().abstraction = Some(PhaseValueAbstraction {
                dimension, link, nodes:Vec::new(), completed:0, fits:0, trained_states:0 });
        true
    }

    pub fn phase_native_value_abstraction_link(&self) -> Option<usize> {
        Some(self.phase_native.as_ref()?.general_policy.as_ref()?.value_learning.as_ref()?.abstraction.as_ref()?.link)
    }

    pub fn phase_native_value_abstraction_status(&self) -> (usize, u32, usize) {
        self.phase_native.as_ref().and_then(|n|n.general_policy.as_ref())
            .and_then(|p|p.value_learning.as_ref()).and_then(|v|v.abstraction.as_ref())
            .map_or((0,0,0), |a|(a.nodes.len(), a.fits, a.trained_states))
    }

    pub fn phase_native_value_predicate_programs(&self) -> Vec<(Option<usize>,usize,usize,Vec<f32>,usize)> {
        self.phase_native.as_ref().and_then(|n|n.general_policy.as_ref())
            .and_then(|p|p.value_learning.as_ref()).and_then(|v|v.abstraction.as_ref())
            .map_or_else(Vec::new,|a|a.nodes.iter().map(|n|
                (n.predicate,n.zero,n.one,n.probabilities.clone(),n.support)).collect())
    }

    fn value_abstraction_features(&self, raw: &[f32], post: bool) -> Option<Vec<u64>> {
        let native = self.phase_native.as_ref()?;
        let policy = native.general_policy.as_ref()?;
        let value = policy.value_learning.as_ref()?;
        let abstraction = value.abstraction.as_ref()?;
        let d = self.config.sensory_cells;
        if raw.len()!=d { return None; }
        let mut out=vec![0u64; abstraction.dimension.div_ceil(64)];
        let mut put=|i:usize, v:bool| { if v {out[i/64] |= 1<<(i%64);} };
        for (i,&v) in raw.iter().enumerate() {put(i,v==1.0);}
        let memory = value.memory_link.is_some_and(|l|conductance(&self.cells,
            &self.synapses[l], native.config.coherence_floor)>1e-7);
        if memory {
            for i in 0..d {put(d+i, PhaseValueAbstraction::bit(&value.initial_frame,i));}
            if let Some(w) = policy.relational_workspace.as_ref() {
                if conductance(&self.cells,&self.synapses[w.synapse],native.config.coherence_floor)>1e-7 {
                    let mut work=w.clone();
                    if post {work.next(raw);}
                    for (i,&v) in work.subject_observation.iter().enumerate() {put(2*d+i,v==1.0);}
                    for (slot,signature) in work.first_appearances.iter().take(12).enumerate() {
                        for (channel,&v) in signature.iter().enumerate() {
                            for bit in 0..8 {put(3*d+(slot*w.identity_channels+channel)*8+bit, v&(1<<bit)!=0);}
                        }
                    }
                    for bit in 0..8 {put(abstraction.dimension-8+bit,work.first_appearances.len()&(1<<bit)!=0);}
                }
            }
        }
        Some(out)
    }

    fn value_abstraction_prediction(&self, raw: &[f32]) -> Option<Vec<f32>> {
        let native=self.phase_native.as_ref()?;
        let abstraction=native.general_policy.as_ref()?.value_learning.as_ref()?.abstraction.as_ref()?;
        if conductance(&self.cells,&self.synapses[abstraction.link],native.config.coherence_floor)<=1e-7 {return None;}
        abstraction.predict(&self.value_abstraction_features(raw,false)?)
    }
}
