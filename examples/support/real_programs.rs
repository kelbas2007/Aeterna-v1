// Recorded fixtures and annotations belong exclusively to the external world.
use aeterna_v1::carrier::{
    PhaseInductionConfig, PhaseNativeConfig, PhaseOnlineConfig, PhasePrimitiveConfig,
    PhaseVectorConfig,
};
use aeterna_v1::scientific_runtime::{RuntimeError, ScientificRuntime, StepOutcome};
use aeterna_v1::{EvoConfig, EvoPhase, HumanProtectionEvidence};
use std::collections::VecDeque;
#[derive(Debug, Clone, Copy)]
pub enum Corpus {
    Images,
    Signals,
}
#[derive(Clone)]
pub struct Record {
    pub input: Vec<f32>,
    pub label: usize,
}
pub fn signal_records(text: &str) -> Vec<Record> {
    let mut data = false;
    let mut rows = Vec::new();
    for line in text.lines() {
        if line.eq_ignore_ascii_case("@data") {
            data = true;
            continue;
        }
        if !data || line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (values, label) = line.rsplit_once(':').unwrap();
        let raw = values
            .split(',')
            .map(|v| v.parse::<f32>().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(raw.len(), 150);
        assert!(raw.iter().all(|v| v.is_finite()));
        let label = label.parse::<usize>().unwrap();
        assert!((1..=2).contains(&label));
        rows.push(Record {
            input: raw
                .chunks_exact(3)
                .map(|c| c.iter().sum::<f32>() / 3.0)
                .collect(),
            label: label - 1,
        });
    }
    rows
}
pub fn data(corpus: Corpus) -> (Vec<Record>, Vec<Record>, usize, usize) {
    match corpus {
        Corpus::Images => {
            let mut counts = [0; 10];
            let mut train = Vec::new();
            let mut test = Vec::new();
            for line in include_str!("../../tests/data/digits.csv").lines() {
                let v = line
                    .split(',')
                    .map(|s| s.parse::<f32>().unwrap())
                    .collect::<Vec<_>>();
                let label = v[64] as usize;
                let r = Record {
                    input: v[..64].iter().map(|x| x / 16.0).collect(),
                    label,
                };
                if counts[label] % 5 == 0 {
                    test.push(r);
                } else {
                    train.push(r);
                }
                counts[label] += 1;
            }
            (train, test, 10, 1)
        }
        Corpus::Signals => {
            let mut train = signal_records(include_str!("../../tests/data/GunPoint_TRAIN.ts"));
            let mut test = signal_records(include_str!("../../tests/data/GunPoint_TEST.ts"));
            assert_eq!(train.len(), 50);
            assert_eq!(test.len(), 150);
            let lo = train
                .iter()
                .flat_map(|r| &r.input)
                .copied()
                .fold(f32::INFINITY, f32::min);
            let hi = train
                .iter()
                .flat_map(|r| &r.input)
                .copied()
                .fold(f32::NEG_INFINITY, f32::max);
            assert!(hi > lo);
            for r in train.iter_mut().chain(&mut test) {
                for x in &mut r.input {
                    *x = ((*x - lo) / (hi - lo)).clamp(0.0, 1.0);
                }
            }
            (train, test, 2, 4)
        }
    }
}
pub fn fresh(width: usize, classes: usize) -> EvoPhase {
    let mut e = EvoPhase::new(EvoConfig {
        sensory_cells: width,
        motor_cells: classes + 1,
        dormant_cells: 320,
        ..Default::default()
    });
    e.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(e.enable_phase_native_online_learning(PhaseOnlineConfig { max_states: 1 }));
    assert!(e.enable_phase_vector_learning(PhaseVectorConfig {
        slots_per_motor: 1,
        exploration_observations: 64,
        ..Default::default()
    }));
    assert!(e.enable_phase_induction(PhaseInductionConfig {
        search_expansions: 128,
        min_future_checks: 2,
        min_leaf_checks: 1,
        coverage_radius: 0.25,
        minimum_outcome: 0.6,
        minimum_margin: 0.05,
        ..Default::default()
    }));
    assert!(e.enable_phase_primitives(PhasePrimitiveConfig {
        capacity: 4,
        ..Default::default()
    }));
    e
}
pub fn safe() -> HumanProtectionEvidence {
    HumanProtectionEvidence {
        human_present: false,
        physical_effect_possible: false,
        predicted_harm_probability: 0.0,
        hazard_confidence: 1.0,
        emergency_stop: false,
    }
}
#[derive(Default)]
pub struct Controls {
    examples: Vec<VecDeque<Vec<f32>>>,
    sums: Vec<Vec<f32>>,
    counts: Vec<usize>,
    observations: Vec<usize>,
}
impl Controls {
    fn observe(&mut self, a: usize, x: &[f32], score: f32, classes: usize) {
        if self.examples.is_empty() {
            self.examples = vec![VecDeque::new(); classes + 1];
            self.sums = vec![vec![0.0; x.len()]; classes + 1];
            self.counts = vec![0; classes + 1];
            self.observations = vec![0; classes + 1];
        }
        self.observations[a] += 1;
        if score < 0.95 {
            return;
        }
        if self.examples[a].len() == 32 {
            self.examples[a].pop_front();
        }
        self.examples[a].push_back(x.to_vec());
        self.counts[a] += 1;
        for (s, &v) in self.sums[a].iter_mut().zip(x) {
            *s += v;
        }
    }
    fn sensor(&self) -> Option<usize> {
        let found = (0..self.counts.len())
            .filter(|&a| self.observations[a] >= 4 && self.counts[a] == 0)
            .collect::<Vec<_>>();
        (found.len() == 1).then(|| found[0])
    }
    fn predict(&self, x: &[f32], linear: bool) -> Option<usize> {
        let mut options = Vec::new();
        for a in 0..self.examples.len() {
            if self.counts[a] == 0 {
                continue;
            }
            if linear {
                let d = self.sums[a]
                    .iter()
                    .zip(x)
                    .map(|(s, &v)| (s / self.counts[a] as f32 - v).powi(2))
                    .sum::<f32>();
                options.push((d, a));
            } else {
                for p in &self.examples[a] {
                    options.push((
                        p.iter().zip(x).map(|(v, w)| (v - w).powi(2)).sum::<f32>(),
                        a,
                    ));
                }
            }
        }
        options.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
        options.first().map(|v| v.1)
    }
}
pub fn train(corpus: Corpus) -> (Vec<u8>, Controls, usize) {
    let (train, _, classes, passes) = data(corpus);
    let mut rt = ScientificRuntime::new(fresh(train[0].input.len(), classes)).unwrap();
    rt.set_outcome_goal(1.0).unwrap();
    let mut control = Controls::default();
    let mut actions = 0;
    for _ in 0..passes {
        for r in &train {
            rt.observe_external(&r.input).unwrap();
            for _ in 0..(2 * classes + 1) {
                if rt.goal_reached().unwrap() {
                    break;
                }
                let result = rt
                    .step_partial(
                        |_| Some(safe()),
                        |a| {
                            let score = f32::from(a == r.label);
                            control.observe(a, &r.input, score, classes);
                            actions += 1;
                            Ok((r.input.iter().copied().map(Some).collect(), score))
                        },
                    )
                    .unwrap();
                assert!(matches!(result, StepOutcome::Executed { .. }));
            }
            assert!(rt.goal_reached().unwrap(), "factual tuition exhausted");
        }
    }
    (
        rt.organism().online_checkpoint_bytes().unwrap(),
        control,
        actions,
    )
}
#[derive(Debug)]
pub struct Report {
    pub corpus: Corpus,
    pub tasks: usize,
    pub correct: usize,
    pub abstentions: usize,
    pub wrong: usize,
    pub nearest: usize,
    pub linear: usize,
    pub nearest_actions: usize,
    pub linear_actions: usize,
    pub training_actions: usize,
    pub test_actions: usize,
    pub sensing_first: usize,
    pub primitives: usize,
    pub primitive_uses: u64,
    pub constructor_updates: u64,
    pub checkpoint_bytes: usize,
    pub useful: bool,
}
pub fn run(corpus: Corpus) -> Report {
    let (bytes, controls, training_actions) = train(corpus);
    let (_, test, classes, _) = data(corpus);
    let mut rt = ScientificRuntime::new(EvoPhase::from_online_checkpoint(&bytes).unwrap()).unwrap();
    rt.set_model_learning_enabled(false);
    rt.set_outcome_goal(1.0).unwrap();
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    let mut report = Report {
        corpus,
        tasks: test.len(),
        correct: 0,
        abstentions: 0,
        wrong: 0,
        nearest: 0,
        linear: 0,
        nearest_actions: 0,
        linear_actions: 0,
        training_actions,
        test_actions: 0,
        sensing_first: 0,
        primitives: rt.organism().phase_primitives().len(),
        primitive_uses: rt
            .organism()
            .phase_primitives()
            .iter()
            .map(|p| p.construction_uses)
            .sum(),
        constructor_updates: rt
            .organism()
            .phase_constructor_info()
            .unwrap()
            .factual_updates,
        checkpoint_bytes: bytes.len(),
        useful: false,
    };
    for r in test {
        // The controls must acquire measurement from those same actual outcomes.
        // The evaluator checks its semantic effect and counts executed actions.
        for linear in [false, true] {
            let mut actions = 0;
            let mut correct = 0;
            if let Some(sensor) = controls.sensor() {
                actions += 1;
                if sensor == classes {
                    if let Some(a) = controls.predict(&r.input, linear) {
                        actions += 1;
                        correct = usize::from(a == r.label);
                    }
                }
            }
            if linear {
                report.linear += correct;
                report.linear_actions += actions;
            } else {
                report.nearest += correct;
                report.nearest_actions += actions;
            }
        }
        rt.observe_external_partial(&vec![None; r.input.len()])
            .unwrap();
        let mut selected = None;
        for turn in 0..2 {
            let result = rt.step_partial(
                |_| Some(safe()),
                |a| {
                    if turn == 0 && a == classes {
                        report.sensing_first += 1;
                    }
                    if a < classes {
                        selected = Some(a);
                    }
                    report.test_actions += 1;
                    Ok((
                        r.input.iter().copied().map(Some).collect(),
                        f32::from(a == r.label),
                    ))
                },
            );
            match result {
                Ok(StepOutcome::Executed { .. }) => {}
                Err(RuntimeError::NoSupportedAction) => break,
                other => panic!("{other:?}"),
            }
            if selected.is_some() {
                break;
            }
        }
        match selected {
            Some(a) if a == r.label => {
                report.correct += 1;
                assert!(rt.goal_reached().unwrap());
            }
            Some(_) => report.wrong += 1,
            None => report.abstentions += 1,
        }
        assert_eq!(
            fingerprint,
            rt.organism().phase_native_learned_fingerprint()
        );
    }
    report.useful = report.correct * 100
        >= report.tasks
            * match corpus {
                Corpus::Images => 60,
                Corpus::Signals => 70,
            };
    report
}
