// Real recorded images and labels are evaluator data only. Production EvoPhase
// receives an image, opaque chosen motor and its actual external outcome.
use aeterna_v1::carrier::{PhaseNativeConfig, PhaseOnlineConfig, PhaseVectorConfig};
use aeterna_v1::scientific_runtime::{RuntimeError, ScientificRuntime, StepOutcome};
use aeterna_v1::{EvoConfig, EvoPhase, HumanProtectionEvidence};
use std::collections::VecDeque;

#[derive(Clone)]
pub struct Record {
    pub pixels: Vec<f32>,
    pub digit: usize,
}
pub fn records() -> Vec<Record> {
    include_str!("../../tests/data/digits.csv")
        .lines()
        .map(|line| {
            let values = line
                .split(',')
                .map(|v| v.parse::<f32>().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(values.len(), 65);
            Record {
                pixels: values[..64].iter().map(|x| x / 16.0).collect(),
                digit: values[64] as usize,
            }
        })
        .collect()
}
pub fn split() -> (Vec<Record>, Vec<Record>) {
    let mut counts = [0; 10];
    let mut train = Vec::new();
    let mut heldout = Vec::new();
    for r in records() {
        let test = counts[r.digit] % 5 == 0;
        counts[r.digit] += 1;
        if test {
            heldout.push(r);
        } else {
            train.push(r);
        }
    }
    (train, heldout)
}
pub fn fresh() -> EvoPhase {
    let mut e = EvoPhase::new(EvoConfig {
        sensory_cells: 64,
        motor_cells: 11,
        dormant_cells: 356,
        ..Default::default()
    });
    e.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(e.enable_phase_native_online_learning(PhaseOnlineConfig { max_states: 1 }));
    assert!(e.enable_phase_vector_learning(PhaseVectorConfig::default()));
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
#[derive(Clone)]
struct World {
    record: Record,
    roles: Vec<usize>,
    pixels: Vec<f32>,
    terminal: bool,
}
impl World {
    fn new(record: Record, variant: usize) -> Self {
        let roles = (0..11)
            .map(|a| (a * (if variant == 0 { 1 } else { 3 }) + variant * 2) % 11)
            .collect();
        let pixels = (0..64)
            .map(|j| record.pixels[(j * (if variant == 0 { 1 } else { 5 }) + variant * 7) % 64])
            .collect();
        Self {
            record,
            roles,
            pixels,
            terminal: false,
        }
    }
    fn act(&mut self, a: usize, testing: bool) -> (Vec<Option<f32>>, f32) {
        assert!(!self.terminal);
        let role = self.roles[a];
        let correct = role == self.record.digit;
        self.terminal = testing && role < 10;
        (
            self.pixels.iter().copied().map(Some).collect(),
            f32::from(correct),
        )
    }
}
#[derive(Default)]
pub struct Controls {
    observations: [usize; 11],
    successes: [usize; 11],
    examples: [VecDeque<Vec<f32>>; 11],
    sums: [Vec<f32>; 11],
}
impl Controls {
    fn observe(&mut self, a: usize, pre: &[f32], outcome: f32) {
        self.observations[a] += 1;
        if outcome < 0.95 {
            return;
        }
        self.successes[a] += 1;
        if self.examples[a].len() == 32 {
            self.examples[a].pop_front();
        }
        self.examples[a].push_back(pre.to_vec());
        if self.sums[a].is_empty() {
            self.sums[a] = vec![0.0; pre.len()];
        }
        for (s, &v) in self.sums[a].iter_mut().zip(pre) {
            *s += v;
        }
    }
    fn sensor(&self) -> Option<usize> {
        let c = (0..11)
            .filter(|&a| self.observations[a] >= 4 && self.successes[a] == 0)
            .collect::<Vec<_>>();
        (c.len() == 1).then(|| c[0])
    }
    fn choice(&self, input: &[Option<f32>], centroid: bool) -> Option<usize> {
        if input.iter().any(Option::is_none) {
            return self.sensor();
        }
        let x = input.iter().map(|v| v.unwrap()).collect::<Vec<_>>();
        let distance = |p: &[f32]| {
            p.iter().zip(&x).map(|(a, b)| (a - b).powi(2)).sum::<f32>() / x.len() as f32
        };
        let mut items = Vec::new();
        for a in 0..11 {
            if self.successes[a] == 0 {
                continue;
            }
            if centroid {
                let p = self.sums[a]
                    .iter()
                    .map(|v| v / self.successes[a] as f32)
                    .collect::<Vec<_>>();
                items.push((a, distance(&p)));
            } else {
                for p in &self.examples[a] {
                    items.push((a, distance(p)));
                }
            }
        }
        items.sort_by(|a, b| a.1.total_cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
        if centroid {
            return items.first().map(|&(a, _)| a);
        }
        let nearest = items.first()?.1.sqrt();
        if nearest > 0.25 {
            return None;
        }
        items.truncate(3);
        let mut votes = [0.0; 11];
        for (a, d) in items {
            votes[a] += 1.0 / (d + 0.0001);
        }
        let mut order = (0..11).collect::<Vec<_>>();
        order.sort_by(|&a, &b| votes[b].total_cmp(&votes[a]).then_with(|| a.cmp(&b)));
        if (votes[order[0]] - votes[order[1]]) / votes.iter().sum::<f32>() < 0.05 {
            return None;
        }
        Some(order[0])
    }
}
pub fn train(variant: usize) -> (Vec<u8>, Controls, usize) {
    let (records, _) = split();
    let mut rt = ScientificRuntime::new(fresh()).unwrap();
    rt.set_outcome_goal(1.0).unwrap();
    let mut controls = Controls::default();
    let mut actions = 0;
    for r in records {
        let mut w = World::new(r, variant);
        rt.observe_external(&w.pixels).unwrap();
        for _ in 0..11 {
            if rt.goal_reached().unwrap() {
                break;
            }
            let pre = w.pixels.clone();
            assert!(matches!(
                rt.step_partial(
                    |_| Some(safe()),
                    |a| {
                        let result = w.act(a, false);
                        controls.observe(a, &pre, result.1);
                        actions += 1;
                        Ok(result)
                    }
                )
                .unwrap(),
                StepOutcome::Executed { .. }
            ));
        }
        assert!(rt.goal_reached().unwrap(), "corrective feedback exhausted");
    }
    (
        rt.organism().online_checkpoint_bytes().unwrap(),
        controls,
        actions,
    )
}
#[derive(Debug)]
pub struct Report {
    pub variant: usize,
    pub training_actions: usize,
    pub tasks: usize,
    pub phase: usize,
    pub centroid: usize,
    pub knn: usize,
    pub abstentions: usize,
    pub sensing_first: usize,
    pub phase_actions: usize,
    pub centroid_actions: usize,
    pub knn_actions: usize,
    pub active_prototypes: usize,
    pub checkpoint_bytes: usize,
    pub confusion: [[usize; 11]; 10],
}
pub fn run(variant: usize) -> Report {
    let (bytes, controls, training_actions) = train(variant);
    let (_, test) = split();
    let mut rt = ScientificRuntime::new(EvoPhase::from_online_checkpoint(&bytes).unwrap()).unwrap();
    rt.set_model_learning_enabled(false);
    rt.set_outcome_goal(1.0).unwrap();
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    let mut r = Report {
        variant,
        training_actions,
        tasks: test.len(),
        phase: 0,
        centroid: 0,
        knn: 0,
        abstentions: 0,
        sensing_first: 0,
        phase_actions: 0,
        centroid_actions: 0,
        knn_actions: 0,
        active_prototypes: rt.organism().phase_vector_info().unwrap().active_prototypes,
        checkpoint_bytes: bytes.len(),
        confusion: [[0; 11]; 10],
    };
    for record in test {
        let label = record.digit;
        let mut w = World::new(record.clone(), variant);
        rt.observe_external_partial(&vec![None; 64]).unwrap();
        let mut answer = 10;
        for step in 0..2 {
            match rt.step_partial(
                |_| Some(safe()),
                |a| {
                    if step == 0 && w.roles[a] == 10 {
                        r.sensing_first += 1;
                    }
                    r.phase_actions += 1;
                    if w.roles[a] < 10 {
                        answer = w.roles[a];
                    }
                    Ok(w.act(a, true))
                },
            ) {
                Ok(StepOutcome::Executed { learned: false, .. }) => {}
                Err(RuntimeError::NoSupportedAction) => {
                    r.abstentions += 1;
                    break;
                }
                other => panic!("frozen outcome {other:?}"),
            }
            if w.terminal {
                break;
            }
        }
        r.phase += usize::from(w.terminal && rt.goal_reached().unwrap());
        r.confusion[label][answer] += 1;
        assert_eq!(
            rt.organism().phase_native_learned_fingerprint(),
            fingerprint
        );
        for centroid in [true, false] {
            let mut w = World::new(record.clone(), variant);
            let mut observed = vec![None; 64];
            let mut success = 0;
            for _ in 0..2 {
                let Some(a) = controls.choice(&observed, centroid) else {
                    break;
                };
                if centroid {
                    r.centroid_actions += 1;
                } else {
                    r.knn_actions += 1;
                }
                let result = w.act(a, true);
                observed = result.0;
                success += usize::from(result.1 >= 0.95);
                if w.terminal {
                    break;
                }
            }
            if centroid {
                r.centroid += success;
            } else {
                r.knn += success;
            }
        }
    }
    r
}
