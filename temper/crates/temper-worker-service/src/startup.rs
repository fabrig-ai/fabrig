//! The worker startup (temper/docs/design/protocol/services.md, 5).
pub mod worker;
pub use temper_service_core::startup::problems::{SharedProblems as Problems, SharedWhat as What, lines};
pub use temper_service_core::startup::{Shape, nearest};
pub use worker::{NamedWorker, WorkerPart, check, shape, worker_alone, worker_part};
