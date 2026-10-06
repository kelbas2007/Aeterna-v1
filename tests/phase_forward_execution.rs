//! P2 evaluator: raw factual tuples only cross the production boundary.
use aeterna_v1::{Authority, EvoConfig, EvoPhase, RasterFieldConfig};
use aeterna_v1::carrier::PhaseNativeConfig;

const REPETITIONS: usize = 160;
const REVISION: usize = 128;
const PERMUTATIONS: [[usize; 3]; 6] = [
    [0,1,2], [0,2,1], [1,0,2], [1,2,0], [2,0,1], [2,1,0],
];

#[derive(Debug, Clone)]
struct Case { length: usize, actions: [usize; 3], x: usize, y: usize, offset: usize }

fn raster(state: usize, x: usize, y: usize) -> Vec<f32> {
    let deltas = [(1,0), (2,0), (3,0), (4,0), (0,1),
                  (0,2), (0,3), (1,1), (1,2), (2,1)];
    let (dx, dy) = deltas[state];
    assert!(x + dx < 12 && y + dy < 12);
    let mut r = vec![0.0; 144];
    r[y * 12 + x] = 1.0;
    r[(y + dy) * 12 + x + dx] = 1.0;
    r
}
fn route_action(w: &Case, stage: usize) -> usize {
    if stage == 0 { w.actions[1] } else { w.actions[(stage + 1) % 3] }
}
fn actions(w: &Case) -> Vec<usize> { (0..w.length).map(|i| route_action(w,i)).collect() }
fn facts(w: &Case) -> Vec<(usize,usize,usize,f32)> {
    let mut rows = vec![(0,w.actions[0],w.length+1,0.25)];
    for s in 0..w.length {
        rows.push((s,route_action(w,s),s+1, if s+1 == w.length { 1.0 } else { 0.0 }));
    }
    rows.push((w.length+2,w.actions[2],w.length+3,0.4));
    let n = rows.len(); rows.rotate_left(w.offset % n); rows
}
fn carrier(mode: usize) -> EvoPhase {
    let mut cfg = EvoConfig {
        sensory_cells:144, motor_cells:3, dormant_cells:64, hdc_dim:192,
        ..EvoConfig::default()
    };
    match mode {
        1 => cfg.phase_learning_rate = 0.0,
        2 => cfg.weight_learning_rate = 0.0,
        3 => cfg.dormant_cells = 0,
        _ => {},
    }
    let mut evo = EvoPhase::new(cfg);
    let mut field = RasterFieldConfig::for_raster(12,12,3);
    field.learning_enabled = false; field.readout_enabled = false;
    evo.attach_raster_field(field);
    evo.enable_phase_native_planning(PhaseNativeConfig::default()); evo
}
fn acquire(evo: &mut EvoPhase, w: &Case) -> usize {
    let rows = facts(w);
    for _ in 0..REPETITIONS {
        for &(from,action,to,value) in &rows {
            evo.observe_phase_native_forward_transition(
                &raster(from,0,0),action,&raster(to,0,0),value);
        }
    }
    evo.set_planning_learning_enabled(false);
    rows.len() * REPETITIONS
}
fn similarity(evo: &EvoPhase, predicted: &[f32], actual: &[f32]) -> f32 {
    let field = evo.raster_field().unwrap();
    match (field.encode_relational_trace(predicted),field.encode_relational_trace(actual)) {
        (Some(a),Some(b)) => a.similarity(&b), _ => -1.0,
    }
}
fn forecast(evo: &EvoPhase, w: &Case) -> bool {
    let predictions = evo.imagine_phase_native_actions(&raster(0,w.x,w.y),&actions(w));
    predictions.len() == w.length && predictions.iter().enumerate().all(|(s,p)| {
        p.authority == Authority::Imagined
            && similarity(evo,&p.sensory,&raster(s+1,w.x,w.y)) >= 0.97
            && (p.need - if s+1 == w.length {1.0} else {0.0}).abs() < 0.02
    })
}
fn cindex(w: &Case, logical: usize) -> usize {
    let n = w.length+2; (logical+n-w.offset%n)%n
}
fn integrated(evo: &mut EvoPhase, w: &Case) -> bool {
    evo.observe_initial_real(&raster(0,w.x,w.y),false);
    let rows = facts(w);
    let mut state = 0;
    for _ in 0..w.length {
        let before = evo.current_real().unwrap().clone();
        let fingerprint = evo.phase_native_learned_fingerprint();
        let Some((decision,prediction)) = evo.choose_phase_native_action_with_prediction() else { return false; };
        assert_eq!(fingerprint,evo.phase_native_learned_fingerprint());
        assert_eq!(before.sensory,evo.current_real().unwrap().sensory);
        assert_eq!(before.tick,evo.current_real().unwrap().tick);
        // Only now, AFTER decision and prediction, the world executes the action.
        let Some(&(_,_,to,value)) = rows.iter().find(|(from,a,_,_)| {
            *from == state && *a == decision.first_action
        }) else { return false; };
        let post = raster(to,w.x,w.y);
        if similarity(evo,&prediction.sensory,&post) < 0.97 { return false; }
        let error = evo.observe_phase_native_action_result(decision.first_action,&post,value).unwrap();
        if error > 0.03 { return false; }
        state = to;
        if value >= 1.0 { return evo.current_real().unwrap().need; }
        if value > 0.0 { return false; }
    }
    false
}

