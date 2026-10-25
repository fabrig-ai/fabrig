//! The crate's test fixture: a binary whose second thread panics.
//!
//! Its exit code 3 says the panic ended that thread alone, which fail-stop
//! never allows; with the hook installed, the process aborts first.

use std::process::ExitCode;

#[expect(clippy::panic, reason = "the panic is what the fixture is for")]
fn main() -> ExitCode {
    skein_fail_stop::install();
    match std::thread::spawn(|| panic!("the fixture's thread panics")).join() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::from(3),
    }
}
