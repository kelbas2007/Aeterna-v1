//! Exposed experiment: automatic absence-of-choice credit versus the older
//! manual control that had to switch U1 plasticity off for every forced lesson.
//! The roles/world are disclosed; this is *engineering equivalence only*.
use aeterna_v1::HumanProtectionEvidence;
use aeterna_v1::construction::Config;
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost,UnifiedStep};

fn safe(_:usize)->HumanProtectionEvidence {
    HumanProtectionEvidence{human_present:false,physical_effect_possible:false,
        predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false}
}
#[derive(Clone)]
struct World{value:i64,ready:bool,roles:[usize;4]}
fn step(rt:&mut ScientificRuntime,w:&mut World,forced:Option<usize>)->UnifiedStep{
    let offers=(0..4).map(|a|ActionOffer{
        available:forced.is_none_or(|f|f==a),max_cost:1
    }).collect::<Vec<_>>();
    let event=rt.step_constructing(Some(&[Some(1)]),&offers,safe,|action|{
        match w.roles.iter().position(|&a|a==action).unwrap(){
            0=>w.ready=true,
            1=>{if w.ready{w.value+=1;}w.ready=false;},
            2=>w.ready=false,
            3=>{w.value=0;w.ready=false;},
            _=>unreachable!()
        }
        Ok(UnifiedPost{observation:vec![w.value],cost:1,reward:0.0})
    }).unwrap();
    if let Some(a)=forced{assert_eq!(event.action,a)}
    event
}
fn acquired(roles:[usize;4],automatic:bool)->(ScientificRuntime,World){
    let cfg=Config{width:1,actions:4,minimum_training:128,
        search_every:4096,planning_depth:3,planning_work:120_000,
        journal_capacity:1024,..Default::default()};
    let mut rt=ScientificRuntime::new_constructing(cfg,
        AutonomyPolicy::default(),vec![0],"agency-credit-equivalence").unwrap();
    if automatic{rt.enable_constructing_agency_aware_credit().unwrap();}
    else{assert!(rt.set_unified_online_learning(false));}
    let mut w=World{value:0,ready:false,roles};
    for i in 0..44{
        step(&mut rt,&mut w,Some(roles[3]));
        step(&mut rt,&mut w,Some(roles[if i%2==0{0}else{2}]));
        step(&mut rt,&mut w,Some(roles[1]));
    }
    step(&mut rt,&mut w,Some(roles[3]));
    if !automatic{assert!(rt.set_unified_online_learning(true));}
    assert_eq!(w.value,0);
    (rt,w)
}
#[test]
fn automatic_agency_rule_matches_manual_control_on_disclosed_role_permutations(){
    // These permutations come from PREVIOUSLY EXPOSED diagnostic fixtures.
    // This cannot count as fresh transfer qualification or method discovery.
    let mut cases=0_usize;let mut solved=[0usize;2];let mut actions=[0usize;2];
    for a in 0..4{for b in 0..4{if a==b{continue}
    for c in 0..4{if c==a||c==b{continue}
        let d=(0..4).find(|&i|i!=a&&i!=b&&i!=c).unwrap();
        let roles=[a,b,c,d];
        let (manual,mw)=acquired(roles,false);
        let (automatic,aw)=acquired(roles,true);
        assert_eq!(manual.unified_report().unwrap().native_weights,
                   automatic.unified_report().unwrap().native_weights);
        assert_eq!(manual.unified_report().unwrap().native_meta_observations,
                   automatic.unified_report().unwrap().native_meta_observations);
        assert_eq!(manual.construction_learner().unwrap().records().len(),
                   automatic.construction_learner().unwrap().records().len());
        assert_eq!(automatic.unified_report().unwrap().agency_credit.as_ref()
                   .unwrap().externally_forced_actions,133);
        let checkpoint=[manual.checkpoint_constructing().unwrap(),
                        automatic.checkpoint_constructing().unwrap()];
        let worlds=[mw,aw];let mut traces=[Vec::new(),Vec::new()];
        for arm in 0..2{
            let mut rt=ScientificRuntime::from_constructing_checkpoint(&checkpoint[arm]).unwrap();
            rt.enable_constructing_self_directed_control().unwrap();
            rt.enable_constructing_predictive_history_states().unwrap();
            let mut w=worlds[arm].clone();
            for _ in 0..24{
                if w.value>=1{break}
                let chosen=step(&mut rt,&mut w,None);
                traces[arm].push((chosen.action,chosen.source.clone(),w.value));
            }
            solved[arm]+=usize::from(w.value>=1);
            actions[arm]+=traces[arm].len();
        }
        assert_eq!(traces[0],traces[1],
            "same factual journal and U1 weights must lead to identical autonomous decisions");
        cases+=1;
    }}}
    assert_eq!(cases,24);
    eprintln!("AGENCY_CREDIT_EQUIVALENCE {}",
        serde_json::json!({"exposed_cases":cases,
        "manual_success":solved[0],"automatic_success":solved[1],
        "manual_actions":actions[0],"automatic_actions":actions[1],
        "independent_qualification":false}));
    assert_eq!(solved[0],solved[1]);
    assert_eq!(actions[0],actions[1]);
}
