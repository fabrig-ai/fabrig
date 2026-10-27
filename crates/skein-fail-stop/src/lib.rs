//! Fail-stop in a binary built to unwind (programming-model.md, 2).
//!
//! The release profile aborts on a panic. The dev profile unwinds, as the
//! test harness needs, so that a binary and its tests share one build of
//! each dependency. Every binary's `main` therefore calls [`install`] before
//! anything else: from then on a panic on any thread reports its message
//! and a forced backtrace once, whatever `RUST_BACKTRACE` says, then aborts
//! the process before anything unwinds. The header follows the default
//! hook's without the thread id: std's OS thread id is private, and a new
//! platform call is not worth a log line when the backtrace says where the
//! panic was. No destructor runs, nothing catches the panic, and the process
//! ends on `SIGABRT`, as under `panic = "abort"`. Each repository's gate
//! checks that every binary calls it first.
//!
//! This is ordinary Rust (programming-model.md, 10.2).
#![forbid(unsafe_code)]

use std::io::Write;

/// Installs the hook: a message and forced backtrace, then an abort.
pub fn install() {
    std::panic::set_hook(Box::new(|panic| {
        let thread = std::thread::current();
        let name = thread.name().unwrap_or("<unnamed>");
        let location = panic.location().expect("std supplies the panic's location");
        let message = panic.payload_as_str().unwrap_or("Box<dyn Any>");
        let backtrace = std::backtrace::Backtrace::force_capture();
        let report = format!("\nthread '{name}' panicked at {location}:\n{message}\nstack backtrace:\n{backtrace}");
        // Hold stderr through the abort so concurrent panic reports cannot
        // interleave. A failed write must still end the process.
        let mut stderr = std::io::stderr().lock();
        let _write_result = stderr.write_all(report.as_bytes());
        std::process::abort();
    }));
}
