# The HTTP/1.1 client

The client's vocabulary is its own; the service's protocol layer
translates between it and the domain's.

```rust
pub enum Request {                         // from the side above
    Call(Call),                            // start an exchange: Done or Failed answers it
    Upload(stream::Down),                  // the request body's stream, written
    Body(stream::Down),                    // the response body's stream, read
    Discard,                               // the rest of the body is read and dropped
    Close,                                 // Closed answers it
}

pub enum Event {                           // to the side above
    Response(Response),                    // the final response's head
    Upload(stream::Up),                    // Room, or Failed once the upload can go no further
    Body(stream::Up),                      // Bytes, End, or Failed
    Done(Reuse),                           // for a Call: over, and whether the connection is kept
    Failed(Error),                         // for a Call: failed, or never began
    Closed,                                // for a Close: terminal
}

pub struct Call { method: Method, target: Box<[u8]>, headers: Box<[Header]>, body: Body, close: bool }
pub enum Body { None, Length(u64) }
pub struct Response { version: Version, status: u16, headers: Box<[Header]>, framing: Framing }
pub enum Framing { Empty, Length(u64), Chunked, UntilEnd }
pub struct Header { name: Box<[u8]>, value: Box<[u8]> }   // names compared without regard to case
```

- **A call starts an exchange,** while none is in progress, and gets
  exactly one terminal event: `Done` once the response body has been read
  to its end or discarded, or `Failed`. A call refused writes nothing and
  leaves the connection as it was; any other failure leaves it for the
  close. A call during an exchange, or after the connection was told it
  is not to be used again, is the side above's bug, asserted.
- **`Response`** goes up once the final head is whole. Its body follows
  on the body stream, even when it is empty: the first demand of an empty
  body gets `End`.
- **`Close` ends the client in any state.** It withdraws what the client
  demanded below, ends the exchange in progress without a word, and
  answers `Closed`, its one terminal event. It does not close the stream
  below, which its owner closes.

## 1. The request

- **The head is written sized** (../../../foundation/programming-model.md, 8): the request
  line, the caller's fields in order, then the client's own:
  `Content-Length` for a body (and `Content-Length: 0` for a POST, PUT or
  PATCH without one, RFC 9110, 8.6), and `Connection: close` for a call
  that closes. It is measured, refused past `Limits::request`, written
  into a box of exactly its length, and sent whole within room it asks
  for, before anything is read.
- **What the caller gets wrong is a refusal,** checked in a fixed order
  before anything is written: a target that is empty or not visible ASCII
  (`Target`), a field name that is not a token (`Name`), a value with a
  control character other than a tab, CR, LF and NUL among them
  (`Value`), a field the client writes itself, `Content-Length`,
  `Transfer-Encoding` or `Connection` (`Reserved`), each field in turn,
  its name before its value; then no `Host`, or more than one (`Host`,
  RFC 9112, 2); and the length (`TooLong`). No header can be injected
  through a call.
- **The methods** are those an API's client sends: GET, HEAD, POST, PUT,
  PATCH, DELETE and OPTIONS. `Host` is a field the caller gives, exactly
  once, anywhere among its fields.
- **The request body is a stream** (`Upload`), written, by length: the
  side above demands room only, of at most `Limits::send`, sends within
  the room granted, and finishes once the call's length is sent. Each
  demand goes below as room, with the next line of a response that may
  come first; the client demands nothing below while it waits for the
  side above's next demand, as a read stated alone would hold a demand
  that the upload's room could not join. A withdrawal of the upload's
  demand is not supported: the side above closes the client instead.

- **A gzip encoder** (added 2026-10-20) is beside the client, for a
  caller whose server takes a compressed request body (RFC 9110, 8.4.1;
  ../../llm/README.md, section 4.8): a pure function of the whole body to its gzip
  member (RFC 1952), DEFLATE inside (RFC 1951), and a worst case from
  the body's length that its output never passes, so the caller sets
  the room aside before it compresses. It is deterministic: one body,
  one output. It keeps a bounded window and match search, chosen for
  JSON that repeats itself, and it trades ratio for speed: compressing
  a body must cost far less than sending the bytes it saves. The client
  machine itself never compresses: the caller hands it the compressed
  body, and puts `Content-Encoding: gzip` among its fields. Nothing in
  skein's production code decodes gzip: the tests decode with a small
  inflater of their own, and a golden output is checked once against
  the system's `gzip`.

