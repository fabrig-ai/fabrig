//! The deployment's client entry point (temper/docs/design/protocol/binaries.md).
#![forbid(unsafe_code)]

use std::process::ExitCode;

#[expect(
    clippy::disallowed_methods,
    reason = "cli C3: the client's arguments configure its command; temper's table will read them"
)]
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
