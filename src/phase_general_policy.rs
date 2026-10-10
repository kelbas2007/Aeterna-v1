// GENERAL-POLICY-1: opt-in *replacement* for scripted policy authorities.
//
// Raw binary sensory frame -> distributed hashed representation ->
// learned action preference / predictive novelty -> protected motor.
// External rewards are credited backward by temporal eligibility traces.
// NO object words, key/door IDs, scene geometry, MiniGrid motor labels,
// navigation routes, privileged simulator state, or handcoded subgoals.
// Learning/selection mathematics is of course programmed Rust; this
// does not establish that the SNN invented its own learning algorithm.
const GENERAL_DIM:usize=96;
const GENERAL_ACTOR_DIM:usize=GENERAL_DIM*2;
const GENERAL_TRACE_DECAY:f32=0.91;
const GENERAL_LEARNING_RATE:f32=0.10;

#[derive(Debug,Clone)]
struct PhaseGeneralPolicy {
    weights: Vec<Vec<f32>>,
    predictions: Vec<Vec<f32>>,
    prediction_visits: Vec<u32>,
    physical_links: Vec<usize>,
    steps:u64,
    updates:u64,
    positive_rewards:u32,
    // One continuing causal subject, not independent visible frames. Opt-in.
    // The second half of its ACTOR features represents prior observations.
    developmental_memory:bool,
    working_trace:Vec<f32>,
    retained_events:u64,
    // Temporary lifetime-episode history, never transferred as knowledge.
    recent:Vec<(u64,usize)>,
    eligibility:Vec<Vec<f32>>,
}
#[derive(Debug,Clone,Copy,PartialEq)]
pub struct PhaseGeneralDecision {
    pub action:usize,
    pub score:f32,
    pub synapse:usize,
}
impl EvoPhase {
    fn general_frame_features(raw:&[f32])->Option<Vec<f32>> {
        if raw.is_empty()||raw.iter().any(|v|!v.is_finite()
            || (*v!=0.0&&*v!=1.0)) {return None;}
        let mut out=vec![0.0f32;GENERAL_DIM];
        out[0]=1.0;
        let mut active=0usize;
        for (index,&value) in raw.iter().enumerate(){
            if value<0.5 {continue;}
            active+=1;
            let mut hash=(index as u64).wrapping_add(0x9E3779B97F4A7C15);
            hash=(hash^(hash>>30)).wrapping_mul(0xBF58476D1CE4E5B9);
            hash=(hash^(hash>>27)).wrapping_mul(0x94D049BB133111EB);
            hash^=hash>>31;
            let a=1+(hash as usize)%(GENERAL_DIM-1);
            let b=1+((hash>>32) as usize)%(GENERAL_DIM-1);
            out[a]+=if hash&1==0{1.0}else{-1.0};
            out[b]+=if hash&2==0{1.0}else{-1.0};
        }
        let divisor=(1.0+(active as f32)*0.45).sqrt();
        for v in out.iter_mut().skip(1){*v=(*v/divisor).clamp(-1.0,1.0);}
        Some(out)
    }
    fn general_dot(weights:&[f32], features:&[f32])->f32{
        weights.iter().zip(features).map(|(w,f)|w*f).sum()
    }
    fn general_hash(raw:&[f32])->u64{
        let mut v=0xcbf29ce484222325u64;
        for &x in raw {
            v^=(x>=0.5) as u64;
            v=v.wrapping_mul(0x100000001b3);
        }
        v
    }
    pub fn enable_phase_native_general_policy(&mut self)->bool{
        let Some(native)=self.phase_native.as_ref() else{return false;};
        if native.general_policy.is_some(){return false;}
        if self.config.motor_cells==0||self.config.motor_cells>32{return false;}
        let n=self.config.motor_cells;
        let mut links=Vec::with_capacity(n);
        for a in 0..n{
            let index=self.native_synapse(0,self.config.sensory_cells+a);
            let syn=&mut self.synapses[index];
            syn.weight=0.65;
            syn.confidence=1.0;
            syn.eligibility=1.0;
            syn.phase_offset=wrap_phase(
                self.cells[syn.to].phase-self.cells[syn.from].phase
            );
            links.push(index);
        }
        let Some(state)=self.phase_native.as_mut() else{return false;};
        state.general_policy=Some(PhaseGeneralPolicy{
            weights:vec![vec![0.0;GENERAL_ACTOR_DIM];n],
            predictions:vec![vec![0.0;GENERAL_DIM];n],
            prediction_visits:vec![0;n],
            physical_links:links,
            steps:0,updates:0,positive_rewards:0,
            developmental_memory:false,
            working_trace:vec![0.0;GENERAL_DIM],retained_events:0,
            recent:Vec::new(),
            eligibility:vec![vec![0.0;GENERAL_ACTOR_DIM];n],
        });
        true
    }
    /// Fundamental experiment: the policy now conditions decisions on a
    /// bounded history of its own actual experience, not just current pixels.
    /// This is a hand-designed, leaky *working memory prototype*, not a human
    /// child brain or unsupervised symbolic world understanding.
    pub fn enable_phase_native_developmental_memory(&mut self)->bool{
        let Some(state)=self.phase_native.as_mut()
            .and_then(|n|n.general_policy.as_mut()) else {return false;};
        if state.developmental_memory{return false;}
        state.developmental_memory=true;
        state.working_trace.fill(0.0);
        state.retained_events=0;
        true
    }
    pub fn phase_native_developmental_memory_enabled(&self)->bool{
        self.phase_native.as_ref().and_then(|n|n.general_policy.as_ref())
            .is_some_and(|m|m.developmental_memory)
    }
    pub fn phase_native_developmental_retained_events(&self)->u64{
        self.phase_native.as_ref().and_then(|n|n.general_policy.as_ref())
            .map_or(0,|m|m.retained_events)
    }
    /// First factual observation starts an episode; a later blank/similar
    /// observation does not erase the preceding cue. Never receives a
    /// hidden state label or a correct motor.
    pub fn observe_phase_native_general_initial(&mut self,raw:&[f32])->bool{
        let Some(features)=Self::general_frame_features(raw) else{return false;};
        let Some(state)=self.phase_native.as_mut()
            .and_then(|n|n.general_policy.as_mut()) else{return false;};
        if !state.developmental_memory{return true;}
        state.working_trace=features;
        state.retained_events=1;
        true
    }
    fn general_actor_features(
        observation:&[f32],model:&PhaseGeneralPolicy
    )->Vec<f32>{
        let mut features=Vec::with_capacity(GENERAL_ACTOR_DIM);
        features.extend_from_slice(observation);
        if model.developmental_memory{
            features.extend_from_slice(&model.working_trace);
        }else{
            features.extend(std::iter::repeat_n(0.0,GENERAL_DIM));
        }
        features
    }
    pub fn phase_native_general_enabled(&self)->bool{
        self.phase_native.as_ref().is_some_and(|n|n.general_policy.is_some())
    }
    pub fn phase_native_general_rewards(&self)->u32{
        self.phase_native.as_ref().and_then(|n|n.general_policy.as_ref())
            .map_or(0,|s|s.positive_rewards)
    }
    pub fn phase_native_general_updates(&self)->u64{
        self.phase_native.as_ref().and_then(|n|n.general_policy.as_ref())
            .map_or(0,|s|s.updates)
    }
    pub fn phase_native_general_synapse(&self,action:usize)->Option<usize>{
        self.phase_native.as_ref()?.general_policy.as_ref()?
            .physical_links.get(action).copied()
    }
    pub fn is_phase_native_general_synapse(&self,index:usize)->bool{
        self.phase_native.as_ref().and_then(|n|n.general_policy.as_ref())
            .is_some_and(|s|s.physical_links.contains(&index))
    }
    pub fn begin_phase_native_general_episode(&mut self){
        if let Some(s)=self.phase_native.as_mut()
            .and_then(|n|n.general_policy.as_mut()){
            s.recent.clear();
            s.working_trace.fill(0.0);
            s.retained_events=0;
            for row in &mut s.eligibility{row.fill(0.0);}
        }
    }
    pub fn choose_phase_native_general_action(
        &self,raw:&[f32]
    )->Option<PhaseGeneralDecision>{
        let native=self.phase_native.as_ref()?;
        let model=native.general_policy.as_ref()?;
        let present=Self::general_frame_features(raw)?;
        let features=Self::general_actor_features(&present,model);
        let h=Self::general_hash(raw);
        let seen=|a:usize|model.recent.iter().any(|&(state,action)|
            state==h && action==a);
        let novelty_possible=(0..self.config.motor_cells).any(|a|!seen(a));
        let mut best=None::<PhaseGeneralDecision>;
        for action in 0..self.config.motor_cells{
            let physical=conductance(
                &self.cells,&self.synapses[model.physical_links[action]],
                native.config.coherence_floor
            );
            if physical<=1e-7 || (novelty_possible&&seen(action)){continue;}
            let acquired=Self::general_dot(&model.weights[action],&features);
            let pred=&model.predictions[action];
            let surprise=(1.0-Self::general_dot(pred,&features)
                /GENERAL_DIM as f32).max(0.0);
            let coverage=1.0/(1.0+(model.prediction_visits[action] as f32).sqrt());
            // Common stochastic tie-breaking derived from physical carrier tick
            // and raw sensed state. There is no world/action meaning.
            let mut salt=h^(action as u64).wrapping_mul(0x9E3779B97F4A7C15)
                ^self.tick;
            salt^=salt>>12;salt^=salt<<25;salt^=salt>>27;
            let jitter=(salt.wrapping_mul(0x2545F4914F6CDD1D)>>40)
                as f32/(1u32<<24) as f32;
            let learning=native.config.learning_enabled;
            let inherited=if learning {
                self.phase_native_innate_action_bias(action)
            }else{0.0};
            let score=acquired+inherited
                +if learning {0.20*surprise+0.10*coverage+0.15*jitter}
                    else {0.07*surprise+0.025*jitter};
            let candidate=PhaseGeneralDecision{
                action,score:score*physical,synapse:model.physical_links[action]
            };
            if best.is_none_or(|value|candidate.score>value.score+1e-7){
                best=Some(candidate);
            }
        }
        best
    }
    /// The only training signal is factual raw PRE, executed opaque motor,
    /// factual raw POST, externally observed reward. Eligibility traces
    /// assign delayed terminal reward to predecessor actions.
    pub fn observe_phase_native_general_transition(
        &mut self,action:usize,pre:&[f32],post:&[f32],reward:f32
    )->bool{
        if action>=self.config.motor_cells||!reward.is_finite()
            || !(0.0..=1.0).contains(&reward){return false;}
        let (Some(before),Some(after))=(
            Self::general_frame_features(pre),
            Self::general_frame_features(post)
        ) else{return false;};
        let h=Self::general_hash(pre);
        let learning=self.phase_native.as_ref()
            .is_some_and(|n|n.config.learning_enabled);
        let Some(model)=self.phase_native.as_mut()
            .and_then(|n|n.general_policy.as_mut()) else{return false;};
        model.steps+=1;
        if model.recent.len()>=512{model.recent.remove(0);}
        model.recent.push((h,action));
        let actor_before=Self::general_actor_features(&before,model);
        if model.developmental_memory {
            // Continuous subjective trace: prior factual events influence
            // future actions even when the next visible frame is identical.
            // No reward or oracle controls the write gate.
            for (memory,next) in model.working_trace.iter_mut().zip(&after){
                *memory=(0.94*(*memory)+0.06*(*next)).clamp(-1.0,1.0);
            }
            model.retained_events=model.retained_events.saturating_add(1);
        }
        if !learning {return true;}
        let model=self.phase_native.as_mut().expect("native")
            .general_policy.as_mut().expect("policy");
        let pred=&mut model.predictions[action];
        let previously=model.prediction_visits[action];
        let alpha=0.12/(1.0+(previously as f32)*0.005);
        let mut error=0.0f32;
        for (v,p) in after.iter().zip(pred.iter_mut()){
            error+=(*v-*p).powi(2);
            *p+=alpha*(*v-*p);
        }
        model.prediction_visits[action]=previously.saturating_add(1);
        let novelty=(error/(GENERAL_DIM as f32)).sqrt().clamp(0.0,1.0);
        for trace in &mut model.eligibility{
            for v in trace.iter_mut(){*v*=GENERAL_TRACE_DECAY;}
        }
        // Competitive temporal synapses: when a physically selected motor
        // succeeds, its actual cue-action state strengthens while other
        // alternatives at the *same preceding state* are inhibited.
        // This prevents a common preparatory motor from absorbing all
        // delayed reward across distinct later decisions.
        // These signed traces are based ONLY on executed action and factual
        // PRE, never on evaluator-supplied correct action labels.
        let rivals=(model.eligibility.len().saturating_sub(1)).max(1) as f32;
        for (motor,trace) in model.eligibility.iter_mut().enumerate(){
            let direction=if motor==action{1.0}else{-1.0/rivals};
            for (t,feature) in trace.iter_mut().zip(&actor_before){
                *t=(*t+direction*(*feature)).clamp(-8.0,8.0);
            }
        }
        // Positive factual terminal reward beats weak novelty by design.
        // Near-zero nonterminal effects are not task-success substitutes.
        let feedback=if reward>0.0{
            model.positive_rewards=model.positive_rewards.saturating_add(1);
            4.0*reward
        }else{
            0.008*novelty-0.002
        };
        for (weights,trace) in model.weights.iter_mut().zip(&model.eligibility){
            for (w,t) in weights.iter_mut().zip(trace){
                *w=(*w+GENERAL_LEARNING_RATE*feedback*(*t))
                    .clamp(-16.0,16.0);
            }
        }
        model.updates+=1;
        true
    }
}
