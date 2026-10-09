#[path = "../examples/support/induction.rs"]
mod induction;
#[test]
fn experience_builds_all_two_input_functions_and_deeper_interactions() {
    let mut failed = Vec::new();
    for f in (0..16).map(induction::Function::Two).chain([
        induction::Function::Parity,
        induction::Function::Majority,
        induction::Function::Selector,
    ]) {
        for variant in 0..2 {
            let r = induction::run(f, variant);
            println!("{r:?}");
            if r.correct * 100 < 95 * r.tasks {
                failed.push(format!("{r:?}"));
            }
            assert_eq!(r.sensing_first, r.tasks);
            assert!(
                r.actions <= 2 * r.tasks
                    && r.prototype_actions <= 2 * r.tasks
                    && r.linear_actions <= 2 * r.tasks
            );
            assert!(r.nodes <= 45 && r.checkpoint_bytes < 1024 * 1024);
            assert!(
                r.tuition > 0
                    && r.prototype <= r.tasks
                    && r.linear <= r.tasks
                    && r.abstentions <= r.tasks
            );
            assert_eq!(r.variant, variant);
            let _ = r.function;
        }
    }
    assert!(
        failed.is_empty(),
        "fixed criteria failed: {}",
        failed.join("\n")
    );
}
