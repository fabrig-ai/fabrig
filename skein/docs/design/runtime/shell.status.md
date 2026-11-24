# shell: status

2026-11-23. Where skein's code lags or departs from [shell.md](shell.md).

`shell-startup.md`, section 1 has no known gap in closing a panic report with its thread's
`panicked at` line and message after one forced backtrace. The fail-stop
test world checks that a 1,024-byte tail retains that closing text when
it fits; the report exceeds the tail. `shell.md`, sections 3 and 12's parent death
on macOS, the watcher of `readiness.md`, 5, runs on Linux in
skein-shell's unit tests alone, over the wait built for them; its
kqueue wait has not been seen run on macOS.

`shell-startup.md`, section 1.3 has no known gap in requested hangup handling, preserving a
hangup ignored at startup, or `Kept::on_abort` restoring the terminal
before a panic's report. *The terminal's foreground* has none either:
`open_terminal` and `take_foreground` are `posix`'s, the end-to-end
world's `a_child_takes_the_terminal_and_gives_it_back` holds a child
taking a pseudo-terminal and its parent taking it back, and the unit
test `the_foreground_is_refused_off_a_terminal` a pipe refused with the
signal mask restored. The macOS build and runtime gaps in `shell.md`, section 8
below remain; the Linux tests do not prove the Darwin arms.

`shell-startup.md`, section 1.5, and `shell-table.md`, section 1 have no known gap but the two departures in [shell-table.status.md](shell-table.status.md): a
setting's set, declared twin and notes and a group's line
(`settings.rs`), the table's types (`table.rs`), its help, command page
and version line (`table/help.rs`) and its reference documents
(`table/reference.rs`), each held by its unit tests on `table/demo.rs`'s
tables, and `tests/boundary/tests/cli_names.rs`'s
`rendered_documents_read_back` reads the environment documents as check 5
does. temper's `temper-box` and `temper-engine` are the first binaries on
a table (`temper/docs/design/cli/README.md`), alongside the six tools and
fixtures (`temper/docs/design/testing/tools.md`); temper's remaining binaries
are milestones C3 and C5 of `temper/docs/design/draft/cli.execution.md`.

- **Not yet:** 8: temper's one story has not
  been seen green on the macOS workflow
  (`temper/docs/design/testing/tiers.md`, 1.8), and no macOS runner runs
  skein's suites. Whether that workflow's compile step has run is unknown;
  a cross-target build does not prove the Darwin runtime.
- **Not yet:** 8: *The Darwin check* runs nowhere before landing:
  `rust-toolchain.toml` installs only the `wasm32-unknown-unknown`
  target, so clippy for `aarch64-apple-darwin` fails on our Linux
  machines for want of the target's `std` and `core` (E0463), and no
  gate step runs it. Darwin's adapters, among them startup's file reads
  and `make_root`'s walk over the shared decisions, are first compiled
  on the macOS runner.
- **Not yet:** 8, 11: `drive` takes its kernel and clock directly, and its
  own tests run in the real loop, not yet over a scripted face of them.
  [in flight: change 13904]
- **Not yet:** 8: startup's bounded and private reads, its trust roots'
  DER and bundle bound, a path's NUL and `make_root`'s walk are checked on
  the real kernel, not yet over handed facts in the shell's own tests.
  [in flight: change 13908]
