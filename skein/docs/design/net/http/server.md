# The HTTP/1.1 server

The server's vocabulary is its own, as the client's is: the service's
protocol layer translates between it and the domain's.

```rust
pub enum Request {                         // from the side above
    Next,                                  // the next request: Call, Ended or Failed answers it
    Respond(Response),                     // the call's response head, once; Refused answers one refused
    Body(stream::Down),                    // the request body's stream, read
    Discard,                               // the rest of the request body is read and dropped, or given up
    Reply(stream::Down),                   // the response body's stream, written
    Close,                                 // Closed answers it
}

pub enum Event {                           // to the side above
    Call(Call),                            // for a Next: a request's head, whole and sound
    Ended,                                 // for a Next: the client ended the connection between requests
    Body(stream::Up),                      // Bytes, End, or Failed
    Reply(stream::Up),                     // Room, or Failed once the reply can go no further
    Refused(Refusal),                      // for a Respond: refused, writing nothing
    Done(Reuse),                           // for a Call: over, and whether the connection is kept
    Failed(Error),                         // for a Call, or for a Next no call answered
    Closed,                                // for a Close: terminal
}

pub struct Call { method: Method, target: Box<[u8]>, version: Version, headers: Box<[Header]>, body: Body }
pub struct Response { status: u16, headers: Box<[Header]>, body: Body, close: bool }
pub enum Body { None, Length(u64), Chunked }
pub enum Error { Rejected(Rejection), Truncated, Stream(Fault), ChunkSize, Chunk, Trailer, Extensions, BodyTooLong }
```

- **`Next` asks for the next request,** while no exchange is in
  progress, and exactly one event answers it: `Call`; `Ended`, the
  client's end with no line of a request; or `Failed`, a request
  rejected with the server's own answer (`Rejected`), one cut short
  (`Truncated`), or the stream's failure. The side above asks only when
  it can take a request: at its entrance it refuses by not asking, or by
  answering a busy status.
- **A call is an exchange,** and exactly one terminal event ends it:
  `Done`, once the response is all queued below and the request body was
  read to its end, discarded, withdrawn from once all read below, or
  given up; or `Failed`. After `Ended`, a
  `Failed` or `Done(Close)` the server waits for its close; a `Next` then
  is the side above's bug, asserted, as is a request of any kind with no
  call in progress.
- **`Close` ends the server in any state.** It withdraws what it
  demanded below, ends the exchange in progress without a word, and
  answers `Closed`, its one terminal event. It does not close the stream
  below, which its owner closes.

## 1. One request at a time

- **Room first.** On `Next`, before it reads anything of a request, the
  server asks below for room for the longest response head it writes,
  `Limits::response`, and holds it once granted, as io holds a grant
  across demands that ask for none (../../runtime/io/sockets.md, 3). Only then does it read.
  Whatever the request comes to, a response head or a rejection's
  answer, goes down at once within that room, so the server never parses
  a request it could not answer (../../../foundation/programming-model.md, 7).
- **The next request is read** only once the side above asks for it,
  which it does after `Done(Keep)`, once the response is queued.
  Pipelined requests wait below meanwhile, in the stream's intake, under
  its cap, and then in the client's socket.

## 2. The request head

- **Read a line at a time,** as the client reads a response head (`client.md`, 2):
  each line a scan to LF of at most what is left of `Limits::head`, its
  CR dropped. Empty lines before the request line are skipped, within the
  budget (RFC 9112, 2.2).
- **The request line** is a method, a space, a target, a space and
  `HTTP/x.y`, exactly so: one that is not is rejected rather than
  repaired (RFC 9112, 3). The method is a token, and one of `Method`'s,
  the methods an API's client sends; the target is visible ASCII, kept as
  it came, in whatever form the client wrote it. In absolute form its
  authority names the host, and `Host` is to be ignored (RFC 9112,
  3.2.2): the server still checks `Host`, and the side above, which
  reads the target, takes the host from it. A minor version past 1 is
  read as 1.1.
