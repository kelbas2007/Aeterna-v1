//! Development-only integration: the existing Learner's factual exploration
//! joins the SAME U1 arbitration / single-use protection boundary. This is not
//! independent general intelligence, cross-domain transfer or a blind benchmark.
use aeterna_v1::{HumanProtectionEvidence};
use aeterna_v1::construction::{Config};
use aeterna_v1::construction::autonomy::{ActionOffer,AutonomyPolicy};
use aeterna_v1::scientific_runtime::{ScientificRuntime,UnifiedPost};

fn safe(_:usize)->HumanProtectionEvidence {
    HumanProtectionEvidence{human_present:false,physical_effect_possible:false,predicted_harm_probability:0.0,hazard_confidence:1.0,emergency_stop:false}
}
fn make(enabled:bool)->ScientificRuntime {
    let cfg=Config{width:3,actions:4,minimum_training:128,search_every:4096,
        planning_depth:4,planning_work:4000,..Default::default()};
    let mut rt=ScientificRuntime::new_constructing(cfg,AutonomyPolicy::default(),vec![0;3],"unscheduled-world").unwrap();
    if enabled{rt.enable_constructing_self_directed_control().unwrap();}
    rt
}
fn world_step(world:&mut [i64;3],a:usize,roles:[usize;4]) {
    if a==roles[0]{world[1]=1}
    else if a==roles[1]{if world[1]==1{world[2]=1}}
    else if a==roles[2]{if world[2]==1{world[0]+=1}}
    else if a==roles[3]{world[2]=0}
}
fn offers()->Vec<ActionOffer>{vec![ActionOffer{available:true,max_cost:1};4]}
#[test]
fn unscheduled_probe_then_exploit_in_joint_u1_and_restore() {
    let roles=[2,0,3,1];
    let mut rt=make(true);let mut world=[0i64;3];
    let goal=[Some(3),None,None];let mut sources=std::collections::BTreeSet::new();
    for _ in 0..112 {
        if world[0]>=3{break}
        let step=rt.step_constructing(Some(&goal),&offers(),safe,|a|{
            world_step(&mut world,a,roles);
            Ok(UnifiedPost{observation:world.to_vec(),cost:1,reward:0.0})
        }).unwrap();
        sources.insert(step.source);
    }
    eprintln!("U1_UNSCHEDULED world={world:?} events={} sources={sources:?}",rt.construction_learner().unwrap().records().len());
    assert!(world[0]>=3,"goal not reached without an action script");
    assert!(!sources.contains("factual-probe"),
        "unverified novelty must abstain instead of overriding U1");
    let checkpoint=rt.checkpoint_constructing().unwrap();
    let mut restored=ScientificRuntime::from_constructing_checkpoint(&checkpoint).unwrap();
    assert!(restored.construction_learner().unwrap().self_directed_control_enabled());
    assert_eq!(restored.checkpoint_constructing().unwrap(),checkpoint);
    let mut more=0;
    for _ in 0..36 {
        if world[0]>=6 {break}
        restored.step_constructing(Some(&[Some(6),None,None]),&offers(),safe,|a|{
            world_step(&mut world,a,roles);
            Ok(UnifiedPost{observation:world.to_vec(),cost:1,reward:0.0})
        }).unwrap();more+=1;
    }
    eprintln!("U1_UNSCHEDULED_TRANSFER world={world:?} extra={more}");
    assert!(world[0]>=6,"failed to reuse physical consequences from prior goal after checkpoint restore");
}
#[test]
fn physical_roles_remain_opaque_under_joint_arbitration() {
    // Paired experiment: a new factual option may ABSTAIN when unsupported.
    // In that case enabling factual mode must not silently worsen physical
    // action choice relative to the original U1 on the same opaque roles.
    let mut all=0;
    let mut solved=[0usize;2];
    let mut experience_choices=[0usize;2];
    let mut actions_total=[0usize;2];
    for a in 0..4 {for b in 0..4 {for c in 0..4 {for d in 0..4 {
        let roles=[a,b,c,d];
        if roles.iter().copied().collect::<std::collections::BTreeSet<_>>().len()!=4 {continue}
        all+=1;
        for (arm,enabled) in [(0,true),(1,false)] {
            let mut rt=make(enabled);
            let mut world=[0i64;3];let mut actions=0;
            while actions<96 && world[0]<3 {
                let step=rt.step_constructing(Some(&[Some(3),None,None]),&offers(),safe,|action|{
                    world_step(&mut world,action,roles);
                    Ok(UnifiedPost{observation:world.to_vec(),cost:1,reward:0.0})
                }).unwrap();
                experience_choices[arm]+=usize::from(step.source=="factual-experience"||step.source=="factual-probe");
                actions+=1;
            }
            solved[arm]+=usize::from(world[0]>=3);actions_total[arm]+=actions;
        }
    }}}}
    eprintln!("JOINT_OPAQUE_ROLE_PARITY roles={all} enabled={}/24 choices={} actions={} baseline={}/24 choices={} actions={}",
        solved[0],experience_choices[0],actions_total[0],solved[1],experience_choices[1],actions_total[1]);
    assert_eq!(all,24);
    assert_eq!(experience_choices,[0,0],"conjunction is not verified, so must abstain");
    assert_eq!(solved[0],solved[1],"unsupported factual prediction cannot worsen goal reachability");
    assert_eq!(actions_total[0],actions_total[1],"unsupported factual prediction cannot perturb U1 actions");
}

