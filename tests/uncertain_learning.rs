use aeterna_v1::carrier::{
    PhaseAdaptiveConfig, PhaseInterval, PhaseNativeConfig, PhaseOnlineConfig, PhasePartialConfig,
    PhasePartialDecisionKind, PhaseRuleConfig, PhaseRuleLanguage, PhaseUncertainConfig,
};
use aeterna_v1::scientific_runtime::{RuntimeError, ScientificRuntime, StepOutcome};
use aeterna_v1::{Authority, EvoConfig, EvoPhase, HumanProtectionEvidence};

fn safe() -> HumanProtectionEvidence {
    HumanProtectionEvidence {
        human_present: false,
        physical_effect_possible: false,
        predicted_harm_probability: 0.0,
        hazard_confidence: 1.0,
        emergency_stop: false,
    }
}
fn fresh(width: usize, motors: usize, max: usize, budget: usize) -> EvoPhase {
    let mut e = EvoPhase::new(EvoConfig {
        sensory_cells: width,
        motor_cells: motors,
        dormant_cells: width * motors * 5 + 4,
        ..Default::default()
    });
    e.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(e.enable_phase_native_online_learning(PhaseOnlineConfig { max_states: 1 }));
    assert!(e.enable_phase_rule_learning_with_language(
        PhaseRuleConfig {
            planning_depth: 2,
            ..Default::default()
        },
        PhaseRuleLanguage::CircularAffine
    ));
    assert!(e.enable_phase_adaptive_rules(PhaseAdaptiveConfig {
        min_inlier_fraction: 1.0,
        ..Default::default()
    }));
    assert!(e.enable_phase_uncertain_observation(
        PhasePartialConfig {
            max_hypotheses: max,
            ..Default::default()
        },
        PhaseUncertainConfig {
            measurement_radius: 0.0004,
            inference_budget: budget,
            ..Default::default()
        }
    ));
    e
}
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> i32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % 257) as i32
    }
}
fn measured(state: &[i32], tick: usize) -> Vec<Option<f32>> {
    state
        .iter()
        .enumerate()
        .map(|(j, &x)| {
            Some(
                (x as f32 / 257.0 + if (tick + j) % 2 == 0 { 0.0003 } else { -0.0003 })
                    .rem_euclid(1.0),
            )
        })
        .collect()
}
fn acquired<F: Fn(usize, &[i32]) -> Vec<i32>>(
    width: usize,
    motors: usize,
    max: usize,
    budget: usize,
    law: F,
) -> EvoPhase {
    let mut rt = ScientificRuntime::new(fresh(width, motors, max, budget)).unwrap();
    let mut rng = Rng(0x1234ABCD);
    for tick in 0..160 {
        let state = (0..width).map(|_| rng.next()).collect::<Vec<_>>();
        rt.observe_external_partial(&measured(&state, tick))
            .unwrap();
        rt.set_partial_goal(&vec![Some(0.1234567); width]).unwrap();
        assert!(matches!(
            rt.step_partial(
                |_| Some(safe()),
                |a| Ok((measured(&law(a, &state), tick + 1), 0.0))
            )
            .unwrap(),
            StepOutcome::Executed { .. }
        ));
    }
    for a in 0..motors {
        assert!(
            rt.organism()
                .phase_adaptive_action_info(a)
                .unwrap()
                .active
                .is_some(),
            "action {a}"
        );
    }
    EvoPhase::from_online_checkpoint(&rt.organism().online_checkpoint_bytes().unwrap()).unwrap()
}
fn inside(rows: &[Vec<Option<PhaseInterval>>], truth: &[f32]) -> bool {
    rows.iter().any(|row| {
        row.iter()
            .zip(truth)
            .all(|(v, &x)| v.is_none_or(|v| v.contains(x)))
    })
}
fn joint(_: usize, s: &[i32]) -> Vec<i32> {
    vec![
        (s[0] + s[1] + 17).rem_euclid(257),
        (s[0] - s[1] + 31).rem_euclid(257),
    ]
}

