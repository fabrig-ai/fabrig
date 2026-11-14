# Testing HTTP

- **Step tests** (`crates/skein-http/src/tests/`): the request head
  written and every refusal in order; the status line, fields, folds and
  every head limit at and past its edge; interim responses; the framing
  each head decides and every conflict refused; reuse by each rule; each
  body framing under fills and scans of every shape, a delimiter split
  across chunks, the reads below shaped by the demand above, every
  framing error, withdrawal, discard, and what an exchange leaves unread;
  the upload, a response that comes first, the stream ending and failing
  in each state, a close in each state, `waiting()`, and the limits'
  `valid`, true at each edge and false past it. A bug of the side above
  is asserted, and no test triggers it
  (`skein/docs/foundation/programming-model.md`, 10.4): the tests reach
  what the assertion guards through the outcomes around it. The reader: every field, every line ending and
  the line scans that read them, a CR's LF skipped, streams of LF, CR,
  CRLF and mixed endings shorter than one scan and the deliveries they
  take, the end and a failure while idle and while reading, every limit
  at its edge, the byte order mark, every cut of a stream reading the
  same, and a close in each state; and `sse::Data`. The server: the room
  set aside before anything is read; the request line, fields, and every
  head limit at and past its edge; every rejection and its answer; the
  framing each head decides; the body under demands of every shape, its
  empty end, every framing error, a withdrawal and a discard on a
  connection kept and on one not; a 100 (Continue), and where none goes;
  the response head written, its reasons and framing, and every refusal
  in order; each reply's framing, the room of a chunk, HTTP/1.0's
  chunked reply to the end of the stream, and a reply withdrawn; reuse
  by each rule, a response given before the body is read and one given
  while it is discarded; the stream ending and failing in each state, a
  close in each state, `waiting()`, and the limits' `valid`, the side
  above's bugs untested as the client's are. The writer: each field and data split at every line ending,
  comments, every refusal, an event in pieces within the room granted,
  the end and a failure, a close in each state, and events read back by
  the reader as they were written.
