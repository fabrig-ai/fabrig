# HTTP

Provisional, 2026-10-04; revised 2026-10-20. The design of
`skein-http`: the protocol machines for HTTP/1.1, client and server, and
for server-sent events, reader and writer. They are step machines of a
connection's stack (../../../foundation/programming-model.md, 4): each depends on lib only,
and meets the stream below it and the user above it through entry points
of the model's shape.

## 1. In one page

- **HTTP/1.1 only.** Heads are read a line at a time under a maximum
  size; bodies are framed by length, by chunks, or by the end of the
  stream; the body goes up as a stream that the side above demands.
- **One exchange at a time.** A client connection carries one exchange
  at a time and is used again when both sides allow it; a server
  connection parses the next request only once the response is queued
  and there is room for the one after it: it sets that room aside before
  it reads anything of a request, so whatever the request comes to, its
  answer goes down at once.
- **Two streams per exchange.** The side above writes the request body
  as a stream and reads the response body as one, each a `lib::stream`
  face (../../runtime/lib.md, 7) of which the client is the side below; the server's
  side above reads the request body and writes the response body the
  same way. A machine stacked on a body (server-sent events, JSON)
  cannot tell it from a socket.
- **Refuse at the entrance.** What is wrong with a request's head the
  server answers itself, small and fixed: 400, 413, 414, 431, 501 or 505,
  and the connection ends. Bad framing in a body closes, unanswered; a
  chunked body past the limit is a 413 if the room set aside is still
  held.
- **Server-sent events** are a machine over a body stream each way. The
  reader reads lines, fields, and an event at each blank line, each
  under a maximum; an event's data goes up as a stream while its lines
  are read, so a JSON tokenizer reads it as it arrives, and the reader
  holds a piece of a line, never an event. The writer frames each event
  sized and sends it within the room it is granted.
- **Both sides of each machine** are skein's (client and server, reader
  and writer), and each is tested against transcripts of real peers'
  formats, generated messages, and the other side: a protocol world
  stacks both ends of an LLM's stream (`testing.md`).

## 2. In skein

`skein-http` depends on lib only. A service's protocol layer stacks it:
over TLS's plaintext, a socket or a pipe, whichever the connection has,
and under the service's own decoders. An LLM client's stack, from the
socket up:

```
domain
  ▲  typed calls
skein-llm's provider decoder (or the service's own decoder)
  ▲  tokens
json                  the selective collector on each event's data stream
  ▲  events
sse                   skein_http::sse
  ▲  body stream
http client           skein_http::client
  ▲  stream: plaintext
tls
  ▲  stream: ciphertext
io socket
```

A fake LLM provider's, the other end:

```
domain
  ▲  typed calls              ▼  what it answers
the service's protocol layer
  ▲  tokens                   ▼  events, their data written by the JSON writer
json (the request body)      sse writer     skein_http::sse::writer
  ▲  request body             ▼  response body
http server                                 skein_http::server
  ▲  stream: plaintext        ▼
tls, or an io socket
```

Each machine has its own `Limits` and edge checks, emits into uncapped
queues, and says what it waits for:

```rust
// skein_http::client, the machine
pub fn up(client: &mut Client, env: &Env<Limits>, ev: stream::Up,
          above: &mut Queue<Event>, below: &mut Queue<stream::Down>)
pub fn down(client: &mut Client, env: &Env<Limits>, rq: Request,
            above: &mut Queue<Event>, below: &mut Queue<stream::Down>)

// skein_http::sse, the machine: the same shape, over a body stream
pub fn up(reader: &mut Reader, env: &Env<Limits>, ev: stream::Up,
          above: &mut Queue<Event>, below: &mut Queue<stream::Down>)
pub fn down(reader: &mut Reader, env: &Env<Limits>, rq: Request,
            above: &mut Queue<Event>, below: &mut Queue<stream::Down>)

// skein_http::server and skein_http::sse::writer: the same shape
pub fn up(server: &mut Server, env: &Env<Limits>, ev: stream::Up,
          above: &mut Queue<Event>, below: &mut Queue<stream::Down>)
pub fn down(server: &mut Server, env: &Env<Limits>, rq: Request,
            above: &mut Queue<Event>, below: &mut Queue<stream::Down>)
```

