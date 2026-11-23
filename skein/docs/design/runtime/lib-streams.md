# lib: streams

## 1. Streams

`lib::stream` is the classic shape of a byte boundary
(../../foundation/programming-model.md, 4.3):

```rust
pub enum Read {
    Nothing,
    Fill(u32),
    Scan { until: Delimiter, max: u32 },
    Line { max: u32 },
    Arrived { max: u32 },               // what has arrived: one byte to max
}

pub enum Down {                         // to the side below
    Demand { read: Read, room: u32 },   // what this state needs, and the output room it wants
    Send(Box<[u8]>),                    // moved down, within the room granted
    Finish,                             // nothing more to send: flush, then end the stream
}

pub enum Up {                           // from the side below
    Bytes(Box<[u8]>),                   // the demand: exactly, or what has arrived of it
    Room,                               // the room asked for is free
    End,                                // the other side will send nothing more
    Failed(Fault),                      // the stream is broken: no more Bytes or Room
}
```

A `Fault` says why a stream broke as far as the side above can act on it:
the peer reset it, the side below could not make sense of the peer's data
(a TLS record that fails to decrypt), or anything else.

**The contract of a stream.** Every side below meets it, io's and every
machine's:

- **A demand is answered at most once:** by `Bytes`, what its read asks
  for (exactly, but for a read of what has arrived, below), or by `Room`,
  whichever the side below can give first;
  either answer ends it, and nothing is outstanding until the side above
  states its next. The side above states its next demand only after an
  answer, never in place of one outstanding. A state that wants nothing
  more after an answer states nothing (../../foundation/programming-model.md, 5.4).
- **`Read::Nothing` with no room withdraws** the outstanding demand, and
  only when the side above will read no more: it is closing, or its read
  crossed `End`. An answer
  already on its way may still arrive, and the side above drops it:
  dropping `Bytes` loses data, so only a reader giving up the stream may.
  No other `Bytes` or `Room` come without a demand.
- **`End` comes once nothing the side below holds can meet a demand:**
  with one outstanding, when it can never be met; with none, only when
  nothing is held. It comes once, and ends reading only. A read that
  crosses it is never met, and `End` does not end the demand: it stays
  outstanding until `Room` answers it, if it asked for room, or until the
  side above withdraws it, and the side above states no other demand
  meanwhile. Room may still be granted after `End`, as the stream can
  still send to a peer that only half-closed. A read larger than what is
  left before the end is never met; a side above that must see every byte
  reads by its framing (a scan, or fills no larger than its framing says
  remain), or reads what has arrived, which is met while a byte is left.
- **`Room` grants one `Send`** of at most the room asked for; the side
  above sends within it (an empty `Send` gives it up), or finishes, before
  it demands room again. **`Finish` comes with no read outstanding,**
  other than one that crossed `End`: a side below that reads another
  stream to meet a read, as TLS does, could not send the end behind it.
  io asserts neither; a side above keeps both, so that it works over any
  side below.
- **`Failed` may come at any time,** a demand outstanding or not, and
  nothing follows it. After `End` it says only that the stream can no
  longer send: what was read stands.
- **A scan that meets no delimiter within its maximum delivers exactly
  the maximum.** What that means is the side above's: a framing error for
  a line or a head, one piece of a longer text for a string scanned to its
  quote (../data/json.md, 3.2).
- **A read of what has arrived (`Arrived`) is met as soon as the side
  below holds a byte,** by all it holds then, up to the read's maximum: at
  least one byte, at most the maximum. It is the read of bytes nothing
  frames: a child's output, a body read to its end. It waits for no more
  than one byte, so a reader of such a stream pays one delivery for what
  arrived together, not one per byte.
  - **What a delivery holds is how the bytes arrived:** the receives,
    records or chunks they came in. A side above that reads what has
    arrived makes the same of the bytes whatever the cuts; only their
    order is the stream's. In a simulated world, how bytes arrive is the
    writers' writes and the cuts the seed draws (../testing/simulator.md, 1 and 4),
    so a world stays one function of its seed.
  - **Its maximum is at least one byte** (a read of none is `Nothing`). A
    maximum past what the side below can hold is met with what it holds:
    the side below never waits to fill it. The side above's memory counts
    a delivery of its maximum.
  - **A side below that reads another stream to meet it** reads below no
    more than it needs: a read of what has arrived, where its framing
    allows one (an HTTP body within what is left of its chunk or its
    length, http.md, 3.3), or by its framing (TLS, a record at a time,
    ../net/tls.md, 3.2).
  - **Exact reads stay where the bytes are framed,** or where the side
    above must not take more than it can act on in one step: a header, a
    length-prefixed frame, a document read up to its end and never past
    (../data/json.md, 3.2), a terminal's keys when each may be an event of its
    own.
