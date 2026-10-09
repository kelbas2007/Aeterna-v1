#[path = "../examples/support/iris.rs"]
mod iris;
#[test]
fn recorded_iris_measurements_use_opaque_actions_partial_sensing_and_a_matched_stump() {
    let reports = iris::run();
    assert_eq!(reports.len(), 3);
    for r in &reports {
        println!("{r:?}");
        assert_eq!(r.tasks, 40);
        assert!(r.training_actions <= 540);
        assert!(r.phase_actions <= 2 * r.tasks && r.stump_actions <= 2 * r.tasks);
        assert!(r.phase_success <= r.tasks && r.stump_success <= r.tasks);
        assert!(r.retained_examples <= 3 * (64 + 6));
        assert!(r.checkpoint_bytes < 16 * 1024 * 1024);
        // Capability regression on separable classes; the overlapping pair's
        // failures remain visible instead of weakening a qualification claim.
        if r.pair[0] == 0 {
            assert_eq!(r.trained_models, 3);
            assert_eq!(r.sensing_first, 40);
            assert!(r.phase_success >= 30);
        }
    }
}
