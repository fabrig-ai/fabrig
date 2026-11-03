//! Stage-one check: `temper/docs/design/domain/testing.md`, 6.5.
use std::process::ExitCode;

/// The check command, supplied by its implementation task.
#[must_use]
pub fn main(_args: &[String]) -> ExitCode {
    eprintln!("temper-stage-one: check is not built");
    ExitCode::from(2)
}
