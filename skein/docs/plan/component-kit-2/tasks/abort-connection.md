+++
id = "abort-connection"
part = "abort"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge-connection/src/**", "skein/tests/forge/**"]
effort = "high"
minutes = 45
+++
# `Request::Abort` in skein-forge-connection

## Goal

Draft, 3.6; programming-model.md, 5.2: every component takes an abort,
"a close that does not wait". skein-forge-connection takes none today.
Add `Request::Abort`: it ends what is open as `Close` does
(forge-connection.md, 8.5) and aborts every socket it keeps instead of
closing it gracefully; `Event::Closed` stays the one terminal. The TLS
`close_notify` is not part of this task.

Acceptance:

- `Request::Abort` exists, with a `///` contract in the boundary's words.
- In-crate tests, one behaviour each, pass:
  - an abort with calls waiting and in flight sends `skein_io::Request::Abort`
    for each socket (no graceful `Close`), no `Answered` for any call,
    and `Event::Closed` once io has settled them;
  - an abort after a close turns the sockets still closing gracefully
    into aborts, and `Closed` still goes up once;
  - a close after an abort, and a second abort, change nothing: no
    second `Closed`, no request to io.
- `heavy cargo nextest run -p skein-forge-connection` passes, and the
  readers' suites below.
- `heavy cargo check -p temper-service --tests` passes (temper's owner of the component).

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`.
- `skein/docs/foundation/programming-model.md`, 5.2 (the lifecycle,
  close and abort); `skein/docs/design/forge-connection.md`, 2 (the
  boundary) and 8.5 (ending).
- The code: `src/boundary.rs:59-82` (`Request`, `Close`'s contract),
  `src/component.rs:300-321` (`Close`), `src/connection.rs:920`
  (`Connection::close(abort, io)`), `src/hooks.rs:106-160` (`close`,
  the webhook connections' aborts). Close's tests: `src/tests.rs:140-170`.

## Owns and may change

- skein-forge-connection's `src/**`.
- Readers in `skein/tests/forge/**` that match `skein_forge_connection::Request`
  exhaustively: add `Request::Abort` to the arm that holds
  `Request::Close`, nothing else. The survey found none; `skein/tests/forge/conformance`
  and `skein/tests/forge/machine` construct requests only.
- Nothing else. `lib-forge-connection` and `stage-forge` change these
  crates later; leave pop loops, deadline merges and io stages as they
  are.

Settled (do not ask):

- **No new event or error variant.** No `Event::Cancelled`, no
  `api::Error::Cancelled`: temper matches the component's events and
  errors exhaustively, so a new terminal changes a public type it uses.
  Each open call ends with its connection and no answer, as under
  `Close`.
- **Order of close and abort** follows 5.2: abort after close upgrades
  what still closes gracefully; close after abort changes nothing;
  either is followed by one `Closed`.
- If `Close`'s path and `Abort`'s share everything but the socket's
  `abort` flag, write one private function taking it.

## Steps

1. `Request::Abort` in `src/boundary.rs`, after `Close`:

   ```rust
   /// After the root's drain, a close that does not wait
   /// (programming-model.md, 5.2): calls end with their connections and no
   /// answer goes up; every socket, the listener and each webhook
   /// connection is aborted. After a `Close` it aborts what still closes;
   /// `Event::Closed` answers either, once.
   Abort,
   ```
2. The component handles it beside `Close` (`src/component.rs`); the
   connections close with `abort = true`; the hooks take an abort (a
   `hooks.abort()` beside `hooks.close()`, or a flag on it).
3. The three tests above, in `src/tests.rs` on its `Harness`.
4. Checks of `common.md`, then the readers: `heavy cargo nextest run -p
   forgejo-conformance -p skein-forge-machine-world` (take the package
   names from `skein/tests/forge/conformance/Cargo.toml` and
   `skein/tests/forge/machine/Cargo.toml`) and the temper check above.
   Commit; push.

## Report

The variant's contract as written, how close and abort share their
path, the tests added (names, counts), every reader you edited, the
checks run.
