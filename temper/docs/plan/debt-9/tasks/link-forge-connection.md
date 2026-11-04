+++
id = "link-forge-connection"
part = "transport"
kind = "feature"
needs = ["link-crate"]
files = ["skein/crates/skein-forge-connection/**", "skein/tests/forge/machine/**", "skein/docs/design/forge-connection.status.md"]
effort = "high"
minutes = 120
+++

# skein: forge-connection on `Link`

## Goal

`skein-forge-connection/src/connection.rs`' `Connection` stands on `Link` and `Upload`, and sends `close_notify` on a graceful
close. It is the largest of the four, and the one whose close does most today: at base `close` sends the socket's `Close` at
once and abandons its queues.

Acceptance:

- `rg -n 'tls_up|tls_down|graceful' skein/crates/skein-forge-connection/src/connection.rs` finds nothing.
- A plaintext connection's `Close` still leaves in `close`, at once, as `tests/abort.rs` and `tests/runtime.rs` pin it.
- `abort` after a graceful close still aborts (programming-model.md, 5.2).
- The pin holds, and `skein/tests/forge/machine`'s suite passes, secure rows and all.
- `connection.rs`'s non-test lines fall.
- `forge-connection.status.md`'s 8.1 **Not yet** sentence is gone. Its **Departs** stand: they're D020's.

## Inputs

- `temper/docs/plan/debt-9/notes/common.md`, `temper/docs/plan/debt-9/notes/link.md`, `AGENTS.md`, `skein/AGENTS.md`.
- `skein/docs/design/link.md`, 3; `skein/docs/design/forge-connection.md`, 7.1, 8.1 and 8.5.

## Owns and may change

- Owns: `skein-forge-connection/src/**`, `skein/tests/forge/machine/**`.
- `skein-forge-connection/Cargo.toml`: one line, `skein-link.workspace = true`, and `Cargo.lock`.
- `skein/docs/design/forge-connection.status.md`: that sentence goes, and the date line.
- `closing`, `phase`, `written`, `deadline` stay, and the tests' reads of them. `graceful` goes.

## Steps

1. `Connection` per `notes/link.md`. `stream` returns on `Phase::Closed` only when the link is closed: closing, the link still
   needs `up`. `connect` reads `Start`.
2. `close` and `abort` gain `env`: 20 calls across `component.rs`, `calls.rs`, `github.rs`, `github_calls.rs`, each edited only
   for that. `abort` is `self.link.close(.., true, io)`.
3. `pump`'s early break on `closing` and `Phase::Closed` goes around the HTTP half only: `link.pump` still runs.
   `Moved::Sent` sets `written` and the phase, as `send` does at base, when a request is in hand.
4. `ready` adds `link.has_work()`, also in `Phase::Closed`. The pass in `component.rs` that skips closed connections must
   still pump them: the connection leaves the pass on io's `Closed`, as it does now.
5. `next_deadline` in `component.rs` adds `link.next_deadline()`; `fire` calls `link.fire`.
6. `at_origin` hands its client to `Link::new`, and `limits::http` reads `link.encrypted()`.
7. The pin: `skein/tests/forge/machine/src/peer.rs`' `Peer` holds the server. A story with a secure peer asserts `closed`
   after a graceful close. Two world facts to read before you judge a failure: `tests/forge/machine/src/machine.rs` skips the
   peers while one is paused, so another peer's TLS close stalls through a pause; and a `Room` draws from the seed.
8. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-forge-connection --all-targets -- -D
   warnings`; `heavy cargo nextest run -p skein-forge-connection` and `skein/tests/forge/machine`'s and
   `skein/tests/forge/protocol`'s packages; `heavy cargo check --tests -p temper-service`. This is a change of meaning for what
   stands on it, so also run `temper/tests/engine/protocol-forge`'s package once.
9. The citations check. Commit and push.

## Report

`notes/link.md`'s break, the common note's items, the heap figures in `tests/suite/memory.rs`, and the net lines. If you report
that no test's events moved, say how you checked.
