# readiness: status

2026-11-11. Where the tree's code lags or departs from [readiness.md](readiness.md).

- **Departs:**  the trust-count reporting departure remains `shell.status.md`, 6.2.
   the user's request of 2026-11-04 asked each part of its
  round to end with fewer lines than it started; the part that built `store_file`
  and `bytes::concat` ended 190 lines up, 215 in skein against 25 down
  in smith, since a host and its first tests replaced two copies.
  5: the `debt-28` request of 2026-11-06 has each of its items end with fewer
  lines than it started; the keeper's fence ended 48 lines of Rust up, 9 in code
  and 39 in tests, since a change of path removes no body.
  5: `linux` and `darwin` each keep `count`, `process::fd` and
  `files::truncate`, differing only in a panic's words or a path, where the
  section has identical calls live once in `posix`. These three were outside
  the six-function task's scope (`shell.md`, section 9), and no debt item holds them.
  Seen by reading.
   the page's `environment:` part, and a command page's, also list
  the host's variables a setting's `Fallback::Base` reads, after the
  twins and the table's variables, in the settings' order, so `HOME`
  comes after the first XDG variable (`table/help.rs`,
  `help_environment`; `table/help/tests.rs`, `HOST_ENVIRONMENT`). That
  reads `shell.md`, 14.1's "a table that holds such a setting reads them" as putting
  them on the page; `shell.md`, 14.1 has the documents say so, and
  `../../foundation/cli.md`, 7's example, the draft's
  (`temper/docs/design/draft/cli.md`, 3.6), lists the directories' twins
  alone. No binary prints a page yet: C2's first tables meet it.
   the test-document departure remains `shell.status.md`, 14.3.
