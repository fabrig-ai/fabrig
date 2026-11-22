//! Native adapters and request fixtures for the benchmark test binaries.

pub(crate) mod request_fixture;
pub use request_fixture::request;

pub use crate::run::testing::record_preparation_refusal;

pub fn verify_seed_digest(actual: Result<String, crate::Refusal>, expected: &str) -> Result<(), crate::Fault> {
    crate::isolate::verify_seed_digest(actual, expected)
}

pub mod arm {
    pub use crate::arm::testing::{build_lock, git, output, outside_repositories};
}

pub mod batch {
    pub use crate::batch::testing::run;
}

pub mod validate {
    pub use crate::validate::testing::validate_catalogue;
}
