//! The checked startup joined to the component (protocol/services.md, 4.1).
pub use temper_engine_service_startup::startup::*;
mod start;
pub use start::{restoring_prepared, restoring_with, with_setup_given};
