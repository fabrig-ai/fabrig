# The channel across layouts: contracts

2026-11-24. The shapes the changes of the goal building
[`layouts.md`](layouts.md) and [`wires.md`](wires.md) share, and no other
area reads (temper's `draft/design-economy.md`, 4.3). They bind as those
documents do. The goal's last change deletes this document once the code
holds them.

## 1. The conversion module

The engine's conversions (`layouts.md`, 2 to 5), in
`jig-protocol-worker-wire`, beside its kinds. Made first, used by no
component, then taken by the engine's workers component when the
opening switches to layouts.

```rust
// jig_protocol_worker_wire::layouts
pub const MAX_TEXTS: u32 = 8;        // worker texts kept per wire (layouts.md, 3)

/// One wire as the worker's credential states it (wires.md, 2), borrowed.
#[derive(Clone, Copy, Debug)]
pub struct Theirs<'a> { pub name: &'a str, pub layout: [u8; 32], pub text: &'a [u8] }

/// A part of another owner's that a jig record carries (layouts.md, 2.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nested { Charter, Result, Turn, Carried(Carried) }

/// Why a value did not convert (layouts.md, 5): skein's `Refused` kinds,
/// a result past its bound, or bytes that are no value of the source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause { Variant, Field, Narrowed, Absent, Large, Malformed }

/// A value that did not convert, named by the source text's names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unconverted { pub wire: Box<str>, pub declaration: Box<str>, pub part: Box<str>, pub cause: Cause }
impl Unconverted {
    /// layouts.md, 5.1: `the worker's layout cannot keep WIRE DECLARATION.PART: WHY`.
    pub fn kept(&self) -> Box<[u8]>;
    /// layouts.md, 5.3's tail: `WIRE DECLARATION.PART: WHY`.
    pub fn read(&self) -> Box<[u8]>;
}

/// The engine's side: its own texts, parsed once, its application's roots,
/// and the plans derived so far by wire and worker text (layouts.md, 3).
#[derive(Debug)]
pub struct Plans { /* … */ }
impl Plans {
    /// `texts`: the current text of each wire the engine speaks, in its
    /// order, its name and layout read from it. Refused with a text when
    /// one does not parse, or a root names a wire or a declaration they lack.
    pub fn new(texts: &[&'static str], roots: &List<Root>) -> Result<Plans, Box<[u8]>>;
    /// wires.md, 3, the wires' checks in their order, with their texts;
    /// a wire at the engine's layout is not parsed.
    pub fn judge(&mut self, theirs: &[Theirs<'_>]) -> Result<Peer, Box<[u8]>>;
}

/// One channel's conversions: per wire, nothing or its plans, and one
/// workspace fitted to them all.
#[derive(Clone, Debug)]
pub struct Peer { /* … */ }
impl Peer {
    /// Every wire at the engine's layout: every call below hands its bytes back as they are.
    pub fn same(&self) -> bool;
    /// wires.md, 4: the worker's kinds' table at jig's version, its bounds within `limits`.
    pub fn table(&self, limits: &v22::Limits) -> Result<skein_channel::Version, Box<[u8]>>;
    /// The frame kind of one of the engine's `FromEngine` kinds at the
    /// worker's layout, or none where the worker's layout lacks it.
    pub fn sends(&self, kind: Kind) -> Option<u16>;
    /// layouts.md, 4.1: an engine body of `kind` to the worker's frame kind and body.
    pub fn send(&mut self, kind: Kind, body: &[u8], out: &mut Buffer) -> Result<u16, Unconverted>;
    /// layouts.md, 4.2: a worker frame to the engine's kind and body.
    pub fn receive(&mut self, kind: u16, body: &[u8], out: &mut Buffer) -> Result<Kind, Unconverted>;
    /// A part down, by its Keep plan; `Turn` serves a window's turns and a resumed part's body.
    pub fn down(&mut self, nested: Nested, bytes: &[u8], out: &mut Buffer) -> Result<(), Unconverted>;
    /// A part up, by its Read plan.
    pub fn up(&mut self, nested: Nested, bytes: &[u8], out: &mut Buffer) -> Result<(), Unconverted>;
}
```

- **`Kind`, `Carried`, `Root` and `v22::Limits`** are the crate's own as
  they stand; `Buffer` is skein lib's, its capacity the bound the caller
  holds the result to, `Large` past it.
- **A `Nested` and a direction it does not cross** (`layouts.md`, 2.1):
  `Malformed`, since no plan exists for it; the components never ask.
- **When the opening switches,** the engine's `texts` are its
  `Spoken`s' (`layouts.md`, 7), and `Theirs` is read from the
  credential's `Wire`s (`wires.md`, 2).

## 2. Frames and the parts they carry

A frame's whole conversion (`layouts.md`, 4.1 and 4.2) over a `Peer`
(1), in a module of its own beside it, so that the engine's workers
component calls one function each way. Made before the opening switches;
the version agreement's `translation` module, which it replaces, goes
when the components take it.

```rust
// jig_protocol_worker_wire::convert
/// layouts.md, 4.1: a frame of the engine's, at its layouts, to the
/// worker's: each part its record carries (layouts.md, 2.1) down by its
/// Keep plan, a resumed window's turns each on its own, an
/// `Assignment.parts` item whose body refuses dropped; then the record by
/// the frames' plan, within the worker's largest body of its kind.
pub fn send(peer: &mut Peer, frame: Frame, limits: &v22::Limits) -> Result<Frame, Unconverted>;
/// layouts.md, 4.2: a worker's frame, its kind at the worker's numbers,
/// to the engine's frame: the record up by the frames' plan, within the
/// engine's largest body of its kind, then each part it carries up by its
/// Read plan.
pub fn receive(peer: &mut Peer, kind: u16, body: &[u8], limits: &v22::Limits) -> Result<Frame, Unconverted>;
```

- **`Frame`** is skein's channel frame (skein's `net/channel.md`), as the
  components hand it now; `limits` the engine's, which its codec decodes
  a record within.
- **A `Peer` whose wires are all at the engine's layout** (`Peer::same`)
  hands the frame back as it is: nothing decoded, nothing allocated.
- **A part with no root** (`layouts.md`, 7) crosses as it is; a part's
  refusal names the part's wire and declaration, as `Peer::down` and
  `Peer::up` name it.
