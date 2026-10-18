//! Canonical application entry point. The compatibility binary uses the same implementation.
#[path = "bin/aeterna-unified.rs"]
mod application;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    application::main()
}
