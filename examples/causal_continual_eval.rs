//! Precommitted, disclosed engineering comparison. World labels stay here;
//! the learner receives only opaque observations, goals, offers and real POST.
use aeterna_v1::construction::autonomy::{ActionOffer, AutonomyPolicy};
use aeterna_v1::construction::causal_memory::CausalConfig;
use aeterna_v1::construction::Config;
use aeterna_v1::scientific_runtime::{ScientificRuntime, UnifiedPost};
use aeterna_v1::HumanProtectionEvidence;
use serde_json::{json, Value};

fn mix(mut n: u64) -> u64 {
    n = n.wrapping_add(0x9e3779b97f4a7c15);
    n = (n ^ (n >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    n = (n ^ (n >> 27)).wrapping_mul(0x94d049bb133111eb);
    n ^ (n >> 31)
}
fn permutation(seed: u64) -> [usize; 4] {
    let mut p = [0, 1, 2, 3];
    for i in (1..4).rev() {
        p.swap(i, (mix(seed + i as u64) % (i + 1) as u64) as usize);
    }
    p
}
fn observation(seed: u64, stage: usize, width: usize) -> Vec<i64> {
    let mut v = vec![0; width];
    // Unique opaque codes, with no numerical distance indicating progress.
    v[0] = (mix(seed + stage as u64 * 113) % 1_000_000) as i64;
    for (i, x) in v.iter_mut().enumerate().skip(1) {
        *x = if width > 2 {
            (stage * (i + 3) % 17) as i64
        } else {
            0
        };
    }
    v
}
fn safe(_: usize) -> HumanProtectionEvidence {
    HumanProtectionEvidence {
        human_present: false,
        physical_effect_possible: false,
        predicted_harm_probability: 0.0,
        hazard_confidence: 1.0,
        emergency_stop: false,
    }
}
fn life(seed: u64, family: &str, arm: &str) -> Value {
    let long = family == "ten_step";
    let direct = family == "direct_goal";
    let noise_control = family == "uninformative_sensor";
    let width = if long { 12 } else { 2 };
    let config = Config {
        width,
        actions: 4,
        planning_depth: if arm == "horizon_one" { 1 } else { 12 },
        planning_work: 120_000,
        minimum_training: 128,
        search_every: 1_000_000,
        journal_capacity: 4096,
        ..Default::default()
    };
    let policy = AutonomyPolicy {
        max_external_cost: 100_000,
        ..Default::default()
    };
    let mut rt = ScientificRuntime::new_constructing(
        config,
        policy,
        observation(seed, 0, width),
        "causal4-held-engineering",
    )
    .unwrap();
    rt.enable_constructing_agency_aware_credit().unwrap();
    rt.enable_constructing_self_directed_control().unwrap();
    if arm != "legacy" {
        rt.enable_constructing_causal_memory(CausalConfig {
            history_depth: 3,
            ..Default::default()
        })
        .unwrap();
    }
    let p = permutation(seed);
    let offers = vec![
        ActionOffer {
            available: true,
            max_cost: 1
        };
        4
    ];
    let target = if long { 10 } else { 5 };
    let mut goal = vec![None; width];
    goal[0] = Some(observation(seed, target, width)[0]);
    let training = if long {
        260
    } else if direct {
        128
    } else {
        600
    };
    let held = if long { 1 } else { 64 };
    let mut trace = Vec::new();
    let mut held_goals = 0;
    let mut held_actions = 0;
    let mut held_queries = 0;
    let mut trained_fingerprint = None;
    for ep in 0..training + held {
        let learning = ep < training;
        if !learning && ep == training {
            rt.freeze_constructing().unwrap();
            let bytes = rt.checkpoint_constructing().unwrap();
            rt = ScientificRuntime::from_constructing_checkpoint(&bytes).unwrap();
            trained_fingerprint = rt
                .construction_learner()
                .unwrap()
                .causal_report()
                .map(|r| r.learned_fingerprint);
        }
        let hidden = (mix(seed * 10_003 + ep as u64 + 0x4500) & 1) as usize;
        let mut state = if long && learning {
            (ep * 7 + 3) % 10
        } else {
            0
        };
        rt.observe_constructing_external_restart(observation(seed, state, width))
            .unwrap();
        let limit = if long {
            if learning {
                9
            } else {
                36
            }
        } else if direct {
            1
        } else {
            4
        };
        for draw in 0..limit {
            if state == target {
                break;
            }
            let mut terminal = false;
            let pre = observation(seed, state, width);
            rt.step_constructing(Some(&goal), &offers, safe, |action| {
                let role = p.iter().position(|&a| a == action).unwrap();
                let random = mix(seed * 31_337 + ep as u64 * 101 + draw as u64 + 0x9321);
                let mut success = false;
                if long {
                    if role == (state * 7 + (seed % 4) as usize) % 4 {
                        state += 1;
                    } else {
                        state = 0;
                    }
                    success = state == target;
                } else if direct {
                    terminal = true;
                    success = role == 1;
                    state = if success { target } else { 6 };
                } else if role == 0 {
                    if !learning {
                        held_queries += 1;
                    }
                    let cue = if noise_control {
                        (random & 1) as usize
                    } else {
                        hidden ^ usize::from(random % 10 == 0)
                    };
                    state = 1 + cue;
                } else if role == 2 {
                    state = 3 + (random & 1) as usize;
                } else {
                    terminal = true;
                    success = (role == 1 && hidden == 0) || (role == 3 && hidden == 1);
                    state = if success { target } else { 6 };
                }
                if !learning {
                    held_actions += 1;
                    if success {
                        held_goals += 1;
                    }
                }
                let post = observation(seed, state, width);
                trace.push(json!([learning, ep, pre, action, post, 1, success]));
                Ok(UnifiedPost {
                    observation: observation(seed, state, width),
                    cost: 1,
                    reward: f32::from(success),
                })
            })
            .unwrap();
            if terminal {
                break;
            }
        }
    }
    let report = rt.construction_learner().unwrap().causal_report();
    assert_eq!(
        trained_fingerprint,
        report.as_ref().map(|r| r.learned_fingerprint)
    );
    assert_eq!(
        rt.construction_learner().unwrap().factual_event_count(),
        trace.len()
    );
    assert_eq!(
        rt.unified_report().unwrap().total_external_cost,
        trace.len() as u64
    );
    json!({"seed":seed,"family":family,"arm":arm,"training_episodes":training,
        "held_episodes":held,"held_goals":held_goals,"held_actions":held_actions,
        "held_queries":held_queries,"motor_permutation":p,"causal_report":report,
        "trace_columns":["learning","episode","pre","action","post","cost","goal"],
        "trace":trace})
}
fn main() {
    for seed in 230000..230032 {
        for family in [
            "ten_step",
            "noisy_sensor",
            "uninformative_sensor",
            "direct_goal",
        ] {
            for arm in ["legacy", "causal", "horizon_one"] {
                let result = life(seed, family, arm);
                eprintln!(
                    "CAUSAL4 {seed} {family} {arm} {}/{} goals {} actions",
                    result["held_goals"], result["held_episodes"], result["held_actions"]
                );
                println!("{result}");
            }
        }
    }
}
