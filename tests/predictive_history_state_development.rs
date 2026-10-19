//! EXPOSED research diagnostic, NOT independent generalization or AGI.
//! Every sensor consequence is from the ordinary protected external POST;
//! "charged" is hidden strictly inside the evaluator and NEVER sent to U1.
use aeterna_v1::HumanProtectionEvidence;
use aeterna_v1::construction::Config;
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost,UnifiedStep};

fn safe(_:usize)->HumanProtectionEvidence{
    HumanProtectionEvidence{human_present:false,physical_effect_possible:false,
        predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false}
}
#[derive(Clone,Copy)]
struct World{progress:i64,charged:bool,hidden_effect:bool}
impl World{
    fn act(&mut self,action:usize){
        match action{
            0=>self.charged=true,   // invisible prerequisite
            1=>{
                if self.charged||!self.hidden_effect{self.progress+=1}
                self.charged=false;
            },
            2=>self.charged=false,  // invisible distractor
            3=>{self.progress=0;self.charged=false}, // observed reset motor
            _=>unreachable!(),
        }
    }
}
fn step(rt:&mut ScientificRuntime,world:&mut World,forced:Option<usize>)->UnifiedStep{
    let offers=(0..4).map(|a|ActionOffer{
        available:forced.is_none_or(|n|n==a),max_cost:1
    }).collect::<Vec<_>>();
    rt.step_constructing(Some(&[Some(1)]),&offers,safe,|a|{
        world.act(a);
        Ok(UnifiedPost{observation:vec![world.progress],cost:1,reward:0.0})
    }).unwrap()
}
fn acquired(hidden_effect:bool)->(ScientificRuntime,World){
    let cfg=Config{width:1,actions:4,minimum_training:128,search_every:4096,
        planning_depth:3,planning_work:120_000,journal_capacity:1024,
        ..Default::default()};
    let mut rt=ScientificRuntime::new_constructing(cfg,AutonomyPolicy::default(),
        vec![0],"invisible-opaque-motor").unwrap();
    let mut world=World{progress:0,charged:false,hidden_effect};
    for i in 0..44{
        step(&mut rt,&mut world,Some(3));
        step(&mut rt,&mut world,Some(if i%2==0{0}else{2}));
        step(&mut rt,&mut world,Some(1));
    }
    step(&mut rt,&mut world,Some(3));
    assert_eq!(world.progress,0);
    assert_eq!(rt.construction_learner().unwrap().records().len(),133);
    (rt,world)
}
#[test]
fn learner_discovers_invisible_prerequisite_from_protected_real_post(){
    let (rt,mut world)=acquired(true);
    let common=rt.checkpoint_constructing().unwrap();
    let mut candidate=ScientificRuntime::from_constructing_checkpoint(&common).unwrap();
    assert_eq!(candidate.checkpoint_constructing().unwrap(),common);
    candidate.enable_constructing_self_directed_control().unwrap();
    candidate.enable_constructing_predictive_history_states().unwrap();
    let round_trip=candidate.checkpoint_constructing().unwrap();
    assert_eq!(ScientificRuntime::from_constructing_checkpoint(&round_trip)
        .unwrap().checkpoint_constructing().unwrap(),round_trip);
    let first=step(&mut candidate,&mut world,None);
    assert!(first.predictive_history_plan_proposed,
        "an action with no visible effect should lead to a certified future gain");
    // A proposed causal action is NOT counted as selected by native U1.
    if first.source=="predictive-history"{
        assert_eq!(first.action,0);
        assert_eq!(world.progress,0);
    }
    assert_eq!(candidate.construction_learner().unwrap().records().len(),134);
}
#[test]
fn identical_outcomes_under_all_histories_do_not_invent_latent_states(){
    let (mut rt,mut world)=acquired(false);
    rt.enable_constructing_self_directed_control().unwrap();
    rt.enable_constructing_predictive_history_states().unwrap();
    let first=step(&mut rt,&mut world,None);
    assert!(!first.predictive_history_plan_proposed,
        "different predecessor actions are not enough: real POST must differ");
}
#[test]
fn unavailable_prerequisite_motor_cannot_be_invented_in_imagined_plan(){
    let (mut rt,mut world)=acquired(true);
    rt.enable_constructing_self_directed_control().unwrap();
    rt.enable_constructing_predictive_history_states().unwrap();
    let offers=(0..4).map(|a|ActionOffer{available:a!=0,max_cost:1})
        .collect::<Vec<_>>();
    let step=rt.step_constructing(Some(&[Some(1)]),&offers,safe,|a|{
        world.act(a);
        Ok(UnifiedPost{observation:vec![world.progress],cost:1,reward:0.0})
    }).unwrap();
    assert!(!step.predictive_history_plan_proposed,
        "an unavailable motor must not survive as a safe plan");
}
#[test]
fn underfunded_history_reasoning_and_legacy_control_fail_closed(){
    let cfg=Config{width:1,actions:4,minimum_training:128,search_every:4096,
        planning_work:4000,..Default::default()};
    let mut rt=ScientificRuntime::new_constructing(cfg,AutonomyPolicy::default(),
        vec![0],"legacy-budget").unwrap();
    rt.enable_constructing_self_directed_control().unwrap();
    assert!(rt.enable_constructing_predictive_history_states().is_err());
}
