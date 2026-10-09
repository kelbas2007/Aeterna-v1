#[path = "support/digits.rs"]
mod digits;
fn main() {
    for variant in 0..3 {
        let r = digits::run(variant);
        println!("variant={} tuition={} heldout={} phase={} centroid={} knn={} abstentions={} sensing={} actions={}/{}/{} prototypes={} checkpoint_bytes={}",r.variant,r.training_actions,r.tasks,r.phase,r.centroid,r.knn,r.abstentions,r.sensing_first,r.phase_actions,r.centroid_actions,r.knn_actions,r.active_prototypes,r.checkpoint_bytes);
        println!("confusion={:?}", r.confusion);
    }
}