Whoever stacks a machine checks at startup that its largest demand fits
the side below: `client::largest_read` and `client::largest_room`, or
`server::largest_read` and `server::largest_room`, against the stream's
intake and output caps; `sse::largest_demand` against the client's
`Limits::read`, and the JSON tokenizer's against the reader's
`Limits::chunk`; the tokenizer's against the server's `Limits::read`,
and `writer::largest_room` against its `Limits::send`. The method and the version a start line names
(`Method`, `Version`) and a field (`Header`) are the crate's, both sides
the same. So is the reading of a head: `skein_http::header` parses a field
line and reads `Content-Length`, `Transfer-Encoding` and persistence for
both machines, each of which keeps its own policy for what it reads
(`client.md`, 2, `server.md`, 2). A side that writes fields of its own checks and measures them
with the module's public predicates (`is_tchar`, `is_token`, `is_field_byte`,
`trim`, `field_len`) and `client::request_head_len`.

A head prints its lengths (../../runtime/lib.md, 6). A `Header`'s `Debug` shows its name
and its value's length. A server's `Call`, and the request line a server is
still reading, show the target's length, and a `Call` its fields so; a
client's written head, held until room comes to send it, shows its own
length. A client's `Call` is the side above's own, and prints its target.

The crate builds for `wasm32-unknown-unknown` too (../../runtime/lib.md, section 2), so
a client compiled to WebAssembly stacks the same machines: the
event-stream reader over the body its platform's fetch streams, and the
client wherever its shell gives it a byte stream.

## 3. Decisions

- **Two stream faces per exchange,** the upload and the body, each a
  `lib::stream` of which the client is the side below, and the exchange's
  events beside them. A machine stacked on the body sees a stream like a
  socket's; one face for both directions would make a reader stacked on
  the body share its demands with the upload's writer, and a response
  that came mid-upload could leave the writer's demand for room never
  answered.
- **The client holds the body's carry-over,** in an intake of
  `Limits::read`, as the side below of a stream does (../../../foundation/programming-model.md,
  4.3), and reads below in the demand's own shape. Reading ahead, with
  fills of what the intake has room for, would hold a slow event stream
  until a fill's worth arrived; passing demands through without an intake
  could not meet a fill or a scan across two chunks.
- **The request body goes by length only.** Every body temper sends is
  measured first (its ../../llm/README.md, 3); chunked uploads wait for a user.
- **A response that comes first ends the exchange** with the connection,
  and the upload's stream hears `Failed(Fault::Other)`, the fault the
  stream vocabulary has for a side below that stops for a reason of its
  own.
- **A withdrawal on the body means the side above reads no more,** as
  the stream contract says; the side above follows it with `Discard` or
  a close. `Discard` exists because a withdrawal alone cannot say it:
  a machine stacked on the body withdraws only what it has outstanding.
- **Framing headers that conflict are refused,** not resolved: a
  `Transfer-Encoding` beside a `Content-Length` is how requests are
  smuggled and responses split, and the client has nothing to gain by
  guessing.
- **The head's budget covers interim heads,** rather than a separate
  limit on how many there may be.
- **An event's data goes up as a stream.** Held whole, an event cost
  every connection its provider's largest event, and an event that
  echoes its request grows with the conversation. Streamed, the reader
  holds a piece of a line, and a decoder that reads selectively holds only
  what it keeps (../../data/json.md, 5.1). The standard's reasons for holding an
  event are kept another way: its type, which may follow its data, goes
  up at dispatch, and an event cut short fails its data stream, so the
  side above drops what it read, as the standard drops the event. A
  decoder that must tell JSON from a sentinel such as `[DONE]` reads the
  data's first byte before it stacks a tokenizer.
- **lib's line scan, for the reader's lines.** A line ends at LF, CRLF
  or CR alone, and a scan has one delimiter. Scanning to LF always would
  read a stream of CR alone only when a scan filled, and lose its last
  events at the end. Following the last line's ending, as the reader
  first did, cost a stream that mixed endings a scan's delay at each
  change, a rest of a delivery held, and at the end what followed a
  change. A read that stops at either byte is lib's own (`Read::Line`),
  met by the intake's search as any scan is: every line is read as its
  end arrives, a line a demand rather than a byte, at the price of a
  one-byte delivery for a CRLF's LF; and nothing is held.
- **Every byte read counts against the event,** comments included, so an
  event that never ends fails whatever the stream sends it as.
- **The reader does not decode UTF-8,** and so does not replace what is
  not UTF-8 with U+FFFD as a browser would: its data goes to a decoder
  that refuses it.
- **A `retry` past a `u64` is ignored,** as a value of other bytes is.
- **One body reader for both sides.** The client reads a response's body
  and the server a request's by the same code (`body.rs`), which returns
  what goes up for each machine to emit as its own event: the demands'
  shapes and a delimiter split between chunks are subtle enough to have
  once.
