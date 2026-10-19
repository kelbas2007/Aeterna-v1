// Included by ScientificRuntime: one EvoPhase and one permission boundary.
use crate::construction::{Config as UConfig,Learner as ULearner};
use crate::construction::autonomy::{ActionOffer as UOffer,AutonomyPolicy as UPolicy};
use crate::construction::vm::{Error as UError,Result as UResult,fingerprint as uhash};
use crate::carrier::{PhaseNativeConfig,PhaseMetaControlConfig};
use serde::{Serialize as USerialize,Deserialize as UDeserialize};
#[derive(Debug,USerialize,UDeserialize)]
struct UnifiedDomain {name:String,learner:ULearner,last_construction:usize,pending_credit:Option<[f32;5]>}
#[derive(Debug,USerialize,UDeserialize)]
struct UnifiedConstruction {
    current:UnifiedDomain,archived:Vec<UnifiedDomain>,nonce:u64,operations:[u64;4],
    computation_work:u64,total_external_cost:u64,max_external_cost:u64,freeze_controller:bool,
    #[serde(default,skip_serializing_if="Option::is_none")]
    education:Option<crate::education::School>,
}
#[derive(Debug,Clone,USerialize,UDeserialize)]
pub struct UnifiedPost {pub observation:Vec<i64>,pub cost:u64,pub reward:f32}
#[derive(Debug,Clone,USerialize,UDeserialize)]
pub struct UnifiedStep {
    pub action:usize,pub source:String,pub operations:Vec<String>,pub selected_fields:[f32;5],
    pub native_score:f32,pub factual_utility:f32,pub event:usize,pub computation_work:u64,
    /// Diagnostic only; a proposed path is not a selected or executed path.
    #[serde(default,skip_serializing_if="std::ops::Not::not")]
    pub certified_chain_proposed:bool,
}
#[derive(Debug,Clone,USerialize,UDeserialize)]
pub struct UnifiedReport {
    pub domain:String,pub archived_domains:Vec<String>,pub learner:crate::construction::SnapshotReport,
    pub native_meta_observations:u64,pub native_weights:[f32;5],pub operations:[u64;4],
    pub computation_work:u64,pub total_external_cost:u64,pub controller_frozen:bool,
}
#[derive(USerialize,UDeserialize)]
#[serde(deny_unknown_fields)]
struct UnifiedStored {version:u32,carrier:Vec<u8>,cognition:UnifiedConstruction,sequence:u64,emergency:bool,protection_sequence:u64,hash:u64}
#[derive(Clone)]
struct UnifiedOption {p:PhaseCognitiveProposal,kind:u8,source:&'static str,output:Option<Vec<i64>>}
fn uword(mut n:u64)->u64 {n=n.wrapping_add(0x9e3779b97f4a7c15);n=(n^(n>>30)).wrapping_mul(0xbf58476d1ce4e5b9);n=(n^(n>>27)).wrapping_mul(0x94d049bb133111eb);n^(n>>31)}
fn usensory(x:&[i64])->Vec<f32>{x.iter().map(|&v| (0.5+0.5*(v as f64)/(1.0+(v as f64).abs())) as f32).collect()}
fn udistance(x:&[i64],goal:Option<&[Option<i64>]>)->f64 {
    goal.map(|g| x.iter().zip(g).filter_map(|(&v,t)|t.map(|a|(v as i128-a as i128).abs() as f64)).sum()).unwrap_or(0.0)
}
fn ufields(p:&[i64],goal:Option<&[Option<i64>]>,novelty:f32,disagreement:f32,confidence:f32,cost:u64)->[f32;5] {
    [if goal.is_some(){(1.0/(1.0+udistance(p,goal))) as f32}else{0.0},novelty.clamp(0.0,1.0),disagreement.clamp(0.0,1.0),confidence.clamp(0.0,1.0),1.0/(1.0+(cost as f32).ln_1p())]
}
impl ScientificRuntime {
    pub fn new_constructing(config:UConfig,mut policy:UPolicy,initial:Vec<i64>,name:&str)->UResult<Self> {
        config.validate()?;
        if initial.len()!=config.width || name.is_empty() || name.len()>128 {return Err(UError::InvalidData);}
        if let Some(e)=&mut policy.evidence {e.total_alpha=e.total_alpha.min(0.005);}
        let mut learner=ULearner::new(config.clone(),initial.clone())?;
        learner.enable_autonomy(policy.clone())?;learner.manage_from_carrier()?;
        let mut evo=EvoPhase::new(crate::EvoConfig{sensory_cells:config.width,motor_cells:config.actions,hdc_dim:32,phase_learning_rate:1.0,..Default::default()});
        evo.enable_phase_native_planning(PhaseNativeConfig::default());
        if !evo.enable_phase_native_meta_control(PhaseMetaControlConfig::default()){return Err(UError::InvalidData);}
        evo.set_phase_native_meta_monotone_evidence(true);evo.observe_initial_real(&usensory(&initial),false);
        let mut rt=Self::new(evo).map_err(|_|UError::InvalidData)?;
        rt.constructing=Some(UnifiedConstruction{current:UnifiedDomain{name:name.into(),learner,last_construction:0,pending_credit:None},
            archived:Vec::new(),nonce:0,operations:[0;4],computation_work:0,total_external_cost:0,max_external_cost:policy.max_external_cost,freeze_controller:false,education:None});
        rt.fresh_observation_required=false;Ok(rt)
    }
    pub fn construction_learner(&self)->Option<&ULearner>{Some(&self.constructing.as_ref()?.current.learner)}
    /// Opt the SAME construction Learner into factual, self-selected probing.
    /// Goal and allowed offers are still host-supplied constraints, while the
    /// chosen action must compete in the existing U1 decision and pass the
    /// original human protection/one-use execution permit.
    pub fn enable_constructing_self_directed_control(&mut self)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller{return Err(UError::Blocked)}
        u.current.learner.enable_self_directed_control()
    }
    /// Abatable research flag, persists in the SAME Learner checkpoint.
    /// Never changes HumanProtection, actuator permissions or reward.
    pub fn enable_constructing_certified_chains(&mut self)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.freeze_controller{return Err(UError::Blocked)}
        u.current.learner.enable_certified_chains()
    }
    pub fn unified_report(&self)->Option<UnifiedReport>{
        let u=self.constructing.as_ref()?;
        Some(UnifiedReport{domain:u.current.name.clone(),archived_domains:u.archived.iter().map(|d|d.name.clone()).collect(),learner:u.current.learner.report(),
            native_meta_observations:self.organism.phase_native_meta_observations(),native_weights:self.organism.phase_native_meta_weights()?,operations:u.operations,
            computation_work:u.computation_work,total_external_cost:u.total_external_cost,controller_frozen:u.freeze_controller})
    }
    // A new sensor contract does not reinterpret numeric variables as pixels.
    // U1 weights and complete old-domain cognitive checkpoints remain present.
    pub fn enter_constructing_domain(&mut self,name:&str,config:UConfig,mut policy:UPolicy,initial:Vec<i64>)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        if u.archived.len()>=7 || u.freeze_controller || name.is_empty() || name.len()>128 || name==u.current.name
            || u.archived.iter().any(|d|d.name==name) || config.width!=self.organism.config().sensory_cells
            || config.actions!=self.organism.config().motor_cells {return Err(UError::InvalidData);}
        let k=u.archived.len()+2;
        if let Some(e)=&mut policy.evidence{e.total_alpha=e.total_alpha.min(0.01/(k*(k+1)) as f64);}
        policy.max_external_cost=policy.max_external_cost.min(u.max_external_cost.saturating_sub(u.total_external_cost));
        let mut learner=ULearner::new(config,initial.clone())?;learner.enable_autonomy(policy)?;learner.manage_from_carrier()?;
        let old=std::mem::replace(&mut u.current,UnifiedDomain{name:name.into(),learner,last_construction:0,pending_credit:None});
        u.archived.push(old);self.organism.observe_initial_real(&usensory(&initial),false);self.record(LifetimeEventKind::ExternalObservation);Ok(())
    }
    pub fn freeze_constructing(&mut self)->UResult<()> {
        let u=self.constructing.as_mut().ok_or(UError::NoEvidence)?;
        u.current.learner.freeze_learning()?;if let Some(s)=&mut u.education{s.freeze();}u.freeze_controller=true;self.organism.set_phase_native_meta_learning_enabled(false);Ok(())
    }
    // Matched ablation: same experience, programs and budgets; only U1 zeroed.
    pub fn ablate_constructing_controller(&mut self)->UResult<()> {
        let u=self.constructing.as_ref().ok_or(UError::NoEvidence)?;if !u.freeze_controller {return Err(UError::Busy);}
        let mut data:serde_json::Value=serde_json::from_slice(&self.organism.constructing_carrier_checkpoint().ok_or(UError::InvalidData)?).map_err(|_|UError::InvalidData)?;
        for s in data["synapses"].as_array_mut().ok_or(UError::InvalidData)? {s["weight"]=serde_json::json!(0.0);}
        self.organism=EvoPhase::from_constructing_carrier_checkpoint(&serde_json::to_vec(&data).map_err(|_|UError::InvalidData)?).ok_or(UError::InvalidData)?;Ok(())
    }
    pub fn step_constructing<H,F>(&mut self,goal:Option<&[Option<i64>]>,offers:&[UOffer],hazard:H,execute:F)->UResult<UnifiedStep>
    where H:FnOnce(usize)->HumanProtectionEvidence,F:FnOnce(usize)->UResult<UnifiedPost> {
        if self.protection.emergency_stop_latched(){return Err(UError::Blocked);}
        let mut u=self.constructing.take().ok_or(UError::NoEvidence)?;
        let result=self.step_constructing_inner(&mut u,goal,offers,hazard,execute);self.constructing=Some(u);result
    }
    fn step_constructing_inner<H,F>(&mut self,u:&mut UnifiedConstruction,goal:Option<&[Option<i64>]>,offers:&[UOffer],hazard:H,execute:F)->UResult<UnifiedStep>
    where H:FnOnce(usize)->HumanProtectionEvidence,F:FnOnce(usize)->UResult<UnifiedPost> {
        let cfg=u.current.learner.carrier_config().clone();
        if offers.len()!=cfg.actions || goal.is_some_and(|g|g.len()!=cfg.width){return Err(UError::InvalidData);}
        // Once all *specified* goal coordinates already match the last
        // factual observation, do not issue an unneeded protected action.
        // Masked goals may intentionally leave unrelated state unconstrained.
        if u.current.learner.self_directed_control_enabled()
            && goal.is_some_and(|g|g.iter().any(Option::is_some)
                && udistance(u.current.learner.current(),Some(g))==0.0)
        {return Err(UError::NoEvidence)}
        let remaining=u.max_external_cost.saturating_sub(u.total_external_cost);
        let allowed:Vec<_>=offers.iter().map(|o|o.available&&o.max_cost<=remaining).collect();
        if !allowed.iter().any(|&a|a){return Err(UError::Blocked);}
        let (predictions,overhead)=u.current.learner.carrier_predictions(&allowed)?;u.computation_work=u.computation_work.saturating_add(overhead);
        let pressure=u.current.learner.carrier_pressure();
        let mut options:Vec<UnifiedOption>=predictions.iter().map(|p|UnifiedOption{p:PhaseCognitiveProposal{proposal_id:0x100+p.action as u64,action:p.action,
            fields:ufields(&p.output,goal,p.novelty,p.disagreement,p.confidence,1)},kind:3,source:"predict",output:Some(p.output.clone())}).collect();
        // The existing Learner already knows how to move from unexplained
        // consequences toward information-gathering, then exploit verified
        // transitions. Previously it was reachable only in the separate
        // ProtectedConstructionRuntime. Here it is a *proposal*, never a
        // bypass around U1 or the human-protection permit.
        if u.current.learner.self_directed_control_enabled() {
            // Experimental multi-step search is enabled only when the caller
            // explicitly allocates a larger bounded work budget. The published
            // 4k/20k development runs retain their EXACT previous behavior.
            // All chain edges must be validated independently against actual
            // chronological holdouts, unlike exploratory imagined plans.
            let chain_budget=if goal.is_some() && u.current.learner.certified_chains_enabled() && cfg.planning_work>=32_000 {
                cfg.planning_work.saturating_sub(4_000)
            } else {0};
            let (chain,chain_work)=if let Some(g)=goal {
                u.current.learner.carrier_certified_goal_chain_metered(g,&allowed,chain_budget)
            } else {(None,0)};
            let residual=cfg.planning_work.saturating_sub(chain_work);
            let (action,chosen_work)=u.current.learner
                .choose_offered_metered_budget(goal,Some(offers),residual)?;
            if !allowed[action] {return Err(UError::Blocked)}
            let evidence=predictions.iter().find(|p|p.action==action).ok_or(UError::InvalidData)?;
            // Share exactly ONE configured factual work budget between the
            // validated chain, legacy probe selection and direct verification.
            let remaining=residual.saturating_sub(chosen_work);
            let (factual_prediction,factual_work)=u.current.learner
                .carrier_factual_action_consequence_metered(action,remaining);
            u.computation_work=u.computation_work.checked_add(
                chain_work.saturating_add(chosen_work).saturating_add(factual_work))
                .ok_or(UError::Capacity)?;
            if let Some(plan)=chain {
                if !allowed[plan.first_action]{return Err(UError::Blocked)}
                let observed=predictions.iter().find(|p|p.action==plan.first_action)
                    .ok_or(UError::InvalidData)?;
                // Terminal goal value is a *discounted* predicted consequence
                // of multiple certified edges; only FIRST action is executed.
                let mut fields=ufields(&plan.endpoint,goal,observed.novelty,
                    observed.disagreement,1.0/(plan.depth as f32),
                    offers[plan.first_action].max_cost.saturating_mul(plan.depth as u64));
                fields[0]/=1.0+0.15*(plan.depth.saturating_sub(1) as f32);
                options.push(UnifiedOption{p:PhaseCognitiveProposal{
                    proposal_id:0x500+plan.first_action as u64,
                    action:plan.first_action,fields
                },kind:3,source:"factual-chain",output:Some(plan.first_consequence)});
            }
            // Only empirically calibrated consequences are allowed to compete.
            // An unknown prediction is not secretly a high-novelty U1 option.
            // Ordinary U1 handles observation and exploratory choice instead.
            if let Some(factual)=factual_prediction {
            // One common five-field contract: verified-goal proximity,
            // local novelty, epistemic disagreement, factual support,
            // and quoted action economy. No motor IDs or world role labels.
            let fields=ufields(&factual,goal,evidence.novelty,
                evidence.disagreement,0.8,offers[action].max_cost);
            options.push(UnifiedOption{p:PhaseCognitiveProposal{
                proposal_id:0x400+action as u64,action,fields
            },kind:3,source:"factual-experience",output:Some(factual)});
            }
            // No separate high-novelty proposal without prospective support.
            // Ordinary U1 already owns uncertainty-driven action selection:
            // an unverified factual hypothesis must ABSTAIN, not override it.
        }
        let certified_chain_proposed=options.iter().any(|o|o.source=="factual-chain");
        let mut tried=[false;3];let mut helpers=Vec::new();let mut operations=Vec::new();let mut selected=None;
        for round in 0..4 {
            let mut candidates=options.clone();
            let can_construct=u.current.learner.carrier_can_construct() && u.current.learner.records().len().saturating_sub(u.current.last_construction)>=cfg.search_every;
            if round<3 {
                let action=allowed.iter().position(|&a|a).ok_or(UError::Blocked)?;
                if !tried[0] && can_construct {candidates.push(UnifiedOption{p:PhaseCognitiveProposal{proposal_id:1,action,fields:[0.0,pressure,pressure,0.0,0.15]},kind:0,source:"construct",output:None});}
                if !tried[1] && goal.is_some(){candidates.push(UnifiedOption{p:PhaseCognitiveProposal{proposal_id:2,action,fields:[0.5,0.5,pressure,0.5,0.4]},kind:1,source:"plan",output:None});}
                if !tried[2] && !u.current.learner.records().is_empty(){candidates.push(UnifiedOption{p:PhaseCognitiveProposal{proposal_id:3,action,fields:[0.0,0.5,pressure,0.5,0.7]},kind:2,source:"recall",output:None});}
            }
            let raw:Vec<_>=candidates.iter().map(|x|x.p).collect();u.nonce=u.nonce.checked_add(1).ok_or(UError::Capacity)?;
            let choice=self.organism.choose_phase_meta_exploratory(&raw,uword(u.nonce),!u.freeze_controller).ok_or(UError::NoEvidence)?;
            let chosen=candidates.into_iter().find(|p|p.p.proposal_id==choice.proposal_id).ok_or(UError::InvalidData)?;
            u.operations[chosen.kind as usize]+=1;operations.push(chosen.source.to_string());
            if chosen.kind==3 {selected=Some((chosen,choice.score));break;}
            tried[chosen.kind as usize]=true;
            match chosen.kind {
                0=>{
                    u.current.last_construction=u.current.learner.records().len();let before=u.current.learner.report().construction_work;
                    let result=u.current.learner.construct();u.computation_work=u.computation_work.saturating_add(u.current.learner.report().construction_work.saturating_sub(before));
                    if result.is_ok() && u.current.learner.report().shadow_pending {u.current.pending_credit=Some(chosen.p.fields);}
                    else if !u.freeze_controller {self.organism.observe_phase_native_meta_utility(chosen.p.fields,0.0);}
                    if let Err(e)=result {if !matches!(e,UError::NoEvidence|UError::Budget|UError::Capacity){return Err(e);}}
                },
                1=>{
                    let (plans,work)=u.current.learner.carrier_plans(goal.ok_or(UError::InvalidData)?,&allowed)?;
                    u.computation_work=u.computation_work.saturating_add(work);helpers.push(("plan",chosen.p.fields));
                    for (a,p,depth) in plans {
                        let confidence=predictions.iter().find(|x|x.action==a).map(|x|x.confidence).unwrap_or(0.0);
                        options.push(UnifiedOption{p:PhaseCognitiveProposal{proposal_id:0x200+a as u64,action:a,fields:ufields(&p,goal,0.0,pressure,confidence,depth as u64)},kind:3,source:"plan",output:Some(p)});
                    }
                },
                2=>{
                    let (memories,work)=u.current.learner.carrier_recall(&allowed)?;
                    u.computation_work=u.computation_work.saturating_add(work);helpers.push(("recall",chosen.p.fields));
                    for (a,p,similarity) in memories {options.push(UnifiedOption{p:PhaseCognitiveProposal{proposal_id:0x300+a as u64,action:a,fields:ufields(&p,goal,0.0,pressure,similarity,1)},kind:3,source:"recall",output:Some(p)});}
                },_=>return Err(UError::InvalidData)
            }
        }
        let (option,score)=selected.ok_or(UError::NoEvidence)?;let a=option.p.action;
        let token=u.current.learner.carrier_prepare(a)?;
        let decision=self.protection.screen(a,hazard(a));let record=decision.record().clone();
        let Some(permit)=decision.into_permit() else {u.current.learner.carrier_cancel(token)?;self.record(LifetimeEventKind::Blocked(record));return Err(UError::Blocked);};
        let permission=permit.sequence();
        if self.protection.consume_permit(permit).is_err(){u.current.learner.carrier_fault(token,"permission fault".into());self.latch_fault("permission fault".into());return Err(UError::Blocked);}
        let post=match execute(a) {Ok(p)=>p,Err(e)=>{u.current.learner.carrier_fault(token,e.to_string());self.latch_fault(e.to_string());return Err(e);}};
        if post.observation.len()!=cfg.width || !post.reward.is_finite() || !(-1.0..=1.0).contains(&post.reward) || post.cost>offers[a].max_cost {
            u.total_external_cost=u.total_external_cost.saturating_add(post.cost);u.current.learner.carrier_fault(token,"invalid external POST or cost".into());self.latch_fault("invalid external POST or cost".into());return Err(UError::InvalidData);
        }
        let before=udistance(u.current.learner.current(),goal);let after=udistance(&post.observation,goal);
        let novel=!u.current.learner.records().iter().any(|r|r.transition.post==post.observation);
        let progress=if goal.is_some(){((before-after)/(1.0+before)).max(0.0) as f32}else{0.0};
        let value=(0.8*post.reward.max(progress).max(0.0)+0.2*(if novel {1.0}else{0.0})).clamp(0.0,1.0);
        let verdicts=u.current.learner.promotions().len();u.total_external_cost=u.total_external_cost.saturating_add(post.cost);
        u.current.learner.carrier_commit(token,post.observation.clone(),permission,post.cost)?;
        self.organism.observe_initial_real(&usensory(&post.observation),false);
        if !u.freeze_controller {
            self.organism.observe_phase_native_meta_utility(option.p.fields,value);
            for (source,fields) in helpers {self.organism.observe_phase_native_meta_utility(fields,if source==option.source{value}else{0.0});}
            if u.current.learner.promotions().len()>verdicts {
                if let Some(fields)=u.current.pending_credit.take(){let accepted=u.current.learner.promotions().last().is_some_and(|p|p.accepted);self.organism.observe_phase_native_meta_utility(fields,if accepted{1.0}else{0.0});}
            }
        }
        let _=option.output;
        self.record(LifetimeEventKind::Executed{action:a,suppressed:0,learned:!u.freeze_controller});
        Ok(UnifiedStep{action:a,source:option.source.into(),operations,selected_fields:option.p.fields,native_score:score,factual_utility:value,event:u.current.learner.records().len(),computation_work:u.computation_work,certified_chain_proposed})
    }
    pub fn checkpoint_constructing(&self)->UResult<Vec<u8>> {
        let u=self.constructing.as_ref().ok_or(UError::NoEvidence)?;u.current.learner.checkpoint()?;
        let carrier=self.organism.constructing_carrier_checkpoint().ok_or(UError::InvalidData)?;
        let hash=uhash(&serde_json::to_vec(&(1u32,&carrier,u,self.sequence,self.emergency_latched(),self.protection.audit_position().0)).map_err(|_|UError::InvalidData)?);
        let value=serde_json::json!({"version":1,"carrier":carrier,"cognition":u,"sequence":self.sequence,"emergency":self.emergency_latched(),"protection_sequence":self.protection.audit_position().0,"hash":hash});
        serde_json::to_vec(&value).map_err(|_|UError::InvalidData)
    }
    // Cognition only; every action still needs fresh trusted hazard evidence.
    pub fn from_constructing_checkpoint(bytes:&[u8])->UResult<Self> {
        if bytes.len()>32*1024*1024{return Err(UError::Capacity);}
        let s:UnifiedStored=serde_json::from_slice(bytes).map_err(|_|UError::InvalidData)?;
        let hash=uhash(&serde_json::to_vec(&(s.version,&s.carrier,&s.cognition,s.sequence,s.emergency,s.protection_sequence)).map_err(|_|UError::InvalidData)?);
        if s.version!=1 || s.hash!=hash || s.cognition.archived.len()>7 {return Err(UError::InvalidData);}
        if let Some(school)=&s.cognition.education {school.validate()?; if s.cognition.freeze_controller && !school.is_frozen(){return Err(UError::InvalidData);}}
        let evo=EvoPhase::from_constructing_carrier_checkpoint(&s.carrier).ok_or(UError::InvalidData)?;
        let mut names=std::collections::BTreeSet::new();let mut costs=0u64;
        for d in s.cognition.archived.iter().chain(std::iter::once(&s.cognition.current)) {
            let _=ULearner::from_checkpoint(&d.learner.checkpoint()?)?;
            if !d.learner.carrier_is_managed() || d.learner.carrier_config().width!=evo.config().sensory_cells
                || d.learner.carrier_config().actions!=evo.config().motor_cells || !names.insert(&d.name)
                || d.name.is_empty() || d.name.len()>128 || d.last_construction>d.learner.records().len()
                || d.pending_credit.is_some_and(|f|f.iter().any(|x|!x.is_finite()||!(0.0..=1.0).contains(x))) {return Err(UError::InvalidData);}
            costs=costs.saturating_add(d.learner.records().iter().map(|r|r.external_cost).sum::<u64>());
        }
        if costs>s.cognition.total_external_cost || s.cognition.max_external_cost==0
            || evo.current_real().map(|r|r.sensory.clone())!=Some(usensory(s.cognition.current.learner.current())) {return Err(UError::InvalidData);}
        let mut rt=Self::new(evo).map_err(|_|UError::InvalidData)?;rt.sequence=s.sequence;rt.constructing=Some(s.cognition);rt.fresh_observation_required=false;
        rt.protection=HumanProtection::from_audit_position(s.protection_sequence,s.emergency);Ok(rt)
    }
    pub fn restore_constructing(&mut self,bytes:&[u8])->UResult<()> {
        let mut candidate=Self::from_constructing_checkpoint(bytes)?;
        let old=self.constructing.as_ref().ok_or(UError::NoEvidence)?;let new=candidate.constructing.as_ref().ok_or(UError::NoEvidence)?;
        if new.current.name!=old.current.name || new.current.learner.checkpoint()?!=old.current.learner.checkpoint()?
            || serde_json::to_vec(&old.education).map_err(|_|UError::InvalidData)?!=serde_json::to_vec(&new.education).map_err(|_|UError::InvalidData)?
            || new.nonce<old.nonce || new.computation_work<old.computation_work || new.total_external_cost!=old.total_external_cost {return Err(UError::Stale);}
        if candidate.emergency_latched(){let mut e=Self::absent_evidence();e.emergency_stop=true;self.protection.screen(0,e);}
        self.organism=candidate.organism;self.constructing=candidate.constructing.take();Ok(())
    }
}

include!("education_runtime.rs");

include!("recursive_search_runtime.rs");