#[derive(Default, Debug)]
struct ResultRow {
    full: bool, integrated: bool, lesion_lost: bool, phase_lost: bool,
    restored: bool, decoder_lost: bool, unrelated_retained: bool,
    revised: bool, frozen_obsolete: bool, unchanged_retained: bool,
    controls: [bool;3], tuition: usize,
}
fn exercise(w: &Case, with_controls: bool) -> ResultRow {
    let mut evo = carrier(0);
    let tuition = acquire(&mut evo,w);
    assert_eq!(evo.planning_transition_count(),0);
    assert_eq!(evo.imagined_rollout_nodes(),0);
    assert_eq!(evo.phase_native_circuits().len(),w.length+2);
    evo.observe_initial_real(&raster(0,w.x,w.y),false);
    let checkpoint = evo.phase_native_learned_fingerprint();
    let full = forecast(&evo,w);
    assert_eq!(checkpoint,evo.phase_native_learned_fingerprint());
    let live = integrated(&mut evo.clone(),w);

    // Internal link: first successor is still forecast correctly before failure.
    let bridge = evo.phase_native_circuits()[cindex(w,2)].successor_synapse;
    let saved = evo.perturb_phase_native_synapse_for_control(bridge,0.0,0.0).unwrap();
    let prefix = evo.imagine_phase_native_actions(&raster(0,w.x,w.y),&actions(w));
    let lesion_lost = prefix.len() == 1
        && similarity(&evo,&prefix[0].sensory,&raster(1,w.x,w.y)) >= 0.97;
    evo.restore_phase_native_synapse_for_control(bridge,saved);
    let restored1 = forecast(&evo,w);
    let saved = evo.perturb_phase_native_synapse_for_control(bridge,1.0,std::f32::consts::PI).unwrap();
    let phase_lost = evo.imagine_phase_native_actions(&raster(0,w.x,w.y),&actions(w)).len() == 1;
    evo.restore_phase_native_synapse_for_control(bridge,saved);
    let restored2 = forecast(&evo,w);
    assert_eq!(checkpoint,evo.phase_native_learned_fingerprint());

    let first_successor = evo.phase_native_synapse(
        evo.phase_native_circuits()[cindex(w,1)].successor_synapse).unwrap().to;
    let decoder: Vec<_> = evo.phase_native_decoder_synapses().into_iter().filter(|i| {
        let syn = evo.phase_native_synapse(*i).unwrap();
        syn.from == first_successor && syn.weight > 0.5
    }).collect();
    assert_eq!(decoder.len(),2,"two learned active pixel outputs, not copied rasters");
    let saved: Vec<_> = decoder.iter().map(|i| {
        (*i,evo.perturb_phase_native_synapse_for_control(*i,1.0,std::f32::consts::PI).unwrap())
    }).collect();
    let decoder_lost = evo.imagine_phase_native_actions(&raster(0,w.x,w.y),&actions(w)).is_empty();
    for (i,syn) in saved { evo.restore_phase_native_synapse_for_control(i,syn); }
    let restored3 = forecast(&evo,w);
    assert_eq!(checkpoint,evo.phase_native_learned_fingerprint());
    let unrelated = evo.phase_native_circuits()[cindex(w,w.length+1)].successor_synapse;
    let saved = evo.perturb_phase_native_synapse_for_control(unrelated,0.0,0.0).unwrap();
    let unrelated_retained = forecast(&evo,w);
    evo.restore_phase_native_synapse_for_control(unrelated,saved);

    let pre = raster(w.length-1,0,0);
    let post = raster(9,0,0);
    let changed_action = route_action(w,w.length-1);
    let old = evo.phase_native_circuits()[cindex(w,w.length)].clone();
    let mut frozen = evo.clone();
    let frozen_hash = frozen.phase_native_learned_fingerprint();
    let mut first_error = 0.0;
    evo.set_planning_learning_enabled(true);
    for i in 0..REVISION {
        let error = evo.observe_phase_native_forward_transition(&pre,changed_action,&post,1.0).unwrap();
        if i == 0 { first_error = error; }
        frozen.observe_phase_native_forward_transition(&pre,changed_action,&post,1.0);
    }
    evo.set_planning_learning_enabled(false);
    assert_eq!(frozen_hash,frozen.phase_native_learned_fingerprint());
    let now = &evo.phase_native_circuits()[cindex(w,w.length)];
    assert_eq!(old.relay_cell,now.relay_cell);
    assert_eq!(old.successor_synapse,now.successor_synapse);
    let revision_evidence = now.revision > old.revision && now.counterexamples.len() > old.counterexamples.len();
    let revised = evo.imagine_phase_native_actions(&raster(w.length-1,w.x,w.y),&[changed_action]);
    let revised = revised.len() == 1 && first_error > 0.03 && revision_evidence
        && similarity(&evo,&revised[0].sensory,&raster(9,w.x,w.y)) >= 0.97;
    let obsolete = frozen.imagine_phase_native_actions(&raster(w.length-1,w.x,w.y),&[changed_action]);
    let frozen_obsolete = obsolete.len() == 1
        && similarity(&frozen,&obsolete[0].sensory,&raster(w.length,w.x,w.y)) >= 0.97
        && similarity(&frozen,&obsolete[0].sensory,&raster(9,w.x,w.y)) < 0.97;
    let unchanged = evo.imagine_phase_native_actions(&raster(0,w.x,w.y),&[w.actions[0]]);
    let unchanged_retained = unchanged.len() == 1
        && similarity(&evo,&unchanged[0].sensory,&raster(w.length+1,w.x,w.y)) >= 0.97;
    let mut controls = [false;3];
    if with_controls {
        for mode in 1..=3 { let mut control = carrier(mode); acquire(&mut control,w); controls[mode-1] = forecast(&control,w); }
    }
    ResultRow { full, integrated:live, lesion_lost, phase_lost,
        restored:restored1 && restored2 && restored3, decoder_lost, unrelated_retained,
        revised, frozen_obsolete, unchanged_retained, controls, tuition }
}
fn accepted(r: &ResultRow) -> bool {
    r.full && r.integrated && r.lesion_lost && r.phase_lost && r.restored && r.decoder_lost
        && r.unrelated_retained && r.revised && r.frozen_obsolete && r.unchanged_retained
        && !r.controls.iter().any(|v| *v)
}

