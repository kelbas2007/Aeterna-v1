//! T-SR1: preregistered source-disclosed virtual-world qualification.
//! Protocol: docs/TRAJECTORY_SELF_REVISION_TRANSFER_PROTOCOL.md
//! Engineering test runs normally; SCIENTIFIC test is ignored unless the
//! one-use GitHub Actions workflow executes it with a nonselectable run ID.
use aeterna_v1::HumanProtectionEvidence;
use aeterna_v1::construction::Config;
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::construction::vm::fingerprint;
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost,UnifiedStep};
use serde_json::{json,Value};

fn safe(_:usize)->HumanProtectionEvidence{
    HumanProtectionEvidence{human_present:false,physical_effect_possible:false,
        predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false}
}
#[derive(Clone,Copy)]
struct World{progress:i64,roles:[usize;4],family:usize}
impl World{
    fn act(&mut self,action:usize){
        let role=self.roles.iter().position(|r|*r==action).expect("four opaque motors");
        match role{
            0=>{if self.family!=1||self.progress<2{
                self.progress+=if self.family==2 {2}else{1};
            }},
            1=>{if self.family==1&&self.progress==2{self.progress=3;}},
            2=>self.progress=0,
            3=>self.progress=(self.progress-if self.family==3{2}else{1}).max(0),
            _=>unreachable!(),
        }
    }
}
fn step(rt:&mut ScientificRuntime,w:&mut World,goal:Option<i64>,forced:Option<usize>)->UnifiedStep{
    let offers:Vec<ActionOffer>=(0..4).map(|a|ActionOffer{
        available:forced.is_none_or(|f|f==a),max_cost:1
    }).collect();
    let target=goal.map(|g|vec![Some(g)]);
    let result=rt.step_constructing(target.as_deref(),&offers,safe,|action|{
        w.act(action);Ok(UnifiedPost{observation:vec![w.progress],cost:1,reward:0.0})
    }).expect("physical callback or permission failed");
    if let Some(a)=forced{assert_eq!(result.action,a)}
    result
}
fn config()->Config{
    Config{width:1,actions:4,minimum_training:128,search_every:4096,
        planning_depth:3,planning_work:4000,journal_capacity:1024,..Default::default()}
}
fn acquire()->(ScientificRuntime,usize){
    let mut rt=ScientificRuntime::new_constructing(config(),AutonomyPolicy::default(),
        vec![0],"T-SR1-identification").unwrap();
    rt.enable_constructing_trajectory_self_revision().unwrap();
    let mut w=World{progress:0,roles:[0,1,2,3],family:0};
    let mut episodes=0;
    for i in 0..24{
        // Disclosed IDENTIFICATION only; no host actions are specified in eval.
        if i%2==0{
            for _ in 0..8{step(&mut rt,&mut w,Some(2),Some(1));}
        }else{
            step(&mut rt,&mut w,Some(2),Some(0));
            step(&mut rt,&mut w,Some(2),Some(0));
        }
        step(&mut rt,&mut w,None,Some(2)); // genuine authorized reset
        assert_eq!(w.progress,0);
        episodes+=1;
        let report=rt.trajectory_self_revision_report().unwrap();
        if report.generations>0 && !report.shadow && episodes>=12{break}
    }
    assert_eq!(rt.construction_learner().unwrap().current(),[0]);
    (rt,episodes)
}
fn mix(mut x:u64)->u64{
    x=x.wrapping_add(0x9e3779b97f4a7c15);
    x=(x^(x>>30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x=(x^(x>>27)).wrapping_mul(0x94d049bb133111eb);
    x^(x>>31)
}
fn permutation(seed:u64,family:usize,case:usize)->[usize;4]{
    let mut arr=[0,1,2,3];
    let mut x=mix(seed^((family as u64+1)<<32)^(case as u64+0x54318));
    for i in (1..4).rev(){x=mix(x);arr.swap(i,(x%(i as u64+1)) as usize);}
    arr
}
fn target(family:usize)->i64{
    match family{0=>2,1=>3,2=>4,3=>3,_=>unreachable!()}
}
fn evaluate(checkpoint:&[u8],seed:u64)->Value{
    let mut families=Vec::new();let mut cases=Vec::new();
    let mut total_success=[0usize;2];let mut total_actions=[0usize;2];
    let mut first_action_divergences=0usize;let mut first_score_divergences=0usize;
    for family in 0..4{
        let mut success=[0usize;2];let mut actions=[0usize;2];
        for case in 0..12{
            let roles=permutation(seed,family,case);
            let mut traces=[Vec::<usize>::new(),Vec::<usize>::new()];
            let mut scores=[None::<f32>,None::<f32>];
            let mut wins=[false;2];
            for arm in 0..2{
                let mut rt=ScientificRuntime::from_constructing_checkpoint(checkpoint).unwrap();
                assert_eq!(rt.checkpoint_constructing().unwrap(),checkpoint);
                if arm==0{rt.ablate_frozen_trajectory_evaluator().unwrap();}
                let mut w=World{progress:0,roles,family};
                for turn in 0..24{
                    if w.progress>=target(family){break}
                    let result=step(&mut rt,&mut w,Some(target(family)),None);
                    if turn==0{scores[arm]=Some(result.selected_fields[0]);}
                    traces[arm].push(result.action);
                }
                wins[arm]=w.progress>=target(family);
                success[arm]+=usize::from(wins[arm]);
                actions[arm]+=traces[arm].len();
            }
            first_action_divergences+=usize::from(traces[0].first()!=traces[1].first());
            first_score_divergences+=usize::from(scores[0]!=scores[1]);
            cases.push(json!({"family":family,"case":case,"roles":roles,
                "old":{"solved":wins[0],"trace":traces[0]},
                "revised":{"solved":wins[1],"trace":traces[1]},
                "old_first_score":scores[0],"revised_first_score":scores[1]}));
        }
        for arm in 0..2{total_success[arm]+=success[arm];total_actions[arm]+=actions[arm];}
        families.push(json!({"family":family,"old_success":success[0],
            "new_success":success[1],"old_actions":actions[0],"new_actions":actions[1]}));
    }
    let no_family_regression=families.iter().all(|v|
        v["new_success"].as_u64().unwrap()>=v["old_success"].as_u64().unwrap());
    let qualified=no_family_regression && total_success[1]>=total_success[0]
        && total_actions[1]*10<=total_actions[0]*9
        && first_action_divergences>0 && first_score_divergences>0;
    json!({"families":families,"cases":cases,
        "old_success":total_success[0],"revised_success":total_success[1],
        "old_actions":total_actions[0],"revised_actions":total_actions[1],
        "first_action_divergences":first_action_divergences,
        "first_score_divergences":first_score_divergences,
        "no_family_regression":no_family_regression,"qualified":qualified})
}
#[test]
fn frozen_evaluator_ablation_requires_admitted_code(){
    let mut rt=ScientificRuntime::new_constructing(config(),AutonomyPolicy::default(),
        vec![0],"T-SR1-ablation").unwrap();
    rt.enable_constructing_trajectory_self_revision().unwrap();
    assert!(rt.ablate_frozen_trajectory_evaluator().is_err());
    rt.freeze_constructing().unwrap();
    assert!(rt.ablate_frozen_trajectory_evaluator().is_err());
}
#[test]
#[ignore = "one-use preregistered external virtual-world qualification"]
fn preregistered_one_use_action_transfer(){
    let seed=std::env::var("GITHUB_RUN_ID").expect("CI run ID")
        .parse::<u64>().expect("numeric run ID");
    let sha=std::env::var("GITHUB_SHA").expect("exact CI source SHA");
    let (mut learned,episodes)=acquire();
    let report=learned.trajectory_self_revision_report().unwrap();
    let acquired_cp=learned.checkpoint_constructing().unwrap();
    if report.generations==0 || report.active_program.is_none(){
        eprintln!("T_SR1_RESULT {}",json!({
            "status":"NOT_ADMITTED","qualified":false,
            "seed":seed,"sha":sha,"episodes":episodes,"learner":report,
            "checkpoint":fingerprint(&acquired_cp)}));
        panic!("scientific negative: no evaluator was autonomously admitted");
    }
    learned.freeze_constructing().unwrap();
    let checkpoint=learned.checkpoint_constructing().unwrap();
    assert_eq!(ScientificRuntime::from_constructing_checkpoint(&checkpoint)
        .unwrap().checkpoint_constructing().unwrap(),checkpoint);
    let trial=evaluate(&checkpoint,seed);
    let qualified=trial["qualified"].as_bool().unwrap_or(false);
    eprintln!("T_SR1_RESULT {}",json!({
        "status":if qualified{"QUALIFIED_SYNTHETIC"}else{"NEGATIVE_ACTION_ADVANTAGE"},
        "qualified":qualified,"sha":sha,"seed":seed,"episodes":episodes,
        "checkpoint":fingerprint(&checkpoint),
        "learned_program":report.active_program,
        "generations":report.generations,"trial":trial}));
    assert!(qualified,"no preregistered behavioral improvement from self-revised evaluator");
}
