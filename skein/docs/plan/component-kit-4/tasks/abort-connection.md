+++
id = "abort-connection"
part = "abort"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge-connection/**"]
effort = "high"
minutes = 75
+++

## Goal

skein-forge-connection takes an `Abort` request, as programming-model.md
5.2 requires of every component (`skein/docs/design/draft/component-kit.md,
3.6`; the review's D9): a close that does not wait. Each call it holds ends
with its one terminal, as cancelled; every socket and the webhooks'
listener it keeps is aborted at io (`skein_io::Request::Abort`), not
closed; `Event::Closed` follows once io has settled them. TLS
`close_notify` is not part of this task.

Acceptance:

- `Request::Abort` exists, documented in the words of 5.2.
- Tests in the crate's own `src/tests/` show, each in one test: an abort
  while live ends every held call with one cancelled terminal and then
  `Closed`; an abort after a close turns what still closes gracefully into
  aborts (io sees `Abort`, not only `Close`, for each entity still
  closing); a close after an abort changes nothing; a second abort changes
  nothing; requests after an abort are ignored as after a close.
- The crate's tests, clippy, and the tests of its readers named in step 4
  pass.

## Inputs

- `skein/docs/plan/component-kit-4/notes/common.md`.
- `skein/docs/foundation/programming-model.md, 5.2` (lifecycle, the
  abort bullet) and `5.4`.
- `skein/docs/design/forge-connection.md, 2` (what crosses; `Close`) and
  `7` (entities and limits); read the design's text on cancellation to find
  how a call's cancelled terminal is spelt (an `Answered` with a refusal or
  failure the design names). If the design names none, pick the nearest
  existing terminal the owner already handles at close, say which and why.
- The crate: `src/boundary.rs` (`Request`), `src/component.rs:300`
  (`Request::Close`), `src/connection.rs` (each connection's close),
  `src/hooks.rs` and `src/hooks/connection.rs` (the listener and its
  connections), `src/tests.rs:140-165` (the close tests to model).
- A component that has it: `skein-llm-connection` (`Request::Abort` in
  `src/boundary.rs:68`, its handling in `src/component.rs`).

## Owns and may change

- Owns `skein/crates/skein-forge-connection/`.
- Readers that match `Request` exhaustively need an arm: in
  `skein/tests/forge/machine/`, `skein/tests/forge/protocol/` and
  `temper/crates/temper-service/`, `temper/tests/engine/protocol-forge/`,
  you may add the `Request::Abort` arm only (the same handling their
  `Close` arm has, or `unreachable!` where they never send it, with its
  reason), nothing else. Constructing sites need no change.
- Do not add `Abort` handling to the owners (temper's routing): sending an
  abort is a later task's.

## Steps

1. Add `Request::Abort` after `Close`, with a doc that says it is 5.2's
   abort.
2. Handle it in the component: a lifecycle of live, closing, aborting,
   closed held in one place (a private enum is fine here: task lib-kit's
   shared `Lifecycle` is not on your base, and a later task moves this
   crate onto it), every connection and the listener aborted, every held
   call answered once as cancelled, `Closed` after io settles. Keep each
   entity's deadlines a function of its state (5.4).
3. The tests of the Goal, in `src/tests/` (a new `src/tests/abort.rs` if
   the layout has topic files), using the crate's harness as the close
   tests do.
4. `cargo fmt`; `heavy cargo clippy -p skein-forge-connection --all-targets
   -- -D warnings`; `heavy cargo nextest run -p skein-forge-connection`;
   then `heavy cargo check --tests` on each reader package you changed and
   on `temper-service` whether or not you changed it.
5. `old_forms.py --write` (common.md: a new `enum Lifecycle` would raise a
   count: name the private enum otherwise, e.g. `Phase` or `Ending`, or
   reuse an existing state); commit and push.

## Report

As common.md says, plus the terminal you chose for a cancelled call and
where the design says it.