## 2. The response head

- **Read a line at a time:** each a scan to LF of at most what is left of
  `Limits::head`, the line's CR before its LF dropped, so a head of LF
  alone is read (RFC 9112, 2.2). Each line is parsed as it comes into a
  bounded head: the status line, then fields into a list of at most
  `Limits::headers`. The budget covers every head of the exchange,
  interim ones included, so no server keeps a client reading interim
  heads for ever.
- **The status line** is `HTTP/x.y`, a space, three digits from 100 to
  599, then nothing or a space and a reason, which is checked and not
  kept (RFC 9110, 15). Another major version is `Version`; a minor past 1
  is read as 1.1 (RFC 9112, 2.3).
- **A field** is a token, a colon with no whitespace before it, and a
  value, trimmed, with no control character but a tab. A line that
  begins with whitespace is an obsolete fold, joined to the last value
  with one space (RFC 9112, `server.md`, 2).
- **Errors at one line are decided in a fixed order:** its length (a line
  that met no LF filled what was left of the head), then its bytes, then
  the room for one more field.
- **Interim responses** (1xx) are read and skipped, a 100 mid-upload
  among them; a 101 is refused (`Upgrade`): the client never asks to
  switch protocols.
- **The framing** (RFC 9112, 6.3): no body for a response to HEAD, a 204
  or a 304, whatever the head says; chunked for `Transfer-Encoding:
  chunked`, alone, in HTTP/1.1; by length for `Content-Length`, however
  many times it is given, as long as it is one length; otherwise, to the
  end of the stream. `Transfer-Encoding` beside `Content-Length` is
  refused rather than read by the one that wins (RFC 9112, 6.3, 3: "ought
  to be handled as an error"), and so is any coding but `chunked`, which
  the client does not undo, and chunked in HTTP/1.0 (`Framing`).

## 3. The response body

The client is the side below of the body's stream, so it keeps the
stream's contract (../../runtime/lib.md, 7) and holds its carry-over, as io does for a
socket (../../../foundation/programming-model.md, 4.3): the bytes read below that no demand
above has taken yet, in an intake of `Limits::read`, allocated with the
client.

- **It reads below only what a demand above still needs,** in the
  demand's own shape, a fill, a scan to the same delimiter, or a read of
  what has arrived no larger than what is left of the chunk or of the
  body's length, and never past the body's framing. A scan for a line
  passes through as a scan, so a slow event stream is read as each event
  comes; nothing is read ahead of a demand. A delivery that meets the
  demand whole, with nothing held, goes up as it came; others are met
  from the intake, across the chunks of a chunked body too. A delimiter
  split between two chunks is completed a byte at a time, while what is
  held ends partway through it, so that nothing past it is read either.
- **The framing is read only when a demand needs what lies past it:** a
  chunk's size line (a scan to LF of at most `Limits::head`, then
  hexadecimal digits, then nothing or an extension, which is ignored),
  the line ending after a chunk's data (a scan to LF of at most two), and
  the trailer section (lines within `Limits::head` in all, ended by a
  blank line, not kept).
- **The end:** `End` answers a demand that what the body holds can no
  longer meet, once the framing says the body is over, and `Done`
  follows it in the same step. A read larger than what is left is never
  met (../../runtime/lib.md, 7): a side above that must see every byte reads by its
  framing, as JSON and the reader do.
- **A withdrawal** of a body demand means the side above reads no more
  (../../runtime/lib.md, 7), as when a machine stacked on the body closes: what was
  read for it is dropped, a demand after it is asserted, and the side
  above discards the rest or closes the client. A withdrawal may cross
  its demand's answer: one that comes after the answer withdraws all the
  same, and one that comes after the body's end and `Done` is dropped, as
  is a `Discard` that crosses them.