#[test]
fn unknown_outcome_cannot_compete_with_real_u1_proposals() {
    let mut rt=make(true);
    let mut world=[0i64;3];
    let step=rt.step_constructing(Some(&[Some(3),None,None]),&offers(),safe,|a|{
        world_step(&mut world,a,[2,0,3,1]);
        Ok(UnifiedPost{observation:world.to_vec(),cost:1,reward:0.0})
    }).unwrap();
    assert_ne!(step.source,"factual-experience",
        "an action with zero verified transitions must not masquerade as factual knowledge");
    assert_eq!(rt.construction_learner().unwrap().records().len(),1);
}

#[test]
fn protection_gate_still_owns_all_action_authority() {
    use aeterna_v1::construction::vm::Error;
    let mut rt=make(true);
    let initial=rt.construction_learner().unwrap().current().to_vec();
    let denied=rt.step_constructing(Some(&[Some(3),None,None]),&offers(),|_|{
        HumanProtectionEvidence{human_present:true,physical_effect_possible:true,
            predicted_harm_probability:1.0,hazard_confidence:1.0,emergency_stop:false}
    },|_|panic!("external callback must not be executed after protection denies"));
    assert!(matches!(denied,Err(Error::Blocked)));
    assert_eq!(rt.construction_learner().unwrap().current(),initial);
    assert_eq!(rt.construction_learner().unwrap().records().len(),0);
    assert_eq!(rt.unified_report().unwrap().total_external_cost,0);
}

