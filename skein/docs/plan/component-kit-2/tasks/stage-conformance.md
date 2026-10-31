+++
id = "stage-conformance"
part = "stage-sites"
kind = "migration"
needs = ["echo-stage", "kit-shell", "counts"]
files = ["skein/tests/conformance/real/**", "skein/crates/skein-sim/src/tests/given_listener.rs", "skein/scripts/old-forms.txt"]
effort = "high"
minutes = 40
+++
# The real-kernel conformance tests and sim's listener test on the shared stage

## Goal

Draft, 3.3: the conformance tests that drive io against the real kernel,
and skein-sim's inherited-listener test, write io's stage by hand; they
call `skein_io::stage` instead, by `notes/stage.md`.

Acceptance:

- Moved, unless a default of `stage.md` leaves one (named):
  `tests/conformance/real/src/bin/signal_sender_fixture.rs:168`,
  `tests/suite/adopted_listener.rs` (two, ~70 and ~174),
  `tests/suite/given_listener.rs:51`, `tests/support/deadline.rs:101`,
  `tests/support/given_listener.rs:47`, and
  `skein/crates/skein-sim/src/tests/given_listener.rs:50`.
- `io_resume` falls by the sites moved (7 expected).
- `heavy cargo nextest run -p skein-conformance-real` and `heavy cargo
  nextest run -p skein-sim` pass (counts named); clippy clean for both.

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`, `notes/stage.md`.
- `skein/docs/design/kernel.md` (conformance) for the words.
- `skein_io::stage` on your base.

## Owns and may change

- The files in the header. In skein-sim, only that test file.

Pre-approved, beside `stage.md`'s defaults:

- Several of these loops resume until nothing is ready (`while
  state.is_ready()`), with no bound and no `fire`. `stage::up` bounds the
  pass and fires due timers: call it until `!io.is_ready()` where the
  test needs everything ready taken, and keep the test's assertions. A
  test that pins that a timer has not yet fired at that point keeps its
  hand loop and is named.
- `resume` before `kernel.reap`, then `up` over what was reaped: two
  `stage::up` calls (the first with an empty completions queue) keep the
  order.

## Steps

1. Each site by `stage.md` item 1 and the defaults above.
2. `--write` from `skein/`; checks of `common.md`; commit; push.

## Report

Per `common.md`; each site left, with its default and the assertion that
pins it.
