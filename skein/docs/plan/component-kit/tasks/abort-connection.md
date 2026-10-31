+++
id = "abort-connection"
part = "abort"
kind = "feature"
repo = "fabrig"
needs = ["ratchets"]
files = ["skein/crates/skein-forge-connection/src/**", "skein/crates/skein-forge-connection/tests/**", "skein/scripts/old-forms.txt"]
effort = "high"
minutes = 45
+++
# Abort in skein-forge-connection

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, 3.6; review D9; `skein/docs/foundation/programming-model.md`,
5.2 (an abort is "a close that does not wait, which ends each request it
holds as cancelled and aborts what it keeps. An abort after a close turns
what still closes gracefully into aborts; a close after an abort changes
nothing. `Closed` follows either.").

## What changes

- `src/boundary.rs:59` `Request` gains `Abort`, documented beside
  `Close` in 5.2's words.
- The component handles it as skein-llm-connection does
  (`skein/crates/skein-llm-connection/src/component.rs:339-357`, its
  `Lifecycle` and `request_close(abort)`): in-flight calls end as
  cancelled, each socket is aborted (`skein_io::Request::Abort`, as
  `hooks.rs:157` and `connection.rs:931` already send per socket), and
  `Event::Closed` is its terminal. A repeated abort changes nothing.
- No TLS `close_notify` (draft, 3.6: it comes with the shared transport).
- The design document is the designer's: put what it should say in your
  report.

## Acceptance

- Step tests in the crate's `src/tests/`: abort while live (calls end
  cancelled, sockets aborted, then Closed); abort after close; close
  after abort; a second abort.
- `heavy cargo nextest run -p skein-forge-connection` and its world in
  `skein/tests/forge/` that reads it, with counts.
- The component's owners only construct `Request`: check they build,
  `heavy cargo check --tests -p temper-service -p
  temper-forge-setup-shell`. One that matches `Request` exhaustively is
  a `Question:`.
