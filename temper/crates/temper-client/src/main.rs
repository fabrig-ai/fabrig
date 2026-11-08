//! The deployment's client entry point (temper/docs/design/protocol/README.md, section 4).
#![forbid(unsafe_code)]

use std::process::ExitCode;

fn main() -> ExitCode {
    skein_fail_stop::install();
    match temper_client::configure(&skein_shell::arguments()).and_then(temper_client::run) {
        Ok(code) => ExitCode::from(code),
        Err(why) => {
            eprint!("{why}");
            ExitCode::FAILURE
        }
    }
}
