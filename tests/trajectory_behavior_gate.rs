//! T-SR2 disclosed engineering tests only. NOT independent generalization.
//! HumanProtection is invoked for every external virtual action, including
//! paired behavioral admission; no evaluator outcome labels reach the learner.
use aeterna_v1::HumanProtectionEvidence;
use aeterna_v1::construction::Config;
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::scientific_runtime::{
    ScientificRuntime,UnifiedPost,MetaBehaviorTask,
};
fn safe(_:usize)->HumanProtectionEvidence {
    HumanProtectionEvidence {
        human_present:false,physical_effect_possible:false,
        predicted_harm_probability:0.0,hazard_confidence:1.0,
        emergency_stop:false,
    }
}
fn blocked(_:usize)->HumanProtectionEvidence {
    HumanProtectionEvidence{
        human_present:false,physical_effect_possible:false,
        predicted_harm_probability:1.0,hazard_confidence:1.0,
        emergency_stop:true,
    }
}
fn config()->Config{
    Config {width:1,actions:4,minimum_training:128,search_every:4096,
        planning_depth:3,planning_work:4000,journal_capacity:1024,
        ..Default::default()}
}
fn acquire()->ScientificRuntime {
    let mut rt=ScientificRuntime::new_constructing(config(),
        AutonomyPolicy::default(),vec![0],"T-SR2-disclosed-training").unwrap();
    rt.enable_constructing_trajectory_self_revision().unwrap();
    rt.enable_constructing_behavioral_gate().unwrap();
    let mut observed=0_i64;
    for episode in 0..24{
        let count=if episode%2==0{8}else{2};
        let action=if episode%2==0{1}else{0};
        for _ in 0..count {
            let offers=(0..4).map(|i|ActionOffer{
                available:i==action,max_cost:1
            }).collect::<Vec<_>>();
            rt.step_constructing(Some(&[Some(2)]),&offers,safe,|a|{
                if a==0 {observed+=1}
                Ok(UnifiedPost{observation:vec![observed],cost:1,reward:0.0})
            }).unwrap();
        }
        // Physically observed, goal-less reset; never a fabricated POST.
        let offers=(0..4).map(|a|ActionOffer{
            available:a==2,max_cost:1
        }).collect::<Vec<_>>();
        rt.step_constructing(None,&offers,safe,|a|{
            assert_eq!(a,2);observed=0;
            Ok(UnifiedPost{observation:vec![0],cost:1,reward:0.0})
        }).unwrap();
        if rt.trajectory_self_revision_report().unwrap().generations>0 {break}
    }
    let report=rt.trajectory_self_revision_report().unwrap();
    assert_eq!(report.generations,1,"a genuine self-synthesized VM evaluator is required");
    assert!(report.behavior_guard_enabled);
    assert!(!report.behavior_attempted);
    assert!(report.behavioral_outcome.is_none(),"prediction alone cannot deploy a scorer");
    assert!(report.active_program.is_some(),"VM expression must exist even when readout remains guarded");
    rt
}
fn tasks()->Vec<MetaBehaviorTask>{
    (0..8).map(|index|MetaBehaviorTask{
        cohort:if index<4{0}else{1},
        goal:vec![Some(2)],
        offers:vec![ActionOffer{available:true,max_cost:1};4],
        max_actions:8,
    }).collect()
}
#[test]
fn real_protected_paired_canary_qualifies_or_blocks_and_survives_checkpoint(){
    let mut rt=acquire();
    let before=rt.checkpoint_constructing().unwrap();
    let restored=ScientificRuntime::from_constructing_checkpoint(&before).unwrap();
    assert_eq!(restored.checkpoint_constructing().unwrap(),before);
    // Each world instance belongs exclusively to (task, arm). Neither
    // variant sees the role map, and no preprogrammed evaluation actions
    // are given to either decision maker.
    let mut states=[[0_i64;2];8];
    let audit=rt.qualify_constructing_behavioral_gate(&tasks(),
        |_,_,action|safe(action),
        |case,arm,action|{
            match action {
                0=>states[case][arm]+=1,
                1=>{},
                2=>states[case][arm]=0,
                3=>states[case][arm]=(states[case][arm]-1).max(0),
                _=>unreachable!(),
            }
            Ok(UnifiedPost{observation:vec![states[case][arm]],cost:1,reward:0.0})
        }).unwrap();
    assert_eq!(audit.cases.len(),8);
    assert_eq!(audit.summary.cases,8);
    assert!(audit.cases.iter().all(|case|
        case.arms.iter().all(|arm|!arm.steps.is_empty())));
    let report=rt.trajectory_self_revision_report().unwrap();
    assert!(report.behavior_attempted);
    assert_eq!(report.behavioral_outcome.as_ref().unwrap(),&audit.summary);
    let saved=rt.checkpoint_constructing().unwrap();
    let restored=ScientificRuntime::from_constructing_checkpoint(&saved).unwrap();
    assert_eq!(restored.checkpoint_constructing().unwrap(),saved);
    assert!(rt.qualify_constructing_behavioral_gate(&tasks(),
        |_,_,a|safe(a),|_,_,_|panic!("a consumed canary cannot reexecute")).is_err());
    assert_eq!(rt.checkpoint_constructing().unwrap(),saved);
}
#[test]
fn safety_denial_consumes_canary_and_cannot_turn_into_fabricated_evidence(){
    let original=acquire();
    let cp=original.checkpoint_constructing().unwrap();
    let mut rt=ScientificRuntime::from_constructing_checkpoint(&cp).unwrap();
    let work_before=rt.unified_report().unwrap().computation_work;
    let result=rt.qualify_constructing_behavioral_gate(&tasks(),
        |_,_,action|blocked(action),
        |_,_,_|panic!("never call actual POST after emergency veto"));
    assert!(result.is_err());
    assert!(rt.unified_report().unwrap().computation_work>work_before,
        "failed behavioral-gate sandbox must charge work already spent before veto");
    let report=rt.trajectory_self_revision_report().unwrap();
    assert!(report.behavior_attempted);
    assert!(report.behavioral_outcome.is_none());
    assert!(rt.qualify_constructing_behavioral_gate(&tasks(),
        |_,_,a|safe(a),|_,_,_|panic!("no second canary trial")).is_err());
    let after=rt.checkpoint_constructing().unwrap();
    assert_eq!(ScientificRuntime::from_constructing_checkpoint(&after)
        .unwrap().checkpoint_constructing().unwrap(),after);
}
