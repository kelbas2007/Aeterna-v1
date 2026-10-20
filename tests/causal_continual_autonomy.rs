//! Open development controls: real native-selected motors and factual POSTs.
use aeterna_v1::construction::autonomy::{ActionOffer, AutonomyPolicy};
use aeterna_v1::construction::causal_memory::CausalConfig;
use aeterna_v1::construction::{Config, Result};
use aeterna_v1::scientific_runtime::{ScientificRuntime, UnifiedPost};
use aeterna_v1::HumanProtectionEvidence;

fn safe(_: usize) -> HumanProtectionEvidence {
    HumanProtectionEvidence {
        human_present: false,
        physical_effect_possible: false,
        predicted_harm_probability: 0.0,
        hazard_confidence: 1.0,
        emergency_stop: false,
    }
}
fn make(
    width: usize,
    actions: usize,
    horizon: usize,
    capacity: usize,
    enabled: bool,
) -> ScientificRuntime {
    let cfg = Config {
        width,
        actions,
        planning_depth: horizon,
        planning_work: 120_000,
        minimum_training: 128,
        search_every: 1_000_000,
        journal_capacity: capacity,
        ..Default::default()
    };
    let mut policy = AutonomyPolicy::default();
    policy.max_external_cost = 1_000_000;
    let mut rt =
        ScientificRuntime::new_constructing(cfg, policy, vec![0; width], "causal-continuous")
            .unwrap();
    rt.enable_constructing_agency_aware_credit().unwrap();
    rt.enable_constructing_self_directed_control().unwrap();
    if enabled {
        rt.enable_constructing_causal_memory(CausalConfig {
            history_depth: 3,
            ..Default::default()
        })
        .unwrap();
    }
    rt
}
fn offers(n: usize) -> Vec<ActionOffer> {
    vec![
        ActionOffer {
            available: true,
            max_cost: 1
        };
        n
    ]
}

#[test]
fn causal_continuous_physical_learning_compacts_restores_and_preserves_frozen_knowledge() {
    let mut rt = make(12, 7, 12, 192, true);
    let mut physical = vec![0; 12];
    for step in 0..6200 {
        rt.step_constructing(None, &offers(7), safe, |a| {
            physical[0] = (physical[0] + 1) % 5;
            physical[11] = a as i64;
            Ok(UnifiedPost {
                observation: physical.clone(),
                reward: 0.0,
                cost: 1,
            })
        })
        .unwrap();
        if step == 3100 {
            let bytes = rt.checkpoint_constructing().unwrap();
            rt = ScientificRuntime::from_constructing_checkpoint(&bytes).unwrap();
            assert_eq!(rt.checkpoint_constructing().unwrap(), bytes);
        }
    }
    let learner = rt.construction_learner().unwrap();
    let r = learner.causal_report().unwrap();
    assert_eq!(learner.factual_event_count(), 6200);
    assert_eq!(rt.unified_report().unwrap().total_external_cost, 6200);
    assert!(learner.records().len() <= 192 && r.compactions > 20);
    assert_eq!(r.learned_transitions, 6200);
    assert!(r.retained_entries > 0 && r.model_bytes <= 2 * 1024 * 1024);
    rt.freeze_constructing().unwrap();
    let before = rt.construction_learner().unwrap().causal_report().unwrap();
    for _ in 0..240 {
        rt.step_constructing(None, &offers(7), safe, |_| {
            Ok(UnifiedPost {
                observation: physical.clone(),
                reward: 0.0,
                cost: 1,
            })
        })
        .unwrap();
    }
    let after = rt.construction_learner().unwrap().causal_report().unwrap();
    assert_eq!(before.learned_fingerprint, after.learned_fingerprint);
    assert_eq!(before.learned_transitions, after.learned_transitions);
    assert_eq!(after.factual_transitions, 6440);
    let bytes = rt.checkpoint_constructing().unwrap();
    let restored = ScientificRuntime::from_constructing_checkpoint(&bytes).unwrap();
    assert_eq!(restored.checkpoint_constructing().unwrap(), bytes);
}

fn observation(stage: usize) -> Vec<i64> {
    let mut v = vec![0; 12];
    v[0] = [107, 31, 809, 53, 211, 19, 503, 73, 401, 97, 907][stage];
    for (i, x) in v.iter_mut().enumerate().skip(1) {
        *x = (stage * (i + 3) % 17) as i64;
    }
    v
}
fn long_world(stage: &mut usize, a: usize, rotation: usize) {
    if *stage == 10 {
        return;
    }
    if a == (*stage * 7 + rotation) % 4 {
        *stage += 1
    } else {
        *stage = 0
    }
}
fn long_life(enabled: bool, rotation: usize) -> Result<(bool, usize)> {
    let mut rt = make(12, 4, 12, 4096, enabled);
    let mut goal = vec![None; 12];
    goal[0] = Some(observation(10)[0]);
    for ep in 0..260 {
        let mut state = (ep * 7 + 3) % 10;
        rt.observe_constructing_external_restart(observation(state))?;
        for _ in 0..9 {
            if state == 10 {
                break;
            }
            rt.step_constructing(Some(&goal), &offers(4), safe, |a| {
                long_world(&mut state, a, rotation);
                Ok(UnifiedPost {
                    observation: observation(state),
                    cost: 1,
                    reward: f32::from(state == 10),
                })
            })?;
        }
    }
    rt.freeze_constructing()?;
    rt = ScientificRuntime::from_constructing_checkpoint(&rt.checkpoint_constructing()?)?;
    let mut state = 0;
    let mut cost = 0;
    rt.observe_constructing_external_restart(observation(state))?;
    while state < 10 && cost < 36 {
        rt.step_constructing(Some(&goal), &offers(4), safe, |a| {
            long_world(&mut state, a, rotation);
            cost += 1;
            Ok(UnifiedPost {
                observation: observation(state),
                cost: 1,
                reward: f32::from(state == 10),
            })
        })?;
    }
    Ok((state == 10, cost))
}
#[test]
fn causal_native_reuses_a_ten_motor_path_with_twelve_sensors_after_own_factual_acquisition() {
    let new = long_life(true, 3).unwrap();
    let baseline = long_life(false, 3).unwrap();
    eprintln!("CAUSAL_LONG_PATH new={new:?} baseline={baseline:?}");
    assert!(new.0, "new learner must actually reach the ten-step goal");
    assert!(
        new.1 <= 24,
        "native controller must reuse the acquired route"
    );
}