- **A field** is read as the client reads one (`client.md`, 2), but an obsolete
  fold, or whitespace before the first field, is rejected (RFC 9112, 2
  lets a server).
- **At the blank line:** one `Host` in HTTP/1.1, at most one in HTTP/1.0
  (RFC 9112, `client.md`, 2), and that one a host and a port, `uri-host [":"
  port]` (RFC 9110, 7.2): an IP literal in brackets, or a name or an IPv4
  address of unreserved bytes, sub-delimiters and percent-escapes, then
  digits, or empty, as a client sends it for a target with no authority;
  `a b/c@` is no host. Then the framing (RFC 9112, 6.1 and 6.3):
  `Transfer-Encoding` is read only in HTTP/1.1 and only alone, so in
  HTTP/1.0 or beside a `Content-Length` it is faulty framing, as it is
  how requests are smuggled; its codings end with `chunked`, given once,
  and hold no other, which the server does not undo. `Content-Length` is
  one length, however many times it is given. Neither: no body. Then a
  length past `Limits::body`.
- **`Call::accepts_gzip`** says whether the head's `Accept-Encoding`
  takes gzip (RFC 9110, 12.5.3), over every such field: a listed `gzip`
  or `x-gzip`, or `*` where neither is listed, and in each case a weight
  above 0, without regard to case. No field says no, since a client not
  asking gets the plain answer (`../../ui/web/bundle.md`, 4). It is a reading of
  the head and chooses nothing; a value it can't read says no.
- **Errors are decided in a fixed order:** at a line, its length, then its
  bytes, then the room for one more field; at the request line, its
  form, then its version, then its method; at the blank line, the
  `Host`, the framing, then the length.
- **Each rejection** has a small answer of a fixed length
  (../../../foundation/programming-model.md, 8): the status line, the `Date` (4),
  `Content-Length: 0` and `Connection: close`. It goes down within the
  room set aside, `Failed(Rejected(_))` answers the `Next`, and the
  connection ends.

  | Rejection | What | Answer |
  |---|---|---|
  | `RequestLine` | a request line that is not one | 400 |
  | `TargetTooLong` | a request line that does not fit in what is left of the head | 414 |
  | `Version` | another major version than HTTP/1 | 505 |
  | `Method` | a method the server does not know | 501 |
  | `Header` | a field line that is not a field, or a fold | 400 |
  | `HeadTooLong` | a head past `Limits::head` | 431 |
  | `TooManyHeaders` | more than `Limits::headers` fields | 431 |
  | `Host` | no `Host` in HTTP/1.1, more than one, or one that is not a host and a port | 400 |
  | `Framing` | framing that cannot be read | 400 |
  | `Coding` | a transfer coding other than `chunked` | 501 |
  | `BodyTooLong` | a body by length past `Limits::body` | 413 |

## 3. The request body

- **Read as the client reads a response body** (`client.md`, 3), by the same code:
  the server is the side below of the body's stream, keeps its contract,
  and holds its carry-over in an intake of `Limits::read`. A request is
  framed by length or by chunks, never to the end of the stream: one with
  neither has no body. The first demand of an empty body gets `End`; the
  exchange is done only once the side above read its `End`, discarded
  it, or withdrew its demand once the body was all read below, the
  withdrawal crossing the last bytes: a side above that responds without
  reading the body discards it, or `Done` never comes.
- **`Discard`** reads the rest and drops it, on a connection that may be
  used again, so that the next request can be read; on one that may not,
  it reads no more: the body is given up, and a read outstanding for it
  withdrawn.
- **Bad framing** (`ChunkSize`, `Chunk`, `Trailer`) fails the exchange,
  and the body's stream hears `Failed(Fault::Invalid)`: nothing is
  answered, and the connection ends.