#[test]
fn p2_forward_composition_and_ordinary_action_fact_loop() {
    let mut passed = 0;
    for permutation in PERMUTATIONS {
        for length in 2..=5 {
            let w = Case { length, actions:permutation, x:4,y:5,offset:length%(length+2) };
            let result = exercise(&w,false);
            println!("P2_DEVELOPMENT case={w:?} result={result:?}");
            passed += usize::from(accepted(&result));
        }
    }
    println!("P2_MECHANISM passed={passed}/24");
    assert_eq!(passed,24);
}
#[test]
fn p2_controls_and_unknown_states_abstain() {
    let w = Case {length:3,actions:[0,1,2],x:3,y:4,offset:0};
    let r = exercise(&w,true); println!("P2_CONTROLS {r:?}"); assert!(accepted(&r));
    let mut evo = carrier(0); acquire(&mut evo,&w);
    assert!(evo.imagine_phase_native_actions(&vec![0.0;144],&[0]).is_empty());
    assert!(evo.imagine_phase_native_actions(&raster(0,4,4),&[2]).is_empty());
    assert!(evo.imagine_phase_native_actions(&raster(0,4,4),&[3]).is_empty());
    assert!(evo.imagine_phase_native_actions(&raster(0,4,4),&vec![0;7]).is_empty());
}
#[test]
fn p2_forward_kernel_has_no_future_prototype_or_search_access() {
    let source = include_str!("../src/phase_forward.rs");
    let kernel = source.split("fn native_forward_tick(").nth(1).unwrap();
    for forbidden in ["receptors", "CarrierTrace", "PlanDecision", "encode_high_level_trace", ".plan(", "Frontier", "raster("] {
        assert!(!kernel.contains(forbidden),"forward kernel leaked {forbidden}");
    }
    for required in ["syn.phase_offset", "active[syn.from].charge", "syn.weight", "atan2"] {
        assert!(kernel.contains(required),"missing physical operation {required}");
    }
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z^(z>>30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z^(z>>27)).wrapping_mul(0x94d049bb133111eb); z^(z>>31)
    }
    fn range(&mut self,n:usize)->usize {(self.next()%n as u64) as usize}
}
fn wilson(k:usize,n:usize)->(f64,f64) {
    let z=1.959963984540054_f64; let n=n as f64; let p=k as f64/n;
    let den=1.0+z*z/n; let center=(p+z*z/(2.0*n))/den;
    let half=z*(p*(1.0-p)/n+z*z/(4.0*n*n)).sqrt()/den; (center-half,center+half)
}
#[test]
#[ignore = "one-use post-freeze CI authority only; not an ordinary regression"]
fn p2_fresh_forward_and_integrated_pack() {
    let authority:u64=std::env::var("AETERNA_FRESH_SEED").unwrap().parse().unwrap();
    let source=std::env::var("AETERNA_SOURCE_SHA").unwrap();
    let spec=std::env::var("AETERNA_SPEC_SHA").unwrap();
    assert_eq!(std::env::var("GITHUB_RUN_ID").unwrap(),authority.to_string());
    assert_eq!(std::env::var("GITHUB_RUN_ATTEMPT").unwrap(),"1");
    assert_eq!(source.len(),40); assert_eq!(spec.len(),40);
    let mut pack=Vec::new(); let mut digest=14_695_981_039_346_656_037_u64;
    for sub in 0..10usize {
        let mut rng=Rng(authority ^ (sub as u64).wrapping_mul(0xd1b54a32d192ed03) ^ 0x50325f465744);
        for _ in 0..8 {
            let length=2+rng.range(4);
            let w=Case {length,actions:PERMUTATIONS[rng.range(6)],x:1+rng.range(6),y:1+rng.range(6),offset:rng.range(length+2)};
            let text=format!("{sub}:{w:?}");
            for b in text.bytes(){digest^=u64::from(b);digest=digest.wrapping_mul(1_099_511_628_211);}
            println!("P2_WORLD {text}"); pack.push((sub,w));
        }
    }
    println!("P2_SEAL source={source} spec={spec} authority={authority} digest={digest:016x} N=80 seeds=10 BEFORE_ALL_TUITION_AND_SCORING");
    let mut per_seed=[0usize;10]; let mut counts=[0usize;10]; let mut controls=[0usize;3]; let mut tuition=0;
    for (index,(sub,w)) in pack.iter().enumerate() {
        let r=exercise(w,true);
        for (i,b) in [r.full,r.integrated,r.lesion_lost,r.phase_lost,r.restored,r.decoder_lost,r.unrelated_retained,r.revised,r.frozen_obsolete,r.unchanged_retained].iter().enumerate() {counts[i]+=usize::from(*b);}
        for i in 0..3 {controls[i]+=usize::from(r.controls[i]);}
        per_seed[*sub]+=usize::from(accepted(&r)); tuition+=r.tuition;
        println!("P2_CASE {index} {r:?}");
    }
    let complete:usize=per_seed.iter().sum(); let (lo,hi)=wilson(complete,80);
    println!("P2_FRESH accepted={complete}/80 wilson95=[{lo:.6},{hi:.6}] per_seed={per_seed:?} counts_full_integrated_lesion_phase_restored_decoder_unrelated_revised_frozen_unchanged={counts:?} control_full_forecasts_zero_phase_zero_weight_no_capacity={controls:?}");
    println!("P2_COST tuition_per_arm_mean={:.3} arms_per_case=4 revision_presentations_per_revision_and_frozen_copy={} full_route_length=2..5",tuition as f64/80.0,REVISION);
    assert!(complete>=76 && lo>0.88 && per_seed.iter().all(|n|*n>=6));
    assert_eq!(controls,[0;3]);
}
