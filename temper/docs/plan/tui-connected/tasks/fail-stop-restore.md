+++
id = "fail-stop-restore"
part = "screen"
kind = "interface"
needs = []
files = [
  "skein/crates/skein-fail-stop/src/lib.rs",
  "skein/tests/fail-stop/**",
  "skein/crates/skein-shell/src/terminal.rs",
  "skein/crates/skein-shell/src/posix/terminal.rs",
  "skein/crates/skein-shell/src/lib.rs",
  "skein/docs/design/shell.status.md",
  "skein/docs/design/ui-screen.status.md",
]
effort = "medium"
minutes = 50
+++

# A panic leaves the terminal as it was found

## Goal

A binary that set its terminal raw and drew on its alternate screen must
leave both as it found them when it panics, and the panic's report must
reach the person's own screen. skein-fail-stop's hook gains a restoration
it runs once before its report, and skein-shell's kept modes make that
restoration: `LEAVE` written to the output, then the modes set back. Nothing
calls them yet; the TUI's binary will (`tui-command`).

This task departs from the implementer's text in one way: two status
entries are yours to change (acceptance 4).

Acceptance:

1. `skein_fail_stop::Restore` and `skein_fail_stop::before_report` exist as
   `temper/docs/plan/tui-connected/notes/contracts.md`, section 4, has them.
   The hook takes the restoration out and runs it before it writes its
   report, so it runs at most once, then writes the report and aborts as
   today; with none set it behaves exactly as today.
2. `skein_shell::Kept::on_abort(&self, output, leave)` exists as section 4
   has it: the function it gives writes all of `leave` to `output` with
   blocking writes (retrying on `EINTR`, stopping on any other error), then
   sets the kept modes back on the kept descriptor with `tcsetattr`,
   ignoring errors. It never closes the kept descriptor: `Kept::restore`
   still does, once.
3. A row of skein's fail-stop fixture sets a restoration that writes a
   mark to stderr, then panics; its test finds the mark once, before the
   report's first line, and the process still ends on `SIGABRT`.
4. `skein/docs/design/shell.status.md` loses its **Not yet** entry for
   6.3's restoration at a panic; `skein/docs/design/ui-screen.status.md`'s
   **Not yet** for 8 loses "`Kept::on_abort` and skein-fail-stop's
   `before_report`" and keeps "the terminal".

## Inputs

- `temper/docs/plan/tui-connected/notes/common.md`.
- `temper/docs/plan/tui-connected/notes/contracts.md`, section 4: binds.
- `skein/docs/design/shell.md`, 6.3, *Restored at a panic*.
- `skein/docs/foundation/programming-model.md`, section 2, *Panics abort*.
- `skein/docs/design/ui-screen.md`, section 8, *Every way out leaves the
  terminal as it was found*.

## Owns and may change

**Owned:** the `files` above.

**Readers and neighbours:** none. Nothing else calls `before_report` or
`on_abort` in this task. `tests/boundary/tests/fail_stop.rs` checks that
`install()` comes first in every `main`: the fixture still calls it first.

**Options, each with its default:**

- The restoration's slot: a `std::sync::Mutex<Option<Restore>>` in a
  `static`, taken with `lock()` and `take()`. If the hook finds the lock
  poisoned, it takes the value through `into_inner` of the poison error,
  so a panic inside a restoration set earlier never hides the report.
- The blocking write in skein-shell: `libc::write` in the posix module,
  beside `set` and `size`, under that module's `#![expect(unsafe_code)]`
  with a `// SAFETY:` line, as its siblings have. If the posix module is
  not the right home on Darwin, put it in the module the Darwin build
  already compiles for both, and say so.
- The fixture's restoration row: an argument (`restore`) to the existing
  fixture binary, that sets `before_report(Some(Box::new(|| { /* write
  "restored\n" to stderr */ })))` before the panic. If an argument breaks
  the fixture's existing argument order, use an environment variable the
  test sets, and say so.

## Steps

1. skein-fail-stop: `Restore`, `before_report`, the hook running the
   restoration first. Module doc: a line on the restoration and why it runs
   before the report (the report must reach the person's own screen).
2. skein-shell: `Kept::on_abort`, its blocking write and the modes set
   back; `Kept` keeps what it needs (the duplicate descriptor and the
   modes) by value, copied into the function's closure. Export nothing new
   beyond the method.
3. The fixture's row and its test in `skein/tests/fail-stop/tests/suite/`:

   ```rust
   #[test]
   fn a_restoration_runs_once_before_the_report() {
       let output = /* the fixture with "restore" */;
       let report = core::str::from_utf8(&output.stderr).expect("UTF-8");
       let restored = report.find("restored\n").expect("the restoration ran");
       let header = report.find("thread '").expect("the report");
       assert!(restored < header, "the restoration ran before the report: {report}");
       assert_eq!(report.matches("restored\n").count(), 1);
       // and the process aborted on SIGABRT, as assert_report checks
   }
   ```

   Bite: run the restoration after the report once, see the test fail on
   the order, and restore.
4. A unit test in skein-shell is not needed: `on_abort` needs a real
   terminal, which no focused test opens. Its compile on both targets is
   the check (`heavy cargo check -p skein-shell --tests`).
5. The status entries (acceptance 4).
6. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo
   clippy -p skein-fail-stop -p skein-fail-stop-tests -p skein-shell
   --all-targets -- -D warnings` (the test crate's package name is in its
   `Cargo.toml`); `heavy cargo nextest run -p skein-fail-stop-tests` and
   `-p skein-shell --lib`; `heavy cargo nextest run -p fabrig-boundary
   --test fail_stop`. Commit ("skein: a panic's hook runs a restoration
   first, and kept terminal modes make one"), push.

## Report

As the implementer's text says, and: where the blocking write lives and
how it builds on Darwin, and the new test's time.
