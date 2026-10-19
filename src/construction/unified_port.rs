// Crate-private port, included by runtime.rs: never external authority.
#[derive(Debug, Clone)]
pub(crate) struct PortPrediction {
    pub action: usize, pub output: Vec<i64>, pub confidence: f32,
    pub novelty: f32, pub disagreement: f32,
}
#[derive(Debug, Clone)]
pub(crate) struct PortCertifiedChain {
    pub first_action:usize,
    pub first_consequence:Vec<i64>,
    pub endpoint:Vec<i64>,
    pub depth:usize,
}

#[derive(Debug,Clone)]
pub(crate) struct PortHistoryPlan{
    pub first_action:usize,
    pub first_consequence:Vec<i64>,
    pub terminal:Vec<i64>,
    pub depth:usize,
}
impl Learner {
    pub(crate) fn manage_from_carrier(&mut self) -> Result<()> {
        if !self.records.is_empty() || self.pending { return Err(Error::Busy); }
        self.carrier_managed = true; Ok(())
    }
    pub(crate) fn carrier_is_managed(&self)->bool {self.carrier_managed}
    /// Read-only consequence derived from real PRE/action/POST journal rows.
    /// Imagined outcomes never enter the journal and cannot become authority.
    pub(crate) fn carrier_factual_action_consequence_metered(&self,action:usize,budget:u64)->(Option<Vec<i64>>,u64) {
        if action>=self.config.actions || budget==0{return (None,0)}
        let mut meter=Meter::new(budget);
        let result=super::runtime::calibrated_factual_prediction(&self.records,&self.current,action,self.frozen_at,&mut meter);
        (result,meter.spent)
    }