#[test]
fn joint_inverse_preserves_both_modular_solutions_and_bounded_sensor_error() {
    let e = acquired(2, 1, 32, 8192, joint);
    let mut rt = ScientificRuntime::new(e).unwrap();
    rt.set_model_learning_enabled(false);
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    for (tick, state) in [[1, 256], [97, 201], [255, 0], [37, 82]]
        .into_iter()
        .enumerate()
    {
        rt.observe_external_partial(&[None, None]).unwrap();
        rt.set_partial_goal(&[Some(0.1234567), None]).unwrap();
        rt.step_partial(
            |_| Some(safe()),
            |_| Ok((measured(&joint(0, &state), tick), 0.0)),
        )
        .unwrap();
        let report = rt
            .organism()
            .phase_uncertain_belief()
            .unwrap()
            .inverse
            .unwrap();
        assert_eq!(report.authority, Authority::Imagined);
        assert!(!report.contradicted);
        assert!(!report.evidence_sources.is_empty());
        assert!(report.operations <= 8192);
        let truth = state.map(|x| x as f32 / 257.0);
        assert!(inside(&report.pre_hypotheses, &truth), "{report:?}");
        let other = truth.map(|x| (x + 0.5).rem_euclid(1.0));
        assert!(
            inside(&report.pre_hypotheses, &other),
            "second root removed: {report:?}"
        );
        assert!(report
            .pre_hypotheses
            .iter()
            .all(|row| row.iter().all(|v| v.is_some_and(|v| v.radius < 0.02))));
        // Independent dense integer evaluator checks every feasible state,
        // rather than comparing against the solver's own formulas/roots.
        let observed = joint(0, &state);
        for x in 0..257 {
            for y in 0..257 {
                if joint(0, &[x, y]) == observed {
                    assert!(inside(
                        &report.pre_hypotheses,
                        &[x as f32 / 257.0, y as f32 / 257.0]
                    ));
                }
            }
        }
    }
    assert_eq!(
        rt.organism().phase_native_learned_fingerprint(),
        fingerprint
    );
}

#[test]
fn modular_gain_roots_and_cap_widening_never_discard_the_true_operand() {
    for (gain, cap) in [(2, 32), (-3, 32), (2, 1)] {
        let e = acquired(2, 1, cap, 8192, |_, s| {
            vec![
                (gain * s[0] + 17).rem_euclid(257),
                (s[1] + 31).rem_euclid(257),
            ]
        });
        let mut rt = ScientificRuntime::new(e).unwrap();
        rt.set_model_learning_enabled(false);
        rt.observe_external_partial(&[None, None]).unwrap();
        rt.set_partial_goal(&[Some(0.1234567), None]).unwrap();
        let state = [255, 37];
        let post = [
            (gain * state[0] + 17).rem_euclid(257),
            (state[1] + 31).rem_euclid(257),
        ];
        rt.step_partial(|_| Some(safe()), |_| Ok((measured(&post, 0), 0.0)))
            .unwrap();
        let report = rt
            .organism()
            .phase_uncertain_belief()
            .unwrap()
            .inverse
            .unwrap();
        for k in 0..gain.unsigned_abs() {
            let x =
                (state[0] as f32 / 257.0 + k as f32 / gain.unsigned_abs() as f32).rem_euclid(1.0);
            assert!(inside(
                &report.pre_hypotheses,
                &[x, state[1] as f32 / 257.0]
            ));
        }
        if cap == 1 {
            assert!(report.widened);
            assert_eq!(report.pre_hypotheses.len(), 1);
        }
    }
}

#[test]
fn dependent_equations_leave_a_continuous_unknown_and_budget_exhaustion_widens() {
    let e = acquired(2, 1, 32, 128, |_, s| {
        vec![
            (s[0] + s[1] + 17).rem_euclid(257),
            (s[0] + s[1] + 31).rem_euclid(257),
        ]
    });
    let mut rt = ScientificRuntime::new(e).unwrap();
    rt.set_model_learning_enabled(false);
    rt.observe_external_partial(&[None, None]).unwrap();
    rt.set_partial_goal(&[Some(0.1234567), None]).unwrap();
    rt.step_partial(|_| Some(safe()), |_| Ok((measured(&[47, 61], 0), 0.0)))
        .unwrap();
    let r = rt
        .organism()
        .phase_uncertain_belief()
        .unwrap()
        .inverse
        .unwrap();
    assert!(r
        .pre_hypotheses
        .iter()
        .any(|r| r.iter().all(Option::is_none)));
    assert!(!r.contradicted);
    // Cartesian alternatives consume a shared budget, never independent
    // per-root budgets that could leave incomplete search looking precise.
    let law = |a, s: &[i32]| {
        let mut out = joint(a, s);
        out.extend_from_slice(&s[2..]);
        out
    };
    let many = acquired(5, 1, 32, 128, law);
    let rows = (0..32).map(|_| vec![None; 5]).collect::<Vec<_>>();
    let prediction = many.phase_uncertain_predict(0, &rows).unwrap();
    assert!(prediction.widened);
    assert_eq!(prediction.hypotheses, vec![vec![None; 5]]);
    assert!(many
        .phase_uncertain_predict(
            0,
            &[vec![
                Some(PhaseInterval {
                    center: 0.2,
                    radius: 0.5
                });
                5
            ]]
        )
        .is_none());
}

