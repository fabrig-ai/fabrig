//! The deployment's agent entry point (temper/docs/design/protocol/binaries.md, 3).
#![forbid(unsafe_code)]

mod entry;

use std::process::ExitCode;

fn main() -> ExitCode {
    skein_fail_stop::install();
    entry::run()
}
