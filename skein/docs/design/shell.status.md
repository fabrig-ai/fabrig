# shell: status

2026-11-09. Where skein's code lags or departs from [shell.md](shell.md).

Section 6 has no known gap in closing a panic report with its thread's
`panicked at` line and message after one forced backtrace. The fail-stop
test world checks that a 1,024-byte tail retains that closing text when
it fits; the report exceeds the tail. Sections 3 and 13's parent-death
exec is Linux's; macOS returns `Unsupported` and its alternative remains
section 10's question.

Section 6.3 has no known gap in requested hangup handling, preserving a
hangup ignored at startup, or `Kept::on_abort` restoring the terminal
before a panic's report. *The terminal's foreground* has none either:
`open_terminal` and `take_foreground` are `posix`'s, the end-to-end
world's `a_child_takes_the_terminal_and_gives_it_back` holds a child
taking a pseudo-terminal and its parent taking it back, and the unit
test `the_foreground_is_refused_off_a_terminal` a pipe refused with the
signal mask restored. The macOS build and runtime gaps in section 9
below remain; the Linux tests do not prove the Darwin arms.

- **Not yet:** 6: the configured peer names resolved at startup (step 5).
  6.4: the helper's dispatch, and the service's own cgroup opened at
  startup.
  6.5: a setting derives its twin from the tool's name and its own, and
  has no set; a table of settings has no line of its own.
  6.6: that macOS answers `ENOPROTOOPT` to `SO_ACCEPTCONN` is
  read from published XNU, whose `main` may lag `macos-26`'s kernel, and
  no recorded run shows it: a Linux test holds `listening`
  (`skein-shell/src/listener_calls.rs`) on both answers, and only the
  constant that selects macOS's goes unrun. 9: temper's one story has not
  been seen green on the macOS workflow
  (`temper/docs/design/domain/testing.md`, 2.8), and no macOS runner runs
  skein's suites. Whether that workflow's compile step has run is unknown;
  a cross-target build does not prove the Darwin runtime.
  15: no binary's table: no `Set`, `Table`, `Command`, `Switch`,
  `Variable` or `TestTable`, no help page, command page or version line
  rendered from one (15.2), and no reference documents (15.3).
- **Departs:** 6.2: `read_trust_roots` gives its caller the counts, and only
  `smith-agent-document`'s `config/native.rs`, `trust_sources`, says them,
  through the shell's re-export too. temper's own two readers
  (`temper/crates/temper-engine-shell/src/serve.rs`, `trust`, and
  `temper-fetch-shell/src/fetch.rs`) say nothing of a malformed block, where
  the section has startup say the counts: the request `first-usable-model` of
  2026-11-06 asked only that smith's line go quiet. Seen by reading.
  6.7: the user's request of 2026-11-04 asked each part of its
  round to end with fewer lines than it started; the part that built `store_file`
  and `bytes::concat` ended 190 lines up, 215 in skein against 25 down
  in smith, since a host and its first tests replaced two copies.
  7.5: the `debt-28` request of 2026-11-06 has each of its items end with fewer
  lines than it started; the keeper's fence ended 48 lines of Rust up, 9 in code
  and 39 in tests, since a change of path removes no body.
  7.5: `linux` and `darwin` each keep `count`, `process::fd` and
  `files::truncate`, differing only in a panic's words or a path, where the
  section has identical calls live once in `posix`. These three were outside
  the six-function task's scope (section 10), and no debt item holds them.
  Seen by reading.
