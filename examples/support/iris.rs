// Evaluator only: raw records and species labels never enter the production
// planner. A label determines the factual outcome of a terminal world action.
use aeterna_v1::carrier::{
    PhaseAdaptiveConfig, PhaseNativeConfig, PhaseOnlineConfig, PhasePartialConfig, PhaseRuleConfig,
    PhaseRuleLanguage, PhaseUncertainConfig,
};
use aeterna_v1::scientific_runtime::{RuntimeError, ScientificRuntime, StepOutcome};
use aeterna_v1::{EvoConfig, EvoPhase, HumanProtectionEvidence};
use std::collections::VecDeque;

#[derive(Clone)]
struct Record {
    raw: [f32; 4],
    species: usize,
}
fn records() -> Vec<Record> {
    include_str!("../../tests/data/iris.csv")
        .lines()
        .skip(1)
        .map(|line| {
            let fields = line.split(',').collect::<Vec<_>>();
            Record {
                raw: std::array::from_fn(|j| fields[j].parse().unwrap()),
                species: fields[4].parse().unwrap(),
            }
        })
        .collect()
}
fn safe() -> HumanProtectionEvidence {
    HumanProtectionEvidence {
        human_present: false,
        physical_effect_possible: false,
        predicted_harm_probability: 0.0,
        hazard_confidence: 1.0,
        emergency_stop: false,
    }
}
struct World {
    record: Record,
    pair: [usize; 2],
    channels: [usize; 5],
    roles: [usize; 3], // evaluator-private: sensor, class A, class B
    state: [f32; 5],
    tick: usize,
    terminal: bool,
}
impl World {
    fn new(record: Record, pair: [usize; 2], variant: usize, tick: usize) -> Self {
        // Fixed unit scaling avoids fitting normalization on held-out records.
        let channels = if variant % 2 == 0 {
            [3, 0, 4, 1, 2]
        } else {
            [2, 4, 1, 0, 3]
        };
        let roles = [[2, 0, 1], [1, 2, 0], [0, 1, 2]][variant % 3];
        let mut state = [0.0; 5];
        for j in 0..4 {
            state[channels[j]] = 0.15 + record.raw[j] / 12.0;
        }
        // Vary the neutral outcome independently of the species and feature
        // values. A fixed neutral value aliases a constant with copy+offset.
        state[channels[4]] = 0.05 + ((tick * 37 + tick / 7) % 17) as f32 / 17.0 * 0.4;
        Self {
            record,
            pair,
            channels,
            roles,
            state,
            tick,
            terminal: false,
        }
    }
    fn measured(&self, full: bool) -> Vec<Option<f32>> {
        self.state
            .iter()
            .enumerate()
            .map(|(j, &x)| {
                (full || j == self.channels[4]).then_some(
                    x + if (self.tick + j) % 2 == 0 {
                        0.00015
                    } else {
                        -0.00015
                    },
                )
            })
            .collect()
    }
    fn goal(&self) -> Vec<Option<f32>> {
        let mut goal = vec![None; 5];
        goal[self.channels[4]] = Some(0.75);
        goal
    }
    fn act(&mut self, a: usize) -> (Vec<Option<f32>>, f32) {
        assert!(!self.terminal, "only one terminal choice is allowed");
        let role = self.roles[a];
        let correct = role > 0 && self.record.species == self.pair[role - 1];
        if role > 0 {
            self.state[self.channels[4]] = if correct { 0.75 } else { 0.25 };
            self.terminal = true;
        }
        self.tick += 1;
        (self.measured(true), f32::from(correct))
    }
}
#[derive(Clone)]
struct Fact {
    pre: Vec<f32>,
    post: Vec<f32>,
}
#[derive(Default)]
struct Stump {
    evidence: [VecDeque<Fact>; 3],
}
#[derive(Clone, Copy)]
struct Split {
    source: usize,
    threshold: f32,
    low: f32,
    high: f32,
}
impl Stump {
    fn observe(&mut self, a: usize, pre: Vec<f32>, post: &[Option<f32>]) {
        if self.evidence[a].len() == 64 {
            self.evidence[a].pop_front();
        }
        self.evidence[a].push_back(Fact {
            pre,
            post: post.iter().map(|x| x.unwrap()).collect(),
        });
    }
    fn sensor(&self) -> Option<usize> {
        (0..3).find(|&a| {
            self.evidence[a].len() >= 6
                && self.evidence[a].iter().all(|f| {
                    f.pre
                        .iter()
                        .zip(&f.post)
                        .all(|(x, y)| (x - y).abs() <= 0.001)
                })
        })
    }
    fn fit(&self, a: usize, need: usize) -> Option<Split> {
        let facts = &self.evidence[a];
        if facts.len() < 6 {
            return None;
        }
        let label = |f: &Fact| usize::from((f.post[need] - 0.75).abs() < 0.01);
        let mut best: Option<(usize, Split)> = None;
        for source in 0..5 {
            if source == need {
                continue;
            }
            let mut thresholds = facts.iter().map(|f| f.pre[source]).collect::<Vec<_>>();
            thresholds.sort_by(f32::total_cmp);
            thresholds.dedup();
            thresholds.insert(0, -1.0);
            thresholds.push(2.0);
            for window in thresholds.windows(2) {
                let threshold = (window[0] + window[1]) / 2.0;
                let mut counts = [[0usize; 2]; 2];
                for f in facts {
                    counts[usize::from(f.pre[source] >= threshold)][label(f)] += 1;
                }
                let score = counts.iter().map(|c| c[0].max(c[1])).sum();
                let probability = |c: [usize; 2]| {
                    if c[0] + c[1] == 0 {
                        0.0
                    } else {
                        c[1] as f32 / (c[0] + c[1]) as f32
                    }
                };
                if best.is_none_or(|(old, _)| score > old) {
                    best = Some((
                        score,
                        Split {
                            source,
                            threshold,
                            low: probability(counts[0]),
                            high: probability(counts[1]),
                        },
                    ));
                }
            }
        }
        best.map(|(_, s)| s)
    }
    fn choice(&self, pre: &[Option<f32>], need: usize) -> Option<usize> {
        let sensor = self.sensor()?;
        if pre.iter().any(Option::is_none) {
            return Some(sensor);
        }
        (0..3)
            .filter(|&a| a != sensor)
            .filter_map(|a| {
                let s = self.fit(a, need)?;
                Some((
                    a,
                    if pre[s.source]? < s.threshold {
                        s.low
                    } else {
                        s.high
                    },
                ))
            })
            .max_by(|(a, x), (b, y)| x.total_cmp(y).then_with(|| b.cmp(a)))
            .map(|(a, _)| a)
    }
}
#[derive(Debug)]
pub struct PairReport {
    pub pair: [usize; 2],
    pub training_actions: usize,
    pub trained_models: usize,
    pub tasks: usize,
    pub phase_success: usize,
    pub stump_success: usize,
    pub phase_actions: usize,
    pub stump_actions: usize,
    pub abstentions: usize,
    pub sensing_first: usize,
    pub retained_examples: usize,
    pub checkpoint_bytes: usize,
}
fn fresh() -> EvoPhase {
    let mut e = EvoPhase::new(EvoConfig {
        sensory_cells: 5,
        motor_cells: 3,
        dormant_cells: 80,
        ..Default::default()
    });
    e.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(e.enable_phase_native_online_learning(PhaseOnlineConfig { max_states: 1 }));
    assert!(e.enable_phase_rule_learning_with_language(
        PhaseRuleConfig {
            planning_depth: 2,
            node_budget: 256,
            ..Default::default()
        },
        PhaseRuleLanguage::CircularAffine
    ));
    assert!(e.enable_phase_adaptive_rules(PhaseAdaptiveConfig {
        fit_budget: 8_000_000,
        ..Default::default()
    }));
    assert!(e.enable_phase_uncertain_observation(
        PhasePartialConfig::default(),
        PhaseUncertainConfig {
            measurement_radius: 0.00025,
            ..Default::default()
        }
    ));
    e
}
pub fn run() -> Vec<PairReport> {
    let records = records();
    assert_eq!(records.len(), 150);
    let mut reports = Vec::new();
    for (variant, pair) in [[0, 1], [0, 2], [1, 2]].into_iter().enumerate() {
        // Per species, record index mod 5 in {0,1} is held out. The split and
        // policy are fixed before fitting; no held-out feedback teaches.
        let mut train = Vec::new();
        let mut heldout = Vec::new();
        for i in 0..50 {
            for &species in &pair {
                let record = records[species * 50 + i].clone();
                if i % 5 < 2 {
                    heldout.push(record);
                } else {
                    train.push(record);
                }
            }
        }
        let mut rt = ScientificRuntime::new(fresh()).unwrap();
        let mut stump = Stump::default();
        let mut training_actions = 0;
        for trial in 0..180 {
            let mut world = World::new(
                train[(trial * 17 + (trial / 60) * 7) % train.len()].clone(),
                pair,
                variant,
                trial,
            );
            rt.observe_external_partial(&world.measured(true)).unwrap();
            rt.set_partial_goal(&world.goal()).unwrap();
            for _ in 0..3 {
                let pre = world
                    .measured(true)
                    .iter()
                    .map(|x| x.unwrap())
                    .collect::<Vec<_>>();
                let result = rt
                    .step_partial(
                        |_| Some(safe()),
                        |a| {
                            let (post, outcome) = world.act(a);
                            stump.observe(a, pre, &post);
                            training_actions += 1;
                            Ok((post, outcome))
                        },
                    )
                    .unwrap();
                if world.terminal || !matches!(result, StepOutcome::Executed { .. }) {
                    break;
                }
            }
        }
        let bytes = rt.organism().online_checkpoint_bytes().unwrap();
        let infos = (0..3)
            .map(|a| rt.organism().phase_adaptive_action_info(a).unwrap())
            .collect::<Vec<_>>();
        let mut report = PairReport {
            pair,
            training_actions,
            trained_models: infos.iter().filter(|i| i.active.is_some()).count(),
            tasks: heldout.len(),
            phase_success: 0,
            stump_success: 0,
            phase_actions: 0,
            stump_actions: 0,
            abstentions: 0,
            sensing_first: 0,
            retained_examples: infos.iter().map(|i| i.retained_examples).sum(),
            checkpoint_bytes: bytes.len(),
        };
        // Exercise actual disk-format restoration before independent frozen
        // tasks. No episode observation or inferred interval is persisted.
        let mut rt =
            ScientificRuntime::new(EvoPhase::from_online_checkpoint(&bytes).unwrap()).unwrap();
        rt.set_model_learning_enabled(false);
        let fingerprint = rt.organism().phase_native_learned_fingerprint();
        for (trial, record) in heldout.into_iter().enumerate() {
            let mut world = World::new(record.clone(), pair, variant, 1000 + trial);
            rt.observe_external_partial(&world.measured(false)).unwrap();
            rt.set_partial_goal(&world.goal()).unwrap();
            for step in 0..2 {
                match rt.step_partial(
                    |_| Some(safe()),
                    |a| {
                        if step == 0 && world.roles[a] == 0 {
                            report.sensing_first += 1;
                        }
                        report.phase_actions += 1;
                        Ok(world.act(a))
                    },
                ) {
                    Ok(StepOutcome::Executed { learned: false, .. }) => {}
                    Err(RuntimeError::NoSupportedAction) => {
                        report.abstentions += 1;
                        break;
                    }
                    other => panic!("unexpected frozen outcome: {other:?}"),
                }
                if world.terminal {
                    break;
                }
            }
            report.phase_success += usize::from(world.terminal && rt.goal_reached().unwrap());
            assert_eq!(
                rt.organism().phase_native_learned_fingerprint(),
                fingerprint
            );
            let mut baseline = World::new(record, pair, variant, 1000 + trial);
            let mut observation = baseline.measured(false);
            for _ in 0..2 {
                let Some(a) = stump.choice(&observation, baseline.channels[4]) else {
                    break;
                };
                report.stump_actions += 1;
                observation = baseline.act(a).0;
                if baseline.terminal {
                    break;
                }
            }
            report.stump_success += usize::from(
                baseline.terminal && (baseline.state[baseline.channels[4]] - 0.75).abs() < 0.01,
            );
        }
        reports.push(report);
    }
    reports
}
