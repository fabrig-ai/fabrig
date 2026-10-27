//! The crate's test fixture: a binary whose second thread panics.
//!
//! Its exit code 3 says the panic ended that thread alone, which fail-stop
//! never allows; with the hook installed, the process aborts first.

use std::io::Write;
use std::process::ExitCode;

fn main() -> ExitCode {
    skein_fail_stop::install();
    let mut args = std::env::args().skip(1);
    let payload = args.next().unwrap_or_default();
    let mut thread = std::thread::Builder::new();
    if let Some(name) = args.next() {
        thread = thread.name(name);
    }
    match thread.spawn(move || panic_in_fixture(&payload)).expect("the fixture's thread starts").join() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::from(3),
    }
}

struct UnwindGuard;

impl Drop for UnwindGuard {
    fn drop(&mut self) {
        let _write_result = std::io::stderr().lock().write_all(b"fixture unwound\n");
    }
}

#[inline(never)]
#[expect(clippy::panic, reason = "the panic is what the fixture is for")]
fn panic_in_fixture(payload: &str) {
    let _guard = UnwindGuard;
    match payload {
        "string" => std::panic::panic_any(String::from("the fixture's owned message")),
        "any" => std::panic::panic_any(7_u8),
        _ => panic!("the fixture's thread panics"),
    }
}
