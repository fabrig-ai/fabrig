+++
id = "connection-entrances"
part = "forge-connection"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge-connection/src/component.rs", "skein/crates/skein-forge-connection/src/component/runtime.rs", "skein/crates/skein-forge-connection/src/boundary.rs", "skein/crates/skein-forge-connection/src/tests/abort.rs", "skein/crates/skein-forge-connection/src/tests/runtime.rs", "skein/docs/design/forge-connection.status.md"]
effort = "medium"
minutes = 30
+++

# forge-connection answers a late `Call`, and a duplicate `Add` is ignored everywhere

## Goal

Two faults at `skein-forge-connection`'s entrance. (Bug 7, re-checked: both hold.)

- **7a:** `component.rs`'s `down` drops a `Request::Call` while Closing or Aborting, with no terminal.
- **7b:** `component/runtime.rs`'s `add` calls a `#[cfg(test)] unique` that panics on a duplicate number
  in this crate's unit tests alone, so the path everyone else runs is untested and a test pins a
  panic.

Acceptance:

- A `Call` while Closing or Aborting pushes `Event::Answered(Answered { call, cost: 0, free: 0, rate:
  None, result: Err(api::Error::Unavailable) })` and returns. `Abort` while Closing is as it is.
  `Closed` is as it is: `tests/abort.rs` sends requests after closed.
- `unique` is gone; a duplicate `Add` is ignored, as `skein/docs/design/forge-connection.md`, 3.1 says.
- The tests below fail on the old code.

## Inputs

- `skein/docs/plan/bugs-skein/notes/common.md`.
- `skein/docs/design/forge-connection.md`, 3.1 (a number the component holds is ignored), 8.5
  ("A call after the close").
- `skein/docs/foundation/programming-model.md`, 5.2.

## Owns and may change

- Owns: `src/component.rs` (`down`), `src/component/runtime.rs` (`add`, `unique`), `src/boundary.rs`
  (the doc comments of `Call` and `Close`, one line each), `src/tests/abort.rs`, `src/tests/runtime.rs`.
- `skein/docs/design/forge-connection.status.md`: remove this plan's **Not yet** entry
  (`notes/common.md`).
- No reader changes. Senders of `Request::Call` (`temper-service/src/route.rs`, its tests,
  `skein/tests/forge/conformance`) send none after the close (surveyed); temper's `Add` returns early
  for a number held. `tests/reads.rs` asserts no answer for a call opened before the close, which
  stands.

## Steps

1. In `src/tests/abort.rs`, write `a_call_after_the_close_is_answered_unavailable`, modelled on
   `abort_after_close_turns_the_graceful_close_into_an_abort`: `fixtures::known()`, `h.run(ZERO)`,
   `h.send(Close)`, `h.call(1, ..)`, then pop `h.up` directly before `h.run` settles io, and assert one
   `Answered` with cost 0 and `Unavailable`; again after `h.send(Abort)`.
2. In `src/tests/runtime.rs`, replace `a_duplicate_number_asserts_only_in_tests` with
   `a_duplicate_number_is_ignored`, modelled on `an_add_past_the_bound_is_ignored`: add the forge with
   `access = None`, add the number again with access, then `start` is still `Unconnected`,
   `connections_opened() == 0` and `h.up` is empty.
3. Run both on the old code and quote the failures. Then the fixes.
4. `heavy cargo nextest run -p skein-forge-connection --lib`, then the focused tests of
   `skein-forge-connection`, `skein-forge-protocol-world`, `forgejo-conformance`. `heavy cargo check -p
   temper-service --tests`. `cargo fmt`, `python3 -I -B skein/scripts/old_forms.py`, `heavy cargo
   clippy -p skein-forge-connection --all-targets -- -D warnings`.
5. Commit, push.

## Report

As the implementer's text says, with what the tests showed on the old code.
