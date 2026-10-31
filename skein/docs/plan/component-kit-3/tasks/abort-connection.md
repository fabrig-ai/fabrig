+++
id = "abort-connection"
part = "abort"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge-connection/**"]
effort = "high"
minutes = 50
+++

## Goal

skein-forge-connection takes an abort from its owner, as every component
must (`skein/docs/foundation/programming-model.md`, 5.2; the review's D9,
quoted in `skein/docs/design/draft/component-kit.md`, 3.6). Today its
boundary has `Request::Close` only. The TLS `close_notify` is not part of
this task.

Acceptance:
- `Request::Abort` in `crates/skein-forge-connection/src/boundary.rs`: a
  close that does not wait. It ends what is open as `Close` does
  (`skein/docs/design/forge-connection.md`, 8.5: each open call ends with
  its connection and has no answer; held and returned answers are dropped),
  and aborts what it keeps (its connections, the listener and each webhook
  connection) with io's `Request::Abort` instead of a graceful close.
- An abort after a close turns what still closes gracefully into aborts; a
  close after an abort changes nothing; a second abort changes nothing.
  `Event::Closed` goes up once, last, after either, when io has settled
  every socket.
- Tests in the crate's own tests show each of those, and the crate's tests,
  clippy and its direct dependents' tests pass.

## Inputs

- `skein/docs/plan/component-kit-3/notes/common.md`.
- `skein/docs/foundation/programming-model.md`, 5.2 (components follow the
  lifecycle; the abort's contract).
- `skein/docs/design/forge-connection.md`, 2.1 (what crosses), 8.1 (a forge
  connection), 8.4 (a webhook connection) and 8.5 (ending).
- `crates/skein-forge-connection/src/component.rs` (`Request::Close`'s
  handling), `src/connection.rs`, `src/hooks.rs` and `src/hooks/connection.rs`
  (how each closes its socket); `skein-io`'s `Request::Abort`.

## Owns and may change

- Owns `skein/crates/skein-forge-connection/**`.
- Readers that match `skein_forge_connection::Request` exhaustively (search
  every tree: `rg -n 'forge_connection::Request|Request::Close' skein temper`)
  get the new arm only, handled as their `Close` arm is or as unreachable
  where the reader never sees an owner's request; call sites only. Today
  the readers construct requests rather than match them; if none matches,
  nothing outside the crate changes.
- Not `docs/design/` (forge-connection.md's 2.1 lists the requests: the
  report says what to add there).
- Pre-approved:
  - The close state: if the component keeps `closed: bool` and per-entity
    `closing` flags, an `aborting` bit or a small private enum beside them
    is yours to choose; lib's `Lifecycle` arrives in another task and is
    not required here.
  - If an entity has no abort path below (a webhook connection that only
    closes gracefully), give it one through io's `Request::Abort`.
  - Owners that send `Close` today keep sending it; no owner is changed to
    send `Abort` in this task.

## Steps

1. Add `Request::Abort` with a doc line in `boundary.rs`'s style, beside
   `Close`.
2. Handle it in `component.rs` and down to each kept entity.
3. Tests in `src/tests/` (the crate's existing fixtures and harness), one
   behaviour each:
   - `an_abort_ends_open_calls_and_aborts_each_connection`;
   - `an_abort_after_a_close_aborts_what_still_closes`;
   - `a_close_after_an_abort_changes_nothing`;
   - `closed_follows_an_abort_once_io_has_settled`.
4. `cargo fmt`; `heavy cargo clippy -p skein-forge-connection --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-forge-connection`; then the direct
   dependents (`cargo tree -i skein-forge-connection -e normal,dev --depth 1`):
   `heavy cargo check -p <each> --tests`, and their tests where an arm changed.
   Commit and push.

## Report

What `Abort` does in each entity, the tests, and the readers changed (or
that none needed it), and the wording forge-connection.md 2.1 and 8.5 need.
