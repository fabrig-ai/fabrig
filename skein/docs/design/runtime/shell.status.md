# shell: status

2026-11-11. Where skein's code lags or departs from [shell.md](shell.md).

Section 6 has no known gap in closing a panic report with its thread's
`panicked at` line and message after one forced backtrace. The fail-stop
test world checks that a 1,024-byte tail retains that closing text when
it fits; the report exceeds the tail. Sections 3 and 12's parent-death
exec is Linux's; macOS returns `Unsupported` and its alternative remains
section 9's question.

Section 6.3 has no known gap in requested hangup handling, preserving a
hangup ignored at startup, or `Kept::on_abort` restoring the terminal
before a panic's report. *The terminal's foreground* has none either:
`open_terminal` and `take_foreground` are `posix`'s, the end-to-end
world's `a_child_takes_the_terminal_and_gives_it_back` holds a child
taking a pseudo-terminal and its parent taking it back, and the unit
test `the_foreground_is_refused_off_a_terminal` a pipe refused with the
signal mask restored. The macOS build and runtime gaps in section 8
below remain; the Linux tests do not prove the Darwin arms.

Sections 6.5 and 14 have no known gap but the two departures below: a
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

- **Not yet:** 14.4: `Table::reach`, `Reached` and `Table::unknown` [in flight: change 86].
- **Not yet:** 6: the configured peer names resolved at startup (step 5).
  6.4: the helper's dispatch, and the service's own cgroup opened at
  startup.
  6.6: that macOS answers `ENOPROTOOPT` to `SO_ACCEPTCONN` is
  read from published XNU, whose `main` may lag `macos-26`'s kernel, and
  no recorded run shows it: a Linux test holds `listening`
  (`skein-shell/src/listener_calls.rs`) on both answers, and only the
  constant that selects macOS's goes unrun. 8: temper's one story has not
  been seen green on the macOS workflow
  (`temper/docs/design/testing/README.md`, 2.8), and no macOS runner runs
  skein's suites. Whether that workflow's compile step has run is unknown;
  a cross-target build does not prove the Darwin runtime.
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
   the `debt-28` request of 2026-11-06 has each of its items end with fewer
  lines than it started; the keeper's fence ended 48 lines of Rust up, 9 in code
  and 39 in tests, since a change of path removes no body.
   the shared-posix-call gap remains `readiness.status.md`, 5.
  14.2: the page's `environment:` part, and a command page's, also list
  the host's variables a setting's `Fallback::Base` reads, after the
  twins and the table's variables, in the settings' order, so `HOME`
  comes after the first XDG variable (`table/help.rs`,
  `help_environment`; `table/help/tests.rs`, `HOST_ENVIRONMENT`). That
  reads 14.1's "a table that holds such a setting reads them" as putting
  them on the page; 14.1 has the documents say so, and
  `../../foundation/cli.md`, 7's example, the draft's
  (`temper/docs/design/draft/cli.md`, 3.6), lists the directories' twins
  alone: `temper-box`'s and `temper-engine`'s pages are the first to
  list them.
  14.3: a binary's documents gain a `## Test` part after `## Internal`
  when its table holds a test entry, where 14.3 has the two alone and
  `../../foundation/cli.md`, 4 puts the test set in the test set's document.
  No table holds one.
