//! A panic on any thread of a binary built to unwind aborts the process,
//! after reporting its message and backtrace (programming-model.md, 2).
#![cfg(test)]

use std::os::unix::process::ExitStatusExt;
use std::process::{Command, Output};

/// The abort signal, on Linux and on aarch64-apple-darwin.
const SIGABRT: i32 = 6;

#[test]
fn a_panic_on_a_second_thread_aborts_the_process_after_its_report() {
    // A borrowed payload and an unnamed thread, with the full backtrace setting.
    let output = panic_report(Some("full"), "str", None);
    let report = core::str::from_utf8(&output.stderr).expect("the report is UTF-8");
    assert!(report.contains("the fixture's thread panics"), "the hook reported first: {report}");
    assert_report(&output, "", "<unnamed>", "the fixture's thread panics");
}

#[test]
fn owned_and_non_string_payloads_follow_the_default_report() {
    // Named threads add owned and opaque payloads, with the other backtrace settings.
    for (backtrace, payload, message) in
        [(None, "string", "the fixture's owned message"), (Some("1"), "any", "Box<dyn Any>")]
    {
        let output = panic_report(backtrace, payload, Some("fixture-worker"));
        assert_report(&output, "", "fixture-worker", message);
    }
}

#[test]
fn a_restoration_runs_once_before_the_report() {
    let output = panic_report(None, "restore", None);
    let report = core::str::from_utf8(&output.stderr).expect("UTF-8");
    let restored = report.find("restored\n").expect("the restoration ran");
    let header = report.find("thread '").expect("the report");
    assert!(restored < header, "the restoration ran before the report: {report}");
    assert_eq!(report.matches("restored\n").count(), 1);
    assert_report(&output, "restored\n", "<unnamed>", "the fixture's thread panics");
}

#[test]
fn a_supervisors_tail_keeps_the_panic_line_and_message() {
    // The fixture models a supervisor keeping the last 1,024 stderr bytes.
    const DETAIL_BYTES: u32 = 1024;
    let output = panic_report(None, "str", Some("fixture-worker"));
    assert!(
        output.stderr.len() > usize::try_from(DETAIL_BYTES).expect("the bound fits usize"),
        "the report must exceed the supervisor's tail: {} bytes",
        output.stderr.len()
    );
    let report = core::str::from_utf8(&output.stderr).expect("the report is UTF-8");
    let header = report.lines().nth(1).expect("the report has a thread line");
    let mut tail = skein_lib::Tail::with_capacity(DETAIL_BYTES);
    tail.put(&output.stderr);
    let kept = tail.into_boxed();
    let kept = core::str::from_utf8(&kept).expect("the fixture's tail is UTF-8");
    assert!(
        kept.ends_with(&format!("{header}\nthe fixture's thread panics\n")),
        "the supervisor keeps the panic's line and message at the end: {kept}"
    );
    assert_report(&output, "", "fixture-worker", "the fixture's thread panics");
}

#[expect(
    clippy::disallowed_methods,
    reason = "skein's round: reads the fixture binary path supplied by Cargo and nextest"
)]
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

fn assert_report(output: &Output, restoration: &str, name: &str, message: &str) {
    assert_eq!(output.status.signal(), Some(SIGABRT), "the process aborts: {:?}", output.status);
    let report = core::str::from_utf8(&output.stderr).expect("the report is UTF-8");
    let report = report.strip_prefix(restoration).expect("the restoration precedes the report");
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
    let mut ending = report.lines().rev();
    assert_eq!(ending.next(), Some(message), "the report closes with the payload: {report}");
    assert_eq!(ending.next(), Some(header), "the closing header repeats the first: {report}");
    assert_eq!(report.matches(header).count(), 2, "the header is printed exactly twice: {report}");
    assert_eq!(report.matches("stack backtrace:").count(), 1, "exactly one backtrace is printed: {report}");
    assert_eq!(report.matches("   0:").count(), 1, "exactly one first frame is printed: {report}");
    assert!(
        report.contains("fail_stop_fixture::panic_in_fixture"),
        "the backtrace names the panicking function: {report}"
    );
    assert!(!report.contains("note: run with"), "the report needs no backtrace hint: {report}");
    assert!(!report.contains("fixture unwound"), "the panic never unwinds: {report}");
}