- **Machine worlds** (../../../foundation/testing-strategy.md, 2.4) in `tests/http`
  (`skein-http-world`), one machine from a seed in one loop with both its
  neighbours:
  - **The client's** runs one connection for one exchange after another.
    Below, the server's stream: its bytes arrive in pieces cut at random,
    late, into an intake under its cap, and meet each read as lib's
    intake does (../../runtime/lib.md, 7); room
    is granted late, one `Send` a grant, while a response may wait; now
    and then the server is patient, and answers each exchange only once
    it has the whole request, so that a client that waited for the
    response before it took the body would wait for ever; the stream ends
    when the bytes run out, early at a cut, idle or with a read on its
    way, and fails, before its end or after it; after a close it may
    still answer what was on its way. Above, a user makes the calls,
    uploads each body in pieces within the room granted, reads with
    fills, line scans, scans to LF, CRLF and a quote, and reads of what
    has arrived, of every size,
    slowly, withdraws a demand and discards, discards the rest now and
    then, sends a withdrawal or a discard that crosses its answer, the body's
    end among them, stops for a while, and closes after the last exchange
    or at any moment. The world checks the stream contract; below, one
    demand at a time, none past the caps or once the stream ended or
    failed, a withdrawal only as the client stops reading, each `Send`
    within the room granted; above, each answer for a demand and exactly
    what it reads, `End` and `Failed` once and nothing after, every stream
    still open told it failed before the exchange's `Failed`, one response
    and one terminal event per call, nothing sent for a call that failed
    `Closed`, `Closed` once and last; and `waiting()` against what the
    neighbours see. Each exchange is held to a reference
    reader: the call refused, or not, as a check of the test's own
    refuses it, one call in fifty getting one thing wrong; the request
    written, against a writer of the test's own; the head; the body, a
    prefix of the reference's, and when `End` came, nothing left that
    meets the demand it answered; and the outcome,
    unless the stream failed or the side above closed first, with a
    connection whose upload stopped, or whose stream ended or failed
    during the exchange, not used again.
  - **The reader's** does the same for an event stream, each demand a
    line scan of the chunk, against a reference that reads by the
    standard's parser a byte at a time, with the reader's limits, and
    knows nothing of scans. The events, the outcome, the reconnection
    time and the last event ID are the reference's, unless the stream
    failed or the side above closed first; the one exception is the
    incomplete line a stream may end with (`sse.md`, 1), of which only whole
    chunks are read, so a failure the reference finds past them reads
    as the end.
  - **The server's** runs one connection for one request after another.
    Below, the client's stream: its bytes arrive in pieces cut at random,
    late, and meet each read as lib's intake does; room is granted late
    and each `Send` held to it as io holds it, one a grant, made before room is
    demanded again, and a read past the end outstanding until withdrawn,
    as TLS holds it; the client pipelines
    its requests, or is patient and sends each once the last exchange is
    over; one that asks for a 100 (Continue) holds its body back until
    the 100 comes, sends none once a final response comes first, and now
    and then tires of waiting; the stream ends when the bytes run out,
    early at a cut, idle or with a read on its way, after a request or
    mid-way, and fails, before its end or after it, or the first time the
    server waits for a thing drawn; an answer to a demand the server
    withdrew may still come, before its close or after.
    Above, a service asks for each request when it feels like it, reads
    the body with demands of every shape, slowly, withdraws a demand and
    discards, now and then a withdrawal that crosses the piece it would
    have read, responds at the moment its plan draws (at once, while a
    demand of its on the body is outstanding, partway through the body,
    once it read it, once it discarded it, or at once without touching
    the body, which it discards a while later) with a response now and
    then one the server must refuse, writes the reply in pieces within the
    room granted, finishes in place of a grant's `Send` now and then,
    withdraws the reply's demand now and then, as a machine stacked on it
    does when it closes, and closes then or a while after, stops for a
    while, and closes after the last request or at any moment. The wall
    clock the server dates its heads by is drawn from each seed. The
    world checks both sides' contracts as the client's does, a 100 before
    any read of a body its client holds back among them, and
    `waiting()` against what the neighbours see. Each call is held to a
    reference reader of requests, which shares nothing with the server:
    the call, or the rejection; the body; and the outcome, the reuse as
    the world reckons it from the request, the response and when the body
    was read. What the server wrote is held, byte for byte, to a writer of
    the test's own: the 100 if it went, the head, and the reply framed as
    the side above sent it; and a rejection's answer.
  - **The writer's** runs one writer for a stream of events and comments
    of every shape, one in fifty flawed. Below, room is granted late, the
    stream says it ended now and then, and fails; above, a user writes
    one at a time when it feels like it, stops for a while, finishes, or
    closes at any moment. The world checks room only, one
    demand at a time, one `Send` a grant within it, one answer each, and
    `waiting()`. Each item is refused exactly when a check of the test's
    own refuses it; what was written is the frames of what was sent, byte
    for byte against a writer of the test's own, and a prefix of the one
    being written; and it reads back, by the reference reader and by the
    reader's own world, as each event was written.
  - A seed replays to the same run.
