//! AGENCY-CREDIT2 preregistered single-use transfer from FORCED factual
//! acquisition to UNFORCED protected action policy. NOT an AGI benchmark.
//! Protocol committed first: docs/AGENCY_CREDIT2_ONE_USE_PROTOCOL.md.
//! TEST IS IGNORED by normal CI, executed only by its exact source-frozen
//! one-use GitHub Actions science gate. Negative outcomes must be preserved.
use aeterna_v1::HumanProtectionEvidence;
use aeterna_v1::construction::Config;
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost};
use serde_json::{json,Value};

fn safe(_:usize)->HumanProtectionEvidence {
    HumanProtectionEvidence {
        human_present:false,physical_effect_possible:false,
        predicted_harm_probability:0.0,hazard_confidence:1.0,
        emergency_stop:false,
    }
}
fn config()->Config{
    Config{width:1,actions:4,minimum_training:128,search_every:4096,
        planning_depth:3,planning_work:120_000,journal_capacity:1024,
        ..Default::default()}
}
// Source-world acquisition (EXPOSED): a hidden one-use prerequisite. The
// evaluator controls the available actions ONLY during identification.
// Neither the task name nor action roles are given to U1 or the Learner.
fn acquire(agency_aware:bool)->ScientificRuntime {
    let mut rt=ScientificRuntime::new_constructing(
        config(),AutonomyPolicy::default(),vec![0],"AC2-acquisition").unwrap();
    if agency_aware {rt.enable_constructing_agency_aware_credit().unwrap();}
    let mut progress=0_i64;let mut primed=false;
    for i in 0..44{
        for forced in [3_usize,if i%2==0{0}else{2},1]{
            let offers=(0..4).map(|a|ActionOffer{
                available:a==forced,max_cost:1
            }).collect::<Vec<_>>();
            rt.step_constructing(Some(&[Some(1)]),&offers,safe,|a|{
                match a {
                    0=>primed=true,
                    1=>{if primed{progress+=1};primed=false},
                    2=>primed=false,
                    3=>{progress=0;primed=false},
                    _=>unreachable!(),
                }
                Ok(UnifiedPost{observation:vec![progress],cost:1,reward:0.0})
            }).unwrap();
        }
    }
    let offers=(0..4).map(|a|ActionOffer{
        available:a==3,max_cost:1
    }).collect::<Vec<_>>();
    rt.step_constructing(Some(&[Some(1)]),&offers,safe,|a|{
        assert_eq!(a,3);progress=0;primed=false;
        Ok(UnifiedPost{observation:vec![0],cost:1,reward:0.0})
    }).unwrap();
    assert_eq!(rt.construction_learner().unwrap().records().len(),133);
    assert_eq!(rt.construction_learner().unwrap().current(),[0]);
    rt.enable_constructing_self_directed_control().unwrap();
    rt.enable_constructing_predictive_history_states().unwrap();
    // No target-cohort experiences or reward updates after this point.
    rt.freeze_constructing().unwrap();
    let cp=rt.checkpoint_constructing().unwrap();
    assert_eq!(ScientificRuntime::from_constructing_checkpoint(&cp)
        .unwrap().checkpoint_constructing().unwrap(),cp);
    rt
}
fn mix(mut x:u64)->u64{
    x=x.wrapping_add(0x9e3779b97f4a7c15);
    x=(x^(x>>30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x=(x^(x>>27)).wrapping_mul(0x94d049bb133111eb);
    x^(x>>31)
}
fn permute(seed:u64,family:usize,case:usize)->[usize;4]{
    let mut p=[0,1,2,3];
    let mut v=mix(seed^((family as u64+1)<<32)^
        ((case as u64+1).wrapping_mul(0xa24baed4963ee407)));
    for i in (1..4).rev(){
        v=mix(v);
        p.swap(i,(v%(i as u64+1)) as usize);
    }
    p
}
#[derive(Clone)]
struct World {
    family:usize,roles:[usize;4],progress:i64,
    ready:bool,counter:u8,
}
impl World {
    fn new(family:usize,roles:[usize;4])->Self{
        Self{family,roles,progress:0,ready:false,counter:0}
    }
    fn act(&mut self,action:usize){
        let role=self.roles.iter().position(|&r|r==action).unwrap();
        match (self.family,role) {
            // One-use latch TOGGLES: double priming erases preparation.
            (0,0)=>self.ready=!self.ready,
            (0,1)=>{if self.ready{self.progress+=1}self.ready=false},
            (0,2)=>self.ready=false,
            (0,3)=>{self.ready=false;self.progress=0},
            // A pulse expires when ANY other motor intervenes.
            (1,0)=>self.ready=true,
            (1,1)=>{if self.ready{self.progress+=1}self.ready=false},
            (1,2)=>self.ready=false,
            (1,3)=>{self.ready=false;self.progress=0},
            // Cannot be represented by a ONE-action suffix:
            // two preparatory events before the use operation.
            (2,0)=>self.counter=self.counter.saturating_add(1).min(2),
            (2,1)=>{if self.counter>=2{self.progress+=1}self.counter=0},
            (2,2)=>self.counter=0,
            (2,3)=>{self.progress=0;self.counter=0},
            // Negative control: no hidden prerequisite to correct.
            (3,0)=>self.ready=!self.ready,
            (3,1)=>self.progress+=1,
            (3,2)=>self.ready=false,
            (3,3)=>{self.progress=0;self.ready=false},
            _=>unreachable!(),
        }
    }
}
fn trial(rt:&mut ScientificRuntime,world:&mut World)->Value {
    let offers=[ActionOffer{available:true,max_cost:1};4];
    let mut trace=Vec::new();
    let mut selected_history=0_usize;
    let mut first_action=None::<usize>;
    for _ in 0..24{
        if world.progress>=1{break}
        let chosen=rt.step_constructing(Some(&[Some(1)]),&offers,safe,|a|{
            world.act(a);
            Ok(UnifiedPost{observation:vec![world.progress],cost:1,reward:0.0})
        }).unwrap();
        if first_action.is_none(){first_action=Some(chosen.action)}
        if chosen.source=="predictive-history"{selected_history+=1}
        trace.push(json!({
            "action":chosen.action,
            "source":chosen.source,
            "history_offered":chosen.predictive_history_plan_proposed,
            "real_post":world.progress,
        }));
    }
    json!({"solved":world.progress>=1,"actions":trace.len(),
        "first":first_action,"selected_history":selected_history,"trace":trace})
}
#[test]
#[ignore="one-use preregistered external-source virtual-world qualification"]
fn agency_credit_fresh_causal_transfer() {
    let seed=std::env::var("GITHUB_RUN_ID").expect("nonselectable GitHub run ID")
        .parse::<u64>().expect("numeric run ID");
    let sha=std::env::var("GITHUB_SHA").expect("exact CI test/merge SHA");
    let baseline=acquire(false);
    let candidate=acquire(true);
    let p0=baseline.construction_learner().unwrap().records();
    let p1=candidate.construction_learner().unwrap().records();
    assert_eq!(p0.len(),p1.len());
    for (a,b) in p0.iter().zip(p1){
        assert_eq!(a.transition.pre,b.transition.pre);
        assert_eq!(a.transition.action,b.transition.action);
        assert_eq!(a.transition.post,b.transition.post);
    }
    let candidate_ledger=candidate.unified_report().unwrap().agency_credit
        .expect("agency control not enabled before real acquisition");
    assert_eq!((candidate_ledger.externally_forced_actions,
        candidate_ledger.self_selected_actions),(133,0));
    let acquired_weights=[
        baseline.unified_report().unwrap().native_weights,
        candidate.unified_report().unwrap().native_weights,
    ];
    let snapshots=[
        baseline.checkpoint_constructing().unwrap(),
        candidate.checkpoint_constructing().unwrap(),
    ];
    let mut families=Vec::new();
    let mut cases=Vec::new();
    let mut total_success=[0usize;2];
    let mut total_actions=[0usize;2];
    let mut total_selected_history=[0usize;2];
    let mut divergence=0usize;
    for family in 0..4{
        let mut success=[0usize;2];let mut actions=[0usize;2];
        let mut chosen_history=[0usize;2];
        for case in 0..8{
            let roles=permute(seed,family,case);
            let mut arms=Vec::new();
            for arm in 0..2{
                let mut rt=ScientificRuntime::from_constructing_checkpoint(
                    &snapshots[arm]).unwrap();
                assert_eq!(rt.checkpoint_constructing().unwrap(),snapshots[arm]);
                let mut w=World::new(family,roles);
                let answer=trial(&mut rt,&mut w);
                success[arm]+=usize::from(answer["solved"]==true);
                actions[arm]+=answer["actions"].as_u64().unwrap() as usize;
                chosen_history[arm]+=answer["selected_history"].as_u64().unwrap() as usize;
                arms.push(answer);
            }
            divergence+=usize::from(arms[0]["first"]!=arms[1]["first"]);
            cases.push(json!({"family":family,"case":case,"roles":roles,
                "legacy":arms[0],"agency_aware":arms[1]}));
        }
        for arm in 0..2{
            total_success[arm]+=success[arm];
            total_actions[arm]+=actions[arm];
            total_selected_history[arm]+=chosen_history[arm];
        }
        families.push(json!({"family":family,
            "legacy_success":success[0],"agency_success":success[1],
            "legacy_actions":actions[0],"agency_actions":actions[1],
            "legacy_history_selected":chosen_history[0],
            "agency_history_selected":chosen_history[1]}));
    }
    let no_family_regression=families.iter().all(|f|
        f["agency_success"].as_u64().unwrap()>=
        f["legacy_success"].as_u64().unwrap());
    let qualified=no_family_regression && total_success[1]>=total_success[0]
        && total_actions[1]*10<=total_actions[0]*9
        && divergence>0 && total_selected_history[1]>0;
    eprintln!("AGENCY_CREDIT2_RESULT {}",json!({
        "verdict":if qualified{"QUALIFIED_SYNTHETIC"}else{"NEGATIVE_ACTION_TRANSFER"},
        "qualified":qualified,"seed":seed,"sha":sha,
        "training_steps":p0.len(),"initial_u1_weights":acquired_weights,
        "total_success":total_success,"total_actions":total_actions,
        "selected_history":total_selected_history,"first_action_divergences":divergence,
        "no_family_regression":no_family_regression,"families":families,"cases":cases
    }));
    assert!(qualified,"preregistered 4-family causal action-transfer gate failed");
}
