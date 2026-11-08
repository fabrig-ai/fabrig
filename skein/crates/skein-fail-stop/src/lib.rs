//! Fail-stop in a binary built to unwind (programming-model.md, 2).
//!
//! The release profile aborts on a panic. The dev profile unwinds, as the
//! test harness needs, so that a binary and its tests share one build of
//! each dependency. Every binary's `main` therefore calls [`install`] before
//! anything else: from then on a panic on any thread reports its thread's
//! `panicked at` line with the panic's location, its message, and a forced
//! backtrace once, whatever `RUST_BACKTRACE` says. The report closes with
//! that line and message again, so a supervisor keeping the last bytes
//! retains them after a long backtrace (skein/docs/design/shell.md, 6).
//! The process then aborts before anything unwinds. The header follows the default
//! hook's without the thread id: std's OS thread id is private, and a new
//! platform call is not worth a log line when the backtrace says where the
//! panic was. No destructor runs, nothing catches the panic, and the process
//! ends on `SIGABRT`, as under `panic = "abort"`. Each repository's gate
//! checks that every binary calls it first.
//! An optional restoration runs once before the report so it reaches the
//! person's own screen (shell.md, 6.3).
//!
//! This is ordinary Rust (programming-model.md, 10.2).
#![forbid(unsafe_code)]

use std::io::Write;
use std::sync::{Mutex, PoisonError};

/// What the hook runs once, before its report (programming-model.md, section 2).
pub type Restore = Box<dyn Fn() + Send + Sync>;

static RESTORE: Mutex<Option<Restore>> = Mutex::new(None);

/// Sets the restoration, or takes it back with None. The hook takes it out before
/// running it, so it runs at most once.
pub fn before_report(restore: Option<Restore>) {
    *RESTORE.lock().unwrap_or_else(PoisonError::into_inner) = restore;
}

/// Installs the hook: a message and forced backtrace, then an abort.
pub fn install() {
    std::panic::set_hook(Box::new(|panic| {
        let restore = RESTORE.lock().unwrap_or_else(PoisonError::into_inner).take();
        if let Some(restore) = restore {
            restore();
        }
        let thread = std::thread::current();
        let name = thread.name().unwrap_or("<unnamed>");
        let location = panic.location().expect("std supplies the panic's location");
        let message = panic.payload_as_str().unwrap_or("Box<dyn Any>");
        let backtrace = std::backtrace::Backtrace::force_capture();
        let line = format!("thread '{name}' panicked at {location}:\n{message}\n");
        let report = format!("\n{line}stack backtrace:\n{backtrace}\n{line}");
        // Hold stderr through the abort so concurrent panic reports cannot
        // interleave. A failed write must still end the process.
        let mut stderr = std::io::stderr().lock();
        let _write_result = stderr.write_all(report.as_bytes());
        std::process::abort();
    }));
}
