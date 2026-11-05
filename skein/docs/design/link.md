# link: one socket and its optional TLS

The transport under a protocol layer's HTTP clients: a socket's token, an
optional TLS client, the queues between them, the handshake, a body's upload,
and how a connection ends. Four components stand on it: skein's forge
connections (forge-connection.md), setup's host (forge-setup.md), OAuth's
accounts (oauth.md) and LLM connections (llm-connection.md). Read it with
[tls.md](tls.md), 3 and [io.md](io.md), 3.

## 1. In one page

- **A stream with a close rule.** A `Link` carries `lib::stream`'s `Down` and
  `Up` between its owner and a socket, through TLS or not, and the owner drives
  both alike: it reads `encrypted` only for the sizes that follow from it. It
  names no HTTP: what stands on it is the owner's.
- **A graceful close sends `close_notify`.** TLS closes first, what it wrote
  reaches io, then the socket closes. Only then.
- **An abort sends nothing:** the socket is aborted, at once, and an abort
  follows a graceful close that's still under way (programming-model.md, 5.2).
- **A graceful close is bounded.** io bounds a socket's close by its
  `close_timeout` and can't bound what comes before it, so the link does: a TLS
  close that hasn't reached the socket by `close_timeout` is aborted.
- **One move at a time.** Each call makes at most one io request, so an owner's
  reserved room holds (programming-model.md, 4.3 and 6.3).

## 2. In skein

`skein-link`: lib, io and TLS. It is a concrete step piece a component holds by
value, with no trait and no generic item.

- **Its role is the protocol layer's** (programming-model.md, 4): it depends on
  lib, io and a machine, and may depend on nothing above. It can't be a machine,
  which names no io, nor a composition, which names no io either, and io can't
  name TLS. It adds no role, so the code's rules don't change;
  `draft/component-kit.md`, 6.2 stays open.
- **It names no HTTP,** since `skein-llm-connection` reaches HTTP only through
  `skein-llm`.
- **What stays the owner's:** the socket's `Connect` and its owner token, its
  deadlines but the close's, what a failure means, and what "sent" means, which
  it reads from `Moved::Sent`.

## 3. The link

### 3.1 Its face

```rust
pub struct Limits { pub tls: tls::Limits, pub close_timeout: Duration }

pub enum Start { Handshaking, Ready, Closing }
pub enum Event { Ready, Stream(Up), Failed }
pub enum Moved { Nothing, Within, Sent, Above(Event) }

impl Link {
    pub fn new(tls: Option<tls::Client>) -> Link;
    pub fn encrypted(&self) -> bool;
    pub fn connecting(&mut self, socket: Token, below: &mut Queue<io::Request>);
    pub fn connected(&mut self, env: &Env<Limits>) -> Start;
    pub fn up(&mut self, env: &Env<Limits>, event: Up);
    pub fn down(&mut self, env: &Env<Limits>, down: Down);
    pub fn pump(&mut self, below: &mut Queue<io::Request>) -> Moved;
    pub fn close(&mut self, env: &Env<Limits>, abort: bool, below: &mut Queue<io::Request>);
    pub fn closed(&mut self);
    pub fn has_work(&self) -> bool;
    pub fn next_deadline(&self) -> Option<Time>;
    pub fn due(&self, now: Time) -> bool;
    pub fn fire(&mut self, now: Time, below: &mut Queue<io::Request>);
}
```

- **`encrypted`** says which, for the sizes an owner derives: what it can hand
  down at once is TLS's `send` with TLS, io's `output` without.
- **`connecting`** keeps io's token, and sends a close asked for before it.
- **`connected`** starts the handshake, `Handshaking`; with no TLS the stream is
  `Ready` at once; once closing it starts nothing, `Closing`.
- **`up`** is the socket's `Up`, **`down`** the owner's `Down`. Without TLS they
  pass through the link's queues as they are.
- **`pump`** makes one move. `Sent` is a non-empty `Send` handed to io;
  `Within` any other move; `Above` the owner's to act on; `Nothing` says there's
  none. It's called until `Nothing`, or until the owner's budget or room ends,
  and `has_work` says what's left.
