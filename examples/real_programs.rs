#[path = "support/real_programs.rs"]
mod recorded;
fn main() {
    for corpus in [recorded::Corpus::Images, recorded::Corpus::Signals] {
        let r = recorded::run(corpus);
        println!("{r:?}");
        println!(
            "open_usefulness_target={}",
            if r.useful { "PASS" } else { "FAIL" }
        );
    }
}
