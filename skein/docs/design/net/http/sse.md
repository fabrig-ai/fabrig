# Server-sent events

Two machines over a body stream (WHATWG HTML, 9.2), each with the same
shape on both sides as the client's: the reader, which a client stacks
on a response body, and the writer (3), which a server stacks on its
reply. The reader:

```rust
pub enum Request {                         // from the side above
    Next,                                  // the next event: Opened, Ended or Failed answers it
    Data(stream::Down),                    // the open event's data, read as a stream
    Close,                                 // Closed answers it
}

pub enum Event {                           // to the side above
    Opened,                                // for a Next: an event's first data line has begun
    Data(stream::Up),                      // Bytes; End at the event's blank line; Failed if it is cut short
    Dispatched(Dispatch),                  // with the data's End: the event's type and the last event ID
    Ended,                                 // for a Next: the stream ended between events
    Failed(Error),                         // for a Next: a limit passed, or the stream failed
    Closed,                                // for a Close: terminal
}

pub struct Dispatch { name: Box<[u8]>, id: Box<[u8]> }
pub enum Error { LineTooLong, EventTooLong, FieldTooLong, Stream(Fault) }
```

- **`Next` asks for the next event.** Exactly one event answers it:
  `Opened`, once an event's first `data` line begins; or the stream's
  outcome, `Ended` or `Failed`, after which nothing follows but `Closed`.
  An event with no data line is never opened, as the standard dispatches
  nothing for it, though its `id` and `retry` take effect. One `Next` at a
  time, none while an event is open, and none after the outcome or the
  close: the side above's bug otherwise, asserted.
- **The open event's data is a stream** (`Data`), a `lib::stream` face of
  which the reader is the side below (../../runtime/lib.md, 7), as the HTTP client is a
  body's (`client.md`, 3). Its bytes are the values of the event's `data` lines,
  joined by LF as the standard joins them, delivered as they are read.
  The side above demands fills and scans; the reader meets each from the
  line pieces it reads below, holding what a demand has not taken yet in
  an intake of `Limits::chunk`, and reads below only what a demand still
  needs. A JSON tokenizer, or skein-json's selective collector, stacks on
  it as on any stream (../../data/json.md, 5.1).
- **The data ends at the event's blank line.** `End` answers a demand
  that what the data holds can no longer meet, and `Dispatched` follows
  it in the same step, with the event's type (`message` when none was
  set) and the last event ID. The type goes up only then, as the standard
  lets a type line follow the data lines.
- **An event cut short is never dispatched.** When the stream ends or
  fails before an open event's blank line, its data stream hears
  `Failed`, with the stream's own fault or `Other` for an end, and the
  side above drops what it read of it, as the standard drops the event.
  The outcome answers the next `Next`. A limit passed while an event is
  open fails its data stream with `Invalid`, and the error answers the
  next `Next`.
- **A withdrawal** of the data's demand means the side above reads no
  more of this event (../../runtime/lib.md, 7): the reader reads the rest of it to its
  blank line, counted and dropped, then goes on as at its end,
  `Dispatched` included.
- **`Close` ends the reader in any state,** as the JSON tokenizer's does
  (../../data/json.md, 3.1): it withdraws what it demanded below, drops a `Next`
  not yet answered and an open event's data without a word, and answers
  `Closed`.
- **`sse::Data`,** a side below over one box, remains for data already
  whole, such as a recorded event: each demand answered at once by what
  it reads (exactly, or all that is left up to the maximum of a read of
  what has arrived), or by `End` when what is left cannot meet it, as
  lib's intake would.
