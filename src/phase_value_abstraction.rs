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
    raw_dimension: usize,
    link: usize,
    nodes: Vec<PhaseValueRuleNode>,
    completed: u32,
    fits: u32,
    trained_states: usize,
    case_width: usize,
    cases: Vec<PhaseValueCase>,
    validation: (usize,usize),
    effects:Vec<Vec<PhaseValueRuleNode>>,
    reward_priors:Vec<f32>,
    activity:Vec<f32>,
    effect_transfer:bool,
    context_unit_bits:usize,
    effect_support:Vec<Vec<u64>>,
}

#[derive(Clone)]
struct PhaseValueCase { words:Vec<u64>, action:usize }
impl std::fmt::Debug for PhaseValueCase {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        let digest=self.words.iter().fold(0xcbf29ce484222325u64,|h,&w|(h^w).wrapping_mul(0x100000001b3));
        f.debug_struct("PhaseValueCase").field("action",&self.action).field("words",&self.words.len())
            .field("digest",&digest).finish()
    }
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
        self.grow(states, &examples, motors, 0, VALUE_RULE_NODE_CAPACITY, &mut nodes,self.dimension);
        self.nodes = nodes;
        self.trained_states = examples.len();
        self.fits = self.fits.saturating_add(1);
        if self.completed % 128 == 0 {
            self.select_representation(states,&examples,motors);
            self.fit_effects(states,motors);
        }
    }

    fn unit(words:&[u64],start:usize,width:usize)->u64 {
        let shift=start%64;
        let mut value=words.get(start/64).copied().unwrap_or(0)>>shift;
        if shift+width>64 {value|=words.get(start/64+1).copied().unwrap_or(0)<<(64-shift);}
        value&((1<<width)-1)
    }

    fn case_vote(cases:&[PhaseValueCase],words:&[u64],width:usize,dimension:usize,motors:usize)->Option<Vec<f32>> {
        if cases.len()<3 {return None;}
        let mut nearest=[(u64::MAX,usize::MAX);3];
        for (index,case) in cases.iter().enumerate() {
            let distance=if width==1 {
                words.iter().zip(&case.words).map(|(a,b)|(a^b).count_ones() as u64).sum()
            } else {
                (0..dimension).step_by(width).map(|i| {
                    let delta=Self::unit(words,i,width) as i64-Self::unit(&case.words,i,width) as i64;
                    (delta*delta) as u64
                }).sum()
            };
            let entry=(distance,index);
            for slot in 0..3 {
                if entry<nearest[slot] {
                    for j in (slot+1..3).rev() {nearest[j]=nearest[j-1];}
                    nearest[slot]=entry;break;
                }
            }
        }
        let mut out=vec![0.0;motors];
        for (rank,&(_,index)) in nearest.iter().enumerate() {
            out[cases[index].action]+=1.0/3.0+0.0001/(rank+1) as f32;
        }
        let total=out.iter().sum::<f32>();
        for value in &mut out {*value/=total;}
        Some(out)
    }

    fn select_representation(&mut self,states:&[PhaseGeneralValueState],examples:&[(usize,usize,f32)],motors:usize) {
        // A bounded internal validation split uses MODEL targets acquired from
        // real actions, never an external test label. Input packing width is
        // inferred among generic 1..8-bit groupings; no dataset names/layouts.
        let chosen=examples.iter().rev().take(512).copied().collect::<Vec<_>>();
        let training=chosen.iter().enumerate().filter(|(i,_)|i%5!=0).map(|(_,r)|*r).collect::<Vec<_>>();
        let held=chosen.iter().enumerate().filter(|(i,_)|i%5==0).map(|(_,r)|*r).take(64).collect::<Vec<_>>();
        if training.len()<8 || held.len()<4 {return;}
        let mut trial_nodes=Vec::new();
        self.grow(states,&training,motors,0,VALUE_RULE_NODE_CAPACITY,&mut trial_nodes,self.dimension);
        let top=|p:&[f32]|p.iter().enumerate().max_by(|(a,x),(b,y)|x.total_cmp(y).then_with(||b.cmp(a))).unwrap().0;
        let mut best=held.iter().filter(|&&(s,a,_)|Self::tree_prediction(&trial_nodes,&states[s].features)
            .is_some_and(|p|top(&p)==a)).count();
        let tree_score=best;let mut width=0;
        let cases=training.iter().map(|&(s,a,_)|PhaseValueCase {words:states[s].features.clone(),action:a}).collect::<Vec<_>>();
        for candidate in 1..=8 {
            let correct=held.iter().filter(|&&(s,a,_)|Self::case_vote(&cases,&states[s].features,candidate,
                self.dimension,motors).is_some_and(|p|top(&p)==a)).count();
            if correct>best || (correct==best && width>0 && candidate>width) {best=correct;width=candidate;}
        }
        self.validation=(best,held.len());
        // Prefer the smaller existing predicate program on equal accuracy.
        self.case_width=if best>tree_score {width} else {0};
        self.cases=if self.case_width>0 {chosen.iter().map(|&(s,a,_)|
            PhaseValueCase{words:states[s].features.clone(),action:a}).collect()} else {Vec::new()};
    }

    fn fit_effects(&mut self,states:&[PhaseGeneralValueState],motors:usize) {
        self.effect_support=vec![Vec::new();self.raw_dimension.div_ceil(self.context_unit_bits)];
        for state in states.iter().filter(|s|!s.features.is_empty()) {
            for (group,support) in self.effect_support.iter_mut().enumerate() {
                let start=group*self.context_unit_bits;
                let word=Self::unit(&state.features,start,self.context_unit_bits.min(self.raw_dimension-start));
                if support.len()<64 && !support.contains(&word) {support.push(word);}
            }
        }
        let mut rewards=vec![0.0;motors];let mut changed_total=0.0;let mut trials_total=0.0;
        self.effects=vec![Vec::new();motors];
        for action in 0..motors {
            let mut rows=Vec::new();
            for (index,state) in states.iter().enumerate() {
                if state.features.len()!=self.dimension.div_ceil(64) {continue;}
                let outcomes=&state.outcomes[action];
                let count=outcomes.iter().map(|o|o.count as f32).sum::<f32>();
                if count<1.0 {continue;}
                let cost=outcomes.iter().map(|o|o.cost_sum).sum::<f32>()/count;
                let changed=(1.0-(cost-0.002)/0.010).clamp(0.0,1.0);
                rewards[action]+=outcomes.iter().map(|o|o.reward_sum).sum::<f32>();
                changed_total+=changed*count;trials_total+=count;
                rows.push((index,usize::from(changed>=0.5),count.sqrt().min(8.0)));
            }
            if rows.len()>=6 {
                let mut nodes=Vec::new();
                self.grow(states,&rows,2,0,63,&mut nodes,self.raw_dimension);
                self.effects[action]=nodes;
            }
        }
        let total=rewards.iter().sum::<f32>();
        self.effect_transfer=total>=8.0 && rewards.iter().copied().fold(0.0,f32::max)/total>=0.90
            && trials_total>0.0 && changed_total/trials_total>=0.10;
        self.reward_priors=if total>0.0 {rewards.iter().map(|r|r/total).collect()} else {vec![0.0;motors]};
    }

    fn effect_scores(&self,words:&[u64],stalled:&[usize])->Option<Vec<f32>> {
        if !self.effect_transfer {return None;}
        let total=self.activity.iter().sum::<f32>().max(1.0);
        Some(self.effects.iter().enumerate().map(|(action,nodes)| {
            let unsupported=nodes.iter().filter_map(|n|n.predicate).any(|bit| {
                let group=bit/self.context_unit_bits;let start=group*self.context_unit_bits;
                let word=Self::unit(words,start,self.context_unit_bits.min(self.raw_dimension-start));
                !self.effect_support.get(group).is_some_and(|known|known.contains(&word))
            });
            // A correlation with a familiar component of a *new* sensory
            // tuple is not evidence that the old action law still applies.
            let changed=if unsupported && stalled.contains(&action) {0.0}
                else if unsupported {0.5} else {Self::tree_probabilities(nodes,words).map_or(0.5,|p|p[1])};
            let goal=self.reward_priors[action];
            // Preferences come from real completion motors and changed views
            // on successful episodes. No turn/forward names or obstacle IDs.
            goal*(changed+0.0001)+0.005*self.activity[action]/total*changed
        }).collect())
    }

    fn grow(&self, states: &[PhaseGeneralValueState], rows: &[(usize, usize, f32)],
        motors: usize, depth: usize, budget: usize, nodes: &mut Vec<PhaseValueRuleNode>,features_limit:usize) -> usize {
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
        for bit in 0..features_limit {
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
        let zero_index = self.grow(states, &zero, motors, depth+1, left_budget, nodes,features_limit);
        let used=nodes.len()-first_child;
        let one_index = self.grow(states, &one, motors, depth+1, budget-1-used, nodes,features_limit);
        nodes[index].predicate = Some(bit);
        nodes[index].zero = zero_index;
        nodes[index].one = one_index;
        index
    }

    fn predict(&self, words: &[u64],stalled:&[usize]) -> Option<Vec<f32>> {
        if words.len() != self.dimension.div_ceil(64) { return None; }
        if let Some(scores)=self.effect_scores(words,stalled) {return Some(scores);}
        if self.case_width>0 {
            return Self::case_vote(&self.cases,words,self.case_width,self.dimension,self.nodes.first()?.probabilities.len());
        }
        Self::tree_prediction(&self.nodes,words)
    }

    fn tree_prediction(nodes:&[PhaseValueRuleNode],words:&[u64])->Option<Vec<f32>> {
        let p=Self::tree_probabilities(nodes,words)?;
        if p.iter().copied().fold(0.0,f32::max)<0.65 {return None;}
        Some(p)
    }

    fn tree_probabilities(nodes:&[PhaseValueRuleNode],words:&[u64])->Option<Vec<f32>> {
        let mut node = nodes.first()?;
        while let Some(bit) = node.predicate {
            node = nodes.get(if Self::bit(words, bit) { node.one } else { node.zero })?;
        }
        if node.support < 3 {
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
        let raw_dimension=self.config.sensory_cells;
        // Public tuple structure, not category meanings or motor semantics.
        let context_unit_bits=policy.relational_workspace.as_ref()
            .map_or(1,|w|(w.channels*w.bits).clamp(1,32));
        let motors=self.config.motor_cells;
        let Some(cell) = self.dormant_range().find(|&c|!self.cells[c].recruited) else {return false;};
        self.cells[cell].recruited = true;
        let link = self.native_synapse(0, cell);
        let syn = &mut self.synapses[link];
        syn.weight=0.6; syn.confidence=1.0; syn.eligibility=1.0;
        syn.phase_offset=wrap_phase(self.cells[syn.to].phase-self.cells[syn.from].phase);
        self.phase_native.as_mut().unwrap().general_policy.as_mut().unwrap()
            .value_learning.as_mut().unwrap().abstraction = Some(PhaseValueAbstraction {
                dimension,raw_dimension, link, nodes:Vec::new(), completed:0, fits:0, trained_states:0,
                case_width:0,cases:Vec::new(),validation:(0,0),effects:Vec::new(),reward_priors:Vec::new(),
                activity:vec![0.0;motors],effect_transfer:false,context_unit_bits,effect_support:Vec::new() });
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

    pub fn phase_native_value_representation_status(&self)->(usize,usize,usize,usize) {
        self.phase_native.as_ref().and_then(|n|n.general_policy.as_ref())
            .and_then(|p|p.value_learning.as_ref()).and_then(|v|v.abstraction.as_ref())
            .map_or((0,0,0,0),|a|(a.case_width,a.cases.len(),a.validation.0,a.validation.1))
    }

    pub fn phase_native_value_effect_status(&self)->(bool,usize) {
        self.phase_native.as_ref().and_then(|n|n.general_policy.as_ref())
            .and_then(|p|p.value_learning.as_ref()).and_then(|v|v.abstraction.as_ref())
            .map_or((false,0),|a|(a.effect_transfer,a.effects.iter().map(Vec::len).sum()))
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
        let value=native.general_policy.as_ref()?.value_learning.as_ref()?;
        let abstraction=value.abstraction.as_ref()?;
        if conductance(&self.cells,&self.synapses[abstraction.link],native.config.coherence_floor)<=1e-7 {return None;}
        let key=self.general_value_key(raw,false)?;
        let stalled=value.transient_stalls.iter().filter(|&&(k,_)|k==key).map(|&(_,a)|a).collect::<Vec<_>>();
        abstraction.predict(&self.value_abstraction_features(raw,false)?,&stalled)
    }
}
