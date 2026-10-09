#[path = "../examples/support/real_programs.rs"]
mod recorded;
#[test]
fn acquired_program_pipeline_on_recorded_images_and_motion_signals() {
    for corpus in [recorded::Corpus::Images, recorded::Corpus::Signals] {
        let r = recorded::run(corpus);
        println!("{r:?}");
        // Pipeline integrity is separate from the declared usefulness targets.
        assert_eq!(r.correct + r.wrong + r.abstentions, r.tasks);
        assert!(r.correct > 0, "no recorded-data transfer at all");
        assert_eq!(r.sensing_first, r.tasks);
        assert!(r.test_actions <= 2 * r.tasks);
        assert!(r.nearest_actions <= 2 * r.tasks && r.linear_actions <= 2 * r.tasks);
        assert!(r.nearest <= r.tasks && r.linear <= r.tasks && r.primitives <= 4);
        assert!(r.checkpoint_bytes < 16 * 1024 * 1024 && r.constructor_updates > 0);
        println!(
            "open_usefulness_target={}",
            if r.useful { "PASS" } else { "FAIL" }
        );
    }
}
