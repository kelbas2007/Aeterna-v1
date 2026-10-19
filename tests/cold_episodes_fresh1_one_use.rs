//! COLD-EPISODES-FRESH1: preregistered ONE-USE independent-within-synthetic
//! causal-family challenge; the protocol predates this source:
//! docs/COLD_EPISODES_FRESH1_PROTOCOL.md (098351c1).
//! Unlike COLD-EPISODES1, these are genuinely different, source-DISCLOSED
//! evaluator-owned mechanics. They are NOT an external AGI qualification.
//!
//! No tutor chooses an action; all four motors are always offered. The only
//! external reset is a truthful sensor observation between goal episodes.
//! Every motor consequence enters the same protected ScientificRuntime journal.
use aeterna_v1::HumanProtectionEvidence;
use aeterna_v1::construction::Config;
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost};
use aeterna_v1::construction::vm::fingerprint;
use serde_json::{json,Value};

const EPISODES:usize=24;
const HORIZON:usize=18;
const OFFER:ActionOffer=ActionOffer{available:true,max_cost:1};

fn safe(_:usize)->HumanProtectionEvidence{
    HumanProtectionEvidence{
        human_present:false,physical_effect_possible:false,
        predicted_harm_probability:0.0,hazard_confidence:1.0,
        emergency_stop:false
    }
}
fn cold(active:bool)->ScientificRuntime{
    let config=Config{
        width:1,actions:4,minimum_training:128,search_every:4096,
        planning_depth:3,planning_work:80_000,journal_capacity:2048,
        ..Default::default()
    };
    let mut rt=ScientificRuntime::new_constructing(
        config,AutonomyPolicy::default(),vec![0],"fresh-cold-lifetime").unwrap();
    rt.enable_constructing_agency_aware_credit().unwrap();
    rt.enable_constructing_self_directed_control().unwrap();
    if active{
        rt.enable_constructing_predictive_history_states().unwrap();
        rt.enable_constructing_active_causal_probes().unwrap();
    }
    assert!(rt.construction_learner().unwrap().records().is_empty());
    assert!(rt.unified_report().unwrap().native_weights.iter().all(|&w|w==0.0));
    rt
}
fn mix(mut x:u64)->u64{
    x=x.wrapping_add(0x9e3779b97f4a7c15);
    x=(x^(x>>30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x=(x^(x>>27)).wrapping_mul(0x94d049bb133111eb);
    x^(x>>31)
}
fn role_maps(seed:u64,family:usize)->Vec<[usize;4]>{
    // Enumerate all twenty-four motor permutations, then seeded shuffle;
    // uniqueness of all eight test mappings is guaranteed, not guessed.
    let mut maps=Vec::new();
    for a in 0..4{for b in 0..4{
        if a==b{continue}
        for c in 0..4{
            if c==a||c==b{continue}
            let d=(0..4).find(|&x|x!=a&&x!=b&&x!=c).unwrap();
            maps.push([a,b,c,d]);
        }
    }}
    assert_eq!(maps.len(),24);
    let mut key=mix(seed^((family as u64+1).wrapping_mul(0xa24baed4963ee407)));
    for index in (1..maps.len()).rev(){
        key=mix(key);
        maps.swap(index,(key%((index+1) as u64)) as usize);
    }
    maps.truncate(8);
    maps
}
#[derive(Clone)]
struct CausalWorld {
    family:usize,roles:[usize;4],progress:i64,
    mode:bool,stage:u8,window:u8,bridged:bool,
}
impl CausalWorld{
    fn new(family:usize,roles:[usize;4])->Self{
        Self{family,roles,progress:0,mode:false,
             stage:0,window:0,bridged:false}
    }
    fn act(&mut self,action:usize){
        let role=self.roles.iter().position(|&id|id==action)
            .expect("all motors opaque, exactly four");
        if self.family==2 && role!=0 {
            // Hidden step-based expiration; this is not a visible sensor.
            self.window=self.window.saturating_sub(1);
        }
        match (self.family,role){
            // Two competing hidden mode setters. The chosen use motor
            // succeeds only when the most recent mode setter was positive.
            (0,0)=>self.mode=true,
            (0,1)=>self.mode=false,
            (0,2)=>{
                if self.mode{self.progress+=1}
                self.mode=false;
            },
            (0,3)=>self.mode=false,
            // Ordered independent preparations; a wrong intervening action
            // destroys stage state. No one-step history is sufficient.
            (1,0)=>self.stage=1,
            (1,1)=>self.stage=if self.stage==1{2}else{0},
            (1,2)=>{
                if self.stage==2{self.progress+=1}
                self.stage=0;
            },
            (1,3)=>self.stage=0,
            // A finite-lived hidden arm needs an intermediate refresh.
            // On an unbridged arm, an immediate use does not succeed.
            (2,0)=>{self.window=2;self.bridged=false},
            (2,1)=>{
                if self.window>0 {self.bridged=true;self.window=2}
                else{self.bridged=false}
            },
            (2,2)=>{
                if self.window>0&&self.bridged{self.progress+=1}
                self.window=0;self.bridged=false;
            },
            (2,3)=>{self.window=0;self.bridged=false},
            // Negative control: no hidden prerequisite; an unconditional
            // motor changes progress on each actual execution.
            (3,0)|(3,1)=>{},
            (3,2)=>self.progress+=1,
            (3,3)=>self.progress=0,
            _=>unreachable!()
        }
    }
}
fn lifetime(family:usize,roles:[usize;4],candidate:bool)->Value{
    let mut rt=cold(candidate);
    let mut episodes=Vec::new();
    let mut successes=0usize;let mut actions=0usize;
    let mut first_six=0usize;let mut last_six=0usize;
    let mut selected_history=0usize;let mut selected_probes=0usize;
    for episode in 0..EPISODES{
        let mut world=CausalWorld::new(family,roles);
        if episode>0 {
            let old_events=rt.construction_learner().unwrap().records().len();
            let old_cost=rt.unified_report().unwrap().total_external_cost;
            rt.observe_constructing_external_restart(vec![world.progress]).unwrap();
            assert_eq!(old_events,rt.construction_learner().unwrap().records().len(),
                "environment observation cannot invent physical actions");
            assert_eq!(old_cost,rt.unified_report().unwrap().total_external_cost,
                "world reset must not mint action credit");
        }
        let mut action_trace=Vec::new();
        for _ in 0..HORIZON {
            if world.progress>=1{break}
            let chosen=rt.step_constructing(
                Some(&[Some(1)]),&[OFFER;4],safe,
                |motor|{
                    world.act(motor);
                    Ok(UnifiedPost{
                        observation:vec![world.progress],
                        cost:1,reward:0.0
                    })
                }).unwrap();
            actions+=1;
            selected_history+=usize::from(chosen.source=="predictive-history");
            selected_probes+=usize::from(chosen.source=="causal-pair-probe");
            action_trace.push(json!({
                "motor":chosen.action,
                "source":chosen.source,
                "physical_post":world.progress,
                "history_offered":chosen.predictive_history_plan_proposed,
                "probe_offered":chosen.active_pair_probe_proposed,
                "actual_external_cost":1,
            }));
        }
        let solved=world.progress>=1;
        successes+=usize::from(solved);
        if episode<6{first_six+=usize::from(solved)}
        if episode>=EPISODES-6{last_six+=usize::from(solved)}
        episodes.push(json!({
            "episode":episode,"solved":solved,
            "action_count":action_trace.len(),
            "action_trace":action_trace
        }));
    }
    let report=rt.unified_report().unwrap();
    let agency=report.agency_credit.as_ref().expect("agency opt-in missing");
    assert_eq!(agency.externally_forced_actions,0,
        "NO experimenter motor choices allowed in acquisition");
    assert_eq!(agency.self_selected_actions,actions as u64);
    let records=rt.construction_learner().unwrap().records();
    assert_eq!(records.len(),actions,"journal must contain every real POST");
    assert_eq!(records.iter().filter(|r|r.transition.restart).count(),EPISODES,
        "start each physical episode with truthful causal-history separator");
    assert!(records.windows(2).all(|pair|
        pair[0].transition.id.checked_add(1)==Some(pair[1].transition.id)));
    let cp=rt.checkpoint_constructing().unwrap();
    assert_eq!(ScientificRuntime::from_constructing_checkpoint(&cp)
        .unwrap().checkpoint_constructing().unwrap(),cp);
    assert_eq!(report.total_external_cost,actions as u64);
    json!({
        "family":family,"motor_roles":roles,
        "candidate":candidate,"successes":successes,"actions":actions,
        "cost":report.total_external_cost,
        "first_six_success":first_six,"last_six_success":last_six,
        "selected_history":selected_history,"selected_probe":selected_probes,
        "native_u1_weights":report.native_weights,
        "journal_events":records.len(),"verified_world_restarts":EPISODES-1,
        "checkpoint_fingerprint_diagnostic":fingerprint(&cp),
        "episode_traces":episodes
    })
}
#[test]
#[ignore="ONE-USE preregistered non-selectable run-ID cohort; source-disclosed synthetic only"]
fn cold_episodes_fresh1_one_use_qualification(){
    let seed=std::env::var("GITHUB_RUN_ID").expect("GitHub run ID required")
        .parse::<u64>().expect("numeric one-use seed");
    let sha=std::env::var("GITHUB_SHA").expect("exact science merge SHA");
    let mut cases=Vec::new();let mut family_summaries=Vec::new();
    let mut total_success=[0usize;2];let mut total_actions=[0usize;2];
    let mut total_cost=[0u64;2];let mut total_history=0usize;
    let mut total_probes=0usize;let mut first_six=0usize;
    let mut last_six=0usize;
    for family in 0..4{
        let mappings=role_maps(seed,family);
        let mut fam_success=[0usize;2];let mut fam_actions=[0usize;2];
        let mut fam_cost=[0u64;2];let mut fam_history=0usize;
        let mut fam_probes=0usize;let mut fam_early=0usize;let mut fam_late=0usize;
        for role_map in mappings{
            let baseline=lifetime(family,role_map,false);
            let candidate=lifetime(family,role_map,true);
            for (arm,row) in [baseline.clone(),candidate.clone()].iter().enumerate(){
                let successes=row["successes"].as_u64().unwrap() as usize;
                let actions=row["actions"].as_u64().unwrap() as usize;
                let cost=row["cost"].as_u64().unwrap();
                fam_success[arm]+=successes;fam_actions[arm]+=actions;
                fam_cost[arm]+=cost;
                total_success[arm]+=successes;total_actions[arm]+=actions;
                total_cost[arm]+=cost;
            }
            fam_history+=candidate["selected_history"].as_u64().unwrap() as usize;
            fam_probes+=candidate["selected_probe"].as_u64().unwrap() as usize;
            fam_early+=candidate["first_six_success"].as_u64().unwrap() as usize;
            fam_late+=candidate["last_six_success"].as_u64().unwrap() as usize;
            cases.push(json!({"family":family,"role_map":role_map,
                "baseline":baseline,"candidate":candidate}));
        }
        total_history+=fam_history;total_probes+=fam_probes;
        first_six+=fam_early;last_six+=fam_late;
        family_summaries.push(json!({
            "family":family,"baseline_successes":fam_success[0],
            "candidate_successes":fam_success[1],
            "baseline_actions":fam_actions[0],
            "candidate_actions":fam_actions[1],
            "baseline_cost":fam_cost[0],"candidate_cost":fam_cost[1],
            "candidate_history_selected":fam_history,
            "candidate_probes_selected":fam_probes,
            "candidate_early_six_success":fam_early,
            "candidate_late_six_success":fam_late
        }));
    }
    let per_family_no_regression=family_summaries.iter().all(|f|
        f["candidate_successes"].as_u64().unwrap()>=
        f["baseline_successes"].as_u64().unwrap());
    let qualified=per_family_no_regression &&
        total_success[1]>=total_success[0] &&
        total_actions[1].saturating_mul(10)<=
            total_actions[0].saturating_mul(9) &&
        total_cost[1]<=total_cost[0] &&
        total_history>0 && total_probes>0 && last_six>first_six;
    let verdict=if qualified{"QUALIFIED_SYNTHETIC_COLD_TRANSFER"}
        else{"NEGATIVE_COLD_TRANSFER"};
    let summary=json!({
        "verdict":verdict,"qualified":qualified,
        "seed":seed,"sha":sha,
        "source_disclosed":true,"pre_registered_protocol":
            "docs/COLD_EPISODES_FRESH1_PROTOCOL.md",
        "case_count":cases.len(),
        "episodes_per_case":EPISODES,
        "actions_per_episode_limit":HORIZON,
        "all_motor_offers_available":true,
        "tutored_motor_actions":0,
        "baseline_successes":total_success[0],
        "candidate_successes":total_success[1],
        "baseline_actions":total_actions[0],
        "candidate_actions":total_actions[1],
        "baseline_cost":total_cost[0],
        "candidate_cost":total_cost[1],
        "selected_history":total_history,
        "selected_probes":total_probes,
        "candidate_first_six":first_six,
        "candidate_last_six":last_six,
        "no_family_regression":per_family_no_regression,
        "families":family_summaries
    });
    std::fs::create_dir_all("cold-fresh-evidence").unwrap();
    let all=json!({"summary":summary,"cases":cases});
    std::fs::write("cold-fresh-evidence/all_action_and_episode_traces.json",
        serde_json::to_vec_pretty(&all).unwrap()).unwrap();
    eprintln!("COLD_EPISODES_FRESH1_RESULT {}",summary);
    assert!(qualified,"one-use preregistered cold causal transfer did not qualify");
}
