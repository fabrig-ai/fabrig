//! The worker startup (temper/docs/design/protocol/services.md, 5).
pub mod worker;
pub use worker::{NamedWorker, WorkerPart, worker_part, worker_alone, shape, check};
pub use temper_service_core::startup::{Shape, nearest};
pub use temper_service_core::startup::problems::{SharedProblems as Problems, SharedWhat as What, lines};
