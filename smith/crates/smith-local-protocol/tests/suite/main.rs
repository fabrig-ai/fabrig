#![expect(
    clippy::disallowed_methods,
    reason = "this integration test harness is ordinary Rust (programming-model.md, section 10.2)"
)]
#[cfg(test)]
pub mod disk;

mod store;
