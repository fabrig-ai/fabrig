# real-journeys: contracts

The run's binding calls: the shapes that cross tasks, in `temper-client-shell-web`'s `journey` module (`temper/docs/design/client/journeys.md`, 3.6 and 3.7). Every module here is native logic without io, as `walk` is. `journey.rs` re-exports what `tools/journeys` needs. Each task adds its own `mod` and `pub use` lines where its spec says, so that parallel branches meet cleanly.

## `journey/fields.rs`: every byte field of a line, one list

The redaction and the scan read fields through this one function, whose `match` lists every variant with no `_`, so that a new record kind can't slip past the guard.

```rust
/// Whether a length prefix counts a field's bytes (client/journeys.md, 3.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Framing { Framed, Text }

/// Each byte field of `line`, in its order. A `heard piece`'s bytes are not among them: they are read through `Streams`.
pub fn fields_mut(line: &mut Line) -> Vec<(&mut Box<[u8]>, Framing)>;
```

- **Framed:** `heard answered`'s body; `exchange`'s body; `save` and `stored`, key and value; `written`, `unwritten`, `erase`.
- **Text:** `ways`, `forge`, `path`, `secret`, `place`; `act`'s value, event and name; `exchange` and `open`, path and each header's name and value; `navigate`, `replace`, `show`'s path; `mark` and `end`'s html.
- **Nothing:** the numbers and words: `exchange`'s method, a checked word; `journey`, `seed`, `clocks`, `primary`, `zone`, `turn`, `start`, `due`, `carried`, `lost`, `close`, `writes`, `deadline`; `heard failed`, `opened`, `refused`, `ended`, `stored`, `not stored`.

## `journey/stream.rs`: a recording's streams, joined

```rust
/// One `data:` line's record, decoded. `line` numbers from 1, the piece it starts on.
pub struct Event { pub line: u32, pub record: Box<[u8]> }
/// Stream bytes outside any `data:` payload that are not bare framing: a comment, a refused stream's body, the undecoded end of a cut last line.
pub struct Plain { pub line: u32, pub bytes: Box<[u8]> }
/// A `data:` line whose payload is not canonical standard base64.
#[derive(Debug, PartialEq, Eq)]
pub struct Undecoded { pub line: u32 }

pub struct Streams { /* private */ }
impl Streams {
    /// Join each stream's pieces and decode its `data:` lines. An `open` or `heard opened` starts a stream.
    pub fn read(lines: &[Line]) -> Result<Self, Undecoded>;
    pub fn events(&self) -> &[Event];
    pub fn events_mut(&mut self) -> &mut [Event];
    pub fn plain(&self) -> &[Plain];
    pub fn plain_mut(&mut self) -> &mut [Plain];
    /// Encode every record back and cut the bytes where the pieces ended.
    /// # Panics
    /// A record or plain run whose length changed; the message names lengths alone.
    pub fn write(&self, lines: &mut [Line]);
}
```

The base64 is `skein_lib::base64`, `Alphabet::Standard`, as `jig-client-protocol-http`'s stream reads it. A line is cut at `\n`; `data: ` is the prefix, one space, as the engine's writer spells it.

## `journey/originals.rs`: what is taken out