#[test]
fn inferred_centers_do_not_teach_and_only_a_factual_visible_goal_counts() {
    let e = acquired(2, 1, 32, 8192, joint);
    let mut rt = ScientificRuntime::new(e).unwrap();
    let before = rt
        .organism()
        .phase_adaptive_action_info(0)
        .unwrap()
        .observations;
    rt.observe_external_partial(&[None, None]).unwrap();
    rt.set_partial_goal(&[Some(0.1234567), None]).unwrap();
    rt.step_partial(|_| Some(safe()), |_| Ok((vec![Some(0.3), None], 0.0)))
        .unwrap();
    assert_eq!(
        rt.organism()
            .phase_adaptive_action_info(0)
            .unwrap()
            .observations,
        before
    );
    assert!(!rt.organism().phase_partial_goal_reached(&[None, Some(0.4)]));
    rt.observe_external_partial(&[Some(0.3), Some(0.4)])
        .unwrap();
    assert!(rt.organism().phase_partial_goal_reached(&[Some(0.3), None]));
    assert!(!rt
        .organism()
        .phase_partial_goal_reached(&[Some(0.3099), None]));
    let mut rt = ScientificRuntime::new(acquired(2, 1, 32, 8192, joint)).unwrap();
    rt.set_model_learning_enabled(false);
    rt.observe_external_partial(&[Some(0.1), Some(0.2)])
        .unwrap();
    rt.set_partial_goal(&[
        Some((0.1 + 0.2 + 17.0 / 257.0) % 1.0),
        Some((0.1 - 0.2 + 31.0 / 257.0_f32).rem_euclid(1.0)),
    ])
    .unwrap();
    let mut calls = 0;
    let mut blocked = safe();
    blocked.emergency_stop = true;
    assert!(matches!(
        rt.step_partial(
            |_| Some(blocked),
            |_| {
                calls += 1;
                Ok((vec![Some(0.1), Some(0.2)], 0.0))
            }
        )
        .unwrap(),
        StepOutcome::Blocked(_)
    ));
    assert_eq!(calls, 0);
}

#[test]
fn noisy_conditional_planning_measures_hidden_context_before_committing() {
    let law = |a, s: &[i32]| {
        if a == 1 {
            s.to_vec()
        } else {
            vec![if s[1] < 130 { 64 } else { 193 }, s[1]]
        }
    };
    let e = acquired(2, 2, 32, 8192, law);
    let mut rt = ScientificRuntime::new(e).unwrap();
    rt.set_model_learning_enabled(false);
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    let state = [21, 220];
    rt.observe_external_partial(&[Some(21.0 / 257.0), None])
        .unwrap();
    rt.set_partial_goal(&[Some(193.0 / 257.0), None]).unwrap();
    let decision = rt
        .organism()
        .phase_partial_decision(&[Some(193.0 / 257.0), None])
        .unwrap();
    assert_eq!(decision.action, 1);
    assert_eq!(
        decision.kind,
        PhasePartialDecisionKind::InformationGathering
    );
    assert!(matches!(
        rt.step_partial(
            |_| Some(safe()),
            |a| {
                assert_eq!(a, 1);
                Ok((measured(&law(a, &state), 0), 0.0))
            }
        )
        .unwrap(),
        StepOutcome::Executed { learned: false, .. }
    ));
    assert!(matches!(
        rt.step_partial(
            |_| Some(safe()),
            |a| {
                assert_eq!(a, 0);
                Ok((measured(&law(a, &state), 1), 0.0))
            }
        )
        .unwrap(),
        StepOutcome::Executed { learned: false, .. }
    ));
    assert!(rt.goal_reached().unwrap());
    assert_eq!(
        rt.organism().phase_native_learned_fingerprint(),
        fingerprint
    );
}

