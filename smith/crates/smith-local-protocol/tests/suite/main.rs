#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "this integration test harness is ordinary Rust (programming-model.md, section 10.2)"
)]
#[cfg(test)]
#[path = "../support/disk.rs"]
pub mod disk;

mod store;