- **The server sets room aside before it reads a request,** room for the
  longest response head, held while it reads, as io keeps a grant across
  demands for none. The answer to whatever the request comes to then
  goes down at once: a rejection never waits on a client that does not
  read, and the server never parses a request it could not answer
  (../../../foundation/programming-model.md, 7). It is the side above that asks for each
  request (`Next`), so a service's entrance is where it chooses to ask.
- **Rejections are the server's own,** small and fixed, and end the
  connection; bad framing in a body closes unanswered, as
  ../../../foundation/programming-model.md, 8 has bad lengths do. The request line that does
  not fit in the head is a 414, as RFC 9112, 3 requires, rather than a 431, which
  is for fields; a fold is rejected, not joined, as RFC 9112, `server.md`, 2 lets a
  server; an unknown method is a 501 (RFC 9110, 9.1) and a coding not
  undone too (RFC 9112, 6.1), while codings that do not end with
  `chunked`, or give it twice, are faulty framing, a 400 (RFC 9112,
  6.3).
- **A body by length past `Limits::body` is refused at the entrance**
  (413), and a chunked body at the size line that takes it past, before
  that chunk's data is read: the server refuses whatever the service
  limits, so a side above that discards, or reads slowly, is not made
  to read an upload without end. Its answer is the same fixed 413 while
  the room set aside is still held, which it is unless a response was
  given or a 100 (Continue) took it; then the connection closes
  unanswered, as for bad framing, since a response given waits for the
  body's end and cannot go. The extensions of a body's size lines are
  bounded in all by the head's limit, the budget the server already
  gives each line of framing, rather than by a limit of their own: they
  are dropped unread, and a client that sends more than a head's worth
  of them is not one an API serves.
- **A response given before the body is all read gives up the rest,** on
  a connection not used again, rather than read the rest to keep it: a
  client may stop its upload when a response comes first, as skein's
  does, and the rest would never come. A side above that wants the
  connection kept discards the body: the head then waits for the body's
  end. This rules out answering while reading the same request's body (a
  streaming echo), which nothing pulls.
- **The server reads the body only before the head goes,** and writes the
  reply only after it, so a demand below is a read or room, never both,
  and the server never holds a read that a client which stopped its
  upload would leave unanswered while the reply waits for room.
- **A 100 (Continue) goes before the body's first read,** the moment the
  server means to read it, which a discard counts as; a final response
  given first answers instead. The client still does not wait for one
  (section 4).
- **A withdrawal on the reply is taken,** as the event writer stacked on
  it sends one when it closes, unlike the client's upload, on which
  nothing is stacked. It means the side above writes no more: the server
  withdraws what it demanded below and waits for its close.
- **A chunked response to HTTP/1.0 goes to the end of the stream,** as
  RFC 9112, 6.1 forbids `Transfer-Encoding` to it, rather than being
  refused: the side above writes the same reply to either.
- **The server writes the `Date`,** from `env.wall`, on every head it
  writes, its own answers included, as RFC 9110, 6.6.1 asks of a server
  with a clock, and refuses one from the side above as a field it writes
  itself: no service forgets it, and none writes it twice. Its length is
  fixed, so the answers stay of a fixed length and the longest of them,
  and the least `Limits::response`, grows by its 37 bytes. The date is
  lib's `Wall::civil` (../../runtime/lib.md, 8). A 100 (Continue) carries none, as 6.6.1 allows of a 1xx.
  Reason phrases are the standard's, or none.
- **Each event or comment the writer writes is a block of its own,**
  ended by a blank line, so a reader's count starts over at each, and
  every event written is one a reader dispatches: a block that sets only
  an id or a reconnection time is not written. Data is split at every
  line ending, so a reader reads its CRs and CRLFs back as LFs.

## 4. Open questions

- **Connection reuse in the client.** Sequential reuse of one connection
  is in. Still open: a pool of connections per peer, who sets its size,
  and how a pool learns that an idle connection ended (the client says
  so in `waiting()` only).
- **`Expect: 100-continue` in the client.** A caller may send the field,
  but the client does not wait for the 100 before it takes the body.
  Waiting would need an interim event plus a read while the upload is
  idle, which `client.md`, section 1 rules out. skein's server sends a 100 when it
  reads, and a client that sends at once is read all the same; whether
  the client should wait waits for a server that needs it.
- **A server's deadlines.** `waiting()` says `Request` both for an idle
  keep-alive and for a head that comes slowly; a connection that wants a
  shorter deadline for the second cannot tell them apart. Whether the
  server says so waits for the engine's webhooks.

## 5. Not built yet

What of this document is not built is in [`http.status.md`](http.status.md).
