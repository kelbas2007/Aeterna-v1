//! Disclosed 2x2 ablation: factual identification versus policy credit.
//! Uses existing online-learning switch, never edits weights or goal reward.
use aeterna_v1::HumanProtectionEvidence;
use aeterna_v1::construction::Config;
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost,UnifiedStep};
use serde_json::json;
fn safe(_:usize)->HumanProtectionEvidence{HumanProtectionEvidence{human_present:false,physical_effect_possible:false,predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false}}
#[derive(Clone)]
struct World{p:i64,ready:bool,roles:[usize;4]}
fn step(rt:&mut ScientificRuntime,w:&mut World,forced:Option<usize>)->UnifiedStep{
    let offers=(0..4).map(|a|ActionOffer{available:forced.is_none_or(|f|f==a),max_cost:1}).collect::<Vec<_>>();
    rt.step_constructing(Some(&[Some(1)]),&offers,safe,|a|{
        match w.roles.iter().position(|&id|id==a).unwrap(){0=>w.ready=true,1=>{if w.ready{w.p+=1}w.ready=false;},2=>w.ready=false,3=>{w.p=0;w.ready=false;},_=>unreachable!()}
        Ok(UnifiedPost{observation:vec![w.p],cost:1,reward:0.0})
    }).unwrap()
}
fn acquired(roles:[usize;4],learn_policy_from_forced_actions:bool)->(ScientificRuntime,World){
    let cfg=Config{width:1,actions:4,minimum_training:128,search_every:4096,planning_depth:3,planning_work:120_000,journal_capacity:1024,..Default::default()};
    let mut rt=ScientificRuntime::new_constructing(cfg,AutonomyPolicy::default(),vec![0],"history-credit-ablation").unwrap();
    assert!(rt.set_unified_online_learning(learn_policy_from_forced_actions));
    let mut w=World{p:0,ready:false,roles};
    for i in 0..44{
        step(&mut rt,&mut w,Some(roles[3]));
        step(&mut rt,&mut w,Some(roles[if i%2==0{0}else{2}]));
        step(&mut rt,&mut w,Some(roles[1]));
    }
    step(&mut rt,&mut w,Some(roles[3]));
    // Factual PRE/action/POST learning has continued throughout. Resume
    // the same native policy plasticity for genuine free choices below.
    assert!(rt.set_unified_online_learning(true));
    (rt,w)
}
#[test]
fn prepared_remainder_with_identification_separated_from_policy_training(){
    let (mut rt,mut w)=acquired([0,1,2,3],false);
    // No action-choice learning on the examiner-required preparation either.
    assert!(rt.set_unified_online_learning(false));
    step(&mut rt,&mut w,Some(0));
    assert!(rt.set_unified_online_learning(true));
    rt.enable_constructing_self_directed_control().unwrap();
    rt.enable_constructing_predictive_history_states().unwrap();
    rt.freeze_constructing().unwrap();
    let cp=rt.checkpoint_constructing().unwrap();
    let mut rt=ScientificRuntime::from_constructing_checkpoint(&cp).unwrap();
    let out=step(&mut rt,&mut w,None);
    eprintln!("HISTORY_CREDIT_REMAINDER {}",json!({"action":out.action,"source":out.source,"post":w.p,"native_weights":rt.unified_report().unwrap().native_weights}));
    assert_eq!((out.action,w.p),(1,1),"a learned remainder must win and reach the actual goal without hand-setting weights");
    assert_eq!(out.source,"predictive-history");
}
#[test]
fn two_factor_policy_credit_and_history_ablation_preserves_all_outcomes(){
    let mut counts=[[0usize;2];4];let mut selected=[0usize;4];let mut cases=Vec::new();
    for a in 0..4{for b in 0..4{if a==b{continue}for c in 0..4{if c==a||c==b{continue}
        let d=(0..4).find(|&d|d!=a&&d!=b&&d!=c).unwrap();let roles=[a,b,c,d];
        for credit in 0..2{
            let (trained,w)=acquired(roles,credit==0);let cp=trained.checkpoint_constructing().unwrap();
            let weights=trained.unified_report().unwrap().native_weights;
            for history in 0..2{
                let arm=credit*2+history;
                let mut rt=ScientificRuntime::from_constructing_checkpoint(&cp).unwrap();
                assert_eq!(rt.checkpoint_constructing().unwrap(),cp);
                rt.enable_constructing_self_directed_control().unwrap();
                if history==1{rt.enable_constructing_predictive_history_states().unwrap();}
                let mut world=w.clone();let mut trace=Vec::new();
                for _ in 0..24{if world.p==1{break}let out=step(&mut rt,&mut world,None);
                    if out.source=="predictive-history"{selected[arm]+=1}
                    trace.push(json!({"action":out.action,"source":out.source,"post":world.p}));
                }
                counts[arm][0]+=usize::from(world.p==1);counts[arm][1]+=trace.len();
                cases.push(json!({"roles":roles,"arm":arm,"forced_policy_learning":credit==0,"history_enabled":history==1,"initial_native_weights":weights,"solved":world.p==1,"trace":trace}));
            }
        }
    }}}
    eprintln!("HISTORY_POLICY_CREDIT_2X2 {}",json!({"counts_success_actions":counts,"selected_history":selected,"cases":cases,"independent_qualification":false,"weight_values_manually_set":false}));
    assert_eq!(cases.len(),96);
}
