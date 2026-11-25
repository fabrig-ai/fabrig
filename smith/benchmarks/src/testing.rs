//! Native adapters and request fixtures for the benchmark test binaries.

pub(crate) mod request_fixture;
pub use request_fixture::request;

pub use crate::run::testing::{record_preparation_refusal, record_scratch_work_with};

pub fn verify_seed_digest(digest: Result<String, crate::Refusal>, expected: &str) -> Result<(), crate::Fault> {
    crate::isolate::verify_seed_digest(digest, expected)
}

pub mod arm {
    pub use crate::arm::testing::{build_lock, git, output, outside_repositories};
}

pub mod batch {
    pub use crate::batch::testing::{DispatchFiles, dispatch_result, run};
}

pub mod task;

pub mod validate {
    pub use crate::validate::testing::{ReplayPaths, validate_catalogue, validate_catalogue_with};
}
