# worker-link-quiet: the shapes that cross tasks

These bind. Names, fields and words are as written here; what is inside
a function is the task's.

## 1. The room

Each link component is given its stream's room: the most bytes the
stream below grants at once, io's `Limits::largest_room()` (its
`output`), in one process as between hosts. A frame's size is its wire
length, the eight-byte header included (`skein_channel::Frame::wire_len`).

## 2. The engine's side, `jig-protocol-engine-workers` (task `engine-room`)

```rust
// limits.rs
pub struct Limits {
    pub sizes: Sizes,
    pub names: u32,
    pub workers: u32,
    pub opening: u32,
    pub chunk: u32,
    /// The most bytes the stream below grants at once: io's
    /// `largest_room` (protocol/channel.md, 9 and 11).
    pub room: u32,
    pub handshake: Duration,
    pub hello: Duration,
    pub ping: Duration,
    pub silence: Duration,
    pub stall: Duration,
}
```

`valid(&Limits)` also holds, besides what it holds now:

- `room` is at least 270, a `Refuse`'s frame (skein's `channel.md`,
  5.1: header, reason, length and 256 bytes of text);
- for every kind the engine sends (`wire::KINDS` whose direction is
  `FromResponder`) but `Kind::Assignment` and `Kind::Job`,
  `sizes::largest_body(&sizes, kind) + 8 <= room`.

```rust
// boundary.rs, exported from lib.rs
/// A record the engine's side did not send (protocol/channel.md, 9).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Unsent {
    pub kind: jig_protocol_worker_wire::Kind,
    /// Its frame's bytes, the header included.
    pub bytes: u32,
    pub bound: Bound,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bound {
    /// Past the stream's room, in bytes.
    Room(u32),
    /// Refused at skein's channel's entrance; never `Unsent::Full`, which
    /// loses the link as it does now.
    Channel(skein_channel::Unsent),
}

pub enum Request {
    // ... as now, and:
    /// Once for each record the component did not send.
    Unsent { worker: u32, channel: Token, unsent: Unsent },
}

// host.rs
pub enum Output {
    // ... as now, and:
    Unsent { worker: Box<[u8]>, channel: Token, unsent: Unsent },
}
```

`kit::host::Workers` maps `Request::Unsent` to `Output::Unsent` with the
worker's name, as it maps `Request::Ended`.

**What happens to a record not sent.** Measured before the channel takes
it; past `room`, it is never handed to the channel. A channel `Unsent`
other than `Full` for a record is the same case with `Bound::Channel`.
For each such record, in this order:

1. `Request::Unsent` once;
2. an assignment: the upward answer `assignment_refused` gives now, with
   the words below in place of `assignment cannot encode`;
3. a job: the upward job answer `LinkUp::JobAnswer` with
   `JobEnding::Refused(JobRefusal::Invalid(JobInvalid::Start { why }))`,
   the words below, as the hub answers a job it refuses at its entrance;
4. a read list: the `ListRead` failure the component gives now for one
   that cannot encode;
5. any other record: dropped, counted in `dropped_down`.

The connection stays as it was: no loss, its ping deadline armed as if
nothing had been queued, the next record sent.

**The words** (ASCII, cut to `sizes.detail` as now):

| Case | Words |
|---|---|
| past the room | `assignment of BYTES bytes past the link's room of ROOM bytes` |
| refused by the channel | `assignment of BYTES bytes refused by the channel: REASON` |
| a job | the same with `job` for `assignment` |
| a job that cannot encode | `job cannot encode` |

`REASON` is `wrong-direction`, `not-taken`, `too-large` or `closed`, for
skein's `WrongDirection`, `PeerDoesNotTake`, `TooLarge` and `Closed`.
`BYTES` and `ROOM` are plain decimals (`skein_lib::Decimal`).

## 3. The worker's side, `jig-protocol-worker-link` (task `worker-room`)

```rust
// limits.rs
pub struct Limits {
    pub sizes: Sizes,
    pub chunk: u32,
    /// The most bytes the stream below grants at once: io's
    /// `largest_room` (protocol/channel.md, 9 and 11).
    pub room: u32,
    pub handshake: Duration,
    // ... the rest as now
}
```

`valid(&Limits)` also holds: `room` is at least 270 and at least the
`Open` frame (the credential's bound, plus 14, plus 8); and for every
kind the worker sends (`wire::KINDS` whose direction is
`FromInitiator`), `sizes::largest_body(&sizes, kind) + 8 <= room`.
`Component::new` refuses limits that are not valid as it does now
(`Unusable::Limits`). Nothing else of the worker's component changes:
`Loss`, `Request` and the handling of a channel `Unsent` stay.

## 4. temper (task `temper-room`)

```rust
// temper-service, lib.rs, beside WorkerLost
pub enum Note {
    // ...
    WorkerUnsent { worker: Box<[u8]>, channel: Token, unsent: jig_protocol_engine_workers::Unsent },
}
```

serve's line, as `temper/docs/design/protocol/README.md`, 15.3 has it:

```
temper serve: worker unsent worker=<worker> channel=<n>: <record> of <bytes> bytes past the room of <room> bytes
temper serve: worker unsent worker=<worker> channel=<n>: <record> of <bytes> bytes refused: <reason>
```

`<record>` is the kind in words, one exhaustive match over
`jig_protocol_worker_wire::Kind`: `assignment`, `job`, `message`,
`relayed`, `turn-kept`, `turn-busy`, `acknowledge`, `cancel`, `grant`,
`sources`, `check`, `sign-out`, `read-list`; a kind the engine never
sends (`hello`, `turn`, ...) by its own name in the same form
(`list-read`, `job-answer`, ...), which no line can show. `<reason>` as
section 2's `REASON`.

The values temper gives the fields (written by the jig tasks as their
required field's lines):

| Field | Where | Value |
|---|---|---|
| engine `Limits.room` | `temper/crates/temper-service/src/workers.rs`, `limits` | `jig.io.largest_room()`, `jig` the engine's `settings.jig.limits` |
| worker `Limits.room` | `temper/crates/temper-service/src/worker_settings.rs`, `link_limits` | `worker.limits.io.largest_room()` |
