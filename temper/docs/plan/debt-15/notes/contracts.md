# debt-15: contracts for skein D011

The shapes that cross D011's tasks, binding. The design is `skein/docs/design/fake-forge.md`, 12. Below, `P` is
`skein/crates/skein-fake-forge-protocol/src`, the Forgejo fake, until `forgejo-rename` moves it, and `G` is
`skein/crates/skein-fake-forge-github/src`.

## 1. The neutral crate

`skein/crates/skein-fake-forge-http`, package `skein-fake-forge-http`, a step crate: `#![cfg_attr(not(test), no_std)]`,
`#![forbid(unsafe_code)]`, the workspace's lints. It has two public modules, `connection` and `git`, and the two types
`Hosted` and `Faults` at its root. It depends on skein-lib and skein-http; `route-move` adds skein-json, `skein-forge` and
`skein-fake-forge-domain`. It names no `forgejo::` or `github::` item.

## 2. `connection`

No closures, fn pointers, traits, generics or `impl Trait` (`skein/docs/foundation/programming-model.md`, 10): the core
answers an enum and the server matches it, as `skein_forge::forgejo::server::BodyStep` does.

```rust
pub enum State { New, Reading, Receiving, Body, Waiting, Streaming, Closing, Closed }

pub struct Connection {
    pub state: State,
    pub events: Queue<http::Event>,
    pub requests: Queue<http::Request>,
    pub http: http::Server,
    // private: the captured head, the reply in hand and its offset
}

/// One http event, as the server has to answer it.
pub enum Turn {
    /// A request's head. The core has kept its snapshot, `Authorization` blanked.
    Call(http::Call),
    Body(Up),
    Done(http::Reuse),
    /// The reply's stream failed, or http refused, failed or ended: the server closes.
    Failed,
    /// Nothing for the server: a reply's room served, or http's `Closed`.
    Handled,
}

pub struct Reply { pub status: u16, pub headers: Box<[Header]>, pub body: Box<[u8]>, pub bodyless: bool }

impl Connection {
    pub fn new(limits: &http::Limits) -> Self;
    pub fn take_head(&mut self) -> Option<http::Call>;
    pub fn head_pending(&self) -> bool;
    pub fn is_settled(&self) -> bool;
    pub fn has_work(&self) -> bool;
}
/// Hands `event` to http unless the state refuses it; false when refused, and the server's pass doesn't run.
pub fn up(c: &mut Connection, env: &Env<http::Limits>, event: Up, below: &mut Queue<Down>) -> bool;
/// The next http event, popped and answered as far as the core can.
pub fn turn(c: &mut Connection, limits: &http::Limits) -> Option<Turn>;
/// Hands http the oldest queued request; false when there is none.
pub fn request(c: &mut Connection, env: &Env<http::Limits>, below: &mut Queue<Down>) -> bool;
pub fn down(c: &mut Connection, env: &Env<http::Limits>, request: http::Request, below: &mut Queue<Down>);
/// `state = Reading`, and asks http for the next request.
pub fn read_next(c: &mut Connection);
pub fn respond(c: &mut Connection, limits: &http::Limits, reply: Reply);
/// Drops the reply and both queues, closes http, `state = Closing`.
pub fn close(c: &mut Connection, env: &Env<http::Limits>, below: &mut Queue<Down>);
```

`up` and `State`:

- `up` refuses in `New`, `Closing` and `Closed`, and turns `Reading` into `Receiving` on `Up::Bytes`.

`turn`:

- `http::Event::Call`: asserts no head is held, keeps the snapshot, answers `Turn::Call`.
- `http::Event::Reply(Up::Room)`: sends the next chunk of `limits.send`, then `Down::Finish` at the end or the next
  `Demand`, and answers `Turn::Handled`.
- `http::Event::Reply(Up::Bytes(_) | Up::End | Up::Failed(_))`, `Refused`, `Failed` and `Ended`: `Turn::Failed`.
- `http::Event::Closed`: `Turn::Handled`.

`respond` is Forgejo's tail of `respond` as it stands, with `reply.bodyless` for its `status == 204`:

- `state = Streaming`, `offset = 0`, the reply in hand is `body` unless it's empty;
- `http::Request::Respond` with `Body::None` when `bodyless`, else `Body::Length(length)`, `close: false`;
- with an empty body and not `bodyless`, `Reply(Down::Finish)`; with a body, `Reply(Down::Demand { read: Read::Nothing,
  room })`, `room` the length within `limits.send`.

GitHub passes an empty body when its reply is bodyless, as it zeroes the length today.

**The core reads `send` from `limits`, per call.** `Connection` keeps no `send`. Forgejo's stored field goes.

**What a server keeps:** its `Event`; its wrappers (`up`, `resume`, `start`, `closed`); its pass, with its bound of 8 and
its break on `head_pending` after every event; `next` and its draw; `down`; its body's reading; its headers; `retire`.
A server's `close` clears what's its own, closes its own (Forgejo's json), then calls `connection::close`, then pushes
`Event::Close`, in that order. The core draws no chance and pushes no `Event`.

## 3. `git`

Written in `P/git.rs` by `route-limits`, moved whole to `skein_fake_forge_http::git` by `route-move`.

```rust
/// What the route reads (fake-forge.md, 12).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub http: skein_http::server::Limits,
    pub documents: skein_forge::limits::Limits,
}

/// A segment under the route's prefix that names no call.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unrouted { NotFound }

/// A domain error's status on the route.
#[must_use]
pub fn status(error: api::Error, name: Name) -> u16;

// Forgejo's layer:
impl protocol::Limits { #[must_use] pub const fn git(&self) -> git::Limits; }
```

- `name`, `fits`, `decoder_bound`, `store_answer`, `client::*` and `server::*` keep their names and take `git::Limits`
  where they took `protocol::Limits`.
- `git::status` gives, for every `api::Error` and `Name`, the `status` `errors::answer(error, Route::Git(name), now)`
  gives. `RateLimited` is 429 whatever the clock. A table test in Forgejo's crate pins the two together.
- GitHub's layer builds `git::Limits` from its own limits, with the values it gives today, and calls `git::status`.

## 4. A commit's bytes

In `skein_fake_forge_domain`, `object.rs`, exported beside `object_bytes`:

```rust
/// A commit count's 32 bytes: its SHA-1 spelling, padded.
#[must_use]
pub fn commit_bytes(count: u64) -> [u8; 32];
/// The count those bytes spell, if they're the fake's.
#[must_use]
pub fn commit_count(bytes: &[u8; 32]) -> Option<u64>;
```

`P/routes.rs`' `object` and `commit` stay, with their signatures, over these two.

## 5. Names at the end

- The Forgejo fake is `skein/crates/skein-fake-forge-forgejo`, package `skein-fake-forge-forgejo`, library
  `skein_fake_forge_forgejo`.
- Forgejo's crate keeps `pub use skein_fake_forge_http::{Faults, Hosted};`, as they are types of its own signatures.
  GitHub's keeps `pub use skein_fake_forge_http::Faults;`.
- `git` stays re-exported from Forgejo's crate from `route-move` to `forgejo-rename`, so temper's paths don't move twice;
  `forgejo-rename` removes it, and every reader names `skein_fake_forge_http::git`.
