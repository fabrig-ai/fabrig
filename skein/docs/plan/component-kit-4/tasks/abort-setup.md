+++
id = "abort-setup"
part = "abort"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge-setup/src/host.rs", "skein/crates/skein-forge-setup/src/host/**", "skein/crates/skein-forge-setup/src/tests/**", "skein/crates/skein-forge-setup/src/tests.rs"]
effort = "high"
minutes = 75
+++

## Goal

skein-forge-setup's host component takes an `Abort` request, as
programming-model.md 5.2 requires of every component
(`skein/docs/design/draft/component-kit.md, 3.6`; the review's D9): a
close that does not wait. Each run it holds ends with its one terminal,
as cancelled; its connections are aborted at io
(`skein_io::Request::Abort`), not closed; `Closed` follows once io has
settled them. TLS `close_notify` is not part of this task.

Acceptance:

- `host::Request::Abort` exists (`src/host/boundary.rs`), documented in the
  words of 5.2.
- Tests in the crate's own tests show, each in one test: an abort while a
  run is in flight ends it with one cancelled terminal and then `Closed`;
  an abort after a close turns what still closes gracefully into aborts;
  a close after an abort changes nothing; a second abort changes nothing;
  a `Run` after an abort is refused or ignored as after a close.
- The crate's tests, clippy, and the readers' checks of step 4 pass.

## Inputs

- `skein/docs/plan/component-kit-4/notes/common.md`.
- `skein/docs/foundation/programming-model.md, 5.2` and `5.4`.
- `skein/docs/design/forge-setup.md, 6.3` (the host component) and `6.5`
  (its limits); find how a run's terminal is spelt there, and use the one
  for a run that did not finish (if none fits, the nearest one the owner
  handles at close, saying which and why).
- The host: `src/host.rs`, `src/host/boundary.rs:111` (`Request`),
  `src/host/connection.rs`, `src/host/run.rs`, `src/host/tests.rs` and
  `src/host/connection_tests.rs` (the close tests to model).
- A component that has it: `skein-llm-connection` (`Request::Abort`,
  `src/boundary.rs:68`, and its handling in `src/component.rs`).

## Owns and may change

- Owns `skein/crates/skein-forge-setup/src/host.rs`, `src/host/`, and the
  crate's tests (`src/tests.rs`, `src/tests/`). The rest of the crate (the
  plans, `github/`, `reconcile/`) is not yours.
- Readers that match `host::Request` exhaustively need an arm: in
  `skein/tests/forge/setup/` and `temper/crates/temper-service/src/setup/`
  you may add the `Request::Abort` arm only (their `Close` arm's handling,
  or `unreachable!` with its reason where they never send it), nothing
  else.
- Do not add abort sending to the owners (temper): a later task's.

## Steps

1. Add `Request::Abort` after `Close`.
2. Handle it in the host: its ending held in one place (a private enum
   is fine, not named `Lifecycle`: task lib-kit's shared one is not on
   your base, and a later task moves this crate onto it), every connection
   aborted, every run answered once as cancelled, `Closed` after io settles.
   Keep each entity's deadlines a function of its state (5.4).
3. The tests of the Goal, beside the host's close tests.
4. `cargo fmt`; `heavy cargo clippy -p skein-forge-setup --all-targets --
   -D warnings`; `heavy cargo nextest run -p skein-forge-setup`; then
   `heavy cargo check --tests` on `skein-forge-setup-world` (the package in
   `skein/tests/forge/setup/`; check its name) and on `temper-service`.
5. `old_forms.py --write` (common.md); commit and push.

## Report

As common.md says, plus the terminal you chose for a cancelled run and
where the design says it.
