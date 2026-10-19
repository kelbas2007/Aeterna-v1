//! AGENCY-CREDIT1 EXPOSED development only; see preregistered protocol.
//! Every chosen motor is screened by HumanProtection; only REAL POSTS enter
//! the same Learner journal, even when action-policy credit is suppressed.
use aeterna_v1::HumanProtectionEvidence;
use aeterna_v1::construction::Config;
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost};

fn safe(_:usize)->HumanProtectionEvidence {
    HumanProtectionEvidence{human_present:false,physical_effect_possible:false,
        predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false}
}
fn initial()->ScientificRuntime {
    ScientificRuntime::new_constructing(Config{
        width:1,actions:2,minimum_training:128,search_every:4096,
        journal_capacity:512,planning_work:4000,..Default::default()
    },AutonomyPolicy::default(),vec![0],"agency-credit-exposed").unwrap()
}
fn step(rt:&mut ScientificRuntime,world:&mut i64,forced:Option<usize>){
    let offers=(0..2).map(|a|ActionOffer{
        available:forced.is_none_or(|id|id==a),max_cost:1
    }).collect::<Vec<_>>();
    let result=rt.step_constructing(Some(&[Some(2)]),&offers,safe,|action|{
        match action{0=>*world=(*world+1).min(2),1=>*world=0,_=>unreachable!()}
        Ok(UnifiedPost{observation:vec![*world],cost:1,reward:0.0})
    }).unwrap();
    if let Some(action)=forced{assert_eq!(result.action,action);}
}
#[test]
fn forced_actions_acquire_facts_but_cannot_change_self_choice_weights(){
    let mut guarded=initial();guarded.enable_constructing_agency_aware_credit().unwrap();
    let mut legacy=initial();let mut a=0_i64;let mut b=0_i64;
    // Both arms experience the same externally forced physical actions and
    // consequences; only the optional agency-aware native U1 credit differs.
    for i in 0..32{
        let forced=if i%4==3 {1}else{0};
        step(&mut guarded,&mut a,Some(forced));
        step(&mut legacy,&mut b,Some(forced));
        assert_eq!(a,b);
    }
    assert_eq!(a,0);
    assert_eq!(guarded.construction_learner().unwrap().records().len(),32);
    assert_eq!(legacy.construction_learner().unwrap().records().len(),32);
    assert_eq!(guarded.unified_report().unwrap().native_meta_observations,0);
    assert!(guarded.unified_report().unwrap().native_weights.iter().all(|&w|w==0.0));
    assert!(legacy.unified_report().unwrap().native_meta_observations>0);
    assert!(legacy.unified_report().unwrap().native_weights.iter().any(|&w|w>0.0));
    let report=guarded.unified_report().unwrap();
    let ledger=report.agency_credit.as_ref().unwrap();
    assert_eq!((ledger.externally_forced_actions,ledger.self_selected_actions),(32,0));
    let bytes=guarded.checkpoint_constructing().unwrap();
    let mut restored=ScientificRuntime::from_constructing_checkpoint(&bytes).unwrap();
    assert_eq!(restored.checkpoint_constructing().unwrap(),bytes);
    let mut restored_world=a;
    step(&mut restored,&mut restored_world,None);
    let report=restored.unified_report().unwrap();
    let ledger=report.agency_credit.as_ref().unwrap();
    assert_eq!((ledger.externally_forced_actions,ledger.self_selected_actions),(32,1));
    assert!(report.native_meta_observations>0,
        "an actual choice between two offered motors must receive native U1 credit");
    assert_eq!(restored.construction_learner().unwrap().records().len(),33);
}
#[test]
fn agency_credit_opt_in_is_one_way_and_pre_experience_only(){
    let mut rt=initial();
    rt.enable_constructing_agency_aware_credit().unwrap();
    assert!(rt.enable_constructing_agency_aware_credit().is_err());
    let mut world=0;
    step(&mut rt,&mut world,Some(0));
    assert!(rt.enable_constructing_agency_aware_credit().is_err());
    let mut late=initial();
    step(&mut late,&mut world,Some(1));
    assert!(late.enable_constructing_agency_aware_credit().is_err());
    let mut frozen=initial();frozen.freeze_constructing().unwrap();
    assert!(frozen.enable_constructing_agency_aware_credit().is_err());
}
#[test]
fn denied_forced_action_never_receives_policy_credit_or_experience(){
    let mut rt=initial();rt.enable_constructing_agency_aware_credit().unwrap();
    let offers=[ActionOffer{available:true,max_cost:1},
                ActionOffer{available:false,max_cost:1}];
    let mut called=false;
    let result=rt.step_constructing(Some(&[Some(2)]),&offers,|_|{
        let mut e=safe(0);e.emergency_stop=true;e
    },|_|{
        called=true;
        Ok(UnifiedPost{observation:vec![1],cost:1,reward:0.0})
    });
    assert!(result.is_err());assert!(!called);
    assert_eq!(rt.construction_learner().unwrap().records().len(),0);
    assert_eq!(rt.unified_report().unwrap().native_meta_observations,0);
    let report=rt.unified_report().unwrap();
    let ledger=report.agency_credit.unwrap();
    assert_eq!((ledger.externally_forced_actions,ledger.self_selected_actions),(0,0));
    let bytes=rt.checkpoint_constructing().unwrap();
    let restored=ScientificRuntime::from_constructing_checkpoint(&bytes).unwrap();
    assert_eq!(restored.checkpoint_constructing().unwrap(),bytes);
}
