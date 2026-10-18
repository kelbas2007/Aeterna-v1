//! Authored teaching example, not an independent intelligence qualification.
//! Learn an unknown actuator increment, then freeze and probe new setpoints.
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost};
use aeterna_v1::construction::Config;
use aeterna_v1::construction::synthesis::SearchConfig;
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::HumanProtectionEvidence;
fn safe(_:usize)->HumanProtectionEvidence {HumanProtectionEvidence{human_present:false,physical_effect_possible:false,predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false}}
fn config()->(Config,AutonomyPolicy){
    let search=SearchConfig{work:200_000,max_expr_nodes:3,max_state:0,max_frontier:2048,max_updates:4,deterministic:true};
    let cfg=Config{width:32,actions:4,search,minimum_training:16,search_every:8,shadow_steps:12,journal_capacity:2048,planning_depth:3,planning_work:100_000};
    let policy=AutonomyPolicy{ceiling:SearchConfig{work:400_000,max_expr_nodes:5,max_frontier:4096,max_updates:8,max_state:1,deterministic:true},failures_per_expansion:2,max_external_cost:10000,evidence:None};
    (cfg,policy)
}
fn perform(r:&mut ScientificRuntime,goal:Option<&[Option<i64>]>)->Result<serde_json::Value,Box<dyn std::error::Error>> {
    let mut observation=r.construction_learner().unwrap().current().to_vec();
    let report=r.step_constructing(goal,&[ActionOffer{available:true,max_cost:1};4],safe,|a|{
        observation[0]+=a as i64;
        let reached=goal.is_some_and(|g|observation.iter().zip(g).all(|(&x,y)|y.is_none_or(|v|x==v)));
        Ok(UnifiedPost{observation,cost:1,reward:if reached{1.0}else{0.0}})
    })?;Ok(serde_json::to_value(report)?)
}
fn evaluate(bytes:&[u8],ablated:bool)->Result<serde_json::Value,Box<dyn std::error::Error>> {
    let mut r=ScientificRuntime::from_constructing_checkpoint(bytes)?;
    r.freeze_constructing()?;
    if ablated {r.ablate_constructing_controller()?;}
    let program=r.construction_learner().unwrap().program().fingerprint();
    let weights=r.unified_report().unwrap().native_weights;
    let mut episodes=Vec::new();
    for offset in [4,5,7,8,6,9,5,8,7,4,9,6] {
        let start=r.construction_learner().unwrap().current()[0];
        let target=start+offset;let mut goal=vec![None;32];goal[0]=Some(target);
        let mut trace=Vec::new();
        for _ in 0..6 {
            trace.push(perform(&mut r,Some(&goal))?);
            if r.construction_learner().unwrap().current()[0]>=target {break;}
        }
        episodes.push(serde_json::json!({"start":start,"target":target,"reached":r.construction_learner().unwrap().current()[0]==target,"steps":trace.len(),"trace":trace}));
    }
    if r.construction_learner().unwrap().program().fingerprint()!=program || r.unified_report().unwrap().native_weights!=weights {return Err("probe mutated learning".into());}
    Ok(serde_json::json!({"controller_ablated":ablated,"same_initial_program":program,"episodes":episodes,"final":r.unified_report()}))
}
fn main()->Result<(),Box<dyn std::error::Error>> {
    let (cfg,policy)=config();let mut r=ScientificRuntime::new_constructing(cfg,policy,vec![0;32],"integer-actuator-v1")?;
    let cold=r.checkpoint_constructing()?;
    let mut sample=Vec::new();
    for i in 0..256 {
        let mut goal=vec![None;32];goal[0]=Some(r.construction_learner().unwrap().current()[0]+(i%3+1) as i64);
        let step=perform(&mut r,if i<64{None}else{Some(&goal)})?;
        if i<8 || i>=248 {sample.push(step);}
    }
    let trained=r.checkpoint_constructing()?;
    if let Some(path)=std::env::args().nth(1){std::fs::write(path,&trained)?;}
    let report=serde_json::json!({"kind":"unified-learning-development","independent_qualification":false,"training_events":256,
        "solution_injected":false,"constructor_injected":false,"training_goal_schedule_authored":true,
        "initial":ScientificRuntime::from_constructing_checkpoint(&cold)?.unified_report(),"trained":r.unified_report(),
        "acquired_program":r.construction_learner().unwrap().program(),"sample_training_decisions":sample,
        "full":evaluate(&trained,false)?,"same_knowledge_zero_controller":evaluate(&trained,true)?,"cold":evaluate(&cold,false)?});
    println!("{}",serde_json::to_string_pretty(&report)?);Ok(())
}
