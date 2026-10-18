use aeterna_v1::construction::search_policy::*;
use aeterna_v1::construction::vm::{Context, Error, Expr, Library, Meter, Op};

#[allow(dead_code)]
#[path = "../examples/rsp1_support/mod.rs"]
mod curriculum;

fn grid(axis: usize) -> SynthesisTask {
    SynthesisTask {
        examples: (-2..=2).flat_map(|x| (-2..=2).map(move |y| {
            let input = vec![x, y];
            Example { output: input[axis], input }
        })).collect(),
        library: Library::default(),
    }
}

fn limits() -> SearchLimits {
    SearchLimits { work: 30_000, evaluations: 64, max_nodes: 11, frontier: 64, edits_per_expansion: 12 }
}

fn config() -> EvolutionConfig {
    EvolutionConfig {
        object: limits(),
        meta: SearchLimits { work: 5_000_000, evaluations: 20, max_nodes: 23, frontier: 64, edits_per_expansion: 12 },
        max_lifetime_work: 100_000_000,
    }
}

#[test]
fn equal_scalar_loss_exposes_different_residual_bindings() {
    let mut a = grid(0);
    let mut b = grid(1);
    let x = a.observe(&Expr::integer(0), &mut Meter::new(50_000), u32::MAX).unwrap();
    let y = b.observe(&Expr::integer(0), &mut Meter::new(50_000), u32::MAX).unwrap();
    assert_eq!(x.loss, y.loss);
    assert_eq!(x.feedback[4], 0);
    assert_eq!(y.feedback[4], 1);
    assert_eq!(x.feedback[5], 1024);
    assert_eq!(y.feedback[5], 1024);
}

#[test]
fn ordering_residuals_are_derived_from_examples() {
    let mut t = grid(0);
    for row in &mut t.examples { row.output = if row.input[0] < row.input[1] { 7 } else { -7 }; }
    let observation = t.observe(&Expr::integer(0), &mut Meter::new(50_000), u32::MAX).unwrap();
    assert_eq!(&observation.feedback[6..8], &[0, 1]);
    assert!(observation.feedback[8] > 0);
}

#[test]
fn residual_bindings_change_actual_edits() {
    let p = SearchPolicy { variation: Expr::Input(14), ..Default::default() };
    let a = search(&p, Expr::integer(0), &mut grid(0), &limits(), 9).unwrap();
    let b = search(&p, Expr::integer(0), &mut grid(1), &limits(), 9).unwrap();
    assert_ne!(a.trace_hash, b.trace_hash);
    assert!(a.objective_work > 0 && b.objective_work > 0);
}

#[test]
fn observed_binding_can_select_the_operand_instead_of_drawing_it_randomly() {
    let variation = Expr::bin(Op::Sub, Expr::integer(0), Expr::bin(Op::Add, Expr::integer(1),
        Expr::bin(Op::Mul, Expr::integer(8), Expr::Input(14))));
    let p = SearchPolicy { variation, location: Expr::integer(0), ..Default::default() };
    for axis in 0..2 {
        let r = search(&p, Expr::integer(0), &mut grid(axis), &limits(), 44).unwrap();
        assert!(r.solved);
        assert_eq!(r.program, Expr::Input(axis));
        assert_eq!(r.evaluations, 2);
    }
}

#[test]
fn diagnostics_are_lazy_and_metered() {
    let mut t = grid(0);
    let mut plain = Meter::new(50_000);
    let mut observed = Meter::new(50_000);
    let a = t.observe(&Expr::integer(0), &mut plain, 0).unwrap();
    let b = t.observe(&Expr::integer(0), &mut observed, u32::MAX).unwrap();
    assert_eq!(a.loss, b.loss);
    assert_eq!(a.feedback, [0; FEEDBACK_FEATURES]);
    assert!(observed.spent > plain.spent);
    let mut insufficient = Meter::new(plain.spent);
    assert_eq!(t.observe(&Expr::integer(0), &mut insufficient, u32::MAX), Err(Error::Budget));
    assert_eq!(insufficient.spent, insufficient.limit);
}

