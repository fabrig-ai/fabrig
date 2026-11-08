+++
id = "report-end"
part = "detail"
kind = "feature"
needs = []
files = ["skein/crates/skein-fail-stop/src/lib.rs", "skein/tests/fail-stop/**", "skein/docs/design/shell.status.md", "skein/docs/design/STATUS.md"]
effort = "medium"
minutes = 40
+++

# A panic's report closes with its message

## Goal

A failed agent's kept detail is the last bytes of its error output
(`smith-host-protocol`, `jig-domain-host`), and `skein-fail-stop` writes
the panic's `panicked at` line and message before a forced backtrace, so
a long backtrace pushes the message out of what is kept. The hook now
closes its report with the same line and message after the backtrace, so
every keeper of a tail holds them within its own bound.

Acceptance:

- `skein_fail_stop::install`'s hook writes the report of
  `smith/docs/plan/subagent-burst/notes/contracts.md`, section 2: the
  head unchanged, then after the backtrace a newline and the same
  `thread '{name}' panicked at {location}:\n{message}\n` again, in the
  one `write_all`, then aborts as today. The module documentation says
  so in the design's words.
- `skein/tests/fail-stop/tests/suite/fail_stop.rs`: `assert_report`, which
  every row calls, also checks the report's end: its last two lines are
  the header line (`thread '<name>' panicked at …:`, the same as the
  first) and the message, the header appearing exactly twice and
  `stack backtrace:` once.
- A row, in the same file, where a supervisor's tail is kept: the
  fixture's stderr is longer than 1,024 bytes (asserted, so the row
  bites), and its last 1,024 bytes, kept through `skein_lib::Tail` with
  capacity 1,024 as a supervisor keeps them, hold the header line and the
  message. Add `skein-lib` to the world's `[dev-dependencies]` if it is
  not there.
- The **Not yet** entry `6: a panic's report closing with its
  `panicked at` line …` is gone from `skein/docs/design/shell.status.md`,
  and the sentence `A panic's report does not yet close with its message
  after the backtrace (`shell.md`).` from `skein/docs/design/STATUS.md`.
- Clippy clean, fmt clean, the package's tests green.

## Inputs

- `smith/docs/plan/subagent-burst/notes/common.md` and `notes/contracts.md`,
  section 2.
- `skein/docs/design/shell.md`, section 6, its last paragraph (*A panic is
  fail-stop*), as this cut wrote it.
- `skein/docs/foundation/programming-model.md`, section 2 (*Panics
  abort*).

Departures from the implementer's text, and why:

- **Status files:** you delete the entries named above, since this task
  closes them; touch no other design text.

## Owns and may change

Owns:

- `skein/crates/skein-fail-stop/src/lib.rs`: the hook's report and the
  module documentation.
- `skein/tests/fail-stop/**`: `assert_report`, the new row, the
  fixture only if a row needs a longer backtrace (a deeper call chain in
  `src/bin/fail_stop_fixture.rs`), and its `Cargo.toml` dev-dependency.
- `skein/docs/design/shell.status.md` and `skein/docs/design/STATUS.md`:
  the two deletions.

Reader and neighbour files: none. Every binary's `main` calls `install`
unchanged; no signature changes.

Pre-approved options:

- **The backtrace under 1,024 bytes** in the fixture's run (a short
  chain in the dev profile): make the panic deeper in the fixture, a
  recursion of a few dozen frames before it, so the report passes
  1,024 bytes; say so in the report.
- **A test elsewhere pins the old report's end** (a search for
  `stack backtrace:` or for the report's last bytes in skein, smith, jig
  or temper tests): if it only pins the head, leave it; if it pins the
  end, follow the new end in that test, one edit, and name it.

## Steps

1. Read `skein-fail-stop/src/lib.rs` and the test world
   (`skein/tests/fail-stop`). `rg -n "stack backtrace|panicked at"` over
   `skein smith jig temper hearth` for other pins.
2. Change `assert_report` first and add the tail row; run
   `heavy cargo nextest run -p skein-fail-stop-tests` and see them fail
   for the missing closing line. Then change the hook; run again, green
   (check the count).
3. Delete the two status lines.
4. Once, at the end: `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`;
   `heavy cargo clippy -p skein-fail-stop -p skein-fail-stop-tests
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   skein-fail-stop -p skein-fail-stop-tests`; `python3 -I -B
   tools/citations/citations.py --base origin/main`. Commit
   (`skein-fail-stop: close a panic's report with its line and message`,
   the body saying why: a supervisor keeps a tail of stderr, and a long
   backtrace pushed the message out of it) and push.

## Report

As the implementer's text says: the first line, the commit pushed, the
tests run and their counts, the rule broken by hand for each new check
and the failure seen, and your implementation and check minutes apart.
