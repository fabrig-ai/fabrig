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
  `Lifecycle` and `request_close(abort)`): in-flight calls end with
  their connections and no `Answered`, as `Close` ends them
  (forge-connection.md, 8.5; see Answers), each socket is aborted (`skein_io::Request::Abort`, as
  `hooks.rs:157` and `connection.rs:931` already send per socket), and
  `Event::Closed` is its terminal. A repeated abort changes nothing.
- No TLS `close_notify` (draft, 3.6: it comes with the shared transport).
- The design document is the designer's: put what it should say in your
  report.

## Acceptance

- Step tests in the crate's `src/tests/`: abort while live (no
  `Answered` follows for a call in flight, held and returned answers
  are dropped, sockets aborted, then Closed); abort after close; close
  after abort; a second abort.
- `heavy cargo nextest run -p skein-forge-connection` and its world in
  `skein/tests/forge/` that reads it, with counts.
- The component's owners only construct `Request`: check they build,
  `heavy cargo check --tests -p temper-service -p
  temper-forge-setup-shell`. One that matches `Request` exhaustively is
  a `Question:`.

## Answers

2026-10-31. Which terminal represents a cancelled forge call, with
`Answered.result` an `api::Error` that has no cancellation variant?
None: `Event::Closed` alone, no `api::Error::Cancelled` and no new
`Event`. The design already ends every call a close finds open "with
its connection" and "no terminal is sent for it" (forge-connection.md,
8.5): the close comes after the owner's drain, and the owner takes no
answers by then. `Abort` ends calls the same way (retire each waiting
call, drop held and returned answers, as `Request::Close` does at
`component.rs:300`), then aborts each socket instead of closing it,
and `Closed` follows once io settles every socket. Abort after close
aborts what still closes gracefully; close after abort, or a second
abort, changes nothing. `api::Error` is skein-forge's, shared by every
forge client and matched by temper; a variant there, or a new `Event`,
is a public change outside this task (the same answer as
abort-setup's). Put in your report the sentence 8.5 should gain for
`Abort`; the design is the designer's.