#[test]
fn physical_phase_support_is_necessary_even_for_hidden_operands() {
    let e = acquired(2, 1, 32, 8192, joint);
    let bytes = e.online_checkpoint_bytes().unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let link = e
        .phase_adaptive_action_info(0)
        .unwrap()
        .active
        .unwrap()
        .synapses[0];
    value["synapses"][link]["weight"] = serde_json::json!(0.0);
    let cut = EvoPhase::from_online_checkpoint(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(cut
        .phase_uncertain_predict(0, &[vec![None, None]])
        .is_none());
    assert!(e.phase_uncertain_predict(0, &[vec![None, None]]).is_some());
    let input = vec![
        Some(PhaseInterval {
            center: 0.1,
            radius: 0.0004,
        }),
        Some(PhaseInterval {
            center: 0.2,
            radius: 0.0004,
        }),
    ];
    let intact = e
        .phase_uncertain_predict(0, std::slice::from_ref(&input))
        .unwrap();
    let mut shifted: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    shifted["synapses"][link]["phase_offset"] = serde_json::json!(
        shifted["synapses"][link]["phase_offset"].as_f64().unwrap() + std::f64::consts::PI
    );
    let changed = EvoPhase::from_online_checkpoint(&serde_json::to_vec(&shifted).unwrap()).unwrap();
    let prediction = changed.phase_uncertain_predict(0, &[input]).unwrap();
    assert!(!inside(
        &prediction.hypotheses,
        &intact.hypotheses[0]
            .iter()
            .map(|x| x.unwrap().center)
            .collect::<Vec<_>>()
    ));
    assert_eq!(
        EvoPhase::from_online_checkpoint(&bytes)
            .unwrap()
            .phase_uncertain_predict(0, &[vec![None, None]]),
        e.phase_uncertain_predict(0, &[vec![None, None]])
    );
}

#[test]
fn frozen_identity_measurements_do_not_loop_or_probe_irrelevant_channels() {
    let e = acquired(2, 1, 32, 8192, |_, s| s.to_vec());
    let mut rt = ScientificRuntime::new(e).unwrap();
    rt.set_model_learning_enabled(false);
    rt.observe_external_partial(&[None, None]).unwrap();
    rt.set_partial_goal(&[Some(0.7), None]).unwrap();
    rt.step_partial(|_| Some(safe()), |_| Ok((vec![Some(0.2), Some(0.3)], 0.0)))
        .unwrap();
    assert_eq!(rt.propose_partial(), Err(RuntimeError::NoSupportedAction));
    rt.observe_external_partial(&[Some(0.2), None]).unwrap();
    assert_eq!(rt.propose_partial(), Err(RuntimeError::NoSupportedAction));
}

#[test]
fn one_noisy_partial_lifetime_adapts_recalls_and_keeps_memory_bounded() {
    let mut rt = ScientificRuntime::new(fresh(2, 2, 32, 8192)).unwrap();
    let mut state = vec![29, 181];
    rt.observe_external_partial(&measured(&state, 0)).unwrap();
    rt.set_partial_goal(&[Some(0.1234567), None]).unwrap();
    let mut tick = 0;
    for stage in 0..3 {
        for _ in 0..256 {
            if rt.goal_reached().unwrap() {
                let actual = rt.organism().phase_uncertain_belief().unwrap().factual[0].unwrap();
                rt.set_partial_goal(&[Some((actual + 0.5).rem_euclid(1.0)), None])
                    .unwrap();
            }
            let outcome = rt
                .step_partial(
                    |_| Some(safe()),
                    |a| {
                        let (mut gain, mut bias) = match (a, state[1] >= 130) {
                            (0, false) => (1, 17),
                            (0, true) => (2, 71),
                            (1, false) => (-1, 31),
                            (1, true) => (1, 53),
                            _ => unreachable!(),
                        };
                        if stage == 1 {
                            gain = -gain;
                            bias += 101;
                        }
                        state = vec![
                            (gain * state[0] + bias).rem_euclid(257),
                            (state[1] + 31).rem_euclid(257),
                        ];
                        tick += 1;
                        let mut post = measured(&state, tick);
                        if tick % 17 == 0 {
                            post[1] = None;
                        }
                        Ok((post, 0.0))
                    },
                )
                .unwrap();
            assert!(
                matches!(outcome, StepOutcome::Executed { .. }),
                "stage={stage} tick={tick} outcome={outcome:?}"
            );
        }
    }
    let infos = (0..2)
        .map(|a| rt.organism().phase_adaptive_action_info(a).unwrap())
        .collect::<Vec<_>>();
    assert!(
        infos
            .iter()
            .all(|i| i.active.is_some() && !i.archives.is_empty() && i.reactivations > 0),
        "{infos:?}"
    );
    let start: serde_json::Value =
        serde_json::from_slice(&rt.organism().online_checkpoint_bytes().unwrap()).unwrap();
    for _ in 0..2048 {
        if rt.goal_reached().unwrap() {
            let actual = rt.organism().phase_uncertain_belief().unwrap().factual[0].unwrap();
            rt.set_partial_goal(&[Some((actual + 0.5).rem_euclid(1.0)), None])
                .unwrap();
        }
        rt.step_partial(
            |_| Some(safe()),
            |a| {
                let (gain, bias) = match (a, state[1] >= 130) {
                    (0, false) => (1, 17),
                    (0, true) => (2, 71),
                    (1, false) => (-1, 31),
                    (1, true) => (1, 53),
                    _ => unreachable!(),
                };
                state = vec![
                    (gain * state[0] + bias).rem_euclid(257),
                    (state[1] + 31).rem_euclid(257),
                ];
                tick += 1;
                Ok((measured(&state, tick), 0.0))
            },
        )
        .unwrap();
    }
    let end: serde_json::Value =
        serde_json::from_slice(&rt.organism().online_checkpoint_bytes().unwrap()).unwrap();
    assert_eq!(
        start["cells"].as_array().unwrap().len(),
        end["cells"].as_array().unwrap().len()
    );
    assert_eq!(
        start["synapses"].as_array().unwrap().len(),
        end["synapses"].as_array().unwrap().len()
    );
    for a in 0..2 {
        let i = rt.organism().phase_adaptive_action_info(a).unwrap();
        assert!(i.retained_examples <= 70 && i.archives.len() <= 3);
    }
    let belief = rt.organism().phase_uncertain_belief().unwrap();
    assert!(belief.hypotheses.len() <= 32);
    println!("continual_actions={} changes=A/B/A sparse_every=17 extra_memory_actions=2048 cells={} synapses={}", tick-2048, end["cells"].as_array().unwrap().len(), end["synapses"].as_array().unwrap().len());
}

#[test]
fn version_six_restore_is_atomic_bounded_and_requires_new_observation() {
    let e = acquired(2, 1, 32, 8192, joint);
    let bytes = e.online_checkpoint_bytes().unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["version"], 9);
    let mut rt = ScientificRuntime::new(e).unwrap();
    rt.set_model_learning_enabled(false);
    rt.observe_external_partial(&[Some(0.1), Some(0.2)])
        .unwrap();
    rt.set_partial_goal(&[Some(0.5), None]).unwrap();
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    for bad in [0, 127, 1_000_001] {
        let mut corrupt = value.clone();
        corrupt["partial"]["uncertainty"]["inference_budget"] = serde_json::json!(bad);
        assert!(rt
            .restore_online_checkpoint(&serde_json::to_vec(&corrupt).unwrap())
            .is_err());
        assert_eq!(
            rt.organism().phase_native_learned_fingerprint(),
            fingerprint
        );
    }
    let mut old = value.clone();
    old["version"] = serde_json::json!(5);
    assert!(EvoPhase::from_online_checkpoint(&serde_json::to_vec(&old).unwrap()).is_err());
    rt.restore_online_checkpoint(&bytes).unwrap();
    assert!(rt.organism().phase_uncertain_belief().is_none());
    assert_eq!(
        rt.organism().phase_native_learned_fingerprint(),
        fingerprint
    );
    assert!(matches!(
        rt.step_partial(
            |_| Some(safe()),
            |_| panic!("restore must require fresh observation")
        ),
        Err(RuntimeError::FreshObservationRequired)
    ));
}