- **`closed`** is io's `Closed`: nothing more leaves.
- **`due`** says `next_deadline` has come: `fire` at `now` aborts.
- **Its `Debug`** gives its queues' lengths, never what they hold: a body going
  down may be a token request (oauth.md, 4).

### 3.2 The order of a move

What TLS wrote goes below before anything TLS said is acted on. TLS answers a
close's last `Up::Room` with a `Send` and `Closed` in one step, and io drops a
stream request once it holds a socket's close (io.md, 3), so the other order
loses `close_notify`.

### 3.3 Close and abort

`Link` holds one state for how it ends, its data in it (programming-model.md,
5.4):

| State | Input | Next | Emits |
|---|---|---|---|
| Open | `close(false)`, with TLS | Flushing, until `now + close_timeout` | TLS's `Close` |
| Open | `close(false)`, without | Graceful | the socket's `Close` |
| Open, Flushing, Graceful | `close(true)` | Aborted | the socket's `Abort` |
| Flushing | TLS's `Closed` | Graceful | the socket's `Close` |
| Flushing | `fire` once `due` | Aborted | the socket's `Abort` |
| any but Closed | io's `Closed` | Closed | nothing |

- **Closing, the stream above is over.** `down` is dropped, and `pump` gives
  nothing but what TLS owes: what TLS would hand up is dropped. `up` still
  reaches TLS, which waits for room.
- **A close before the socket is known** holds its request until `connecting`.
  TLS that hasn't started closes at once, with nothing to write.
- **TLS's `Close` is asked once,** in Open: TLS panics at a second, and at a
  stream request after it. A failed TLS answers `Closed` at once.
- **`close` makes at most one io request.** Over TLS it makes none: `pump`
  carries the close. A close that changes nothing is no fault.
- **A close doesn't wait for the peer's `close_notify`** (RFC 8446, 6.1): io
  half-closes and discards until the end or its deadline.

### 3.4 Upload

`Upload` is a whole body going down a stream within the room each `Up::Room`
gives. It's beside the link because three of the four write it, and stands
apart from it, since a body may go through HTTP's framing.

```rust
impl Upload {
    pub fn new(body: Box<[u8]>) -> Upload;
    /// A request's optional body.
    pub fn of(body: Option<Box<[u8]>>) -> Option<Upload>;
    /// At `Up::Room`: the next piece of at most `room` bytes, and what follows
    /// it: `Finish` at the end, otherwise a demand for `room` more.
    pub fn room(&mut self, room: u32) -> (Down, Down);
}
```

An owner that acts at the end reads it as `next == Down::Finish`. Its `Debug`
gives the body's length, never its bytes, as the link's does.

`skein-llm-connection` has none: `skein-llm`'s client uploads its own request.

## 4. Testing

`skein/tests/link` drives a `Link` against skein's in-memory TLS server
(`skein/tests/tls/src/server.rs`). Each component's own world pins that its
graceful close reaches the server, whose `closed` says it read `close_notify`,
and that its stalled close is aborted at the link's deadline.

- A graceful close: `close_notify` is read, its `Send` precedes the socket's
  `Close`, and nothing follows it.
- An abort sends nothing, and follows a graceful close under way.
- A close after the peer's `close_notify`; before the handshake is done; before
  `Connecting`; twice.
- A close whose room never comes: `next_deadline` is `close_timeout` on, and
  `fire` aborts.
- A stream that fails during a close ends it, and the socket closes. One that
  ends leaves it to `fire`.
- A close after TLS failed: TLS answers `Closed` at once, and the socket closes.
- Without TLS, `close` makes its request at once.
- `Debug` shows no byte of a body or a queue.

## 5. Later

- **A deadline for the handshake or a read** stays the owner's. One here would
  need all four to agree on what it fails.
- **The connections' lifecycles** stay the owners' (`D023` of
  `skein/docs/development/debt/`): the link holds only its own.
- **Whether the link is a "component kit"** is `draft/component-kit.md`, 6.2.
- **`Debug` over what passes through.** `Event` and `Moved` derive it over an
  `Up`'s bytes, as lib's `Up` does: they carry those bytes and keep none, and
  `clippy::use_debug` denies formatting them. They stay so until something keeps
  one (oauth.md, 4).
