+++
id = "starved"
part = "starved"
kind = "feature"
needs = []
files = ["skein/testing/skein-world/src/deadline.rs", "skein/testing/skein-world/src/lib.rs", "skein/testing/skein-world/src/real.rs", "skein/tests/world/tests/suite/real.rs", "skein/tests/world/tests/suite/deadline.rs", "skein/tests/conformance/real/tests/suite/signal_sender.rs", "skein/tests/conformance/real/src/bin/signal_sender_fixture.rs", "skein/docs/design/testing.status.md"]
effort = "high"
minutes = 100
+++

# inproc-runner: a starved pass in a one-process run ends only its test

## Goal

Under `TIERS_ONE_PROCESS`, the variable the new one-process runner sets in every test
binary it starts, a starved pass writes the same `skein-world: starved:` line and
unwinds rather than exiting, so that only its test ends and the process goes on to the
next. Under nextest and outside both it behaves as now. Every place that keys on
`NEXTEST` for starvation keys on the new `starved_passes()`.

Acceptance:

- `skein_world::{StarvedPass, starved_passes, pass_starved}` and `Starved::exit`, and
  `Starved::pass`'s three ends in their order of precedence, exactly as
  `temper/docs/plan/inproc-runner/notes/contracts.md`, section 1 gives them.
- `real.rs`'s two deadline paths (`deadline_reached`, `referee_deadline_reached`) take
  the starved arm where `starved_passes()`, after `forced_cleanup` as now.
- `forward_starved` in `skein/tests/conformance/real/tests/suite/signal_sender.rs`
  forwards the helper's line with `pass_starved` where `starved_passes()`;
  `signal_sender_fixture`'s patience is armed where `starved_passes()`, and its starved
  end is `starved.exit()` (it is a helper process: its test reads its stderr).
- `skein/tests/world/tests/suite/real.rs`'s two children have `TIERS_ONE_PROCESS`
  removed beside `NEXTEST`.
- One table test in `skein/tests/world/tests/suite/deadline.rs` (beside its siblings,
  the child pattern of `real.rs`'s `a_zero_patience_deadline_reports_the_failing_iteration`:
  the test runs its own binary, `--exact NAME --nocapture --test-threads=1`, with a
  child variable set), a row per end:
  - `TIERS_ONE_PROCESS=1`, `NEXTEST` removed: in the child, a zero-patience deadline's
    `Starved::pass()` inside `std::panic::catch_unwind` unwinds with a payload that is
    `StarvedPass` (`downcast_ref`), and the child prints a marker after it and returns:
    the child succeeds, its stderr holds the `skein-world: starved:` line, the marker
    follows it. The process went on.
  - the same, without `catch_unwind`: libtest reports the child's test failed (exit
    101), and the last line of its stderr is the `skein-world: starved:` line: no
    `panicked at` after it.
  - `NEXTEST=1`, `TIERS_ONE_PROCESS` removed: exit 0, the line, no marker (as now).
  - both removed: the test fails with the figures (as now).
  - `Starved::exit()` under `TIERS_ONE_PROCESS=1`: exit 0 with the line.
  - `pass_starved(line)` under `TIERS_ONE_PROCESS=1` unwinds with `StarvedPass`, after
    writing `line`; with both removed it fails with `line`.
- In `real.rs`'s `a_starved_referee_drops_hosts_before_reporting_its_figures`, or a
  row beside it: the starved referee's world run in a child under
  `TIERS_ONE_PROCESS=1` (and no `NEXTEST`) drops its hosts before the line, and the line
  is the last line of the child's stderr.
- The **Not yet** entry `5: a starved pass in a one-process run: ...` in
  `skein/docs/design/testing.status.md` deleted, as a stated departure from the
  implementer's text (`notes/common.md`).

## Inputs

- `skein/docs/design/testing.md`, 5, the bullet *The real loop classifies its patience
  and the referee's overdue liveness deadlines*.
- `skein/docs/foundation/testing-strategy.md`, 2.10 (*Starved passes are counted*).
- `temper/docs/plan/inproc-runner/notes/contracts.md`, section 1;
  `temper/docs/plan/inproc-runner/notes/common.md`.

## Owns and may change

- Owns the files in `files`.
- Readers of `Starved::pass` keep their calls unchanged: its signature stays. They are
  `skein/testing/skein-world/src/{real.rs,tree.rs,end_to_end.rs}`,
  `skein/testing/skein-throwaway-forgejo/src/instance.rs`,
  `skein/testing/skein-fake-person-browser/src/harness.rs` and its `peers/tests.rs`,
  `skein/examples/echo/shell/tests/end_to_end.rs`, and
  `skein/tests/world/tests/suite/{drive.rs,deadline.rs}`; each runs inside a test, so
  it unwinds there, as it should. If one of them runs in a helper binary (a `main`, not
  a libtest test), switch it to `Starved::exit()` and name it in the report.
- If a destructor that runs during the unwind from `real.rs`'s paths prints or panics
  (the line would no longer be last), fix that destructor in skein-world and say so;
  if it is outside skein-world, report it under `## Left` with the test that shows it.
- `#[expect(clippy::print_stderr, reason = ...)]` on the new items that write the line,
  as `pass` has one.

## Steps

1. The table test's rows; run them
   (`heavy cargo nextest run -p skein-world-tests --test suite -E 'test(/deadline::/)'`)
   and see the `TIERS_ONE_PROCESS` rows fail.
2. `deadline.rs`: `StarvedPass`, `starved_passes`, `pass_starved`, `Starved::exit`,
   `pass`'s precedence; `lib.rs`'s export line.
3. `real.rs`'s two paths; the starved referee's row; the children's `env_remove`.
4. The conformance's `forward_starved` and the fixture's patience and end;
   `heavy cargo nextest run -p skein-conformance-real`.
5. The status entry.
6. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p
   skein-world -p skein-world-tests -p skein-conformance-real --all-targets -- -D
   warnings`; the focused tests of `skein-world`, `skein-world-tests` and
   `skein-conformance-real` (`heavy cargo nextest run -p ...`). Commit and push.

## Report

As the implementer's text gives it, with the rule each new row broke and the failure
seen.
