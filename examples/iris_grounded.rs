#[path = "support/iris.rs"]
mod iris;
fn main() {
    for r in iris::run() {
        println!("pair={:?} tuition={} models={} heldout={} phase={} stump={} phase_actions={} stump_actions={} abstentions={} sensing_first={} retained={} checkpoint_bytes={}", r.pair, r.training_actions, r.trained_models, r.tasks, r.phase_success, r.stump_success, r.phase_actions, r.stump_actions, r.abstentions, r.sensing_first, r.retained_examples, r.checkpoint_bytes);
    }
}