- **What it waits for**, `waiting()`: `Next`, `Bytes`, `Above` (an open
  event's data waits for a demand), `Close`, `Nothing`. Every line is
  progress, a comment sent to keep the stream alive among them, and so is
  each piece of a data line delivered. The reconnection time and the last
  event ID are `retry()` and `last_event_id()`, for a client that
  reconnects.

## 1. Lines, fields and events

- **A line ends at LF, at CRLF, or at CR alone.** A CR ends its line at
  once; an LF right after it is its pair, and ends nothing.
- **The reader reads a line at a time,** by line scans of at most
  `Limits::chunk` (`Read::Line`, ../../runtime/lib.md, 7): each delivery ends at the
  first CR or LF, or is a chunk of a line longer than one. Every line is
  read as soon as its end arrives, whatever the stream's convention, and
  a stream shorter than a scan as it comes. An LF-only or a CR-only
  stream is one delivery a line; a CRLF is two, the line to its CR and
  then the LF alone, which the reader skips as the CR's pair.
- **A delivery never crosses a line's end,** as each line scan stops at
  one, so the reader keeps nothing of a delivery past the step that reads
  it but a data line's piece, which waits in the data intake for the side
  above's demands. An end or a failure that comes while the reader
  demands nothing answers the next `Next`; after the end, a failure says
  only that the stream can no longer send, and what was read stands.
- **At the end of the stream, only an incomplete last line goes unread,**
  as the standard drops it: a line scan larger than what is left is never
  met (../../runtime/lib.md, 7). Of a last line longer than a chunk, the whole chunks
  are read, so one that passes a limit fails if a whole chunk past the
  limit came, and otherwise ends the stream.
- **Fields** (WHATWG HTML, 9.2.6), each value going where it belongs as
  its bytes come: `data` onto the event's data stream, an LF before each
  data line but the first; `event` into its type; `id` into a value held
  until its line ends, then
  into the last event ID buffer unless it holds a NUL; `retry` into the
  reconnection time if its value is ASCII digits, and no more than a
  `u64`. A line that begins with a colon is a comment; one whose name is
  no field's is ignored; a name with no colon is the field with an empty
  value; one space after the colon is dropped.
- **Dispatch at each blank line:** the last event ID is set from its
  buffer; with no data line, nothing is dispatched; otherwise the data
  stream ends, and the dispatch goes up with its type (`message` when
  none was set) and the last event ID. No LF trails the data: one goes
  before each data line but the first, which is what the standard's
  dropping of the last LF leaves.
- **One leading byte order mark** is skipped, as UTF-8 decoding does.
  Nothing else is decoded: the data goes to a decoder that checks UTF-8
  (JSON does), and a type or an id is compared as bytes.

## 2. Limits and validity

```rust
pub struct Limits {
    pub line: u32,    // the longest line before its ending, counted as read: past it, LineTooLong
    pub event: u32,   // the bytes read from the end of the last event to the blank line of this one,
                      // endings included, counted as read: past it, EventTooLong
    pub field: u32,   // the longest event type or id: past it, FieldTooLong
    pub chunk: u32,   // the most demanded at once: each line scan's maximum, and the data intake's cap;
                      // at least one
}
```

- **Every byte read counts against the event being read,** a comment's
  and an unknown field's too, so a stream that never ends an event fails
  whatever it sends; each blank line starts over, so a stream kept alive
  by comment blocks runs on.
- **`line` and `event` bound what a peer sends, not what the reader
  holds.** A data line's bytes pass through the data intake a piece at a
  time, so an event of any size within `event` costs the reader no more
  than one of a few bytes. The LLM client sets both from its skip cap
  (../../llm/README.md, section 4.2).
- **Edges bound retained data:** `field` bounds the event type and ID
  buffers, and `chunk` the data intake and current delivery. A dispatch's
  boxes and each piece of data pass to their owner when emitted. A chunk
  must be positive. There is no reader heap sum; worlds check measured
  peaks under scenario ceilings.
- **`largest_demand`** is `chunk`, and so is the largest demand the data
  face meets: whoever stacks a tokenizer on it checks the tokenizer's
  largest demand against it at startup.
- **`up` emits at most** two events and one request: an open event's data
  ending and its dispatch; below, the next demand. **`down` emits at most**
  one event and one request: only a `Close` while reading emits both,
  `Closed` and the demand withdrawn.

## 3. The writer

```rust
pub enum Request {                         // from the side above
    Event(Outgoing),                       // write an event: Sent, Refused or Failed answers it
    Comment(Box<[u8]>),                    // write a comment, to keep the stream alive: the same
    Finish,                                // end the body, once nothing is being written
    Close,                                 // Closed answers it
}

pub enum Event { Sent, Refused(Refusal), Failed(Fault), Closed }

pub struct Outgoing { name: Box<[u8]>, data: Box<[u8]>, id: Option<Box<[u8]>>, retry: Option<u64> }
pub enum Refusal { Name, Id, Comment, TooLong }
```

- **One at a time.** Exactly one event answers each event or comment:
  `Sent`, once all of it went down; `Refused`, for what the side above
  got wrong, writing nothing and leaving the writer as it was; or
  `Failed`, the stream's failure, now or before, after which nothing
  follows but `Closed`. Another while one is written, or one after
  `Finish`, is the side above's bug, asserted.
- **Framed sized, a block each:** `event: ` and the type, if it has one;
  `id: ` and the id, if it has one, an empty one resetting a reader's
  last event ID; `retry: ` and its digits; a `data: ` line for each line
  of the data, split at each LF, CRLF or CR, so one line more than its
  endings; and a blank line. One space follows each colon, which a reader
  drops, so a value's own leading space survives. A comment is `: ` and
  its text, or the colon alone, then a blank line, so that a reader's
  count for an event starts over after it (2).
- **Refused in a fixed order:** a type with a CR or an LF (`Name`), an id
  with a CR, an LF or a NUL, which a reader ignores (`Id`), a comment
  with a CR or an LF (`Comment`), then a frame past `Limits::event`
  (`TooLong`).
- **Every event written is one a reader dispatches,** and reads back as
  it was written: a type of none as `message`, the data's line endings
  as LFs, the last event ID as each `id` left it.
- **Sent in pieces** of at most `Limits::chunk`, each within the room
  granted for it, one `Send` a grant; the frame itself goes, uncopied,
  when it fits one. The server's chunked reply makes each a chunk.
- **`Finish`** ends the body below, once nothing is being written; after
  the stream failed there is nothing to end. The stream's `End` changes
  nothing for a writer, which reads nothing: room may still come after
  it.
- **`Close` ends the writer in any state:** it withdraws what it
  demanded below, drops an event not yet sent, and answers `Closed`.
- **What it waits for**, `waiting()`: `Above`, `Room` (the reader is not
  reading), `Close`, `Nothing`.

## 4. The writer's limits and frame bound

```rust
pub struct Limits {
    pub event: u32,   // the longest event or comment framed, its blank line included: past it, TooLong
    pub chunk: u32,   // the most room demanded at once; at least one
}
```

- **`frame_bound(&limits)`** is the event being written, framed, at most
  `event`, held until all of it went down. What the side above gives it
  is the side above's to count, and a piece copied out is handed out as
  it is sent. `None` for a chunk of zero.
- **`largest_room`** is `chunk`, which whoever stacks the writer checks
  against the server's `Limits::send`. A reader whose `Limits::event` is
  at least the writer's reads every event it writes.
- **`up` emits at most** one event and two requests: room granted sends a
  piece and asks room for the next, or sends the last and says `Sent`.
  **`down` emits at most** one and one.
