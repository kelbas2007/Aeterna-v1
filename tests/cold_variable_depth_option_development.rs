//! VARIABLE-LENGTH-OPTIONS-0: DISCLOSED cold no-tutor developmental trial.
//! Protocol: docs/VARIABLE_LENGTH_CAUSAL_OPTIONS_DEVELOPMENT.md
//! Absolutely no forced acquisition motor or fake PRE/action/POST rows.
use aeterna_v1::HumanProtectionEvidence;
use aeterna_v1::construction::Config;
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost};
use serde_json::{json,Value};

const OFFER:ActionOffer=ActionOffer{available:true,max_cost:1};
const EPISODES:usize=24;
const HORIZON:usize=18;
fn safe(_:usize)->HumanProtectionEvidence{
    HumanProtectionEvidence{human_present:false,physical_effect_possible:false,
        predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false}
}
#[derive(Clone)]
struct World{
    roles:[usize;4],family:usize,stage:u8,progress:i64
}
impl World{
    fn new(family:usize,roles:[usize;4])->Self{
        Self{roles,family,stage:0,progress:0}
    }
    fn act(&mut self,motor:usize){
        let role=self.roles.iter().position(|&a|a==motor).unwrap();
        match (self.family,role){
            // F0: four essential latent actions A,B,C,USE.
            // A prerequisite is never visible in the numeric observation.
            (0,0)=>self.stage=1,
            (0,1)=>self.stage=if self.stage==1{2}else{0},
            (0,2)=>self.stage=if self.stage==2{3}else{0},
            (0,3)=>{
                if self.stage==3{self.progress+=1}
                self.stage=0;
            },
            // F1: five necessary actions A,B,A,B,USE.
            // Order and repetition must be acquired, not scripted for U1.
            (1,0)=>self.stage=match self.stage{0=>1,2=>3,_=>0},
            (1,1)=>self.stage=match self.stage{1=>2,3=>4,_=>0},
            (1,2)=>{
                if self.stage==4{self.progress+=1}
                self.stage=0;
            },
            (1,3)=>self.stage=0,
            // F2: regression retention of prior 3-motor causal world.
            (2,0)=>self.stage=1,
            (2,1)=>self.stage=if self.stage==1{2}else{0},
            (2,2)=>{
                if self.stage==2{self.progress+=1}
                self.stage=0;
            },
            (2,3)=>self.stage=0,
            // F3: direct-goal negative control (no latent prerequisite).
            (3,0)|(3,1)=>{},
            (3,2)=>self.progress+=1,
            (3,3)=>self.progress=0,
            _=>unreachable!()
        }
    }
}
fn fresh(variable_depth:bool)->ScientificRuntime{
    let cfg=Config{width:1,actions:4,minimum_training:128,
        search_every:4096,planning_depth:5,planning_work:120_000,
        journal_capacity:2048,..Default::default()};
    let mut rt=ScientificRuntime::new_constructing(
        cfg,AutonomyPolicy::default(),vec![0],"cold-adaptive-depth").unwrap();
    rt.enable_constructing_agency_aware_credit().unwrap();
    rt.enable_constructing_self_directed_control().unwrap();
    rt.enable_constructing_predictive_history_states().unwrap();
    rt.enable_constructing_adaptive_history_suffix().unwrap();
    rt.enable_constructing_temporal_options().unwrap();
    rt.enable_constructing_autobiographical_options().unwrap();
    if variable_depth{rt.enable_constructing_variable_depth_options().unwrap();}
    rt.enable_constructing_active_causal_probes().unwrap();
    assert!(rt.construction_learner().unwrap().records().is_empty());
    assert_eq!(rt.unified_report().unwrap().native_meta_observations,0);
    rt
}
fn paired_lifetime(family:usize,roles:[usize;4],variable:bool)->Value{
    let mut rt=fresh(variable);
    let mut goals=0usize;
    let mut actions=0usize;
    let mut early=0usize;
    let mut late=0usize;
    let mut probes=0usize;
    let mut veto_safety_verified=0usize;
    let mut episodes=Vec::new();
    for ep in 0..EPISODES{
        let mut world=World::new(family,roles);
        if ep>0{
            let n=rt.construction_learner().unwrap().records().len();
            let c=rt.unified_report().unwrap().total_external_cost;
            rt.observe_constructing_external_restart(vec![0]).unwrap();
            assert_eq!(n,rt.construction_learner().unwrap().records().len());
            assert_eq!(c,rt.unified_report().unwrap().total_external_cost);
        }
        let mut trace=Vec::new();
        for _ in 0..HORIZON{
            if world.progress>=1{break}
            let result=rt.step_constructing(Some(&[Some(1)]),&[OFFER;4],
                safe,|motor|{
                    world.act(motor);
                    Ok(UnifiedPost{
                        observation:vec![world.progress],cost:1,reward:0.0
                    })
                }).unwrap();
            actions+=1;
            probes+=usize::from(result.source=="causal-pair-probe");
            let lane=rt.unified_report().unwrap().temporal_options.unwrap();
            if result.source=="autobiographical-option" &&
                lane.pending_steps>=3 && veto_safety_verified==0{
                // Additional CLONED evidence does not influence this
                // organism's own future actions or motor-selection stream.
                let cp=rt.checkpoint_constructing().unwrap();
                let mut shadow=ScientificRuntime::from_constructing_checkpoint(&cp)
                    .unwrap();
                let before=shadow.construction_learner().unwrap().records().len();
                let mut touched=false;
                let denied=shadow.step_constructing(
                    Some(&[Some(1)]),&[OFFER;4],
                    |_a|{let mut h=safe(0);h.emergency_stop=true;h},
                    |_a|{touched=true;Ok(UnifiedPost{
                        observation:vec![1],cost:1,reward:0.0})}
                );
                assert!(denied.is_err() && !touched);
                assert_eq!(shadow.construction_learner().unwrap().records().len(),before);
                assert_eq!(shadow.unified_report().unwrap()
                    .temporal_options.unwrap().pending_steps,0);
                veto_safety_verified=1;
            }
            trace.push(json!({"action":result.action,"source":result.source,
                "real_post":world.progress,
                "option_proposed":result.autobiographical_option_proposed,
                "continued_option_steps":lane.pending_steps}));
        }
        let solved=world.progress>=1;
        goals+=usize::from(solved);
        if ep<6{early+=usize::from(solved)}
        if ep>=EPISODES-6{late+=usize::from(solved)}
        episodes.push(json!({"episode":ep,"solved":solved,
            "steps":trace.len(),"actual_protected_actions":trace}));
    }
    let report=rt.unified_report().unwrap();
    let agency=report.agency_credit.as_ref().unwrap();
    assert_eq!(agency.externally_forced_actions,0);
    assert_eq!(agency.self_selected_actions+
        agency.option_continuation_actions,actions as u64);
    assert_eq!(report.total_external_cost,actions as u64);
    let records=rt.construction_learner().unwrap().records();
    assert_eq!(records.len(),actions);
    assert_eq!(records.iter().filter(|r|r.transition.restart).count(),EPISODES);
    let lane=report.temporal_options.unwrap();
    assert_eq!(lane.variable_depth_enabled,variable);
    let cp=rt.checkpoint_constructing().unwrap();
    assert_eq!(ScientificRuntime::from_constructing_checkpoint(&cp)
        .unwrap().checkpoint_constructing().unwrap(),cp);
    json!({"family":family,"roles":roles,"variable_depth":variable,
        "goals":goals,"actions":actions,"early_goals":early,"late_goals":late,
        "self_selected_options":lane.autobiographical_started,
        "selected_depths":lane.learned_depth_selected,
        "completed_depths":lane.learned_depth_completed,
        "veto_safety_verified":veto_safety_verified,
        "self_selected_causal_probes":probes,
        "episodes":episodes})
}
fn mix(mut n:u64)->u64{
    n=n.wrapping_add(0x9e3779b97f4a7c15);
    n=(n^(n>>30)).wrapping_mul(0xbf58476d1ce4e5b9);
    n=(n^(n>>27)).wrapping_mul(0x94d049bb133111eb);
    n^(n>>31)
}
fn roles(family:usize,n:usize)->Vec<[usize;4]>{
    let mut v=Vec::new();
    for a in 0..4 {for b in 0..4 {if a==b{continue}
        for c in 0..4 {if c==a||c==b{continue}
            let d=(0..4).find(|&m|m!=a&&m!=b&&m!=c).unwrap();
            v.push([a,b,c,d]);
        }
    }}
    let mut seed=mix(0x6ba4_6c4d_1701_32f0 ^ family as u64);
    for i in (1..v.len()).rev(){
        seed=mix(seed);v.swap(i,(seed%(i as u64+1)) as usize);
    }
    v.truncate(n);v
}
#[test]
fn untutored_four_and_five_action_skills_are_acquired_from_real_history(){
    let mut cases=Vec::new();
    let mut completed=[0u64;6];
    let mut selected=[0u64;6];
    let mut successes=[0usize;2];
    let mut actions=[0usize;2];
    let mut veto=0usize;
    for family in 0..4{
        let number=if family<2{12}else{6};
        for order in roles(family,number){
            let old=paired_lifetime(family,order,false);
            let new=paired_lifetime(family,order,true);
            for (arm,row) in [old.clone(),new.clone()].iter().enumerate(){
                successes[arm]+=row["goals"].as_u64().unwrap() as usize;
                actions[arm]+=row["actions"].as_u64().unwrap() as usize;
            }
            for depth in 2..=5{
                completed[depth]+=new["completed_depths"][depth].as_u64().unwrap();
                selected[depth]+=new["selected_depths"][depth].as_u64().unwrap();
            }
            veto+=new["veto_safety_verified"].as_u64().unwrap() as usize;
            cases.push(json!({"family":family,"roles":order,
                "fixed_three_only":old,"variable_depth":new}));
        }
    }
    let outcome=json!({"cold_tutor_actions":0,"worlds":cases.len(),
        "old_successes":successes[0],"new_successes":successes[1],
        "old_actions":actions[0],"new_actions":actions[1],
        "variable_depth_selected":selected,
        "variable_depth_completed":completed,
        "physical_veto_checkpoint_controls":veto,
        "independent_qualification":false});
    std::fs::create_dir_all("variable-depth-evidence").unwrap();
    std::fs::write("variable-depth-evidence/all_real_traces.json",
        serde_json::to_vec_pretty(&json!({"summary":outcome,"cases":cases}))
            .unwrap()).unwrap();
    eprintln!("VARIABLE_DEPTH_OPTION0 {}",outcome);
    assert!(completed[4]>0 && completed[5]>0,
        "both 4-step and 5-step options must be SELF-SELECTED and COMPLETE under actual protected POST");
    assert!(veto>0,"at least one saved long option must fail closed on fresh HumanProtection veto");
}
#[test]
fn cannot_turn_on_depth_search_after_seeing_motor_outcomes(){
    let mut rt=fresh(false);
    let mut world=World::new(0,[0,1,2,3]);
    rt.step_constructing(Some(&[Some(1)]),&[OFFER;4],safe,|a|{
        world.act(a);
        Ok(UnifiedPost{observation:vec![world.progress],
            cost:1,reward:0.0})
    }).unwrap();
    assert!(rt.enable_constructing_variable_depth_options().is_err());
}