```rust
/// How an event shows a secret's length (client/journeys.md, 3.3).
pub const SECRET_EVENT: &[u8] = b"Secret { len: ";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind { Link, Stored, Typed, Given }

/// One original, sought everywhere. Its Debug prints its kind and length alone.
pub struct Original { /* private */ }
impl Original {
    pub fn kind(&self) -> Kind;
    /// The placeholder's length, the original's bytes.
    pub fn len(&self) -> usize;
    /// Its forms, its bytes first, each once: percent-encoded in both hex cases, `dom::print`'s escapes, Rust's Debug escapes.
    pub fn forms(&self) -> &[Box<[u8]>];
    /// Each sha-256 that may stand for it, and the placeholder's: of its bytes and, for a value with surrounding spaces, tabs, CRs or LFs, of the value whole.
    pub fn digests(&self) -> &[([u8; 32], [u8; 32])];
}

pub struct Originals { /* private */ }
impl Originals {
    /// Read the originals of a recording's lines and the given texts (journeys.md, 3.6).
    pub fn of(lines: &[Line], given: &[Box<[u8]>]) -> Self;
    /// Longest first, each once.
    pub fn sought(&self) -> &[Original];
    /// Distinct originals of a kind, sought or not: a typing run's heads count with its value, as one.
    pub fn count(&self, kind: Kind) -> u32;
}

/// Whether the line is an act whose event shows a secret's length.
pub fn typed(line: &Line) -> bool;
/// `value` with its placeholder in place of its bytes, surrounding spaces, tabs, CRs and LFs standing.
pub fn placeholder(value: &[u8]) -> Box<[u8]>;
/// The length of a `Kept` value's head, by `jig_client_protocol_wire::Kept::decode` under `CEILINGS`: None for a value that isn't one.
pub fn kept_head(value: &[u8]) -> Option<usize>;
```

- An original of `x`s alone, or empty, is none.
- A `Kept` record's whitespace isn't trimmed: only a typed value's and a given text's are.

## `journey/redact.rs`

```rust
pub struct Redacted { pub recording: Box<[u8]>, pub originals: Originals, pub places: u32 }

#[derive(Debug, PartialEq, Eq)]
pub enum Unredacted { Unreadable(Unreadable), Undecoded(Undecoded) }

/// Take the originals out of a recording, its live prints among them (journeys.md, 3.6).
pub fn redact(recording: &[u8], given: &[Box<[u8]>]) -> Result<Redacted, Unredacted>;
```

`places` counts each replacement, an in-place typed value among them. Its Debug prints no bytes (`Redacted` has none).

## `journey/scan.rs`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape { Key, Jwt, Bearer, Code, RefreshToken }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum What { Original, Shape(Shape) }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place { Field, Record, Plain, RunText }
/// What the scan found, by the record and its kind, never the bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Found { pub line: u32, pub record: &'static str, pub place: Place, pub what: What }

pub fn bytes(bytes: &[u8], originals: &Originals) -> Option<What>;
/// A page's html, its four escapes read back first.
pub fn page(html: &str, originals: &Originals) -> Option<What>;
/// Every field, a stream's records and plain text, and each run's joined text.
pub fn lines(lines: &[Line], originals: &Originals) -> Result<Option<Found>, Undecoded>;
```

`Line::kind` becomes `pub` and returns `&'static str` where the scan needs the record's word; a line reading its kind as bytes changes to match.

## `journey/replay.rs`

```rust
/// As `prove`, giving `drawn` the turn's number and the page at each turn's end (journeys.md, 3.7).
pub fn prove_drawn(recording: &[u8], drawn: &mut dyn FnMut(u32, &Page<'static>)) -> Result<Box<[u8]>, Refusal>;
```

`prove(recording)` stays, and is `prove_drawn` with a visitor that does nothing.

## `tools/journeys`

```rust
pub struct Added { pub link: u32, pub stored: u32, pub typed: u32, pub given: u32, pub places: u32 }

pub fn add(journeys: &Path, name: &str, recording: &[u8], given: &[Box<[u8]>]) -> Result<Added, Refusal>;

// Refusal gains
Undecoded { name: String, line: u32 },
Found { name: String, line: u32, record: &'static str, place: journey::Place, what: journey::What },
FoundDrawn { name: String, turn: u32, what: journey::What },
FoundPrinted { name: String, page: String, what: journey::What },
```

`add` goes: the name; `journey::redact`; `journey::scan::lines` on the redacted lines; `journey::prove_drawn`, scanning each drawn page by `scan::page`; the pages built in memory; `scan::lines` on the html-free recording and `scan::bytes` on each printed document; then, and only then, its first `fs` call.
