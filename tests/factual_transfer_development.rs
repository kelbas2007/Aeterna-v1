//! Post-design, disclosed synthetic developmental diagnostics.
//! Two previously unexecuted rule families. Do NOT call held-out after this run.
use aeterna_v1::HumanProtectionEvidence;
use aeterna_v1::construction::{Config};
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost};
fn safe(_:usize)->HumanProtectionEvidence {HumanProtectionEvidence{
    human_present:false,physical_effect_possible:false,predicted_harm_probability:0.0,
    hazard_confidence:1.0,emergency_stop:false}}
fn organism(mode:bool)->ScientificRuntime {
    let cfg=Config{width:4,actions:4,minimum_training:128,search_every:4096,
        planning_depth:4,planning_work:4000,..Default::default()};
    let mut rt=ScientificRuntime::new_constructing(cfg,AutonomyPolicy::default(),vec![0;4],"unseen-compound").unwrap();
    if mode {rt.enable_constructing_self_directed_control().unwrap();}
    rt
}
fn physical(world:&mut [i64;4],action:usize,roles:[usize;4],kind:usize){
    let role=roles.iter().position(|&a|a==action).unwrap();
    match role {
        0 => {world[1]=1;},
        1 => {if world[1]==1 {world[2]=1;} else if kind==1 {world[2]=0;}},
        2 => {world[3]=if kind==0 {3}else{2};},
        3 => {
            if world[1]==1&&world[2]==1&&world[3]>0 {
                world[0]+=1;world[1]=0;world[2]=0;world[3]-=1;
            } else if kind==1 && world[3]>0 {world[3]-=1;}
        },
        _=>unreachable!()
    }
}
#[test]
fn compound_prerequisite_repeated_reset_development_controls(){
    for kind in 0..2 {
        let mut solved=[0usize;2];let mut actions=[0usize;2];let mut sources=[0usize;2];let mut credible=[0usize;2];let mut runs=0;
        for a in 0..4 {for b in 0..4 {for c in 0..4 {for d in 0..4 {
            let roles=[a,b,c,d];
            if roles.iter().copied().collect::<std::collections::BTreeSet<_>>().len()!=4{continue}
            runs+=1;
            for (arm,enabled) in [(0,true),(1,false)] {
                let mut rt=organism(enabled);let mut world=[0;4];let offers=vec![ActionOffer{available:true,max_cost:1};4];
                let mut taken=0;
                while taken<96 && world[0]<3 {
                    let response=rt.step_constructing(Some(&[Some(3),None,None,None]),&offers,safe,|action|{
                        physical(&mut world,action,roles,kind);
                        Ok(UnifiedPost{observation:world.to_vec(),cost:1,reward:0.0})
                    }).unwrap();
                    sources[arm]+=usize::from(response.source=="factual-experience"||response.source=="factual-probe");
                    credible[arm]+=usize::from(response.source=="factual-experience");
                    taken+=1;
                }
                solved[arm]+=usize::from(world[0]>=3);actions[arm]+=taken;
            }
        }}}}
        eprintln!("NEW_COMPOUND_DEV family={kind} n={runs} enabled={}/{} actions={} factual_proposals={} certified={} baseline={}/{} actions={}",
                  solved[0],runs,actions[0],sources[0],credible[0],solved[1],runs,actions[1]);
        assert_eq!(runs,24);
        assert_eq!(sources[1],0);
    }
}
#[test]
fn acquired_consequences_can_be_offered_after_prospective_validation(){
    let cfg=Config{width:2,actions:2,minimum_training:128,search_every:4096,planning_work:4000,..Default::default()};
    let mut rt=ScientificRuntime::new_constructing(cfg,AutonomyPolicy::default(),vec![0;2],"sensor-bound").unwrap();
    rt.enable_constructing_self_directed_control().unwrap();
    let mut world=[0i64;2];
    let forced=[ActionOffer{available:true,max_cost:1},ActionOffer{available:false,max_cost:1}];
    let all=[ActionOffer{available:true,max_cost:1};2];
    for _ in 0..12 {let s=rt.step_constructing(Some(&[Some(100),None]),&forced,safe,|a|{
        if a==0{world[0]+=1};Ok(UnifiedPost{observation:world.to_vec(),cost:1,reward:0.0})
    }).unwrap();assert_eq!(s.action,0);}
    // A separate lifetime restores the SAME acquired journal before reading
    // any fresh evidence. The transferred rule was not hard-coded in the test.
    let cp=rt.checkpoint_constructing().unwrap();
    let mut rt=ScientificRuntime::from_constructing_checkpoint(&cp).unwrap();
    assert_eq!(rt.checkpoint_constructing().unwrap(),cp);
    let mut certified=0;let mut probes=0;
    for _ in 0..12 {let s=rt.step_constructing(Some(&[Some(100),None]),&all,safe,|a|{
        if a==0{world[0]+=1};Ok(UnifiedPost{observation:world.to_vec(),cost:1,reward:0.0})
    }).unwrap();certified+=usize::from(s.source=="factual-experience");probes+=usize::from(s.source=="factual-probe");}
    eprintln!("CERTIFIED_EFFECT_PROGRESS source={} probe={} state={world:?}",certified,probes);
    assert!(certified>0,"a prospectively validated acquired effect must influence U1 after restore");
    assert!(world[0]>=12,"observed successful effect must persist after checkpoint restoration");
    // An environment change invalidates the formerly acquired belief. The
    // organism should stop calling it a verified factual consequence.
    for _ in 0..3 {rt.step_constructing(Some(&[Some(200),None]),&forced,safe,|a|{
        if a==0{world[0]+=10};Ok(UnifiedPost{observation:world.to_vec(),cost:1,reward:0.0})
    }).unwrap();}
    let out=rt.step_constructing(Some(&[Some(200),None]),&forced,safe,|a|{
        if a==0{world[0]+=10};Ok(UnifiedPost{observation:world.to_vec(),cost:1,reward:0.0})
    }).unwrap();
    assert_ne!(out.source,"factual-experience","invalidated belief must abstain, not hallucinate certainty");
}