#[test]
fn accomplished_goal_cannot_trigger_unnecessary_factual_action() {
    use aeterna_v1::construction::vm::Error;
    let mut rt=make(true);
    let stop=rt.step_constructing(Some(&[Some(0),None,None]),&offers(),safe,
        |_|panic!("no action after known goal already satisfied"));
    assert!(matches!(stop,Err(Error::NoEvidence)));
    assert_eq!(rt.construction_learner().unwrap().records().len(),0);
}
#[test]
fn fixed_budget_ablation_reports_both_arms_without_relabeling() {
    let mut counts=[0usize;2];let mut steps=[0usize;2];
    let mut factual=[0usize;2];
    for a in 0..4 {for b in 0..4 {for c in 0..4 {for d in 0..4 {
        let roles=[a,b,c,d];
        if roles.iter().copied().collect::<std::collections::BTreeSet<_>>().len()!=4 {continue}
        for (arm,enabled) in [(0,true),(1,false)] {
            let mut rt=make(enabled);
            let mut world=[0i64;3];let mut count=0;
            for _ in 0..32 {
                if world[0]>=3{break}
                let out=rt.step_constructing(Some(&[Some(3),None,None]),&offers(),safe,|action|{
                    world_step(&mut world,action,roles);
                    Ok(UnifiedPost{observation:world.to_vec(),cost:1,reward:0.0})
                }).unwrap();
                factual[arm]+=usize::from(out.source=="factual-experience"||out.source=="factual-probe");
                count+=1;
            }
            counts[arm]+=usize::from(world[0]>=3);steps[arm]+=count;
        }
    }}}}
    eprintln!("UNSCHEDULED_ABLATION_32 with={}/24 steps={} factual={} without={}/24 steps={} factual={}",counts[0],steps[0],factual[0],counts[1],steps[1],factual[1]);
    assert_eq!(factual[1],0,"default U1 must not pretend to have used the new factual path");
    // Without a validated factual rule, enabling factual inference must not
    // change the physical choices made by the ordinary U1 controller.
    assert_eq!(factual[0],0,"conjunctive effect is not prospectively validated");
    assert_eq!(counts[0],counts[1],"abstaining cannot worsen U1 success");
    assert_eq!(steps[0],steps[1],"abstaining cannot disturb baseline decisions");
}

#[test]
fn partially_satisfied_goal_stops_direct_learner_and_protected_executor() {
    use aeterna_v1::{EvoPhase,EvoConfig};
    use aeterna_v1::construction::runtime::{Learner,ProtectedConstructionRuntime};
    use aeterna_v1::construction::vm::Error;
    let cfg=Config{width:3,actions:4,minimum_training:128,search_every:4096,
        planning_work:100,..Default::default()};
    let mut learner=Learner::new(cfg.clone(),vec![0,0,0]).unwrap();
    learner.enable_autonomy(AutonomyPolicy::default()).unwrap();
    learner.enable_self_directed_control().unwrap();
    assert!(matches!(learner.choose_masked(Some(&[Some(0),None,None])),Err(Error::NoEvidence)));
    let organism=EvoPhase::new(EvoConfig{sensory_cells:3,motor_cells:4,..Default::default()});
    let mut runtime=ProtectedConstructionRuntime::new(organism,cfg,vec![0;3]).unwrap();
    runtime.enable_autonomy(AutonomyPolicy::default()).unwrap();
    runtime.enable_self_directed_control().unwrap();
    let denied=runtime.step_autonomous(Some(&[Some(0),None,None]),&offers(),safe,
        |_|panic!("external action must not occur when masked goal is satisfied"));
    assert!(matches!(denied,Err(Error::NoEvidence)));
}

#[test]
fn jsonl_multi_domain_keeps_small_sensor_contract() {
    use std::io::Write;
    use std::process::{Command,Stdio};
    let mut proc=Command::new(env!("CARGO_BIN_EXE_aeterna-v1"))
        .arg("--jsonl").stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .spawn().unwrap();
    let input=concat!("{\"op\":\"init\",\"width\":3,\"observation\":[0,0,0]}\n",
        "{\"op\":\"enter\",\"name\":\"other-sensor-world\",\"observation\":[0,0,0]}\n",
        "{\"op\":\"quit\"}\n");
    proc.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let output=proc.wait_with_output().unwrap();
    assert!(output.status.success(),"process error: {}",String::from_utf8_lossy(&output.stderr));
    let responses=String::from_utf8(output.stdout).unwrap();
    let rows=responses.lines().map(|x|serde_json::from_str::<serde_json::Value>(x).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(rows.len(),3);
    assert_eq!(rows[0]["type"],"initialized");
    assert_eq!(rows[1]["type"],"entered");
    assert_eq!(rows[1]["report"]["domain"],"other-sensor-world");
    assert_eq!(rows[2]["type"],"bye");
}
