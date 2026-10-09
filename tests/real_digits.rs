#[path = "../examples/support/digits.rs"]
mod digits;
#[test]
fn real_handwriting_uses_factual_feedback_opaque_motors_and_frozen_sensing() {
    for variant in 0..3 {
        let r = digits::run(variant);
        println!("{r:?}");
        assert_eq!(r.variant, variant);
        assert!(r.centroid <= r.tasks && r.knn <= r.tasks && r.abstentions <= r.tasks);
        assert_eq!(r.tasks, 364);
        assert!(r.training_actions <= 1433 * 11);
        assert_eq!(r.sensing_first, r.tasks);
        assert!(
            r.phase_actions <= 2 * r.tasks
                && r.centroid_actions <= 2 * r.tasks
                && r.knn_actions <= 2 * r.tasks
        );
        assert!(
            r.phase * 100 >= 80 * r.tasks,
            "fixed 80% capability target failed: {r:?}"
        );
        assert!(r.active_prototypes <= 352 && r.checkpoint_bytes < 16 * 1024 * 1024);
        assert_eq!(r.confusion.iter().flatten().sum::<usize>(), r.tasks);
    }
}
