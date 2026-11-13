//! Worker-root stories at the protocol's boundary (jig's `domain/hosts/README.md`, section 4 and `domain/hosts/hub.md`
//! ; `domain/worker/README.md`, sections 2, 4.1, 4.3 and
//! 6).

mod admission;
mod entrance;
pub(crate) mod fixtures;
mod harness;
mod jobs;
mod lifecycle;
mod limits;
mod live;
mod model_failures;
mod orders;
mod stopping;

mod translate;
