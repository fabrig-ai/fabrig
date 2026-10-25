//! Fail-stop in a binary built to unwind (programming-model.md, 2).
//!
//! The release profile aborts on a panic. The dev profile unwinds, as the
//! test harness needs, so that a binary and its tests share one build of
//! each dependency. Every binary's `main` therefore calls [`install`] before
//! anything else: from then on a panic on any thread is reported as the
//! default hook reports it, then aborts the process before anything
//! unwinds. No destructor runs, nothing catches the panic, and the process
//! ends on `SIGABRT`, as under `panic = "abort"`. Each repository's gate
//! checks that every binary calls it first.
//!
//! This is ordinary Rust (programming-model.md, 10.2).
#![forbid(unsafe_code)]

/// Installs the hook: the default report, then an abort.
pub fn install() {
    let report = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        report(panic);
        std::process::abort();
    }));
}
