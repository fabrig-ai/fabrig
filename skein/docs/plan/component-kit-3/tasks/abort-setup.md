+++
id = "abort-setup"
part = "abort"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge-setup/**"]
effort = "high"
minutes = 45
+++

## Goal

skein-forge-setup's host component takes an abort from its owner, as every
component must (`skein/docs/foundation/programming-model.md`, 5.2; the
review's D9, quoted in `skein/docs/design/draft/component-kit.md`, 3.6).
Today `crates/skein-forge-setup/src/host/boundary.rs`'s `Request` has
`Run`, `Answer` and `Close`. The TLS `close_notify` is not part of this
task.

Acceptance:
- `Request::Abort` in `src/host/boundary.rs`: a close that does not wait.
  Each run it holds ends with its one terminal as cancelled, if the
  boundary's `Failure` has a variant that says so; and each connection is
  aborted (io's `Request::Abort`), never closed gracefully.
- An abort after a close turns what still closes gracefully into aborts; a
  close after an abort changes nothing; a second abort changes nothing.
  `Event::Closed` goes up once, last, after either.
- Tests in the crate show each of those; the crate's tests, clippy and its
  direct dependents' builds pass.

## Inputs

- `skein/docs/plan/component-kit-3/notes/common.md`.
- `skein/docs/foundation/programming-model.md`, 5.2 (the abort's contract:
  "a close that does not wait, which ends each request it holds as
  cancelled and aborts what it keeps").
- `skein/docs/design/forge-setup.md`, 6.3 (the host component) and 6.5 (its
  limits).
- `crates/skein-forge-setup/src/host.rs` (`Request::Close`'s handling: it
  already aborts each connection), `src/host/connection.rs`,
  `src/host/boundary.rs`.

## Owns and may change

- Owns `skein/crates/skein-forge-setup/**`.
- Readers that match `skein_forge_setup::host::Request` exhaustively
  (`rg -n 'forge_setup::host' skein temper`: `temper/crates/temper-service`
  and `skein/tests/forge/setup` use the host) get the new arm only, handled
  as their `Close` arm is; call sites only. No owner is changed to send
  `Abort`.
- Not `docs/design/`: the report says what forge-setup.md 6.3 needs.
- Pre-approved:
  - What `Close` does today stays as it is, unless a test shows it breaks
    5.2's "every request gets exactly one terminal"; then report it, do not
    change it.
  - If `Failure` has no cancelled variant, `Abort` ends runs the way `Close`
    does today and the report says so; adding a variant is a `Question:`.
  - The close state's private shape (a bit or a small enum beside `closing`)
    is yours; lib's `Lifecycle` arrives in another task and is not required.

## Steps

1. Add `Request::Abort` beside `Close`, with a doc line.
2. Handle it in `host.rs`.
3. Tests in the host's tests (`src/host/tests.rs` or `src/tests/`, with their
   fixtures), one behaviour each:
   - `an_abort_aborts_each_connection_and_ends_each_run`;
   - `an_abort_after_a_close_changes_only_what_still_closes`;
   - `a_close_after_an_abort_changes_nothing`;
   - `closed_follows_an_abort_once`.
4. `cargo fmt`; `heavy cargo clippy -p skein-forge-setup --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-forge-setup`; `heavy cargo check -p <each
   direct dependent> --tests` (`cargo tree -i skein-forge-setup -e normal,dev --depth 1`).
   Commit and push.

## Report

What `Abort` does to runs and connections, the tests, the readers changed
(or none), and what forge-setup.md 6.3 needs.