- **A chunked body is bounded as one by length is** (RFC 9112, 7.1.1):
  its chunks' data, in all, by `Limits::body`, and its size lines'
  extensions, in all, by `Limits::head`, the budget of a head, as each
  size line and the trailer section are. Each size line takes its share
  of both as it is read, before its chunk's data, a discard's reading
  too. Past the data's, the exchange fails with `BodyTooLong`: a 413
  goes down in the room set aside if it is still held, no response given
  and no 100 (Continue) sent in it, and the connection otherwise closes
  unanswered, as for bad framing. Past the extensions', it fails with
  `Extensions`, unanswered, as bad framing. The body's stream hears
  `Failed(Fault::Invalid)` either way.
- **A 100 (Continue)** (RFC 9110, 10.1.1): for an HTTP/1.1 request with a
  body whose `Expect` lists `100-continue`, the server writes `HTTP/1.1
  100 Continue` before the body's first read below, for a demand of the
  side above's or a discard, within the room set aside; the head then
  asks room of its own. A final response given first answers instead, and
  no 100 goes. An HTTP/1.0 client's expectation, and any other, is
  ignored. As the 100 goes only at the side above's first read, a side
  above given such a call demands the body, discards it or responds
  promptly (RFC 9110, 10.1.1): its client waits for the 100, or for a
  while, before it sends the body, and the server sends nothing until the
  side above moves.

## 4. The response

- **`Respond` gives the head,** once a call. What the side above gets
  wrong is a refusal, checked in a fixed order, which writes nothing and
  leaves the exchange as it was: a status outside 200 to 599 (`Status`:
  the server writes no interim response but its own 100, and switches no
  protocols); each field, a name that is not a token (`Name`), a value
  with a control character but a tab (`Value`), a field the server
  writes itself, `Content-Length`, `Transfer-Encoding`, `Connection` or
  `Date` (`Reserved`); a body for a 204, or one in chunks for a 304,
  which may say by its length the one a 200 would have had (RFC 9110,
  8.6) but sends none (`Body`); then a head
  past `Limits::response` (`TooLong`).
- **Written sized:** `HTTP/1.1` whatever the request's version (RFC 9110,
  2.5), the status and the standard's reason phrase for it, or none; the
  `Date`, `env.wall` as the step that writes the head sees it, as an
  IMF-fixdate (RFC 9110, 5.6.7 and 6.6.1): 37 bytes, `Date: ` and 29 of
  the date, a `Wall` ending in 2554, and a line ending; the side above's
  fields in order; the framing as the response says it,
  even to `HEAD` (RFC 9110, 9.3.2), `Content-Length` for a length, `0`
  for no body but in a 204 or a 304, `Transfer-Encoding: chunked` for
  chunks; and `Connection: close` for a connection that does not persist,
  `Connection: keep-alive` for an HTTP/1.0 one that does.
- **The head goes down** once the request body is all read below, or
  given up (5), within the room set aside, or room asked for it after a
  100.
- **The body is a stream** (`Reply`), written as the client's upload is
  (`client.md`, 1): the side above demands room of at most `Limits::send`, sends
  within it, and finishes. By length, each `Send` goes down as it is, and
  `Finish` once the length is sent. In chunks, each `Send` is one chunk,
  framed into a box of its own, its size line, the bytes and a line
  ending, below room asked for the chunk; an empty `Send` is no chunk,
  and goes down empty, giving up the room granted below, which grants one
  `Send` (../../runtime/lib.md, 7); `Finish` writes the last chunk, `0` and a blank
  line, with no trailer section, within room asked for it, or within the
  room granted for a chunk if it comes in place of that grant's `Send`,
  as the contract lets it. To an HTTP/1.0 client, which knows
  no chunks, a chunked response goes to the end of the stream instead
  (RFC 9112, 6.1), on a connection that ends with it. A response to
  `HEAD`, a 204, a 304 or one without a body has no reply stream: a
  request on one is the side above's bug, asserted.
