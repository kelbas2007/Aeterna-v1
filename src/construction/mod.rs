//! Directed construction research lane: executable programs acquired from
//! interaction, followed by prospective validation. This is not a new AGI claim.
//! The interpreter and the EvoPhase human-protection boundary are immutable.
pub mod vm;
pub mod synthesis;
pub mod constructors;
pub mod runtime;
pub mod recursive;
pub mod curriculum;
pub use runtime::{Config, ExternalPost, Forecast, Learner, ProtectedConstructionRuntime, SnapshotReport};
pub use vm::{Error, Result};
mod exact_completion;
pub mod epistemic;
