// Crate-private port, included by runtime.rs: never external authority.
#[derive(Debug, Clone)]
pub(crate) struct PortPrediction {
    pub action: usize, pub output: Vec<i64>, pub confidence: f32,
    pub novelty: f32, pub disagreement: f32,
}
impl Learner {
    pub(crate) fn manage_from_carrier(&mut self) -> Result<()> {
        if !self.records.is_empty() || self.pending { return Err(Error::Busy); }
        self.carrier_managed = true; Ok(())
    }
    pub(crate) fn carrier_is_managed(&self)->bool {self.carrier_managed}
    /// Read-only consequence derived from real PRE/action/POST journal rows.
    /// Imagined outcomes never enter the journal and cannot become authority.
    pub(crate) fn carrier_factual_action_consequence(&self,action:usize)->Option<Vec<i64>> {
        if action>=self.config.actions{return None}
        super::runtime::factual_prediction(&self.records,&self.current,action)
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
