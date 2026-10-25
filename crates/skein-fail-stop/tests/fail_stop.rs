//! A panic on any thread of a binary built to unwind aborts the process,
//! once the default hook has reported it (programming-model.md, 2).
#![cfg(test)]

use std::os::unix::process::ExitStatusExt;
use std::process::Command;

/// The abort signal, on Linux and on aarch64-apple-darwin.
const SIGABRT: i32 = 6;

#[test]
fn a_panic_on_a_second_thread_aborts_the_process_after_its_report() {
    // Cargo and nextest name a package's binaries to its tests when they run.
    let fixture = std::env::var_os("CARGO_BIN_EXE_fail_stop_fixture").expect("the fixture's path, set at run time");
    let output = Command::new(fixture).output().expect("the fixture starts");
    assert_eq!(output.status.signal(), Some(SIGABRT), "the process aborts: {:?}", output.status);
    let report = std::str::from_utf8(&output.stderr).expect("the report is UTF-8");
    assert!(report.contains("the fixture's thread panics"), "the default hook reported first: {report}");
}