- **A line scan (`Line`) ends at the first line end, a CR or an LF,
  whichever comes first,** for text whose lines end with LF, CRLF or CR
  alone, as an event stream's do (http.md, 4.1): a CRLF is two ends, the
  CR ending one delivery and the LF the next, and the side above pairs
  them. Otherwise it is a scan: within its maximum, or exactly the
  maximum. A `Delimiter` stays one exact sequence of bytes; "either of
  two bytes" is a read of its own.

**`Intake`** is the carry-over of a stream: the bytes the side below has
received and the side above has not yet demanded, up to a cap fixed when
it is made. It allocates as it fills, growing exactly and never past the
cap.

- It appends what arrives while it has room, and reports the room left,
  so receiving stops at the cap.
- It meets a fill, a scan or a line scan as soon as it can, each delivery
  a box of exactly the demanded length, and a read of what has arrived as
  soon as it holds a byte, with all it holds up to the read's maximum. A
  scan that reaches its maximum with no delimiter delivers exactly the
  maximum: the side above sees a scan that does not end with the
  delimiter, and decides what it means.
- A fill larger than the cap, or a scan whose maximum is shorter than the
  delimiter (one byte, for a line scan) or longer than the cap, could
  never be met: the caller's bug, asserted. So is a read of what has
  arrived with a maximum of zero; any other maximum can be met.
- A scan remembers how far it has searched, so a scan for the same
  delimiter, or a line scan after a line scan, does not search the same
  bytes again.
- It says whether what it holds ends partway through a delimiter
  (`ends_partway`): a side below that fills its own intake from another
  stream, as the HTTP client does a body's, then reads a byte at a time,
  so that it never reads past a delimiter its next bytes complete.

A **complete buffer** is its own side below. `stream::Held` owns one and
`stream::Slice` borrows one; both answer a demand at once, in the same
step, as an intake would meet it, and answer `End` once what is left can
never meet a demand. Neither asks anything of room: the data is read, not
written. A `Slice` copies only each delivery, never the whole buffer.

### 1.1 Independent output reservations

A framed connection can need to send while its next read waits for a peer
that is itself waiting for that output. Its lower face supports an explicit
independent output vocabulary alongside the classic read vocabulary:

```rust
pub enum OutputDown {
    Room { right: Token, bytes: u32 },
    Cancel { right: Token },
    Send { right: Token, bytes: Box<[u8]> },
    Release { right: Token },
}

pub enum OutputUp {
    Settled { right: Token, outcome: OutputOutcome },
}

pub enum OutputOutcome { Granted, Cancelled, Failed(Fault) }
```

One admitted Room request owns one obligation. Its token is generated by
the caller, never reused for another request on that stream, and never a
wire identity. Token exhaustion prevents a new request before effects.
Its positive byte count fits the lower output cap. The lower grants only
when both those bytes and one Send record slot are reserved, including
output held in flight or stalled. Exactly one Settled terminal consumes
the demand. Granted creates an affine reservation with the same token;
one matching Send, Release, Finish or actual failure/close consumes it.
A Send moves at most the granted bytes and spends the whole reservation,
including unused capacity. An empty Send also consumes it.

There is one independent output demand or grant at a time. It can coexist
with a classic read-only Demand, which it neither answers nor withdraws.
Bytes, End and classic Room cannot settle it. Classic room demands and
grants cannot coexist with independent output ownership, and a classic
Send cannot spend an independent grant. Classic consumers keep the joined
Demand.

Cancel names a pending output demand. Cancellation wins only before the
lower has emitted its terminal. A Granted terminal already queued above
remains the winner; the caller consumes it and releases its grant while
closing. Old Cancel, Send and Release tokens cannot consume a newer right.
They are inert, including a Send with no live matching grant. A matching
Send larger than its reservation is the caller's bug, asserted before
retaining or submitting its bytes. There is no
second terminal when an already granted reservation is retired.

Actual failure settles a pending output demand with Failed before telling
classic stream Failed. Close or Abort settles it with Cancelled before the
entity's real Closed. The caller stops admitting output when closing;
requests naming an already closing or absent entity are not admitted.
Only a positive, within-cap Room on an open stream with idle output and
no competing classic room ownership creates an obligation; the lower
retains only its active cell, without a history of the caller's tokens.
Neither a grant nor Closed proves peer receipt or durable commitment.

Finish keeps its classic restriction: withdraw a read only when closing or
after End, settle any pending output demand, and finish once. There is no
general live-read cancellation and no Finish terminal. End concerns
reading alone; independent output can progress after it. Native lower
support is required: an adapter over an arbitrary classic combined Demand
cannot manufacture independent room.