#[test]
fn residual_arithmetic_handles_extreme_predictions() {
    let o = grid(0).observe(&Expr::integer(i64::MIN), &mut Meter::new(50_000), u32::MAX).unwrap();
    assert_eq!(o.feedback[3], 1_000_000);
    assert_eq!(o.feedback[2], 1024);
}

#[test]
fn malformed_observations_are_rejected_before_feature_indexing() {
    let mut t = grid(0);
    t.examples[1].input.clear();
    assert_eq!(t.observe(&Expr::integer(0), &mut Meter::new(50_000), u32::MAX), Err(Error::InvalidData));
}

#[test]
fn probes_cannot_be_search_examples() {
    let t = grid(0);
    assert_eq!(ValidationCase { support: t.clone(), probes: t }.validate(), Err(Error::InvalidData));
}

#[test]
fn transfer_and_producer_evidence_are_mandatory_before_spending() {
    for field in 0..2 {
        let mut g = curriculum::admission(700_000);
        if field == 0 { g.transfer.clear(); } else { g.producers.clear(); }
        let mut state = RecursiveSearch::new(config()).unwrap();
        let prior = state.clone();
        assert_eq!(state.improve(&[grid(0)], &g, 1).unwrap_err(), Error::Capacity);
        assert_eq!(state, prior);
    }
}

#[test]
fn hidden_examples_and_producer_exams_are_single_use() {
    let mut state = RecursiveSearch::new(config()).unwrap();
    let g = curriculum::admission(800_000);
    state.improve(&[grid(0)], &g, 2).unwrap();
    for forbidden in [&g.transfer[0].probes, &g.producers[0].examination[0].probes] {
        assert_eq!(state.improve(&[forbidden.clone()], &curriculum::admission(900_000), 3).unwrap_err(), Error::Stale);
    }
}

#[test]
fn reordering_consumed_examples_cannot_reopen_admission() {
    let mut state = RecursiveSearch::new(config()).unwrap();
    let mut g = curriculum::admission(850_000);
    state.improve(&[grid(0)], &g, 2).unwrap();
    for c in &mut g.fresh { c.support.examples.reverse(); c.probes.examples.reverse(); }
    assert_eq!(state.improve(&[grid(0)], &g, 3).unwrap_err(), Error::Stale);
}

#[test]
fn qualifications_cannot_be_removed_from_new_state() {
    let mut state = RecursiveSearch::new(config()).unwrap();
    state.improve(&[grid(0)], &curriculum::admission(1_000_000), 3).unwrap();
    state.validate().unwrap();
    let mut bytes = serde_json::to_value(&state).unwrap();
    bytes["audits"][0].as_object_mut().unwrap().remove("qualification");
    let altered: RecursiveSearch = serde_json::from_value(bytes).unwrap();
    assert_eq!(altered.validate(), Err(Error::InvalidData));
}

#[test]
fn old_checkpoint_remains_readable_but_cannot_bypass_new_admission() {
    let state = RecursiveSearch::new(config()).unwrap();
    let mut legacy = serde_json::to_value(&state).unwrap();
    legacy.as_object_mut().unwrap().remove("admission_version");
    let mut loaded: RecursiveSearch = serde_json::from_value(legacy.clone()).unwrap();
    loaded.validate().unwrap();
    assert_eq!(serde_json::to_value(&loaded).unwrap(), legacy);
    assert_eq!(loaded.improve(&[grid(0)], &curriculum::admission(2_000_000), 1).unwrap_err(), Error::Stale);
    // Solving remains available; the stored legacy policy itself is unchanged.
    assert!(search(loaded.policy(), Expr::integer(0), &mut grid(0), &limits(), 1).is_ok());
}

#[test]
fn complete_actual_work_is_charged_including_probe_checks() {
    let mut state = RecursiveSearch::new(config()).unwrap();
    let a = state.improve(&[grid(0)], &curriculum::admission(3_000_000), 11).unwrap().clone();
    let q = a.qualification.as_ref().unwrap();
    let pair_work = |p: &PairedSuite| p.before.work() + p.after.work();
    let expected = a.production.work + pair_work(&q.fresh) + pair_work(&q.transfer) + pair_work(&q.retention)
        + q.producers.iter().map(|p| p.before_creation.work + p.after_creation.work + pair_work(&p.children)).sum::<u64>();
    assert_eq!(state.work(), expected);
    assert_eq!(a.consumed_work, expected);
    assert!(q.fresh.after.probes.iter().all(|p| p.probe_work > 0));
    assert!(!a.accepted || (q.direct_pass(&config().object) && q.producer_pass(&config().object)));
}

