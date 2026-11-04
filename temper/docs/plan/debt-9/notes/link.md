# debt-9: the four components on `Link`

Every `link-*` task names this note, beside `notes/common.md`. `skein/docs/design/link.md`, 3 is the contract, and
`skein-link` is on the trunk when these tasks start.

## What this item changes

**One behaviour:** a graceful close over TLS sends `close_notify`, so 2 to 3 `Stream` requests now precede the socket's
`Close`, `Close` waits for one `Room`, and a component's `Closed` comes later. Its events keep their order. The link's
close deadline is what keeps that bounded. Nothing else moves. Every other visible difference is a stop and ask.

Plaintext is untouched: the same requests, at the same moment.

## What keeps each owner as it is

Each component keeps these, because tests see them:

| | forge-connection | forge-setup | oauth-accounts | llm-connection |
|---|---|---|---|---|
| `Close` or `Abort` on a connecting socket | its callers' choice, `in_flight` aborts | `Close` | `Close` | `Abort` |
| when a plaintext `Close` leaves | in `close`, at once | in `pump` | in `close` | in `route` |
| evidence that bytes went | `Moved::Sent` with a request | none | a non-empty plain `Send` | deadline on a non-empty `Send` in Head |
| TLS `Failed` | `Seen` or `Unseen` by `written` | `fail()` | only in Handshaking; HTTP heard it otherwise | `llm::abort` |

## The shape of a migration

- The transport's fields go: socket, TLS client, `tls_up`/`tls_events`, `tls_down`/`cipher`/`cipher_down`, and the close
  flags. A `link: Link` stands for them. HTTP's or LLM's own queues stay.
- Each owner gets one private helper:

  ```rust
  fn link_env(env: &Env<Limits>) -> Env<skein_link::Limits> {
      Env { now: env.now, wall: env.wall, limits: skein_link::Limits { tls: env.limits.tls, close_timeout: env.limits.io.close_timeout } }
  }
  ```

  All four hold `io::Limits`. In forge-connection `tls` and `io` sit on `Limits` and `shared`: read them where they are.
- `io::Event::Connecting` calls `link.connecting`; `Connected` calls `link.connected`, whose `Start` picks the owner's next
  phase; a stream `Up` calls `link.up`; io's `Closed` calls `link.closed`.
- The owner's pump pops `self.link.pump(io)` where it popped TLS's queues, within its own budget, and leaves its own
  order among what's left:

  ```rust
  match self.link.pump(io) {
      Moved::Above(event) => { /* the arms that were on tls::Event, less Closed */ }
      Moved::Sent => { /* its evidence */ }
      Moved::Within => {}
      Moved::Nothing => { /* its next source */ }
  }
  ```

- **`has_work`, or `ready`, adds `link.has_work()`,** and a closing connection stays pumped until `Nothing`. This is the
  easy one to miss, as each owner stops pumping when it closes. Without it the close stalls, and the deadline turns it
  into an abort: `close_notify` never goes, with no test failing unless your pin runs.
- **`next_deadline` adds `link.next_deadline()`, and `fire` calls `link.fire(now, io)`.** Add them where the owner
  reads the connection's deadlines.
- **`Upload`:** forge-connection, forge-setup and oauth-accounts take `skein_link::Upload` in place of their body and
  offset. `Upload::room` gives the `Send` and what follows it, each passed on as `http::Request::Upload(..)`, as they
  are at base. llm-connection has none.
- A test that set a private field names a moved item: `connection.socket = Some(token)` becomes
  `link.connecting(token, &mut io)`.

## Tests

- **Your pin:** in your component's lowest world with a TLS server, a graceful close ends with the server's `closed`
  true (`skein/tests/tls/src/server.rs`, whichever server your world holds). Break it by hand: skip `link.has_work()` in
  your owner's check, and see the pin fail.
- **Replays.** A world's `Room` draws from its seed (`skein/testing/skein-world/src/stream.rs`, `Wire::answer`), so secure
  replays take another path. Expected values should hold, and a seed-pinned outcome that moves is a stop and ask.
- **Heap ceilings:** the two `Stream`s are small, but `skein/tests/forge/setup/tests/suite/heap.rs` and
  `skein/tests/forge/machine/tests/suite/memory.rs` hold counted peaks. A ceiling that's passed is a stop and ask, with
  the measured figure: never raise it.

## Not yours

Each component's lifecycle stays: `D020`, `D022`, `D023` of `skein/docs/development/debt/` are other items. Don't fold
`Phase` or `Control` into the link, and don't touch `skein-link`: a change its interface needs is a question.