- **A withdrawal on the reply,** as a machine stacked on it sends when it
  closes, means the side above writes no more: the response can never
  end. The server withdraws what it demanded below, demands nothing more,
  drops an answer on its way, and waits for its close.

## 5. The exchange

- **Reuse** (RFC 9112, 9.3): `Done(Keep)` when the request persists
  (HTTP/1.1 without `Connection: close`; HTTP/1.0 with `Connection:
  keep-alive` and not `close`), the response did not ask to close and is
  not sent to the end of the stream, the request body was all read below
  when the head was written, and the stream did not end. Otherwise
  `Done(Close)`, and the head says so.
- **A response given first,** before the request body is all read below,
  gives up the rest of it: the body's stream, if the side above still
  reads it, hears `Failed(Fault::Other)`, the fault the stream
  vocabulary has for a side below that stops for a reason of its own; a
  read outstanding for it is withdrawn; and the head says `Connection:
  close`, as the client may stop its upload when a response comes first,
  as skein's does (`client.md`, 4), so the rest may never come. Unless the side
  above discards the body on a connection that may be kept: the head then
  waits for the body's end below, and keeps the connection.
- **The stream ending or failing:**

  | When | `End` | `Failed(fault)` |
  |---|---|---|
  | idle | the next `Next`: `Ended` | the next `Next`: `Failed(Stream(fault))` |
  | setting room aside, or before a request line | `Ended` | `Failed(Stream(fault))` |
  | in a head, after its request line | `Failed(Truncated)` | `Failed(Stream(fault))` |
  | in a body by length or chunks | `Failed(Truncated)` | `Failed(Stream(fault))` |
  | once the request is all read below | the response still goes, room may still come; `Done(Close)` | `Failed(Stream(fault))` |

  A stream of the side above's that is still open hears its end first,
  `Failed` with the stream's own fault, `Other` for an end that cut the
  request short, and `Invalid` for bad framing or a body past its
  limits.
- **What it waits for**, `waiting()`, a function of its state, so the
  connection can arm deadlines: `Next` (idle), `Room` (the client is not
  reading: the room set aside, a head after a 100, the reply's room or
  the last chunk's), `Request` (a request's head or its next line: an
  idle keep-alive, or a head that comes slowly), `Body` (the request body
  or its framing), `Above` (the side above must respond, demand, discard,
  send or finish), `Close` (the connection is not to be used again, or
  the side above withdrew the reply and closes next) and `Nothing`.

## 6. Limits and their validity

```rust
pub struct Limits {
    pub head: u32,      // a request's head: past it 431, or 414 for a request line that does not fit;
                        // also the longest chunk size line, the most chunk extensions of a body
                        // in all, and the longest trailer section
    pub headers: u32,   // fields in a head: past it 431
    pub body: u64,      // the longest body: by length, past it 413 at the head; in chunks,
                        // Failed(BodyTooLong) at the size line that passes it
    pub read: u32,      // the most the side above demands of the request body at once: the intake's cap
    pub response: u32,  // the longest response head: past it Refused(TooLong); also the room set aside
                        // before each request, at least the longest of the server's own answers
    pub send: u32,      // the most room the side above demands at once for the reply; at least 1
}
```

- **`server::valid(&limits)`** checks a usable head, read and reply
  room, enough response room for the server's own answers, and a framed
  chunk's room fitting `u32`. The request head and body, field count,
  intake and queued reply retain their caps. A sum of heap prices is
  not an admission check.
- **`largest_read`** is the larger of `head` and `read`, and at least 2;
  **`largest_room`** the larger of `response` and the room of a chunk of
  `send`: `send`, its size in hexadecimal, and four.
- **`up` emits at most** three events and two requests: the stream's
  failure told to both bodies' streams and the exchange; below, the head
  and the reply's room, or a 100 and the read it goes before.
  **`down` emits at most** two and two: the body given up and `Done`; below,
  a read withdrawn and the head.