// Build a paired comparison from real searches and withheld predictions. A
// policy is deliberately bad on some tasks; no claimed learned-policy witness.
fn assessed(p: &SearchPolicy, task: &ValidationCase, seed: u64) -> VerifiedSuite {
    let score = evaluate(p, &[task.support.clone()], &limits(), seed).unwrap();
    let program = &score.runs[0].program;
    let mut m = Meter::new(limits().work);
    let mut errors = 0;
    for row in &task.probes.examples {
        m.charge(1).unwrap();
        let y = program.eval(Context { input: &row.input, action: 0, state: &[], args: &[] }, &task.probes.library, &mut m).and_then(|v| v.int());
        if y != Ok(row.output) { errors += 1; }
    }
    VerifiedSuite { search: score, probes: vec![VerifiedRun { case_id: 0, seed, probe_rows: task.probes.examples.len(), probe_errors: errors, probe_work: m.spent }] }
}

fn real_tradeoff() -> (PairedSuite, PairedSuite) {
    let cases = curriculum::cases(4_000_000, 12, 0);
    for op in 0..8 {
        let policy = SearchPolicy { variation: Expr::integer(op), ..Default::default() };
        let mut gain = None;
        let mut loss = None;
        for seed in 1..=8 {
            for task in &cases {
                let pair = PairedSuite { before: assessed(&SearchPolicy::default(), task, seed), after: assessed(&policy, task, seed) };
                if pair.after.solved() > pair.before.solved() { gain = Some(pair.clone()); }
                if pair.after.solved() < pair.before.solved() { loss = Some(pair); }
                if gain.is_some() && loss.is_some() { return (gain.unwrap(), loss.unwrap()); }
            }
        }
    }
    panic!("adversarial search fixture did not produce a tradeoff");
}

fn empty_pair() -> PairedSuite {
    let s = VerifiedSuite { search: SuiteScore { tasks: 0, solved: 0, value: 0, work: 0, runs: vec![] }, probes: vec![] };
    PairedSuite { before: s.clone(), after: s }
}

#[test]
fn local_improvement_cannot_replace_a_policy_that_retains_an_old_solution() {
    let (gain, loss) = real_tradeoff();
    let mut q = QualificationAudit { fresh: gain.clone(), transfer: gain, retention: empty_pair(), producers: vec![], added_retention: vec![] };
    assert!(q.direct_pass(&limits()));
    q.retention = loss;
    assert!(!q.direct_pass(&limits()));
}

#[test]
fn local_improvement_cannot_hide_failed_transfer() {
    let (gain, loss) = real_tradeoff();
    let q = QualificationAudit { fresh: gain, transfer: loss, retention: empty_pair(), producers: vec![], added_retention: vec![] };
    assert!(!q.direct_pass(&limits()));
}

#[test]
fn a_solver_gain_is_insufficient_without_a_producer_gain() {
    let (gain, _) = real_tradeoff();
    let q = QualificationAudit { fresh: gain.clone(), transfer: gain, retention: empty_pair(), producers: vec![], added_retention: vec![] };
    assert!(q.direct_pass(&limits()));
    assert!(!q.producer_pass(&limits()));
}

#[test]
fn all_feedback_coordinates_are_executable_policy_inputs() {
    for i in LEGACY_FEATURES..FEATURES {
        SearchPolicy { priority: Expr::bin(Op::Add, Expr::Input(0), Expr::Input(i)), ..Default::default() }.validate().unwrap();
    }
}

