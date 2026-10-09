// External task definitions and comparators only. No labels enter EvoPhase.
use aeterna_v1::carrier::{
    PhaseInductionConfig, PhaseNativeConfig, PhaseOnlineConfig, PhaseVectorConfig,
};
use aeterna_v1::scientific_runtime::{RuntimeError, ScientificRuntime, StepOutcome};
use aeterna_v1::{EvoConfig, EvoPhase, HumanProtectionEvidence};
use std::collections::VecDeque;
#[derive(Debug, Clone, Copy)]
pub enum Function {
    Two(u8),
    Parity,
    Majority,
    Selector,
}
impl Function {
    pub fn arity(self) -> usize {
        if matches!(self, Self::Two(_)) {
            2
        } else {
            3
        }
    }
    pub fn outcome(self, bits: usize) -> usize {
        match self {
            Self::Two(table) => usize::from(table & (1 << (bits & 3)) != 0),
            Self::Parity => (bits & 7).count_ones() as usize % 2,
            Self::Majority => usize::from((bits & 7).count_ones() >= 2),
            Self::Selector => {
                if bits & 1 == 0 {
                    (bits >> 1) & 1
                } else {
                    (bits >> 2) & 1
                }
            }
        }
    }
}
pub fn roles(variant: usize) -> Vec<usize> {
    (0..3).map(|a| (a * 2 + variant) % 3).collect()
}
pub fn features(
    f: Function,
    bits: usize,
    distractors: usize,
    variant: usize,
    heldout: bool,
) -> Vec<f32> {
    let width = f.arity() + 2;
    let raw = (0..width)
        .map(|j| {
            let on = if j < f.arity() {
                bits >> j & 1 != 0
            } else {
                distractors >> (j - f.arity()) & 1 != 0
            };
            if heldout {
                if on {
                    0.72
                } else {
                    0.28
                }
            } else if on {
                0.8
            } else {
                0.2
            }
        })
        .collect::<Vec<_>>();
    (0..width).map(|j| raw[(j + variant * 2) % width]).collect()
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
pub fn fresh(width: usize) -> EvoPhase {
    let mut e = EvoPhase::new(EvoConfig {
        sensory_cells: width,
        motor_cells: 3,
        dormant_cells: 64,
        ..Default::default()
    });
    e.enable_phase_native_planning(PhaseNativeConfig::default());
    assert!(e.enable_phase_native_online_learning(PhaseOnlineConfig { max_states: 1 }));
    assert!(e.enable_phase_vector_learning(PhaseVectorConfig {
        slots_per_motor: 1,
        exploration_observations: 64,
        ..Default::default()
    }));
    assert!(e.enable_phase_induction(PhaseInductionConfig::default()));
    e
}
#[derive(Default)]
pub struct Controls {
    examples: [VecDeque<Vec<f32>>; 3],
    sums: [Vec<f32>; 3],
    counts: [usize; 3],
    observations: [usize; 3],
    full_post: [usize; 3],
    positives: [usize; 3],
}
impl Controls {
    fn observe(&mut self, a: usize, pre: Option<&[f32]>, post_full: bool, outcome: f32) {
        self.observations[a] += 1;
        self.full_post[a] += usize::from(post_full);
        self.positives[a] += usize::from(outcome >= 0.95);
        if outcome < 0.95 {
            return;
        }
        let Some(x) = pre else {
            return;
        };
        self.counts[a] += 1;
        if self.sums[a].is_empty() {
            self.sums[a] = vec![0.0; x.len()];
        }
        for (s, &v) in self.sums[a].iter_mut().zip(x) {
            *s += v;
        }
        if self.examples[a].len() == 32 {
            self.examples[a].pop_front();
        }
        self.examples[a].push_back(x.to_vec());
    }
    fn sensor(&self) -> Option<usize> {
        let found = (0..3)
            .filter(|&a| {
                self.observations[a] >= 4
                    && self.full_post[a] == self.observations[a]
                    && self.positives[a] == 0
            })
            .collect::<Vec<_>>();
        (found.len() == 1).then(|| found[0])
    }
    fn choose(&self, x: &[f32], linear: bool) -> Option<usize> {
        let mut scores = Vec::new();
        for a in 0..3 {
            if self.counts[a] == 0 {
                continue;
            }
            if linear {
                let c = self.sums[a]
                    .iter()
                    .map(|s| s / self.counts[a] as f32)
                    .collect::<Vec<_>>();
                let distance = c.iter().zip(x).map(|(v, w)| (v - w).powi(2)).sum::<f32>();
                scores.push((distance, a));
            } else {
                for p in &self.examples[a] {
                    let d = p.iter().zip(x).map(|(v, w)| (v - w).powi(2)).sum::<f32>();
                    scores.push((d, a));
                }
            }
        }
        scores.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
        scores.first().map(|&(_, a)| a)
    }
}
pub fn teach(
    rt: &mut ScientificRuntime,
    x: &[f32],
    correct: usize,
    mapping: &[usize],
    mut controls: Option<&mut Controls>,
) -> usize {
    rt.observe_external(x).unwrap();
    let mut observed = true;
    let mut actions = 0;
    for _ in 0..11 {
        if rt.goal_reached().unwrap() {
            break;
        }
        let actual_pre = if observed { Some(x.to_vec()) } else { None };
        let result = rt
            .step_partial(
                |_| Some(safe()),
                |a| {
                    actions += 1;
                    let outcome = f32::from(mapping[a] == correct);
                    if let Some(c) = controls.as_mut() {
                        c.observe(a, actual_pre.as_deref(), mapping[a] == 2, outcome);
                    }
                    observed = mapping[a] == 2;
                    Ok((
                        if observed {
                            x.iter().copied().map(Some).collect()
                        } else {
                            vec![None; x.len()]
                        },
                        outcome,
                    ))
                },
            )
            .unwrap();
        assert!(matches!(result, StepOutcome::Executed { .. }));
    }
    assert!(
        rt.goal_reached().unwrap(),
        "factual tuition exhausted, x={x:?}, correct={correct}, programs={:?}",
        rt.organism().phase_induction_programs()
    );
    actions
}
pub fn train(f: Function, variant: usize) -> (Vec<u8>, Controls, usize) {
    let mut rt = ScientificRuntime::new(fresh(f.arity() + 2)).unwrap();
    rt.set_outcome_goal(1.0).unwrap();
    let mapping = roles(variant);
    let mut controls = Controls::default();
    let mut actions = 0;
    for cycle in 0..16 {
        for bits in 0..(1 << f.arity()) {
            for k in 0..4 {
                let nuisance = (k + cycle) % 4;
                let x = features(f, bits, nuisance, variant, false);
                actions += teach(&mut rt, &x, f.outcome(bits), &mapping, Some(&mut controls));
            }
        }
    }
    (
        rt.organism().online_checkpoint_bytes().unwrap(),
        controls,
        actions,
    )
}
#[derive(Debug)]
pub struct Report {
    pub function: Function,
    pub variant: usize,
    pub tasks: usize,
    pub correct: usize,
    pub prototype: usize,
    pub prototype_actions: usize,
    pub linear: usize,
    pub linear_actions: usize,
    pub abstentions: usize,
    pub sensing_first: usize,
    pub actions: usize,
    pub tuition: usize,
    pub nodes: usize,
    pub checkpoint_bytes: usize,
}
pub fn evaluate(
    bytes: &[u8],
    f: Function,
    variant: usize,
    controls: Option<&Controls>,
    tuition: usize,
) -> Report {
    let mut rt = ScientificRuntime::new(EvoPhase::from_online_checkpoint(bytes).unwrap()).unwrap();
    rt.set_outcome_goal(1.0).unwrap();
    rt.set_model_learning_enabled(false);
    let fingerprint = rt.organism().phase_native_learned_fingerprint();
    let mapping = roles(variant);
    let mut r = Report {
        function: f,
        variant,
        tasks: 0,
        correct: 0,
        prototype: 0,
        prototype_actions: 0,
        linear: 0,
        linear_actions: 0,
        abstentions: 0,
        sensing_first: 0,
        actions: 0,
        tuition,
        nodes: rt
            .organism()
            .phase_induction_programs()
            .iter()
            .map(|p| p.nodes.len())
            .sum(),
        checkpoint_bytes: bytes.len(),
    };
    for bits in 0..(1 << f.arity()) {
        for nuisance in 0..4 {
            let x = features(f, bits, nuisance, variant, true);
            let label = f.outcome(bits);
            let mut terminal = false;
            rt.observe_external_partial(&vec![None; x.len()]).unwrap();
            r.tasks += 1;
            for step in 0..2 {
                let result = rt.step_partial(
                    |_| Some(safe()),
                    |a| {
                        assert!(!terminal);
                        r.actions += 1;
                        if mapping[a] == 2 {
                            if step == 0 {
                                r.sensing_first += 1;
                            }
                            Ok((x.iter().copied().map(Some).collect(), 0.0))
                        } else {
                            terminal = true;
                            Ok((vec![None; x.len()], f32::from(mapping[a] == label)))
                        }
                    },
                );
                match result {
                    Ok(StepOutcome::Executed { learned: false, .. }) => {}
                    Err(RuntimeError::NoSupportedAction) => {
                        r.abstentions += 1;
                        break;
                    }
                    other => panic!("{other:?}"),
                }
                if terminal {
                    break;
                }
            }
            r.correct += usize::from(terminal && rt.goal_reached().unwrap());
            if let Some(c) = controls {
                for linear in [true, false] {
                    let mut actions = 0;
                    let mut correct = 0;
                    if let Some(sensor) = c.sensor() {
                        actions += 1;
                        if mapping[sensor] == 2 {
                            if let Some(a) = c.choose(&x, linear) {
                                actions += 1;
                                correct = usize::from(mapping[a] == label);
                            }
                        }
                    }
                    if linear {
                        r.linear += correct;
                        r.linear_actions += actions;
                    } else {
                        r.prototype += correct;
                        r.prototype_actions += actions;
                    }
                }
            }
            assert_eq!(
                fingerprint,
                rt.organism().phase_native_learned_fingerprint()
            );
        }
    }
    r
}
pub fn run(f: Function, variant: usize) -> Report {
    let (bytes, c, actions) = train(f, variant);
    evaluate(&bytes, f, variant, Some(&c), actions)
}
