+++
id = "link-crate"
part = "transport"
kind = "feature"
needs = []
files = ["skein/crates/skein-link/**", "skein/tests/link/**", "Cargo.toml", "Cargo.lock"]
effort = "high"
minutes = 110
+++

# skein: `skein-link`, one socket and its optional TLS

## Goal

Build `skein/docs/design/link.md`: the `Link` and `Upload` of its section 3, and its world. No component moves yet.

Acceptance:

- `skein-link` holds 3.1's face as written, and 3.3's table as one enum, each state holding its data (`Flushing { until }`).
- `skein/tests/link` holds link.md, 4's stories, against a real rustls server.
- `src` is at most 320 lines, tests apart: this is what replaces four copies.

## Inputs

- `temper/docs/plan/debt-9/notes/common.md`, `AGENTS.md`, `skein/AGENTS.md`.
- `skein/docs/design/link.md`, whole; `skein/docs/design/tls.md`, 3.1 and 3.3; `skein/docs/design/io.md`, 3;
  `skein/docs/foundation/programming-model.md`, 5.2, 5.4 and 10.
- `skein-tls/src/client/close.rs`, what TLS does at `Close`. `skein-llm-connection/src/call.rs`' `route` and
  `skein-oauth-accounts/src/exchange.rs`' `close_progress` show the wrong order and the right one.

## Owns and may change

- Owns: `skein/crates/skein-link/**` and `skein/tests/link/**`, both new.
- The root `Cargo.toml`: one `members` line for each, in order, one `skein-link` line in `[workspace.dependencies]`; and
  `Cargo.lock`. No other manifest: the components add their dependency in their own tasks.
- A world needs its `tests/suite/main.rs` layout: follow `skein/tests/llm-connection` (`skein/docs/design/testing.md`, 6).
  It names `skein-tls-world = { path = "../tls" }` for the server, as `skein/tests/oauth-accounts` does.
- If a design line can't be built as written, stop and ask: the four tasks after this one are written to it.

## Steps

1. `skein/crates/skein-link`: `Cargo.toml` on `skein-lib`, `skein-io`, `skein-tls`, under the workspace's lints; `no_std`;
   `src/lib.rs` citing `skein/docs/design/link.md`, `src/link.rs`, `src/upload.rs`.

   ```rust
   enum Ending { Open, Flushing { until: Time }, Graceful, Aborted, Closed }

   pub struct Link {
       socket: Option<Token>,
       tls: Option<tls::Client>,
       encrypted: bool,
       events: Queue<tls::Event>,   // what TLS said, and without TLS what the socket said, as `Stream`
       cipher: Queue<Down>,         // what leaves for the socket
       ending: Ending,
       /// A close asked before `Connecting`: the socket's request, which `connecting` sends.
       held: Option<Ask>,
   }
   ```

   The fields are a sketch; 3.1 binds.
2. **The rules of 3.3 are where this goes wrong:**
   - `tls::Request::Close` once, from Open. TLS panics at a second, at `Stream` after it and at `Handshake` after it, so
     `down` and `connected` check `Ending` first.
   - Once closing, `up` reaches TLS only in `Flushing`, and is dropped in Graceful, Aborted and Closed.
   - `pump`, once closing, gives no `Above`: it drops `Ready`, `Stream` and `Failed`, and turns `Closed` into the socket's
     `Close` and Graceful. It moves `cipher` first, always (3.2).
   - Abort clears both queues, so `has_work` goes false.
   - `Moved::Sent` is a non-empty `Down::Send`; an empty one, a demand or a finish is `Within`.
   - `has_work` is true while either queue holds anything.
3. `Upload` per 3.4. `room` is `(Down::Send(piece), next)`, where `next` is `Down::Finish` at the end, otherwise
   `Down::Demand { read: Read::Nothing, room }`. A body of no bytes gives an empty `Send` and `Finish`.
4. The world, in `skein/tests/link`: a small harness playing io by hand, carrying `below`'s `Stream` requests to
   `skein_tls_world::server::Server` and its flight back as `Up`s, and answering demands itself, so a test can withhold
   `Up::Room`. Stories, each one test:
   - `a_graceful_close_sends_close_notify_before_the_socket_closes`: `closed`, and the `Send`'s index is below `Close`'s;
   - `nothing_follows_the_sockets_close`;
   - `an_abort_sends_nothing`; `an_abort_follows_a_graceful_close`, in Flushing and in Graceful;
   - `a_close_after_the_peers_close_notify`;
   - `a_close_before_the_handshake_is_done`, `a_close_before_connecting`, `a_second_close_changes_nothing`;
   - `a_close_whose_room_never_comes_is_aborted_at_its_deadline`: `next_deadline` is `now + close_timeout`, `fire` before it
     does nothing, `fire` at it aborts;
   - `without_tls_a_close_is_at_once`, `without_tls_the_stream_passes_through`;
   - `upload_gives_pieces_then_finish`, in `skein-link`'s own `src`.
5. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-link -p skein-link-world
   --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-link -p skein-link-world`; `heavy cargo check -p skein-link`
   alone, as skein-tls is.
6. The citations check. Commit and push. `link.status.md` stays: `link-close` writes it.

## Report

The common note's items. Two breaks at least: swap `pump`'s two sources and name the failure of the first story; remove the
`Flushing` deadline and name the failure of the room's story. Say whether the close after the peer's `close_notify` passed
at once: `skein-tls/src/session.rs`' `close_notify` has an `unreachable!` that, by reading rustls 0.23.41, this path doesn't
reach. If it does, stop and ask: that's a panic to fix in `skein-tls`, and the forge pair would reach it.
