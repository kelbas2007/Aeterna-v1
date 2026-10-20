//! T-SR2 one-use protected-action challenge. PRE-REGISTERED before outcomes.
//! New source-disclosed latent-gate worlds, NOT the exposed T-SR1 families.
//! Never claim virtual-world qualification implies real-world AGI.
use aeterna_v1::HumanProtectionEvidence;
use aeterna_v1::construction::Config;
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::construction::vm::fingerprint;
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost,MetaBehaviorTask};
use serde_json::json;

fn safe(_:usize)->HumanProtectionEvidence{
    HumanProtectionEvidence{human_present:false,physical_effect_possible:false,
        predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false}
}
fn config()->Config{
    Config{width:1,actions:4,minimum_training:128,search_every:4096,
        planning_depth:3,planning_work:4000,journal_capacity:1024,
        ..Default::default()}
}
// Disclosure: teacher-free evaluator-code generation still uses an authored
// *acquisition action schedule* solely to expose diverse real consequences.
fn acquire()->(ScientificRuntime,usize){
    let mut rt=ScientificRuntime::new_constructing(config(),AutonomyPolicy::default(),
        vec![0],"T-SR2-protected-experience").unwrap();
    rt.enable_constructing_trajectory_self_revision().unwrap();
    rt.enable_constructing_behavioral_gate().unwrap();
    let mut observed=0_i64;
    let mut episodes=0_usize;
    for i in 0..24{
        let action=if i%2==0{1}else{0};
        let n=if action==1{8}else{2};
        let offers:Vec<_>=(0..4).map(|a|ActionOffer{available:a==action,max_cost:1}).collect();
        for _ in 0..n{
            rt.step_constructing(Some(&[Some(2)]),&offers,safe,|a|{
                if a==0{observed+=1}
                Ok(UnifiedPost{observation:vec![observed],cost:1,reward:0.0})
            }).unwrap();
        }
        let reset:Vec<_>=(0..4).map(|a|ActionOffer{available:a==2,max_cost:1}).collect();
        rt.step_constructing(None,&reset,safe,|a|{
            assert_eq!(a,2);observed=0;
            Ok(UnifiedPost{observation:vec![0],cost:1,reward:0.0})
        }).unwrap();
        episodes+=1;
        if rt.trajectory_self_revision_report().unwrap().generations>0{break}
    }
    (rt,episodes)
}
fn mix(mut x:u64)->u64{
    x=x.wrapping_add(0x9e3779b97f4a7c15);
    x=(x^(x>>30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x=(x^(x>>27)).wrapping_mul(0x94d049bb133111eb);x^(x>>31)
}
fn roles(seed:u64,family:usize,case:usize)->[usize;4]{
    let mut arr=[0,1,2,3];
    let mut x=mix(seed^((family as u64+1)<<32)^
        ((case as u64+1).wrapping_mul(0xA24BAED4963EE407)));
    for i in (1..4).rev(){x=mix(x);arr.swap(i,(x%(i as u64+1)) as usize);}
    arr
}
#[derive(Clone,Copy)]
struct World{family:usize,roles:[usize;4],progress:i64,gate:u8,energy:u8,stage:u8}
impl World{
    fn new(family:usize,roles:[usize;4])->Self{
        Self{family,roles,progress:0,gate:0,energy:0,stage:0}
    }
    fn execute(&mut self,action:usize){
        let r=self.roles.iter().position(|&a|a==action).unwrap();
        match (self.family,r) {
            (0,0)=>{if self.gate>0{self.progress+=1;self.gate=0}},
            (0,1)=>self.gate=1,
            (0,2)=>self.gate=0,
            (0,3)=>self.progress=(self.progress-1).max(0),
            (1,0)=>{if self.energy>0{self.progress+=1;self.energy-=1}},
            (1,1)=>self.energy=2,
            (1,2)=>self.energy=0,
            (1,3)=>self.progress=(self.progress-1).max(0),
            (2,0)=>{if self.stage>=2{self.progress+=2;self.stage=0}},
            (2,1)=>self.stage=self.stage.saturating_add(1).min(2),
            (2,2)=>self.stage=0,
            (2,3)=>self.progress=(self.progress-1).max(0),
            (3,0)=>{if self.gate>0&&self.energy>0{
                self.progress+=2;self.gate=0;self.energy-=1;
            }},
            (3,1)=>self.gate=1,
            (3,2)=>self.energy=2,
            (3,3)=>self.progress=(self.progress-1).max(0),
            _=>unreachable!(),
        }
    }
}
#[test]
#[ignore="T-SR2 one-use disjoint virtual-world qualification; requires GITHUB_RUN_ID"]
fn t_sr2_one_use_behavioral_gate(){
    let seed=std::env::var("GITHUB_RUN_ID").expect("nonselectable GitHub run ID")
        .parse::<u64>().expect("numeric seed");
    let sha=std::env::var("GITHUB_SHA").expect("exact source SHA");
    let (mut rt,episodes)=acquire();
    let before=rt.trajectory_self_revision_report().unwrap();
    let cp=rt.checkpoint_constructing().unwrap();
    if before.generations==0 || before.active_program.is_none(){
        eprintln!("T_SR2_RESULT {}",json!({
            "verdict":"NOT_ADMITTED","qualified":false,
            "sha":sha,"seed":seed,"training_episodes":episodes,"report":before,
            "checkpoint":fingerprint(&cp)}));
        panic!("No autonomously acquired VM evaluator, so no behavioral admission");
    }
    assert!(!before.behavior_attempted && before.behavior_guard_enabled);
    let tasks=(0..4).flat_map(|family|(0..8).map(move |_|MetaBehaviorTask{
        cohort:family as u16,
        goal:vec![Some(match family{0=>2,1=>3,2=>4,3=>4,_=>unreachable!()})],
        offers:vec![ActionOffer{available:true,max_cost:1};4],
        max_actions:24,
    })).collect::<Vec<_>>();
    let mut worlds=(0..32).map(|i|{
        let f=i/8;
        let perm=roles(seed,f,i%8);
        [World::new(f,perm),World::new(f,perm)]
    }).collect::<Vec<_>>();
    let audit=match rt.qualify_constructing_behavioral_gate(&tasks,
        |_,_,a|safe(a),|i,arm,a|{
            worlds[i][arm].execute(a);
            Ok(UnifiedPost{observation:vec![worlds[i][arm].progress],
                cost:1,reward:0.0})
        }){
        Ok(result)=>result,
        Err(error)=>{
            eprintln!("T_SR2_RESULT {}",json!({"verdict":"PROTECTED_EXECUTION_FAILURE",
                "qualified":false,"seed":seed,"sha":sha,"error":format!("{error:?}")}));
            panic!("Protected behavioral trial failed");
        }
    };
    let report=rt.trajectory_self_revision_report().unwrap();
    assert!(report.behavior_attempted);
    assert_eq!(report.behavioral_outcome.as_ref(),Some(&audit.summary));
    let after=rt.checkpoint_constructing().unwrap();
    assert_eq!(ScientificRuntime::from_constructing_checkpoint(&after)
        .unwrap().checkpoint_constructing().unwrap(),after);
    let accepted=audit.summary.qualified;
    eprintln!("T_SR2_RESULT {}",json!({
        "verdict":if accepted{"QUALIFIED_SYNTHETIC"}else{"NEGATIVE_BEHAVIORAL_ADMISSION"},
        "qualified":accepted,"sha":sha,"seed":seed,"training_episodes":episodes,
        "learned_program":before.active_program,
        "predictive_generations":before.generations,
        "before_checkpoint":fingerprint(&cp),"after_checkpoint":fingerprint(&after),
        "summary":audit.summary,
        "case_results":audit.cases.iter().map(|c|json!({
            "cohort":c.task.cohort,"goal":c.task.goal,
            "old_actions":c.arms[0].steps.len(),"new_actions":c.arms[1].steps.len(),
            "old_posts":c.arms[0].steps.iter().map(|s|s.post[0]).collect::<Vec<_>>(),
            "new_posts":c.arms[1].steps.iter().map(|s|s.post[0]).collect::<Vec<_>>()
        })).collect::<Vec<_>>()
    }));
    assert!(accepted,"self-revised evaluator failed preregistered behavior/cost/cohort gate");
}
