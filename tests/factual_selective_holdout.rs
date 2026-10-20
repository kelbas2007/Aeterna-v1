//! One-shot five-sensor factual-development challenge, frozen before first run.
//! A handwritten evaluator world is NOT an independent external AGI benchmark.
use aeterna_v1::HumanProtectionEvidence;
use aeterna_v1::construction::Config;
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost};
fn safe(_:usize)->HumanProtectionEvidence {HumanProtectionEvidence{
    human_present:false,physical_effect_possible:false,
    predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false}}
fn organism(enabled:bool)->ScientificRuntime {
    let cfg=Config{width:5,actions:5,minimum_training:128,search_every:4096,
        planning_depth:4,planning_work:4000,..Default::default()};
    let mut rt=ScientificRuntime::new_constructing(cfg,AutonomyPolicy::default(),vec![0;5],"five-sensor-holdout").unwrap();
    if enabled{rt.enable_constructing_self_directed_control().unwrap();}
    rt
}
// Evaluator-only: [progress, access, gate, energy, alarm].
fn physical(w:&mut [i64;5],action:usize,roles:[usize;5]) {
    match roles.iter().position(|&a|a==action).unwrap(){
        0=>{if w[4]==0{w[1]=1;}},
        1=>{if w[1]==1&&w[4]==0{w[2]=1;}},
        2=>{if w[4]==0{w[3]=2;}},
        3=>{
            if w[1]==1&&w[2]==1&&w[3]>0&&w[4]==0 {
                w[0]+=1;w[1]=0;w[2]=0;w[3]-=1;
            }else if w[3]>0 {
                w[3]-=1;w[1]=0;w[2]=0;w[4]=1;
            }
        },
        4=>{w[4]=0;},
        _=>unreachable!(),
    }
}
#[test]
fn one_shot_five_sensor_conjunction_recovery() {
    let mut cases=0;let mut solved=[0usize;2];let mut taken=[0usize;2];
    let mut probes=[0usize;2];let mut certified=[0usize;2];
    let offers=vec![ActionOffer{available:true,max_cost:1};5];
    for a in 0..5 {for b in 0..5 {for c in 0..5 {for d in 0..5 {for e in 0..5 {
        let roles=[a,b,c,d,e];
        if roles.iter().copied().collect::<std::collections::BTreeSet<_>>().len()!=5{continue}
        cases+=1;
        for (arm,enable) in [(0,true),(1,false)] {
            let mut rt=organism(enable);let mut world=[0i64;5];let mut n=0;
            while n<144&&world[0]<2 {
                let result=rt.step_constructing(Some(&[Some(2),None,None,None,None]),&offers,safe,|act|{
                    physical(&mut world,act,roles);
                    Ok(UnifiedPost{observation:world.to_vec(),cost:1,reward:0.0})
                }).unwrap();
                probes[arm]+=usize::from(result.source=="factual-probe");
                certified[arm]+=usize::from(result.source=="factual-experience");
                n+=1;
            }
            solved[arm]+=usize::from(world[0]>=2);taken[arm]+=n;
        }
    }}}}}
    assert_eq!(cases,120);
    assert_eq!(certified[1]+probes[1],0);
    eprintln!("ONE_SHOT_FIVE_SENSOR cases={cases} acquired={}/{} actions={} probes={} certified={} baseline={}/{} actions={}",
        solved[0],cases,taken[0],probes[0],certified[0],solved[1],cases,taken[1]);
}