    /// Search a bounded multi-step path only through consequences that passed
    /// an older-prefix / three-new-real-POST prospective validation.
    /// Unlike the legacy exploratory planner, an unverified imagined edge
    /// cannot become a high-scoring U1 candidate. The returned first action
    /// still needs the original single-use HumanProtection permit.
    pub(crate) fn carrier_certified_goal_chain_metered(
        &self,goal:&[Option<i64>],allowed:&[bool],budget:u64
    )->(Option<PortCertifiedChain>,u64) {
        if !self.self_directed || self.pending || self.poisoned || budget==0
            || goal.len()!=self.config.width || allowed.len()!=self.config.actions
            || !goal.iter().any(Option::is_some)
            || goal_distance(&self.current,goal)==0 {
            return (None,0);
        }
        let mut meter=Meter::new(budget);
        let mut visited=BTreeSet::new();
        visited.insert(self.current.clone());
        let mut frontier=VecDeque::from([(self.current.clone(),None::<(usize,Vec<i64>)>,0usize)]);
        let mut expansions=0_usize;
        'search: while let Some((state,first,depth))=frontier.pop_front() {
            if depth>=self.config.planning_depth || expansions>=96 {continue}
            for action in 0..self.config.actions {
                if !allowed[action]{continue}
                if meter.charge(1+state.len() as u64).is_err(){break 'search}
                // Cheap raw inference only prunes impossible/no-op branches.
                // It NEVER authenticates the edge or enters a factual U1 slot.
                let Some(proposed)=factual_prediction(
                    &self.records,&state,action,self.frozen_at,&mut meter
                ) else {
                    if meter.remaining()==0{break 'search}
                    continue
                };
                if proposed==state{continue}
                let Some(next)=calibrated_factual_prediction(
                    &self.records,&state,action,self.frozen_at,&mut meter
                ) else {
                    if meter.remaining()==0{break 'search}
                    continue
                };
                if next!=proposed||next==state{continue}
                expansions+=1;
                let initial=first.clone().unwrap_or((action,next.clone()));
                if goal_distance(&next,goal)==0 && depth+1>=2 {
                    return (Some(PortCertifiedChain{
                        first_action:initial.0,first_consequence:initial.1,
                        endpoint:next,depth:depth+1,
                    }),meter.spent);
                }
                if visited.len()<128 && depth+1<self.config.planning_depth {
                    if meter.charge(12+next.len() as u64).is_err(){break 'search}
                    if visited.insert(next.clone()){
                        frontier.push_back((next,Some(initial),depth+1));
                    }
                }
            }
        }
        (None,meter.spent)
    }

    /// Small BFS over (visible observation, most recent executed motor).
    /// Two visibly identical states are separated only after real
    /// independently contrasted, chronologically validated outcomes.
    /// A stable no-op may be a prerequisite, never a terminal claim.
    pub(crate) fn carrier_predictive_history_plan_metered(
        &self,goal:&[Option<i64>],allowed:&[bool],budget:u64
    )->(Option<PortHistoryPlan>,u64){
        if !self.predictive_history_states || self.pending || self.poisoned ||
            budget==0 || goal.len()!=self.config.width ||
            allowed.len()!=self.config.actions ||
            !goal.iter().any(Option::is_some) ||
            goal_distance(&self.current,goal)==0{
            return (None,0)
        }
        let mut meter=Meter::new(budget);
        let predecessor=if self.restart_next{None}
            else{self.records.last().map(|r|r.transition.action)};
        let mut q=VecDeque::new();let mut visited=BTreeSet::new();
        q.push_back((self.current.clone(),predecessor,
            None::<(usize,Vec<i64>)>,0usize,false));
        visited.insert((self.current.clone(),predecessor,false));
        let mut expanded=0_usize;
        'search:while let Some((state,previous,first,depth,used_history))=q.pop_front(){
            if depth>=self.config.planning_depth||expanded>=48{continue}
            for action in 0..self.config.actions{
                if !allowed[action]{continue}
                if meter.charge((state.len()+2) as u64).is_err(){break 'search}
                expanded+=1;
                let contextual=previous.and_then(|p|
                    calibrated_history_effect(&self.records,&state,action,p,
                        self.frozen_at,&mut meter));
                if meter.remaining()==0{break 'search}
                let (next,has_context)=if let Some(predicted)=contextual{
                    (predicted,true)
                }else{
                    let p=calibrated_factual_prediction(&self.records,&state,
                        action,self.frozen_at,&mut meter)
                        .or_else(||certified_history_noop(&self.records,
                            &state,action,self.frozen_at,&mut meter));
                    if meter.remaining()==0{break 'search}
                    let Some(p)=p else{continue};
                    (p,false)
                };
                let depth=depth+1;
                let used=used_history||has_context;
                let first=first.clone().unwrap_or((action,next.clone()));
                if used && depth>=2 && goal_distance(&next,goal)==0{
                    return (Some(PortHistoryPlan{
                        first_action:first.0,first_consequence:first.1,
                        terminal:next,depth,
                    }),meter.spent)
                }
                if visited.len()<96 && depth<self.config.planning_depth {
                    if meter.charge((next.len()+12) as u64).is_err(){break 'search}
                    if visited.insert((next.clone(),Some(action),used)){
                        q.push_back((next,Some(action),Some(first),depth,used));
                    }
                }
            }
        }
        (None,meter.spent)
    }
    pub(crate) fn carrier_config(&self)->&super::runtime::Config {&self.config}
    pub(crate) fn carrier_prepare(&mut self,a:usize)->Result<Commitment> {self.prepare(a)}
    pub(crate) fn carrier_cancel(&mut self,t:Commitment)->Result<()> {self.cancel(t)}
    pub(crate) fn carrier_fault(&mut self,t:Commitment,error:String) {self.fault(t,error);}
    pub(crate) fn carrier_commit(&mut self,t:Commitment,post:Vec<i64>,permit:u64,cost:u64)->Result<()> {self.commit(t,post,permit,cost)}
    pub(crate) fn carrier_can_construct(&self)->bool {
        !self.pending && !self.poisoned && self.frozen_at.is_none() && self.shadow.is_none()
            && self.records.len()>=self.config.minimum_training
    }
    pub(crate) fn carrier_predictions(&self, allowed:&[bool])->Result<(Vec<PortPrediction>,u64)> {
        if allowed.len()!=self.config.actions || self.pending || self.poisoned {return Err(Error::InvalidData);}
        let mut meter=Meter::new(self.config.planning_work);let mut out=Vec::new();
        for a in 0..self.config.actions {
            if !allowed[a] {continue;}
            let (output,_)=self.active.program.step(&self.current,a,&self.active.state,&self.library,&mut meter)?;
            let mut equal=0usize;let mut count=0usize;let mut local=0usize;
            for r in self.records.iter().rev().take(128) {
                meter.charge(1)?;
                if r.transition.action==a {
                    if r.transition.pre==self.current {local+=1;}
                    if r.model==self.active.program.fingerprint() {count+=1;equal+=usize::from(r.prediction.center==r.transition.post);}
                }
            }
            let disagreement=match &self.shadow {
                Some(s)=> {let (p,_)=s.model.program.step(&self.current,a,&s.model.state,&self.library,&mut meter)?;
                    p.iter().zip(&output).filter(|(x,y)|x!=y).count() as f32/self.config.width as f32},None=>0.0
            };
            out.push(PortPrediction {action:a,output,confidence:equal as f32/(count as f32+1.0),novelty:1.0/(1.0+local as f32),disagreement});
        }
        Ok((out,meter.spent))
    }
    pub(crate) fn carrier_pressure(&self)->f32 {
        let recent:Vec<_>=self.records.iter().rev().take(32).collect();if recent.is_empty(){return 0.0;}
        recent.iter().map(|r|r.prediction.center.iter().zip(&r.transition.post).filter(|(a,b)|a!=b).count() as f32
            /self.config.width as f32).sum::<f32>()/recent.len() as f32
    }
    // Bounded model-based plans are options. The physical carrier chooses.
    pub(crate) fn carrier_plans(&self,goal:&[Option<i64>],allowed:&[bool])->Result<(Vec<(usize,Vec<i64>,usize)>,u64)> {
        if goal.len()!=self.config.width || allowed.len()!=self.config.actions {return Err(Error::InvalidData);}
        let actions:Vec<_>=(0..self.config.actions).filter(|&a|allowed[a]).collect();
        let mut meter=Meter::new(self.config.planning_work);
        let mut frontier=vec![(None,self.current.clone(),self.active.state.clone())];
        let mut best:std::collections::BTreeMap<usize,(f64,Vec<i64>,usize)>=std::collections::BTreeMap::new();
        'depth: for depth in 1..=self.config.planning_depth {
            let mut next=Vec::new();
            for (first,input,state) in frontier {
                for &a in &actions {
                    let result=self.active.program.step(&input,a,&state,&self.library,&mut meter);
                    let (p,s)=match result {Ok(v)=>v,Err(Error::Budget)=>break 'depth,Err(_)=>continue};
                    let first=first.unwrap_or(a);
                    let distance=p.iter().zip(goal).filter_map(|(&x,y)|y.map(|v|(x as i128-v as i128).abs() as f64)).sum::<f64>();
                    let score=distance+depth as f64*0.001;
                    if best.get(&first).is_none_or(|x|score<x.0){best.insert(first,(score,p.clone(),depth));}
                    if next.len()<256 {next.push((Some(first),p,s));}
                }
            }
            frontier=next;
        }
        Ok((best.into_iter().map(|(a,(_,p,d))|(a,p,d)).collect(),meter.spent))
    }
    // HDC content addressing proposes a consequence, never a solved action.
    pub(crate) fn carrier_recall(&self,allowed:&[bool])->Result<(Vec<(usize,Vec<i64>,f32)>,u64)> {
        use crate::hdc::PhaseVector;
        fn encode(values:&[i64])->PhaseVector {
            let roles:Vec<_>=values.iter().enumerate().map(|(i,&v)|
                PhaseVector::from_seed(32,i as u64+0x517a).bind(&PhaseVector::from_seed(32,v as u64^0x7733))).collect();
            PhaseVector::bundle(&roles.iter().collect::<Vec<_>>())
        }
        let current=encode(&self.current);let mut best=std::collections::BTreeMap::new();let mut work=0u64;
        for r in self.records.iter().rev().take(128) {
            if !allowed[r.transition.action] {continue;}
            let similarity=current.similarity(&encode(&r.transition.pre)).clamp(0.0,1.0);work+=self.config.width as u64*32;
            if best.get(&r.transition.action).is_none_or(|(s,_)|similarity>*s) {best.insert(r.transition.action,(similarity,r.transition.post.clone()));}
        }
        Ok((best.into_iter().map(|(a,(s,p))|(a,p,s)).collect(),work))
    }
}