#[test]
fn causal_memory_does_not_bypass_protection_or_accept_a_late_enable() {
    let mut rt = make(12, 7, 12, 192, true);
    let before = rt.checkpoint_constructing().unwrap();
    let mut calls = 0;
    let result = rt.step_constructing(
        None,
        &offers(7),
        |_| HumanProtectionEvidence {
            emergency_stop: true,
            ..safe(0)
        },
        |_| {
            calls += 1;
            Ok(UnifiedPost {
                observation: vec![0; 12],
                cost: 1,
                reward: 0.0,
            })
        },
    );
    assert!(result.is_err());
    assert_eq!(calls, 0);
    assert_eq!(rt.construction_learner().unwrap().factual_event_count(), 0);
    let restored = ScientificRuntime::from_constructing_checkpoint(&before).unwrap();
    assert_eq!(
        restored
            .construction_learner()
            .unwrap()
            .causal_report()
            .unwrap()
            .factual_transitions,
        0
    );
    let mut late = make(2, 4, 3, 192, false);
    late.step_constructing(None, &offers(4), safe, |_| {
        Ok(UnifiedPost {
            observation: vec![1, 0],
            cost: 1,
            reward: 0.0,
        })
    })
    .unwrap();
    assert!(late
        .enable_constructing_causal_memory(CausalConfig::default())
        .is_err());
    assert_eq!(
        late.construction_learner().unwrap().factual_event_count(),
        1
    );
    assert!(!late.construction_learner().unwrap().causal_memory_enabled());
}

fn mix(mut n: u64) -> u64 {
    n = n.wrapping_add(0x9e3779b97f4a7c15);
    n = (n ^ (n >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    n = (n ^ (n >> 27)).wrapping_mul(0x94d049bb133111eb);
    n ^ (n >> 31)
}
fn sensor_life(enabled: bool) -> (usize, usize, u64) {
    let mut rt = make(2, 4, 8, 4096, enabled);
    let goal = [Some(7), None];
    let mut solved = 0;
    let mut queries = 0;
    for ep in 0..700u64 {
        if ep == 600 {
            rt.freeze_constructing().unwrap();
        }
        let hidden = (mix(ep + 0x4500) & 1) as usize;
        let mut draws = 0;
        rt.observe_constructing_external_restart(vec![0, 0])
            .unwrap();
        for _ in 0..4 {
            let mut finished = false;
            let mut success = false;
            rt.step_constructing(Some(&goal), &offers(4), safe, |a| {
                draws += 1;
                let random = mix(ep.wrapping_mul(101) + draws + 0x9321);
                let post = if a == 0 {
                    if ep >= 600 {
                        queries += 1;
                    }
                    let cue = hidden ^ usize::from(random % 10 == 0);
                    11 + 2 * cue as i64
                } else if a == 2 {
                    17 + 2 * (random & 1) as i64
                } else {
                    finished = true;
                    success = (a == 1 && hidden == 0) || (a == 3 && hidden == 1);
                    if success {
                        7
                    } else {
                        9
                    }
                };
                Ok(UnifiedPost {
                    observation: vec![post, 0],
                    cost: 1,
                    reward: f32::from(success),
                })
            })
            .unwrap();
            if finished {
                if ep >= 600 && success {
                    solved += 1;
                }
                break;
            }
        }
    }
    let credits = rt
        .construction_learner()
        .unwrap()
        .causal_report()
        .map_or(0, |r| r.delayed_experiment_credits);
    (solved, queries, credits)
}
#[test]
fn causal_native_selects_a_paid_sensor_and_uses_its_noisy_result_for_a_later_goal() {
    let new = sensor_life(true);
    let old = sensor_life(false);
    eprintln!("CAUSAL_PAID_SENSOR new={new:?} old={old:?}");
    assert!(
        new.0 >= 75 && new.0 >= old.0 + 15,
        "querying must improve actual goal success, not just model fit"
    );
    assert!(
        new.1 >= 50,
        "the selected physical queries must really execute"
    );
    assert!(
        new.2 > 0,
        "later actual goals must assign delayed credit to native information decisions"
    );
}
