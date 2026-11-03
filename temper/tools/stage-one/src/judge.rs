//! Stage-one judge: `temper/docs/design/domain/testing.md`, 6.5.
use std::process::ExitCode;

/// The judge command, supplied by its implementation task.
#[must_use]
pub fn main(_args: &[String]) -> ExitCode {
    eprintln!("temper-stage-one: judge is not built");
    ExitCode::from(2)
}

/// The verdict command, supplied by the judge implementation task.
#[must_use]
pub fn verdict_main(_args: &[String]) -> ExitCode {
    eprintln!("temper-stage-one: verdict is not built");
    ExitCode::from(2)
}
