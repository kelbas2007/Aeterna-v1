//! Paired EXPOSED developmental integration, not blind AGI qualification.
//! Evidence comes from actual safeguarded external POST callbacks, never a
//! synthetic model inserted into the Learner. Training offers ARE scheduled
//! for identifiability; autonomous discovery from scratch remains unproven.
use aeterna_v1::HumanProtectionEvidence;
use aeterna_v1::construction::Config;
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost,UnifiedStep};

fn safe(_:usize)->HumanProtectionEvidence {
    HumanProtectionEvidence{human_present:false,physical_effect_possible:false,
        predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false}
}
// Evaluator-only opaque motor assignment. [progress, prerequisite A, prerequisite B].
// Positive conjunctive action leaves preconditions intact for repeat testing;
// there is a separate physically executed reset motor for varied contrasts.
fn physical(w:&mut [i64;3],action:usize,roles:[usize;4]) {
    if action==roles[0]{w[1]=1}
    else if action==roles[1]{w[2]=1}
    else if action==roles[2]{if w[1]==1&&w[2]==1{w[0]+=1}}
    else if action==roles[3]{w[1]=0;w[2]=0}
    else {panic!("unoffered motor")}
}
fn step(rt:&mut ScientificRuntime,w:&mut [i64;3],roles:[usize;4],
        offered:Option<usize>,goal:i64)->UnifiedStep {
    let offers=(0..4).map(|a|ActionOffer{
        available:offered.is_none_or(|only|a==only),max_cost:1
    }).collect::<Vec<_>>();
    rt.step_constructing(Some(&[Some(goal),None,None]),&offers,safe,|action|{
        physical(w,action,roles);
        Ok(UnifiedPost{observation:w.to_vec(),cost:1,reward:0.0})
    }).unwrap()
}
#[test]
fn same_observation_checkpoint_candidate_and_u1_execute_protected_actions() {
    let roles=[1,3,0,2];
    let cfg=Config{width:3,actions:4,minimum_training:128,search_every:4096,
        planning_depth:4,planning_work:120_000,..Default::default()};
    let mut acquisition=ScientificRuntime::new_constructing(
        cfg,AutonomyPolicy::default(),vec![0;3],"actual-observation-training").unwrap();
    let mut world=[0_i64;3];
    // Disclosed identification phase: probe each possible conjunction input
    // using safely executed motors. No motor meanings or simulated POSTs are
    // delivered to the organism. Both eval arms receive the SAME checkpoint.
    for i in 0..48 {
        let (a,b)=[(false,false),(true,false),(false,true),(true,true)][i%4];
        step(&mut acquisition,&mut world,roles,Some(roles[3]),100);
        if a {step(&mut acquisition,&mut world,roles,Some(roles[0]),100);}
        if b {step(&mut acquisition,&mut world,roles,Some(roles[1]),100);}
        step(&mut acquisition,&mut world,roles,Some(roles[2]),100);
    }
    step(&mut acquisition,&mut world,roles,Some(roles[3]),100);
    assert_eq!(world,[12,0,0],"all positive transitions must be actual external consequences");
    let shared=acquisition.checkpoint_constructing().unwrap();
    let mut baseline=ScientificRuntime::from_constructing_checkpoint(&shared).unwrap();
    let mut direct=ScientificRuntime::from_constructing_checkpoint(&shared).unwrap();
    let mut candidate=ScientificRuntime::from_constructing_checkpoint(&shared).unwrap();
    assert_eq!(baseline.checkpoint_constructing().unwrap(),shared);
    assert_eq!(direct.checkpoint_constructing().unwrap(),shared);
    assert_eq!(candidate.checkpoint_constructing().unwrap(),shared);
    direct.enable_constructing_self_directed_control().unwrap();
    candidate.enable_constructing_self_directed_control().unwrap();
    candidate.enable_constructing_certified_chains().unwrap();
    let mut results=Vec::new();
    for (rt,arm) in [(&mut baseline,"u1"),(&mut direct,"factual-direct"),(&mut candidate,"certified-chain")] {
        let mut current=world;
        let mut sources=Vec::new();
        let mut chain_offers=0_usize;
        for _ in 0..48 {
            if current[0]>=13{break}
            let r=step(rt,&mut current,roles,None,13);
            chain_offers+=usize::from(r.certified_chain_proposed);
            sources.push(r.source);
        }
        results.push((arm,current,sources,chain_offers));
    }
    let base_steps=results[0].2.len();
    let direct_steps=results[1].2.len();
    let upgraded_steps=results[2].2.len();
    let chain_selections=results[2].2.iter().filter(|s|s.as_str()=="factual-chain").count();
    eprintln!("THREE_ARM_ACTUAL_POST same_checkpoint baseline={:?}/{} direct={:?}/{} chain={:?}/{} chain_offered={} chain_selected={} chain_sources={:?}",
        results[0].1,base_steps,results[1].1,direct_steps,results[2].1,upgraded_steps,results[2].3,chain_selections,results[2].2);
    assert!(results.iter().all(|(_,state,_,_)|state[0]>=13),"all protected arms must reach the goal");
    assert!(direct_steps<=base_steps,"actual acquired effects must not regress this exposed case");
    assert!(upgraded_steps<=base_steps,"new chain mechanism must not regress original U1");
    // Selection of a new chain is a separate SCIENTIFIC milestone, not an
    // integration-safety assertion. Print the negative result without falsifying
    // the measured gain due to the already existing factual-experience path.
    // No 'chain advantage' claim is allowed unless direct and chain differ
    // in an independently specified paired challenge.
}
