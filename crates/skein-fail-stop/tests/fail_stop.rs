//! A panic on any thread of a binary built to unwind aborts the process,
//! after reporting its message and backtrace (programming-model.md, 2).
#![cfg(test)]

use std::os::unix::process::ExitStatusExt;
use std::process::{Command, Output};

/// The abort signal, on Linux and on aarch64-apple-darwin.
const SIGABRT: i32 = 6;

#[test]
fn a_panic_on_a_second_thread_aborts_the_process_after_its_report() {
    for backtrace in [None, Some("1"), Some("full")] {
        let output = panic_report(backtrace, "str", None);
        let report = core::str::from_utf8(&output.stderr).expect("the report is UTF-8");
        assert!(report.contains("the fixture's thread panics"), "the hook reported first: {report}");
        assert_report(&output, "<unnamed>", "the fixture's thread panics");
    }
}

#[test]
fn owned_and_non_string_payloads_follow_the_default_report() {
    for (payload, message) in [("string", "the fixture's owned message"), ("any", "Box<dyn Any>")] {
        let output = panic_report(None, payload, Some("fixture-worker"));
        assert_report(&output, "fixture-worker", message);
    }
}

fn panic_report(backtrace: Option<&str>, payload: &str, name: Option<&str>) -> Output {
    // Cargo and nextest name a package's binaries to its tests when they run.
    let fixture = std::env::var_os("CARGO_BIN_EXE_fail_stop_fixture").expect("the fixture's path, set at run time");
    let mut command = Command::new(fixture);
    command.arg(payload).env_remove("RUST_BACKTRACE").env_remove("RUST_LIB_BACKTRACE");
    if let Some(value) = backtrace {
        command.env("RUST_BACKTRACE", value);
    }
    if let Some(name) = name {
        command.arg(name);
    }
    command.output().expect("the fixture starts")
}

fn assert_report(output: &Output, name: &str, message: &str) {
    assert_eq!(output.status.signal(), Some(SIGABRT), "the process aborts: {:?}", output.status);
    let report = core::str::from_utf8(&output.stderr).expect("the report is UTF-8");
    let mut lines = report.lines();
    assert_eq!(lines.next(), Some(""), "the report starts on a new line: {report}");
    let header = lines.next().expect("the report has a thread line");
    let prefix = format!("thread '{name}' panicked at ");
    let location = header.strip_prefix(&prefix).expect("the header names the thread without an id");
    let mut location = location.rsplitn(4, ':');
    assert_eq!(location.next(), Some(""), "the header ends with a colon: {report}");
    for coordinate in ["column", "line"] {
        let number = location.next().expect("the location has both coordinates");
        assert!(number.parse::<u32>().expect(coordinate) > 0, "{coordinate} is positive: {report}");
    }
    let file = location.next().expect("the location names the source file");
    assert!(file.ends_with("src/bin/fail_stop_fixture.rs"), "the location names the fixture: {report}");
    assert_eq!(lines.next(), Some(message), "the payload follows the header: {report}");
    assert_eq!(lines.next(), Some("stack backtrace:"), "the backtrace follows the payload: {report}");
    assert_eq!(report.matches("stack backtrace:").count(), 1, "exactly one backtrace is printed: {report}");
    assert_eq!(report.matches("   0:").count(), 1, "exactly one first frame is printed: {report}");
    assert!(
        report.contains("fail_stop_fixture::panic_in_fixture"),
        "the backtrace names the panicking function: {report}"
    );
    assert!(!report.contains("note: run with"), "the report needs no backtrace hint: {report}");
    assert!(!report.contains("fixture unwound"), "the panic never unwinds: {report}");
}