- **Transcripts** in `tests/http/transcripts/`, each `<name>.http` beside
  `<name>.expect`, what it must decode to: the head, the body or the
  events its body holds, and the outcome. Each is written by hand after
  the public format of its peer, and says so, but one captured from the
  user's own forge with curl, unauthenticated; its expectation was
  drafted by the reference readers and checked by hand. Forty-seven:
  - an LLM provider's streams: Anthropic's Messages, a text answer
    chunked, a tool call by length (a synthetic variation: Anthropic
    sends its streams chunked), an overloaded error as a 529, and one as
    an `error` event mid-stream; OpenAI's Chat Completions, a text answer
    chunked and tool calls by length, each ending with `[DONE]`;
  - a forge's API: a Forgejo pull request by length, a page of issues
    chunked with its `Link` and `X-Total-Count`, a 404, and a repository
    as a real Forgejo behind nginx sent it, chunked;
  - responses curl accepts: a head of LF alone, a folded field, HTTP/1.0
    kept alive and to the end of the stream, a 100 and a 103 before the
    response, a 204, a 304, chunk extensions and trailers, a length
    repeated, no reason phrase, fields in odd case and whitespace, a
    response to HEAD, and `Connection: close`;
  - hostile ones: an oversized head, a head that never ends, chunk sizes
    that overflow or are not hexadecimal, a chunk without its line
    ending, a chunked body and a body by length cut short, bad status
    lines (a bad one, HTTP/2's, not HTTP at all), conflicting framing
    (both headers, two lengths, a coding not undone, chunked in
    HTTP/1.0), a CR in a field, whitespace before a colon, a fold before
    any field, too many fields, an upgrade, a trailer section that never
    ends, and event streams whose event never ends, whose line is too
    long, and whose comments never reach a blank line.

  Each decodes to its expectation under several seeds, read a byte at a
  time so that every byte of the body is seen, and cut anywhere, as the
  reference reads the prefix; each event stream reads to its events under
  scans of several sizes. A transcript without an expectation fails,
  printing the reference readers' reading as a draft to check.

  Requests are kept the same way, in `tests/http/transcripts/requests/`,
  each with what the server must make of it, served by a side above that
  reads each body to its end and answers `200`: the call or the
  rejection, the body, and the outcome. Each is written by hand after the
  public format of its client, and says so. Thirty-six:
  - curl's: a GET, a HEAD, a JSON POST, a body that waits for a 100
    (Continue), its `Expect` asked for, a chunked upload from standard input
    that waits for one too, HTTP/1.0, two requests on one connection, and
    `Connection: close` before a request never read;
  - an LLM client's POST with a JSON body, as Anthropic's and OpenAI's
    Python SDKs send them, their keys redacted;
  - hostile ones: an oversized head and an oversized request line, a
    chunk size past a `u64`, smuggling with both framing headers in
    either order and with whitespace before a colon, a bad request line,
    HTTP/2's preface, a method not implemented, no `Host` and two, a
    fold, a NUL and a bare CR in a field, a coding not undone, a body
    past the limit, by length and in chunks, a head and a body cut short,
    a chunk without its line ending, a trailer section that never ends,
    chunk extensions past the head in all, two lengths, a length with a
    sign, a `Host` that names no host, and too many fields.

  Each comes to its expectation through the server's world under several
  seeds, read a byte at a time, and cut anywhere, as the reference reads
  the prefix.
- **The machines stacked** (`tests/http/tests/suite/stack.rs`), a small step
  towards the protocol worlds: the reader and a tokenizer per event over
  the client, as a connection routes between them, the request body
  written by the JSON writer and uploaded, over the LLM transcripts cut
  at random; every event is the transcript's and every document the JSON
  reference parser's reading of its data.
- **Protocol worlds** (../../../foundation/testing-strategy.md, 2.5) in `tests/protocol`
  (`skein-protocol-world`): both ends of an LLM streaming exchange, built
  as two services would build them. The client's end stacks the client,
  the reader on its body and skein-json's selective collector on each
  event's data, as an LLM client's stack has it (`README.md`, section 2), its request
  written by the JSON writer; the server's stacks the server, a
  tokenizer on the request body and the writer on the reply, each event's
  data written by the JSON writer; each routes as a connection routes,
  outputs queued without reservation, and a scripted user at each top.
  The two bottoms are joined by a stream each way, carried in pieces cut
  at random and joined in the receiving intake, each end's side below
  keeping the stream's contract as io keeps it; a closed end drains what
  comes for its linger, and what comes after resets the other's stream,
  as on a socket, and a read past the end stays outstanding until it is
  withdrawn, as TLS keeps it. The referee is `skein-world`'s
  (`Referee`, `Expectations`), and watches what the users saw. Safety,
  at every observation: what the top of one end sent is what the top of
  the other received, each request and every event, token for token,
  each event with the last event ID and reconnection time the reader's
  face shows once it came; and what the writer sent and the reader's
  user has not read is never more than the caps between them. Liveness,
  goals with a deadline each, drawn from the bytes between the ends and
  the pace of the wire and the users: each scenario's outcome, both ends
  settled, the end that did not close hearing of it, and what a scenario
  aims at, a write held back as its user sees it, an event or an upload
  that waits while the other end's user does not read. The referee
  injects the reset and ends the run once its goals are met, or fails it
  with what is pending. The scenarios: an answer streamed whole; two or
  three calls, one after another on the connection the last kept,
  through both stacks; a slow reader at the client's top, far more
  events than the caps hold, which must stop the writer at the server's;
  a slow consumer at the server's top, which stops reading a request far
  larger than the caps hold, and must stop the client's upload; a
  response that comes mid-upload, an error before the body is read,
  which stops the client's upload and is read; one end closing while the
  other sends, in one of up to three calls; and the wire resetting at
  any moment. Events carry an id or a reconnection time now and then.
  Once both ends are closed, every machine is, and each stack withdrew
  what it demanded below, but a read past the end, before its owner
  closed the stream. A seed replays to the same run. The worlds run in
  plaintext. `skein-world`'s `World` drives processes' `iterate` over the
  simulator, through the kernel's records, which a world joined by bytes
  has none of, so these keep their own loop, of the same shape: the
  contracts as it goes, the referee beside it.
- **Memory** (`tests/http/tests/suite/memory.rs`, with the counting allocator,
  ../../testing/README.md, 5): every call of an entry point a step of the meter. The
  client at its limits: a head at the head limit with folds among its
  fields, bodies by length, chunked with a trailer section at its limit,
  and to the end of the stream, an interim head, an upload, a request
  head near its limit, a response cut short, each read with three
  demands, and closed, failed and discarded after every step; and the
  carry-over held with a delivery. The reader at its limits: an event at
  every limit with lines ended every way, a line, an event and a type
  past theirs, each closed after every step. Every peak stays under
  its scenario's ceiling with headroom. The server and the writer
  (`tests/http/tests/suite/memory_server.rs`): the server at its limits, a head
  at the head limit with its fields at theirs, bodies by length and
  chunked with a trailer section at its limit, a 100 (Continue),
  responses at the response limit by length, chunked and without a
  body, a rejection and a body cut short, each read with three demands,
  responded to before and after its body, and closed, failed and
  discarded after every step; and the carry-over held with a delivery.
  The server's scenarios cover both phases: a request line filling the head, with
  its target's copy; and a response head at its limit, held while a
  discard reads a piece of the most a read is. The writer's peak at its
  limits stays under its scenario's ceiling.
- **The fuzzy suite** (`tests/http/tests/suite/fuzzy/*.rs`): 20,000 connections
  of one to four generated exchanges, valid, mutated, and corrupted where
  a random edit seldom lands (another major version, a code past 599, an
  upgrade, conflicting framing, a chunk size past a `u64`, a trailer
  section without end), and 2,000 transcripts cut and mutated, each
  under limits and neighbours drawn from its seed; 12,000 event streams
  generated and mutated, and 2,000 of the transcripts' event bodies. Each
  sweep asserts that what it injects fell (../../../foundation/testing-strategy.md, 3): every
  outcome of an exchange, every refusal of a call and every error of a
  stream, each way a stream can end or fail below and what the machine
  waited for when it failed, each thing it waits for, a close while it
  waited for each thing, room granted while a response
  waited, a response read mid-upload, a withdrawal and an answer after
  it, a discard, a connection reused, a CR's LF delivered alone, and a
  line longer than a chunk read in pieces. The server's: 10,000
  connections of one to four generated requests, valid, mutated, and
  corrupted where a random edit seldom lands (another major version, a
  method not implemented, a fold, whitespace before a colon, both framing
  headers, a coding not undone, two lengths, a chunk size past a `u64`,
  no `Host`, a field and a request line past the head, too many fields, a
  body past the limit, a trailer section without end, HTTP/2's preface),
  and 1,000 request transcripts cut and mutated; it asserts that every
  outcome, every rejection and refusal, each way a stream ends or fails
  and what the server waited for then, a close while it waited for each
  thing, a 100 (Continue), a client tired of waiting for one, a body
  given up, a head that waited for a discard, room after the end, a
  reply withdrawn, pipelining and reuse fell. The writer's: 5,000 runs,
  every answer and refusal, an event in pieces, the end and room after
  it, a failure idle and while writing, and a close in each state. The
  protocol worlds' (`tests/protocol/tests/suite/fuzzy/worlds.rs`): 300 runs of
  the scenarios under caps drawn down to the least the stacks allow,
  asserting that a writer and an upload were held back, an upload
  stopped, a writer heard its stream fail, a stream reset, a connection
  carried several calls, an event's id or reconnection time was checked,
  and each outcome at each end fell. Fuzz targets run beside it, on a
  nightly toolchain.
- **The reader's data face** (`sse.md`), joins each tier
  above. Step tests: the data under fills and scans of every shape, data
  lines joined across a CRLF split and a CR alone, comments and other
  fields between data lines, a type line after the data, an event with no
  data line, an event cut short by the end and by a failure, a limit
  passed mid-event, a withdrawal mid-event, and a close while an event is
  open. The reader's world holds each event's data, joined from its
  stream, its type and its ID to the reference's, and an event cut short
  to a data stream that failed where the reference drops the event. The
  memory test adds an event of a megabyte under a chunk of a few bytes,
  whose peak stays under the same scenario ceiling as an event of a
  few bytes. The stacked machines and the protocol worlds read each
  event's data through the face, the JSON reference judging what the
  tokenizer, or the collector, makes of it.
