//! Read-only score diagnosis, not a policy repair or qualification.
use aeterna_v1::HumanProtectionEvidence;
use aeterna_v1::construction::Config;
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost};
use serde_json::json;
fn safe(_:usize)->HumanProtectionEvidence{HumanProtectionEvidence{human_present:false,physical_effect_possible:false,predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false}}
#[test]
fn inspect_native_feature_conductances_without_changing_them(){
    let cfg=Config{width:1,actions:4,minimum_training:128,search_every:4096,planning_depth:3,planning_work:120_000,journal_capacity:1024,..Default::default()};
    let mut rt=ScientificRuntime::new_constructing(cfg,AutonomyPolicy::default(),vec![0],"history-action-regression").unwrap();
    let mut value=0_i64;let mut primed=false;
    let mut schedule=Vec::new();
    for i in 0..44{schedule.extend([3,if i%2==0{0}else{2},1]);}
    schedule.extend([3,0]);
    for forced in schedule{
        let offers=(0..4).map(|a|ActionOffer{available:a==forced,max_cost:1}).collect::<Vec<_>>();
        rt.step_constructing(Some(&[Some(1)]),&offers,safe,|a|{
            match a{0=>primed=true,1=>{if primed{value+=1}primed=false;},2=>primed=false,3=>{value=0;primed=false},_=>unreachable!()}
            Ok(UnifiedPost{observation:vec![value],cost:1,reward:0.0})
        }).unwrap();
    }
    rt.enable_constructing_self_directed_control().unwrap();
    rt.enable_constructing_predictive_history_states().unwrap();
    rt.freeze_constructing().unwrap();
    let cp=rt.checkpoint_constructing().unwrap();
    let rt=ScientificRuntime::from_constructing_checkpoint(&cp).unwrap();
    let components=(0..5).map(|i|{let mut f=[0.0_f32;5];f[i]=1.0;rt.organism().phase_native_meta_score(f).unwrap()}).collect::<Vec<_>>();
    let history=[1.0,1.0/44.0,0.0,0.9,1.0/(1.0+1.0_f32.ln_1p())];
    let legacy=[0.5,0.0,0.3125,0.5,1.0/(1.0+1.0_f32.ln_1p())];
    eprintln!("HISTORY_NATIVE_DIAG {}",json!({"synapse_weights":rt.unified_report().unwrap().native_weights,"unit_feature_scores":components,"history_fields":history,"history_score":rt.organism().phase_native_meta_score(history),"legacy_fields":legacy,"legacy_score":rt.organism().phase_native_meta_score(legacy)}));
    assert_eq!(rt.checkpoint_constructing().unwrap(),cp,"diagnosis must not mutate any learned state");
}
