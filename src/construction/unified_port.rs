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
    /// Depth of actual executed-history support for an accepted edge.
    /// This is an evidence statistic, NOT motor authority.
    pub history_suffix_depth:usize,
    /// A bounded, prospectively validated candidate OPTION selected only
    /// by native U1. Never read as a fabricated executed journal.
    pub route:Vec<(usize,Vec<i64>)>,
}

#[derive(Debug,Clone)]
pub(crate) struct PortCausalProbe{
    pub action:usize,
    /// Distinguishing this candidate from older physical observations is a
    /// proposer-side *epistemic statistic*, not predicted goal progress.
    pub novelty:f32,
    pub disagreement:f32,
    pub quoted_cost:u64,
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

    /// Bounded factual observed-state graph; a proposed route is TENTATIVE.
    /// Edges are only actually executed PRE/motor/POST tuples. Contradicting
    /// transitions from the exact same visible PRE and motor are withheld.
    /// No externally supplied world topology, hidden role or fabricated POST.
    pub(crate) fn carrier_observable_goal_route_metered(
        &self,goal:&[Option<i64>],allowed:&[bool],budget:u64
    )->(Option<PortHistoryPlan>,u64){
        if self.pending||self.poisoned||!self.self_directed||budget==0 ||
            allowed.len()!=self.config.actions ||
            goal.len()!=self.config.width ||
            !goal.iter().any(Option::is_some) ||
            goal_distance(&self.current,goal)==0 {
            return (None,0)
        }
        let end=self.frozen_at.map_or(self.records.len(),|id|
            self.records.partition_point(|r|r.transition.id<id));
        if end<2{return (None,0)}
        let mut meter=Meter::new(budget);
        let mut transitions=std::collections::BTreeMap::<
            (Vec<i64>,usize),Option<(Vec<i64>,usize)>>::new();
        for row in &self.records[end.saturating_sub(1024)..end]{
            if meter.charge((self.config.width*2+6) as u64).is_err(){
                // An unfinished scan may hide a physical contradiction:
                // never promote a partial journal to a causal option.
                return (None,meter.spent)
            }
            let t=&row.transition;
            let key=(t.pre.clone(),t.action);
            match transitions.entry(key){
                std::collections::btree_map::Entry::Vacant(v)=>{
                    v.insert(Some((t.post.clone(),1)));
                },
                std::collections::btree_map::Entry::Occupied(mut o)=>{
                    if let Some((old,count))=o.get_mut(){
                        if old==&t.post {
                            *count=count.saturating_add(1);
                        }else{
                            // One true conflicting POST invalidates
                            // the exact-state edge for THIS source scan.
                            *o.get_mut()=None;
                        }
                    }
                }
            }
        }
        let depth_cap=self.config.planning_depth.min(5);
        if depth_cap<2{return (None,meter.spent)}
        let mut visited=std::collections::BTreeSet::new();
        visited.insert(self.current.clone());
        let mut frontier=std::collections::VecDeque::new();
        frontier.push_back((self.current.clone(),Vec::<(usize,Vec<i64>)>::new()));
        while let Some((state,route))=frontier.pop_front(){
            if route.len()>=depth_cap{continue}
            for action in 0..self.config.actions{
                if !allowed[action]{continue}
                if meter.charge((state.len()+4) as u64).is_err(){
                    return (None,meter.spent)
                }
                let Some(Some((post,_count)))=
                    transitions.get(&(state.clone(),action))
                else{continue};
                // A physical no-op cannot supply the useful observable
                // intermediate state whose absence caused the regression.
                if post==&state {continue}
                let mut next=route.clone();
                next.push((action,post.clone()));
                if next.len()>=2 && goal_distance(post,goal)==0 {
                    return (Some(PortHistoryPlan{
                        first_action:next[0].0,
                        first_consequence:next[0].1.clone(),
                        terminal:post.clone(),
                        depth:next.len(),history_suffix_depth:0,route:next
                    }),meter.spent)
                }
                if next.len()<depth_cap && visited.len()<128 &&
                    visited.insert(post.clone()){
                    frontier.push_back((post.clone(),next));
                }
            }
        }
        (None,meter.spent)
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
                // Replanning starts AFTER the previous real action. A validated
                // one-step suffix can be the remainder of a multi-step plan.
                // Requiring depth>=2 here discards the terminal action and
                // can repeatedly propose preparation. No confidence, evidence
                // threshold, native U1 weights or safety boundary is changed.
                if used && goal_distance(&next,goal)==0{
                    return (Some(PortHistoryPlan{
                        first_action:first.0,first_consequence:first.1,
                        terminal:next,depth,history_suffix_depth:1,
                        route:Vec::new(),
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

    /// This organism itself chooses an experiment intended to discriminate
    /// under-sampled action pairs in the SAME observed environment state.
    /// Only an executed no-visible-effect predecessor motivates a new pair:
    /// it might have changed a latent state without sensor change.
    ///
    /// Observations after a reset / physical journal gap are never a
    /// predecessor; inference and metering use only the pre-freeze journal.
    /// No semantic motor roles, rewards, goals or synthetic future POSTs.
    pub(crate) fn carrier_active_causal_probe_metered(
        &self,allowed:&[bool],quoted:&[super::autonomy::ActionOffer],
        budget:u64
    )->(Option<PortCausalProbe>,u64){
        if !self.active_pair_probes||self.pending||self.poisoned||
            self.frozen_at.is_some()||budget==0||
            allowed.len()!=self.config.actions||
            quoted.len()!=self.config.actions{
            return (None,0)
        }
        let Some(previous)=self.records.last()else{return (None,0)};
        if self.restart_next || previous.transition.pre!=previous.transition.post ||
            previous.transition.post!=self.current {return (None,0)}
        let context=previous.transition.action;
        let mut meter=Meter::new(budget);
        let mut rank=None::<(usize,usize,u64,usize)>;
        // Prioritise the least-tested (PRE, predecessor, next-motor)
        // context, with an independent global-action frequency tie-break.
        // The selector cannot know in advance which motor is "charge".
        for action in 0..self.config.actions{
            if !allowed[action] {continue}
            let mut within=0usize;
            let mut total=0usize;
            let mut other_context=false;
            let mut outcomes=std::collections::BTreeSet::new();
            let end=self.frozen_at.map_or(self.records.len(),|id|
                self.records.partition_point(|r|r.transition.id<id));
            for index in (0..end).rev().take(128){
                if meter.charge((self.current.len()+3) as u64).is_err(){
                    return (None,meter.spent)
                }
                let row=&self.records[index].transition;
                if row.action!=action{continue}
                total+=1;
                if row.pre!=self.current{continue}
                let p=factual_predecessor(&self.records[..end],index);
                if p==Some(context) {within+=1}
                if p.is_some_and(|n|n!=context) {other_context=true}
                if outcomes.len()<8 {outcomes.insert(row.post.clone());}
            }
            // Uncertainty is epistemic, not presumed success. Once a pair
            // has at least three direct real witnesses we stop probing it.
            if within>=3{continue}
            let tie=exploration_word(self.next_id^
                (action as u64).wrapping_mul(0x9e3779b97f4a7c15));
            let key=(within,total,tie,action);
            if rank.is_none_or(|b|key<b){rank=Some(key)}
            // Every supported alternative context may indicate a useful
            // contrast; a single unknown observation does not.
            let _=(other_context,outcomes);
        }
        let Some((within,total,_,action))=rank else{return (None,meter.spent)};
        let novelty=1.0/(1.0+within as f32);
        let disagreement=1.0/(1.0+total as f32);
        (Some(PortCausalProbe{
            action,novelty,disagreement,
            quoted_cost:quoted[action].max_cost,
        }),meter.spent)
    }
    /// An epistemic action, NOT an imagined result. The learner itself
    /// measures which opaque motor extension of the most recent actually
    /// executed no-visible-effect sequence is least physically observed.
    /// All historical comparisons are PRE-matched, restart-safe and
    /// charged against the existing finite causal-probe work budget.
    pub(crate) fn carrier_contextual_experiment_metered(
        &self,allowed:&[bool],quoted:&[super::autonomy::ActionOffer],
        budget:u64
    )->(Option<PortCausalProbe>,u64){
        if !self.active_pair_probes||self.pending||self.poisoned||
            self.frozen_at.is_some()||budget==0||
            allowed.len()!=self.config.actions||
            quoted.len()!=self.config.actions||self.restart_next {
            return (None,0)
        }
        let end=self.records.len();
        let max_history=self.config.planning_depth.min(5).saturating_sub(1);
        if end==0||max_history==0{return (None,0)}
        // Most recent connected no-visible-effect sequence. The physical
        // history itself supplies motor IDs; neither hidden state nor
        // evaluator role labels are accessible here.
        let mut suffix=Vec::with_capacity(max_history);
        let mut next_id=self.next_id;
        for index in (0..end).rev().take(max_history){
            let row=&self.records[index].transition;
            if row.id.checked_add(1)!=Some(next_id) ||
                row.pre!=self.current||row.post!=self.current{break}
            suffix.push(row.action);
            next_id=row.id;
            if row.restart{break}
        }
        if suffix.is_empty(){return (None,0)}
        suffix.reverse();
        let depth=suffix.len();
        let mut meter=Meter::new(budget);
        let mut within=vec![0usize;self.config.actions];
        let mut global=vec![0usize;self.config.actions];
        // No partial scan is evidence of being "least tested": return
        // NONE if the caller cannot pay for all relevant rows.
        for index in (0..end).rev().take(256){
            if meter.charge((self.current.len()+depth+4) as u64).is_err(){
                return (None,meter.spent)
            }
            let row=&self.records[index].transition;
            if !allowed[row.action]{continue}
            global[row.action]+=1;
            if row.pre!=self.current || row.restart || index<depth{
                continue
            }
            let mut matched=true;
            let mut expected_id=row.id;
            for prior in (index-depth..index).rev().zip(suffix.iter().rev()){
                let (prior_index,expected_action)=prior;
                let p=&self.records[prior_index].transition;
                if p.id.checked_add(1)!=Some(expected_id) ||
                    p.action!=*expected_action ||
                    p.pre!=self.current||p.post!=self.current{
                    matched=false;break
                }
                expected_id=p.id;
                // An external restart marks the FIRST physical motor in
                // its episode. A restart on ANY interior suffix motor
                // invalidates the candidate even if IDs/observations match.
                // The old prior_index+1<index check inadvertently allowed
                // a restart at the final predecessor (index-1).
                if p.restart && prior_index!=index-depth{
                    matched=false;break
                }
            }
            if matched{within[row.action]+=1}
        }
        let mut best=None::<(usize,usize,u64,usize)>;
        for action in 0..self.config.actions{
            if !allowed[action]{continue}
            let tie=exploration_word(self.next_id ^
                ((action as u64).wrapping_mul(0x9e3779b97f4a7c15)));
            let score=(within[action],global[action],tie,action);
            if best.is_none_or(|b|score<b){best=Some(score)}
        }
        let Some((count,total,_,action))=best else{
            return (None,meter.spent)
        };
        (Some(PortCausalProbe{
            action,novelty:1.0/(1.0+count as f32),
            disagreement:1.0/(1.0+total as f32),
            quoted_cost:quoted[action].max_cost
        }),meter.spent)
    }

    /// TRUSTED EXTERNAL OBSERVATION: a virtual environment reset is not
    /// an imagined POST, has no action permit, and never erases the journal.
    /// The next real action must record a restart marker, preventing
    /// causal/predecessor inference from crossing independent episodes.
    pub(crate) fn carrier_observe_external_restart(
        &mut self,observed:Vec<i64>
    )->Result<()>{
        self.record_external_restart(observed)
    }

    /// A tentative executable memory, inferred ONLY from the organism's own
    /// actual chronological protected motor outcomes. The goal is externally
    /// stated, but the three motor IDs and their order are never supplied.
    /// It must have a real success, a contrasting real failure for the last
    /// motor at the SAME visible PRE, and no contradictory repeat of the
    /// complete candidate. This is evidence for a test, NOT causal proof.
    pub(crate) fn carrier_autobiographical_option_metered(
        &self,goal:&[Option<i64>],allowed:&[bool],budget:u64
    )->(Option<PortHistoryPlan>,u64){
        if self.pending || self.poisoned || budget==0 ||
            self.config.planning_depth<3 ||
            allowed.len()!=self.config.actions || goal.len()!=self.config.width ||
            !goal.iter().any(Option::is_some) ||
            goal_distance(&self.current,goal)==0 {return (None,0)}
        let end=self.frozen_at.map_or(self.records.len(),|id|
            self.records.partition_point(|r|r.transition.id<id));
        if end<3{return (None,0)}
        let mut meter=Meter::new(budget);
        let start=end.saturating_sub(512);
        let mut negative_last=std::collections::BTreeSet::new();
        // A direct-effect motor is not evidence of a hidden prerequisite.
        for row in &self.records[start..end] {
            if meter.charge((self.current.len()+2) as u64).is_err(){
                return (None,meter.spent)
            }
            let t=&row.transition;
            if t.pre==self.current && goal_distance(&t.post,goal)!=0{
                negative_last.insert(t.action);
            }
        }
        // Opaque executable triples: all transitions connected, two genuinely
        // invisible preparations, REAL final effect. No episode crossing.
        let mut observed=std::collections::BTreeMap::<[usize;3],(usize,usize,Vec<i64>)>::new();
        for index in (start+2)..end {
            if meter.charge((3*self.current.len()+9) as u64).is_err(){
                return (None,meter.spent)
            }
            let a=&self.records[index-2].transition;
            let b=&self.records[index-1].transition;
            let c=&self.records[index].transition;
            if b.restart || c.restart ||
                a.id.checked_add(1)!=Some(b.id) ||
                b.id.checked_add(1)!=Some(c.id) ||
                a.post!=b.pre || b.post!=c.pre ||
                a.pre!=self.current || a.post!=a.pre || b.post!=b.pre ||
                !allowed[a.action] || !allowed[b.action] || !allowed[c.action]{
                continue
            }
            let entry=observed.entry([a.action,b.action,c.action])
                .or_insert((0,0,Vec::new()));
            if goal_distance(&c.post,goal)==0{
                entry.0=entry.0.saturating_add(1);
                entry.2=c.post.clone();
            }else{
                entry.1=entry.1.saturating_add(1);
            }
        }
        let mut best=None::<(usize,[usize;3],Vec<i64>)>;
        for (seq,(wins,failures,terminal)) in observed {
            if wins==0 || failures!=0 || !negative_last.contains(&seq[2]) {
                continue
            }
            // Prefer repeatedly verified actual successes. No motor roles,
            // target action IDs, or source-disclosed evaluator states.
            if best.as_ref().is_none_or(|(n,old,_)|
                wins>*n || (wins==*n && seq<*old)){
                best=Some((wins,seq,terminal));
            }
        }
        let Some((_,sequence,terminal))=best else{return (None,meter.spent)};
        let route=vec![
            (sequence[0],self.current.clone()),
            (sequence[1],self.current.clone()),
            (sequence[2],terminal.clone()),
        ];
        (Some(PortHistoryPlan{
            first_action:sequence[0],first_consequence:self.current.clone(),
            terminal,depth:3,history_suffix_depth:0,route
        }),meter.spent)
    }

    /// Experimental DATA-SELECTED causal memory depth. This is a bounded
    /// algorithm authored by developers, NOT a discovered abstract VM grammar.
    /// An observed failure of an otherwise matching shorter suffix motivates
    /// recruiting ONE more preceding action into the causal hypothesis.
    /// Any real contradicting execution of the full sequence withdraws it.
    pub(crate) fn carrier_variable_depth_option_metered(
        &self,goal:&[Option<i64>],allowed:&[bool],budget:u64,
        require_complexity_evidence:bool
    )->(Option<PortHistoryPlan>,u64){
        if self.pending || self.poisoned || budget==0 ||
            self.config.planning_depth<2 ||
            allowed.len()!=self.config.actions ||
            goal.len()!=self.config.width ||!goal.iter().any(Option::is_some)||
            goal_distance(&self.current,goal)==0{
            return (None,0)
        }
        let end=self.frozen_at.map_or(self.records.len(),|id|
            self.records.partition_point(|r|r.transition.id<id));
        if end<2{return (None,0)}
        // The bounded search never reads outside the organism's genuine
        // physical journal or crosses its independently observed restarts.
        let start=end.saturating_sub(512);
        let max_depth=self.config.planning_depth.min(5).min(end-start);
        let mut meter=Meter::new(budget);
        let mut failed_last=std::collections::BTreeSet::new();
        let mut observed=std::collections::BTreeMap::<Vec<usize>,
            (usize,usize,Vec<i64>)>::new();
        for index in start..end{
            let t=&self.records[index].transition;
            if meter.charge((t.pre.len()+2) as u64).is_err(){
                return (None,meter.spent)
            }
            if t.pre==self.current && goal_distance(&t.post,goal)!=0{
                failed_last.insert(t.action);
            }
            for depth in 2..=max_depth{
                if index+1<start+depth{continue}
                if meter.charge((depth+self.current.len()+3) as u64).is_err(){
                    // Partial corpus scans can hide actual contradictory
                    // evidence. Never promote on an underfunded scan.
                    return (None,meter.spent)
                }
                let first=index+1-depth;
                let rows=&self.records[first..=index];
                let beginning=&rows[0].transition;
                if beginning.pre!=self.current{continue}
                let mut valid=true;
                for (position,row) in rows.iter().enumerate(){
                    let actual=&row.transition;
                    if !allowed[actual.action] ||
                        (position>0 && actual.restart) ||
                        (position<depth-1 &&
                            (actual.pre!=self.current ||
                             actual.post!=self.current)){
                        valid=false;break
                    }
                }
                if !valid{continue}
                for two in rows.windows(2){
                    let a=&two[0].transition;
                    let b=&two[1].transition;
                    if a.id.checked_add(1)!=Some(b.id) || a.post!=b.pre{
                        valid=false;break
                    }
                }
                if !valid{continue}
                let motors=rows.iter().map(|r|r.transition.action)
                    .collect::<Vec<_>>();
                let evidence=observed.entry(motors)
                    .or_insert((0,0,Vec::new()));
                if goal_distance(&t.post,goal)==0{
                    evidence.0=evidence.0.saturating_add(1);
                    evidence.2=t.post.clone();
                }else{
                    evidence.1=evidence.1.saturating_add(1);
                }
            }
        }
        let mut best=None::<(usize,Vec<usize>,Vec<i64>)>;
        for (motors,(successes,failures,terminal)) in &observed{
            if *successes==0 || *failures>0 ||
                !failed_last.contains(motors.last().expect("nonempty")){
                continue
            }
            // A longer memory is justified only if that EXACT shorter
            // suffix has previously been executed and FAILED with the same
            // observable PRE. Hence length is supported by contrast, not
            // by an arbitrary user-specified 3-action pattern.
            let shorter=observed.get(&motors[1..]);
            if motors.len()>2 &&
                !shorter.is_some_and(|old|old.1>0){
                continue
            }
            if require_complexity_evidence && motors.len()>3{
                let Some((short_success,short_failure,_))=shorter else{
                    continue
                };
                // Compare empirical goal predictability with the same
                // shorter executed suffix, using real successes AND real
                // failures, never a hidden-stage or authored motor role.
                // An extension costs another physical action, so mere
                // novelty is not justification for added capacity.
                let full_prob=(*successes as f64+1.0)/
                    (*successes as f64+*failures as f64+2.0);
                let short_prob=(*short_success as f64+1.0)/
                    (*short_success as f64+*short_failure as f64+2.0);
                let complexity_tax=0.03*(motors.len()-3) as f64;
                if !full_prob.is_finite() || !short_prob.is_finite() ||
                    full_prob-short_prob<=complexity_tax{
                    continue
                }
            }
            let better=best.as_ref().is_none_or(|(n,old,_)|
                successes>n ||
                (successes==n &&
                    (motors.len()<old.len() ||
                     (motors.len()==old.len() && motors<old))));
            if better{
                best=Some((*successes,motors.clone(),terminal.clone()));
            }
        }
        let Some((_,motors,terminal))=best else{
            return (None,meter.spent)
        };
        let len=motors.len();
        let route=motors.into_iter().enumerate().map(|(index,motor)|
            (motor,if index+1==len{
                terminal.clone()
            }else{self.current.clone()})).collect::<Vec<_>>();
        let first_action=route[0].0;
        (Some(PortHistoryPlan{
            first_action,first_consequence:self.current.clone(),
            terminal,depth:len,history_suffix_depth:0,route
        }),meter.spent)
    }

    /// Optional deeper reconstruction within the SAME Learner. We do not
    /// inject the hidden state or manually name a prerequisite: the suffix
    /// length is selected because shorter histories cannot predict an
    /// otherwise REAL, prospectively contrasted motor consequence.
    pub(crate) fn carrier_adaptive_history_plan_metered(
        &self,goal:&[Option<i64>],allowed:&[bool],budget:u64
    )->(Option<PortHistoryPlan>,u64){
        if !self.adaptive_history_suffix || !self.predictive_history_states ||
            self.pending || self.poisoned || budget==0 ||
            goal.len()!=self.config.width ||
            allowed.len()!=self.config.actions ||
            !goal.iter().any(Option::is_some) ||
            goal_distance(&self.current,goal)==0{
            return (None,0)
        }
        let mut meter=Meter::new(budget);
        let mut suffix=Vec::new();
        if !self.restart_next{
            if let Some(last)=self.records.last(){
                suffix.push(last.transition.action);
                if self.records.len()>=2 && !last.transition.restart{
                    let before=&self.records[self.records.len()-2].transition;
                    if before.id.checked_add(1)==Some(last.transition.id)
                        && before.post==last.transition.pre{
                        suffix.push(before.action)
                    }
                }
            }
        }
        // The visible sensor is NOT itself a complete Markov state.
        // Two execution histories may have identical PRE but different
        // controllability, so the suffix belongs in the visited key.
        let mut visited=BTreeSet::new();
        visited.insert((self.current.clone(),suffix.clone(),0usize));
        let mut q=VecDeque::new();
        q.push_back((self.current.clone(),suffix,
            None::<(usize,Vec<i64>)>,0usize,0usize,
            Vec::<(usize,Vec<i64>)>::new()));
        let mut expanded=0usize;
        'search:while let Some((state,previous,first,depth,used_history,route))=q.pop_front(){
            if depth>=self.config.planning_depth||expanded>=96{continue}
            for action in 0..self.config.actions{
                if !allowed[action]{continue}
                if meter.charge((state.len()+3) as u64).is_err(){break 'search}
                expanded+=1;
                // Prefer the shorter explanation whenever it is supported.
                // Only an underexplained consequence justifies more memory.
                let shallow=previous.first().and_then(|&p|
                    calibrated_history_effect(&self.records,&state,action,p,
                        self.frozen_at,&mut meter));
                if meter.remaining()==0{break 'search}
                let deeper=if shallow.is_none() && previous.len()>=2{
                    calibrated_second_order_effect(&self.records,&state,action,
                        [previous[0],previous[1]],self.frozen_at,&mut meter)
                }else{None};
                if meter.remaining()==0{break 'search}
                let (next,validated_depth)=if let Some(output)=shallow{
                    (output,1)
                }else if let Some(output)=deeper{
                    (output,2)
                }else{
                    // Same causal no-op safeguards as the old one-step lane.
                    let next=calibrated_factual_prediction(&self.records,
                        &state,action,self.frozen_at,&mut meter)
                        .or_else(||certified_history_noop(
                            &self.records,&state,action,self.frozen_at,&mut meter));
                    if meter.remaining()==0{break 'search}
                    let Some(next)=next else{continue};
                    (next,0)
                };
                let history=used_history.max(validated_depth);
                let first=first.clone().unwrap_or((action,next.clone()));
                let depth=depth+1;
                let mut path=route.clone();
                path.push((action,next.clone()));
                if history>0&&goal_distance(&next,goal)==0{
                    return (Some(PortHistoryPlan{
                        first_action:first.0,first_consequence:first.1,
                        terminal:next,depth,
                        history_suffix_depth:history,
                        route:path,
                    }),meter.spent)
                }
                if visited.len()<128&&depth<self.config.planning_depth{
                    if meter.charge((next.len()+14) as u64).is_err(){break 'search}
                    let mut following=Vec::with_capacity(2);
                    following.push(action);
                    following.extend(previous.iter().take(1).copied());
                    if visited.insert((next.clone(),following.clone(),history)){
                        q.push_back((next,following,Some(first),depth,history,path));
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


#[cfg(test)]
mod active_context_restart_boundary_tests{
    use super::*;
    use crate::HumanProtectionEvidence;
    use crate::scientific_runtime::{ScientificRuntime,UnifiedPost};
    use crate::construction::autonomy::{ActionOffer,AutonomyPolicy};

    fn safe(_:usize)->HumanProtectionEvidence{
        HumanProtectionEvidence{
            human_present:false,physical_effect_possible:false,
            predicted_harm_probability:0.0,hazard_confidence:1.0,
            emergency_stop:false
        }
    }
    fn step(rt:&mut ScientificRuntime,motor:usize){
        let offers=(0..3).map(|a|ActionOffer{
            available:a==motor,max_cost:1
        }).collect::<Vec<_>>();
        rt.step_constructing(None,&offers,safe,|executed|{
            assert_eq!(executed,motor);
            Ok(UnifiedPost{observation:vec![0],cost:1,reward:0.0})
        }).unwrap();
    }
    fn run(with_interior_restart:bool)->f32{
        let cfg=Config{
            width:1,actions:3,planning_depth:5,planning_work:100_000,
            minimum_training:128,search_every:4096,
            ..Default::default()
        };
        let mut rt=ScientificRuntime::new_constructing(cfg,
            AutonomyPolicy::default(),vec![0],"restarts").unwrap();
        rt.enable_constructing_self_directed_control().unwrap();
        rt.enable_constructing_predictive_history_states().unwrap();
        rt.enable_constructing_active_causal_probes().unwrap();
        step(&mut rt,0);
        if with_interior_restart {
            rt.observe_constructing_external_restart(vec![0]).unwrap();
        }
        step(&mut rt,1);
        step(&mut rt,2);
        // Current after-reset history is a real 0,1 pair, ready to
        // investigate action 2. Both worlds have the SAME visible PRE.
        rt.observe_constructing_external_restart(vec![0]).unwrap();
        step(&mut rt,0);
        step(&mut rt,1);
        let history=rt.construction_learner().unwrap();
        let t=history.records();
        assert_eq!(t.len(),5);
        assert_eq!(t[1].transition.restart,with_interior_restart);
        assert!(t[3].transition.restart);
        assert!(!t[4].transition.restart);
        let allowed=[false,false,true];
        let quoted=[ActionOffer{available:true,max_cost:1};3];
        let (probe,work)=history.carrier_contextual_experiment_metered(
            &allowed,&quoted,6000);
        assert!(work>0 && work<=6000);
        let candidate=probe.expect("actual connected suffix can nominate offered next motor");
        assert_eq!(candidate.action,2);
        // No simulation or inserted journal entries: this is purely
        // an inspection of real protected steps.
        assert_eq!(history.records().len(),5);
        candidate.novelty
    }

    #[test]
    fn restart_inside_suffix_cannot_be_credited_as_observed_pair(){
        assert_eq!(run(true),1.0,
            "failed old logic counted real actions across two separate episodes");
    }
    #[test]
    fn actual_connected_pair_is_still_counted(){
        assert_eq!(run(false),0.5,
            "fix must retain valid same-episode evidence from full real POSTs");
    }
}
