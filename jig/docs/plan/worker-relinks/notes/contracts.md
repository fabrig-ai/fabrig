# worker-relinks: the shapes that cross tasks

Binding for every task that cites this note. Doc comments are the design's
words; a task may add derives a test needs (`Clone`, `Copy`, `Hash`), never
change a name, a variant or a field.

## 1. The engine's side: `jig-protocol-engine-workers` (`engine-link`)

In `src/boundary.rs`, exported from `lib.rs` beside `Request`:

```rust
/// Why a worker's connection ended (protocol/channel.md, 9 and 10.2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Loss {
    /// Nothing heard for the silence deadline.
    Silence,
    /// Output queued with no frame drained for the stall deadline.
    Stall,
    /// The output cap filled.
    Full,
    /// The opening's handshake or the hello did not come within its deadline.
    Deadline,
    /// The stream ended or failed.
    Stream,
    /// A frame that does not parse, or a record that does not decode or
    /// comes outside its phase.
    Framing,
    /// Refused here, with the reason sent: jig's (`jig_protocol_worker_wire::Reason`),
    /// or skein's channel's (2, limits; 3, framing).
    Refused { reason: u16 },
    /// The worker refused, with its reason.
    RefusedPeer { reason: u16 },
    /// No room for another connection: turned away at its accept.
    NoRoom,
    /// The engine closed it, stopping.
    Stopping,
}
```

`Request` gains one variant, beside `Lost`, which stays as it is:

```rust
/// Once per accepted connection, when it ends, whatever ended it: the
/// worker it authenticated as, the channel its hello opened, and why.
Ended { connection: Token, worker: Option<u32>, channel: Option<Token>, why: Loss },
```

`host::Output` gains one variant; the host fills in the configured name
of the worker (`None` before authentication):

```rust
Ended { worker: Option<Box<[u8]>>, channel: Option<Token>, why: Loss },
```

The host also gives `Output::Ended { worker: None, channel: None, why: Loss::NoRoom }`
for a socket it rejects itself because its socket table is full.

The first cause decides `why`: a refusal sent is `Refused` even if the
stream then fails; a loss by a deadline is `Silence`, `Stall` or `Deadline`
even if the channel then ends as `Closed::Owner`. A connection that ends with
no cause recorded (io's `Closed` reaching `Event::Gone` first) is `Stream`.

## 2. The worker's side: `jig-protocol-worker-link` (`worker-link`)

In `src/boundary.rs`, exported from `lib.rs`:

```rust
/// Why a channel, or a dial, ended (protocol/channel.md, 9 and 10.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Loss {
    /// As skein's channel ended it: the stream, a framing error, a refusal
    /// sent or received, or the owner.
    Closed(skein_channel::Closed),
    /// Nothing heard for the silence deadline.
    Silence,
    /// Output queued with no frame drained for the stall deadline.
    Stall,
    /// The opening did not finish within the handshake's deadline.
    Handshake,
    /// The output cap filled: the channel refused a record as full.
    Full,
    /// The engine's busy answers outran the table that keeps them.
    Busy,
    /// The root faulted the link (`Event::SourceFault`).
    Fault,
    /// The dial failed (`Event::Unreachable`).
    Unreachable,
}
```

`Request::Fact { why: skein_channel::Closed }` becomes `Request::Fact { why: Loss }`;
`Request::Lost` stays a unit variant. Each loss site gives its own cause:

| Site in `component.rs` today | `why` today | `why` from now |
|---|---|---|
| `fire`, `Phase::Open`, silence passed | `Closed::Stream` | `Loss::Silence` |
| `fire`, `Phase::Open`, stall passed | `Closed::Stream` | `Loss::Stall` |
| `fire`, `Phase::Opening`, past `until` | `Closed::Owner` | `Loss::Handshake` |
| `received`, `Event::Unsent { why: Unsent::Full, .. }` | `Closed::Owner` | `Loss::Full` |
| `received`, any other `Unsent` | `Closed::Owner` | `Loss::Closed(Closed::Owner)` |
| `body`, a `TurnBusy` the busy table refuses | `Closed::Owner` | `Loss::Busy` |
| `down`, `Event::SourceFault` | `Closed::Owner` | `Loss::Fault` |
| `down`, `Event::Unreachable` | `Closed::Stream` | `Loss::Unreachable` |
| every other site (`Closed { why }`, `Refused`, `Ended`, `OutputFailed`, `Opening`, `gone`, the refusals sent) | its `Closed` | `Loss::Closed(` the same `)` |

## 3. Who reads them

- `worker-reads` (temper) turns `Output::Ended` and a linked link's `Fact`
  into its log lines (`temper/docs/design/protocol/README.md`, 15.3).
- Until then, temper's readers take the new shapes with compiler-forced
  edits only: an arm `kit::host::Output::Ended { .. } => {}` in
  `temper-service/src/workers.rs`, and `Loss::Closed(..)` around the
  `Closed` patterns and values of `Fact` in `temper-service/src/worker.rs`
  and its tests. These cross from jig into temper as a required field's
  lines do: each edited only in its type path, its value or one arm.