- **`Discard`** gives up the rest of the body, a demand outstanding or
  withdrawn among it: the client reads and drops it, by fills of at most
  `Limits::read` within the framing, so that the connection can be used
  again, then says `Done`. A body that runs to the end of the stream has
  no end to discard to: the exchange is done at once, on a connection not
  used again, and the read below for it is withdrawn.

## 4. The exchange

- **A response that comes first:** a final response read before the
  request body is all sent stops the upload, whose stream is told
  `Failed(Fault::Other)`, and the exchange ends with that response, on a
  connection not used again: the server's view of the request's framing
  is not known.
- **Reuse** (RFC 9112, 9.3): the connection carries another exchange
  when the response persists (HTTP/1.1 without `Connection: close`;
  HTTP/1.0 with `Connection: keep-alive` and not `close`), its body was
  framed by length or by chunks and read to its end or discarded, the
  call did not ask to close, and the upload was not stopped. `Done(Keep)`
  then leaves the client waiting for the next call; `Done(Close)`, for
  its close.
- **The stream ending or failing:**

  | When | `End` | `Failed(fault)` |
  |---|---|---|
  | idle | the next call fails at once, `Closed(None)` | the next call fails at once, `Closed(Some(fault))` |
  | before the request head went down | `Closed(None)` | `Closed(Some(fault))` |
  | after it, before any line of a response | `Truncated { answered: false }` | `Stream(fault)` |
  | in the response's heads | `Truncated { answered: true }` | `Stream(fault)` |
  | in a body by length or chunks | `Truncated { answered: true }` | `Stream(fault)` |
  | in a body to the end of the stream | the body's end | `Stream(fault)` |
  | once the body is all read below | the body stands, not reused | the body stands, not reused |

  `Closed` says the server never saw the request, so a pool can send it
  on another connection whatever its method. `Truncated` with no line of
  a response is the race of RFC 9112, 9.3.1, a connection kept idle that
  the server closed as the request went out: an idempotent request may
  be sent again. A stream of the side above's that is still open, and not
  withdrawn or discarded, hears its end first, `Failed` with: the stream's
  own fault, for a failure; `Other`, for an end that cut the exchange
  short (`Closed(None)`, `Truncated`), which is no error of the peer's
  data; and `Invalid`, for an error of the peer's data. A demand that
  still asks for room, which may come after the end, is withdrawn.
- **What it waits for**, `waiting()`, a function of its state, so the
  connection can arm deadlines (../../../foundation/programming-model.md, 4): `Call` (idle),
  `Room` (the peer is not reading the request), `Response` (the peer has
  the request, or as much as the side above wrote, and has not
  answered), `Body` (the body or its framing from the peer), `Above` (the
  side above must demand, send or finish), `Close` and `Nothing`.
  Progress below is a line of a head or of the framing, a piece of the
  body, or room granted, each a demand met.

## 5. Limits and validity

```rust
pub struct Limits {
    pub request: u32,   // the longest request head written: past it, Refused(TooLong)
    pub head: u32,      // the bytes of a response's heads, interim ones included: past it, HeadTooLong;
                        // also the longest chunk size line, and the longest trailer section
    pub headers: u32,   // fields in a head: past it, TooManyHeaders
    pub read: u32,      // the most the side above demands of the body at once: the intake's cap
    pub send: u32,      // the most room the side above demands at once for the request body; at least 1
}
```

- **Edges bound retained data:** `request` bounds the held request head,
  `headers` the response's field list, `head` the head and parsing lines,
  and `read` the body intake and current delivery. Calls and request-body
  pieces belong to the side above; emitted values pass to their owner
  (../../testing/README.md, 5). `valid(&limits)` requires a head of at least two bytes,
  a positive read cap and positive request-body room (`send`). There is
  no client heap sum; worlds check measured peaks under scenario ceilings.
- **`largest_read`** is the larger of `head` and `read`, and at least 2;
  **`largest_room`** the larger of `request` and `send`.
- **`up` emits at most** two events and two requests: a response and the
  upload stopped, a body's end and `Done`, or a stream told it failed
  and `Failed`; below, the request head sent and the next demand.
  **`down` emits at most** two events and one request: a body's end and
  `Done`, or `Closed`; below, a demand, a send or a withdrawal.
