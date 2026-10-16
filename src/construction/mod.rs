//! Directed construction: factual discrepancy -> typed program -> prospective
//! validation -> acquired search schema. Opt-in research, not AGI qualification.
//! See docs/DIRECTED_CONSTRUCTION.md for contracts and implementation limits.
pub mod vm;
pub mod synthesis;
pub mod constructors;
pub mod runtime;

pub use runtime::{Config, ExternalPost, Forecast, Learner, ProtectedConstructionRuntime, SnapshotReport};
pub use vm::{Error, Result};