#[test]
fn producer_test_executes_children_from_the_same_starting_target() {
    let cfg = config();
    let g = curriculum::admission(5_000_000);
    let before = SearchPolicy::default();
    let after = curriculum::loss_size();
    let pair = compare_producers(&before, &after, &g.producers[0], &cfg.object, &cfg.meta, 55).unwrap();
    for (creation, producer, child) in [(&pair.before_creation, &before, &pair.children.before), (&pair.after_creation, &after, &pair.children.after)] {
        let mut program = SearchPolicy::default().code();
        for edit in &creation.edits_to_winner { program = edit.apply(&program).unwrap(); }
        assert_eq!(program, creation.program);
        assert_eq!(creation.policy_id, producer.fingerprint());
        let descendant = SearchPolicy::from_code(&program).unwrap();
        assert!(child.search.runs.iter().all(|r| r.policy_id == descendant.fingerprint()));
        assert!(creation.objective_work > 0 && child.work() > 0);
    }
    assert_eq!(pair.children.before.probes.iter().map(|p| (p.case_id, p.seed)).collect::<Vec<_>>(),
               pair.children.after.probes.iter().map(|p| (p.case_id, p.seed)).collect::<Vec<_>>());
}

#[test]
fn identical_producers_cannot_be_reported_as_recursive_improvement() {
    let cfg = config();
    let g = curriculum::admission(6_000_000);
    let policy = SearchPolicy::default();
    let mut producers = Vec::new();
    for (i, case) in g.producers.iter().enumerate() {
        let pair = compare_producers(&policy, &policy, case, &cfg.object, &cfg.meta, i as u64 + 1).unwrap();
        assert_eq!(pair.before_creation, pair.after_creation);
        assert_eq!(pair.children.before, pair.children.after);
        producers.push(pair);
    }
    let q = QualificationAudit { fresh: empty_pair(), transfer: empty_pair(), retention: empty_pair(), producers, added_retention: vec![] };
    assert!(!q.producer_pass(&cfg.object));
}

#[test]
fn accepted_history_survives_checkpoint_and_is_checked_by_the_next_admission() {
    use aeterna_v1::construction::{Config, autonomy::AutonomyPolicy};
    use aeterna_v1::scientific_runtime::ScientificRuntime;

    // Regression fixture from repair-check/run-02, not an independent
    // scientific trial. No candidate body or acceptance flag is injected.
    let mut cfg = curriculum::config();
    cfg.meta.work = 20_000_000;
    cfg.meta.evaluations = 32;
    let lessons = curriculum::tasks(203_000, 8, false).0;
    let mut runtime = ScientificRuntime::new_constructing(
        Config { width: 3, ..Default::default() }, AutonomyPolicy::default(),
        vec![0; 3], "RSP2-retention-regression",
    ).unwrap();
    runtime.enable_recursive_search(cfg.clone()).unwrap();
    assert!(runtime.improve_recursive_search(
        &lessons, &curriculum::admission(213_700_000), 203_007,
    ).unwrap().accepted);
    let retained = runtime.construction_learner().unwrap().recursive_search().unwrap().retained().to_vec();
    assert_eq!(retained.len(), 10);

    let bytes = runtime.checkpoint_constructing().unwrap();
    let mut restored = ScientificRuntime::from_constructing_checkpoint(&bytes).unwrap();
    assert_eq!(bytes, restored.checkpoint_constructing().unwrap());
    let state = restored.construction_learner().unwrap().recursive_search().unwrap();
    assert_eq!(state.retained(), retained.as_slice());
    let mut tampered = serde_json::to_value(state).unwrap();
    tampered["retained"].as_array_mut().unwrap().pop();
    let altered: RecursiveSearch = serde_json::from_value(tampered).unwrap();
    assert_eq!(altered.validate(), Err(Error::InvalidData));

    let audit = restored.improve_recursive_search(
        &lessons, &curriculum::admission(1_700_000_000), 203_008,
    ).unwrap();
    let q = audit.qualification.as_ref().unwrap();
    assert_eq!(q.retention.before.search.tasks, retained.len());
    assert_eq!(q.retention.before.solved(), retained.len());
    assert_eq!(q.retention.after.search.tasks, retained.len());
    assert!(q.retention.before.work() > 0 && q.retention.after.work() > 0);
    assert!(!audit.accepted || q.retention.after.solved() == retained.len());
    assert_eq!(q.retention.before.probes.iter().map(|p| p.seed).collect::<Vec<_>>(),
               retained.iter().map(|r| r.seed).collect::<Vec<_>>());
    restored.construction_learner().unwrap().recursive_search().unwrap().validate().unwrap();
}
