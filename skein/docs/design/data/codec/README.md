# Codecs from schemas

Provisional, 2026-10-06, revised 2026-11-25. How an application protocol
built on skein encodes its records, and how a value written at one
layout of its schema is read at another. A schema says each record's fields
and their bounds. skein's generator turns it into plain Rust codecs,
limits and bounds, and `skein-codec` holds the little
the generated code shares, and reads schemas and their records at run
time. Codecs carry the bodies of a framed channel (../../net/channel.md), and
records kept in files or stores.

- [`golden.md`](golden.md): golden bytes and tests.
- [`run-time.md`](run-time.md): run-time schemas and generic values.
- [`fingerprints.md`](fingerprints.md): family and value-type fingerprints.
- [`layouts.md`](layouts.md): plans between two layouts, either way, in Read or Keep mode.
- [`command.md`](command.md): an owner's files and the command.
- [`maps.md`](maps.md): translation from maps files.

## 1. In one page

- **The schema is the specification.** A small file per family and version
  gives each record's fields, their types and their bounds. Design
  documents name records and bounds; the schema gives the bytes.
- **Generated, not written.** A generator, ordinary Rust in skein, turns a
  schema into:
  - the records' types, with public fields, checked against their
    limits where bytes are made or read;
  - limits, whose ceilings are the schema's bounds;
  - measured encoding;
  - bounded decoding with typed problems;
  - each record's encoded and decoded bounds, from a table;
  - for tests, each record's fixed-rule values (`golden.md`, section 1).
- **The subset's Rust, calling generic helpers** (../../../foundation/programming-model.md,
  section 10): each declaration implements `skein-codec`'s `Record`,
  one helper call per field. The helpers are `skein-codec`'s, under
  lib's exception (10.2), written once and tested hard; the generated
  code defines no generic item, closure or macro. The output is kept in
  the consumer's repository. A test fails when the code and the schema
  drift.
- **One pin on the bytes** (`golden.md`, section 1). The `every` fixture uses every
  construct of the language, and its golden files pin the encoding for
  every family, whose tests build their values from their schemas.
- **Sized** (../../../foundation/programming-model.md, section 8). Every length comes before
  its bytes, and is checked against its limit and what is left before
  anything is set aside. Types form a tree of fixed depth, so decoding
  never recurses on a peer's choice.
- **Canonical.** One value has one encoding, and decoding refuses any
  other, so what decodes encodes back to the same bytes.
- **Versions frozen, layouts by fingerprint** (section 5). A schema is
  one version of a family, and a family has one current version, its
  header's, which no change of layout moves. A layout is named by its
  fingerprint (`fingerprints.md`, section 1). Generated code reads and
  writes the current text's layout and refuses another version. Formats
  outside the store's registry are fixed at their versions; a change to
  one brings its own route for the deployed store.
- **Defaults and former names in the schema** (section 3): a field may
  give the value an older layout without it takes, and a declaration,
  field or variant the names it had, so that a layout converts to the
  next with no text beside the schema.
- **Read at run time too.** `skein-codec` reads a schema's text as step
  code (`run-time.md`, section 1), and decodes the bytes of any of its records into a
  generic value, an index over those bytes, with the generated code's
  checks (`run-time.md`, section 2). The generator reads schemas through the same
  parser, so there is one language.
- **One conversion, either way: plans.** A value type's layout
  fingerprint (`fingerprints.md`, 1.4), and a plan derived from two
  layouts' texts, in Read or Keep mode, with no notion of older or
  newer: a pass over a value's bytes that gives the other layout's bytes
  or refuses that value (`layouts.md`). A store runs one on each value
  it reads at another layout than its binary's, and an engine on each
  message to or from a worker at another (section 5). No generated code
  is kept for another layout, and no repository keeps another layout's
  text.

## 2. In skein

- **`skein-codec`** depends on lib only, and builds for
  `wasm32-unknown-unknown` as lib does (../../runtime/lib.md, section 2), with the code
  generated for it. It holds what the generated code shares:
  - `Record`, the trait each generated declaration implements, and the
    generic helpers that check, measure, encode and decode each kind of
    field (section 4);
  - the walk that computes bounds from a module's shapes (section 4);
  - the reasons a decoding fails, and the problem naming its path;
  - bounded reads of lengths, counts and text;
  - reading a versioned record's version;
  - writing a piece of items and splitting one (section 3).

  And, as step code, what reads schemas and records at run time:
  - `schema`: the parser and the layout checks (`run-time.md`, section 1);
  - `value`: generic values (`run-time.md`, section 2);
  - `layout`: derived plans from one layout of a value type to another,
    and their pass over a value's bytes (`layouts.md`).

  And `convert`, the scalar conversions a family's generated translation
  calls (`maps.md`, 5).
- **`skein-codegen`** is the generator: the command and an owner's files
  (`command.md`), and the translation from maps files (`maps.md`).
  Ordinary Rust, not
  step code. It reads schemas through `skein-codec`'s parser and adds
  the checks only Rust needs (`run-time.md`, section 1). A consumer depends on it in its
  tests and tools only.
- **A consumer's codec crate** holds a family's generated record code,
  `src/generated/<stem>.rs` (section 4), and depends
  on lib and `skein-codec` alone; its dev-dependencies are free,
  `skein-codegen` among them. The family's **owner**, the crate that
  keeps it, holds the rest: its schema texts and maps file
  (`command.md`, 1), its drift test (`golden.md`, section 1), and the
  translation between its records and a domain, with the domain's crates
  as its dependencies. So the generated
  code compiles beside the domains rather than after them, and only a
  change of schema recompiles it (temper's `draft/archive/crate-graph.md`, 4.1).
- **A codec crate is named for its owner,** `<owner>-codec`, in a
  directory beside the owner's that takes the owner's directory name
  with `-codec` (`skein/crates/skein-oauth-codec`,
  `skein/examples/ui/view-codec`); an owner whose families are split by
  size names each crate for its family instead (jig's store,
  `jig-protocol-store-people`). Each generated module keeps its Rust name
  at the codec crate's root (`v3`, `cursor_v1`), whatever its file's
  name (section 4), and the owner names it
  where it named it before: a public module is re-exported at the
  owner's same path, which callers keep, and either path names it.
- **An owner with no domain to translate is its own codec crate,**
  holding schemas, code and drift test together: smith's
  `smith-charter`, `smith-channel` and `smith-transcript`, and the
  client wires, are such crates.
- **A large family has a codec crate of its own:** a family whose
  generated code passes 20,000 lines, the user's figure, shares its
  crate with no other family (`tools/crates/design.md`, 3.1), so that a
  change to one family recompiles it alone and the families compile side
  by side. An owner's smaller families may share one codec crate.
- **No package holds generated record code beside other workspace
  dependencies,** and the codec check keeps it so, with no exception
  (`tools/crates/design.md`, 3.2). skein's two owners, `skein-oauth` and
  `skein-ui-example-view`, keep theirs in `skein-oauth-codec` and
  `skein-ui-example-view-codec`.

## 3. The schema

```
family smith-charter 1

record Section {
    title: text 256
    body: text 65536
}

enum Effect { read, write }

record HostTool {
    name: text 64
    description: text 4096
    input: bytes 65536
    effect: Effect
    deadline: duration
}

record Report {
    fields: list 32 Field
}

enum Form {
    report: Report
    failure
}
```

| Type | Encoded as | In Rust |
|---|---|---|
| `u8`, `u16`, `u32`, `u64` | fixed width, big-endian | the same |
| `bool` | a byte, 0 or 1 | `bool` |
| `duration` | a `u64` of nanoseconds | `lib::Duration` |
| `fixed N` | N bytes | `[u8; N]` |
| `bytes N` | a `u32` length, then at most N bytes | `Box<[u8]>` |
| `secret N` | as `bytes N` | `lib::Secret` |
| `text N` | a `u32` length, then at most N bytes of UTF-8 | `Box<[u8]>`, checked UTF-8 |
| `list N T` | a `u32` count, then at most N items | `lib::List<T>` |
| `option T` | a byte, 0 for none or 1 for some, then the value | `Option<T>` |
| a record | its fields, in order | a struct |
| an enumeration | a byte tag, the variants numbered in order from 0, then the variant's record, if it has one | an enum; a `boxed` variant's record in a `Box` |

- **Durations, never times.** A time means something only on its writer's
  clock. A peer turns a duration into a deadline on its own clock
  (../../../foundation/programming-model.md, section 9).
- **Every field is present.** An absent value is an explicit `option`.
  A value's bytes hold every field of its layout, whatever its default.
- **A default is the value another layout's records take** for a field
  their layout lacks (`layouts.md`, 2), written after the type:
  `infrastructure: u32 = 0`. An `option` without one defaults to none.
  Its forms, each checked against
  the field's type and encoded once, as the text is read (`run-time.md`, 1.3): a
  decimal word for an integer or a duration's nanoseconds, `true` or
  `false`, `0x` and lower-case hex for `fixed` and `bytes`, a quoted
  text, `[]`, `none` or `some` and a value, a variant without a record
  by its name.
  A record, a variant with one, a `secret` and a `bytes` that holds a
  record take none: a field of those an older layout lacks is a written
  change (`layouts.md`, 2). A default changes neither the bytes, the
  generated code, the golden bytes nor any fingerprint (`fingerprints.md`, section 1): it
  speaks only to a layout without its field.
- **Former names:** a declaration, a field or a variant may name what it
  was called, `was` and one or more names after its own:
  `record Task was Job {`, `assignee was owner: u32`, `paused was
  suspended`. A plan between two layouts matches parts by them
  (`layouts.md`, 2), so a text renames a part
  and changes another in the same landing with nothing beside it. A
  former name is none of its scope's current names (the declarations',
  a record's fields', an enumeration's variants') and none of its other
  former names; it stays as long as a store may hold a layout that
  spells it. Like names, former names are in no fingerprint.
- **A tombstone keeps a dropped field in its record's text:** `dropped
  hint: text 256 = ""`, the word `dropped`, then the field as it was,
  its former names among it, and its default, which it must have. It
  takes no bytes, no generated code, no golden byte and no fingerprint:
  it lets a plan toward a layout that still has the field write a value
  there, in either mode (`layouts.md`, 2). Its type is one that takes a
  default; its name and former names are none of its record's fields'
  names or former names, nor another tombstone's. It stays until fabrig
  keeps no code for a layout that has the field
  (`temper/docs/design/draft/archive/layouts-both-ways.md`, 7.4).
- **`droppable`** before a field's name, `droppable hint: text 256`,
  lets a plan in Keep mode from this layout drop the field whatever its
  value, where the target lacks it (`layouts.md`, 2). Nothing else reads
  it: it moves no byte, no generated code and no fingerprint.
- **Both words are read from this grammar on.** A build before it
  refuses a text that uses either, so a family's schema uses them once
  dev.fabrig.ai runs a build that reads them: its store's registry, and
  a peer it opens with, may hand that text to it
  (`temper/docs/design/draft/archive/layouts-both-ways.md`, 7.4). A field may
  still be named `dropped` or `droppable` (`run-time.md`, 1.1).
- **A variant holds one record, or nothing.** The record seals its own
  bounds, which an enum variant's fields could not.
- **A variant marked `boxed`** (`report: boxed Report`) holds its record
  in a `Box`, and its bytes are the record's as they would be. Its owner
  marks a variant whose record is much larger than its siblings', where
  the workspace's lints report it (`golden.md`, section 1): only the compiler knows a
  type's size (`README.md`, section 4), so the generator never guesses one, and every
  other variant's record stays inline, costing no allocation. Like
  `secret`, the word changes the Rust type and not the bytes, so marking
  a variant or unmarking it moves no golden bytes and no version
  (`README.md`, section 5).
- **A sum crosses as one piece.** A schema bounds each field and each
  count, so a list's worst case is its count times its item's, and lists
  of items that hold lists multiply. Where an owner bounds a collection
  by its bytes in all, not by its count times its largest item, the
  collection is one `bytes N` field, N the owner's sum: its items each
  after a `u32` length, written and split with `skein-codec`'s piece
  helpers, each item decoded by its own type's code as the reader splits
  the piece. A piece whose lengths do not end exactly at its end is
  refused as malformed. Its worst case is its bound, and its items'
  decoded size is the owner's to count against the same sum. An owner
  bounding the items by that sum splits with `piece::split_within`,
  which charges each item and its length and refuses past the owner's
  most; one reserving by count counts first with `piece::count` and takes
  each item with `piece::counted`.
- **A type uses only types declared before it.** The types form a tree, and
  decoding nests to a fixed depth.
- **No imports.** A record of another family, such as a charter carried in
  a channel's body, travels as a `bytes N` field and is decoded by its own
  family's code.
- **A `bytes` field says what it holds,** when it holds a record or
  pieces: `refusal: bytes 517 holds jig-store-people Refusal` holds a
  record of another family, named by its family and type;
  `segments: bytes 65536 pieces Segment` holds pieces (above) whose items
  are a type of this schema, declared before. The words change neither
  the bytes nor the Rust: the generated code reads the field as any
  `bytes N`, and its owner decodes what it holds as before. They let a
  layout plan convert it with its holder (`layouts.md`, 2). A field
  of a stored family or a wire whose bytes are a record or pieces says
  so. A `secret N` may say so too, `secret N holds skein-oauth SavedToken`,
  and stays a secret: never
  printed, its bytes converted by a plan as a held field's are.
- **A record marked `versioned`** starts with its family's version, a
  `u16`. A reader of bytes kept elsewhere learns from it which code reads
  them.
- **A credential is a `secret N`,** never `bytes N`: a token, a key, a
  secret, a grant's value. Its bytes cross as `bytes N` would; its type,
  lib's `Secret` (../../runtime/lib-bytes.md, section 1), shows only its length in `Debug`, so
  a record that holds one is printed, logged or traced without it. Its
  owner reads the bytes with `expose`, to use or send them.

## 4. What is generated

For each family:

- **Its limits.** A plain struct with one limit per bound in the schema,
  its `u32` fields in schema order under `#[repr(C)]`, so that a bound's
  table names a limit by its field (below), and a constant holding the
  schema's bounds as ceilings. A side's limits
  may be smaller than the ceilings, never larger (../../../foundation/programming-model.md,
  section 7). One generated function checks a family's limits against
  its ceilings, naming the first past one; the family's owner calls it
  once, where it makes its limits. Encoding and decoding check values
  only, never the limits: each value against its limit no higher than
  its ceiling, so no value past a ceiling becomes bytes or decodes,
  whatever limits they are given, and a call costs what its value holds,
  not the family's size. `Limits::at_most(&self, most: u32) -> Limits`
  lowers each limit to at most `most`, so that a test takes a family's
  limits at one clamp without naming them (`golden.md`, section 1).
- **Its version.** `pub const VERSION: u16`, the schema's version, which
  its versioned records' encoding and decoding name; a hand-written
  constant that restates it is defined by it.
- **Its layout.** `pub const LAYOUT: [u8; 32]`, the family's fingerprint
  (`fingerprints.md`, section 1) of the text it was generated from, so that
  a build names its own layout of the family without parsing its text,
  and a peer or a kept record is judged by it (jig's
  `protocol/client/versions.md`, section 1). A hand-written constant that
  restates it is defined by it.
- **Its limits by name.** `LIMIT_NAMES: &[&str]`, each limit's field
  name in the struct's order, and `Limits::values(&self)`, their values
  in the same order, read as a `&[u32]`, so that the bounds of a text
  read at run time, another layout's among them, can be
  computed with these limits (`run-time.md`, 1.3, `most_bytes_within`). A
  limit's name is the rule's `schema::limit_name` gives (`run-time.md`, 1.3), the one
  the generator names the field by.
- **Its paths, by name.** A path is a part of a declaration (a
  record's field, its version or its trailing bytes, an enumeration's
  tag), held as its name as the schema spells it: `Record.field`,
  `Record (version)`, `Record (tail)` and `Enum (tag)`. The module's
  `paths` names each one a constant, its schema name in capitals
  (`paths::SPEC_WORDS`, `paths::ROW_VERSION`, `paths::ROW_TAIL`,
  `paths::PART_KIND_TAG`), and `Path::name` reads the name back, so a
  step can name a refusal's field in its words without formatting
  (`../../../foundation/programming-model.md`, 3). Two paths are equal
  when their names are; no path holds a position. A problem is a path
  with `skein-codec`'s reason, both typed by the family's `Limits`
  (`pub type Problem = skein_codec::Problem<Limits>`, and `Path` so);
  the module re-exports `Record`. A `?` carrying one family's refusal
  into another's result does not compile, so no refusal is named by
  the wrong family's table.

For each record and enumeration:

- **Its type, with public fields,** each the Rust type of its kind: no
  parts twin, constructor, accessor or `into_parts`. A value is built and
  taken apart with the type's own syntax, so bytes move and are not
  copied. The derives are `Debug`, `Clone`, equality and `Hash`; a
  `secret` field's `Debug` is its length (section 3). A value past its
  limits may exist in memory; it can't become bytes.
- **`Record`,** its codec as one helper call per field, in schema order,
  for each of checking, measuring, encoding and decoding.
- **Checked where bytes are made:** measuring checks each field against
  the limits it is given, no higher than its ceiling, in schema order,
  depth first: a length or a list's count past its limit refuses with
  `Bound`, text that is not UTF-8 with `Utf8`, naming the field. What it
  passes it measures, which cannot overflow, since the ceilings bound it.
- **Encoding:** a box of the measured length, or the rest of a frame's
  writer (../../net/channel.md, section 7) after measuring.
- **Decoding** from a `lib::Reader`, against the limits it is given. It
  refuses with a problem naming the field:
  - too short;
  - beyond a limit;
  - an unknown tag;
  - a `bool` that is neither 0 nor 1;
  - text that is not UTF-8;
  - another version;
  - bytes left over at the end.

  Every box is allocated at its final length, once that length has passed
  its limit and what is left.
- **Its bounds,** for given limits:
  - the most bytes it encodes to, which must fit a `u32` at the ceilings or
    the schema is refused;
  - the most heap its decoded value holds, as a function over the limits
    and the sizes of the generated types, which only the compiler knows,
    a `boxed` variant's record counting its own size.

  Each is `None` for limits past a ceiling. Bounds are data: the module
  holds one shape per declaration, `DECLARATIONS`, public, in the order
  of their names: each a `skein_codec::bounds::Declaration`, the
  declaration's name and its nodes in postorder, its own root last, with
  the size of each type a list or a `boxed` variant holds. A node's
  edges count within its declaration; a nested declaration is a node
  naming it, and a limit is named by its field's offset in `Limits`
  (`core::mem::offset_of!`), which the compiler places. One walk in
  `skein-codec` computes every bound from the shapes and the limits'
  values, finding a named declaration by its name, with checked
  arithmetic, no recursion and memory bounded by the shapes. It checks
  the family's limits once, where it is asked: a bound costs the types
  it reaches, not their number times the family's limits. The family's
  bounds are the most of its declarations'.
  The generator also computes each record's encoded bound at zero limits
  from the same shapes and emits it as `ENCODED_AT_ZERO`.
- **Stable under insertion.** No generated line spells a position in
  its family: not a declaration's, a part's, a limit's, a node's outside
  its own declaration, nor a maps file's line (`maps.md`, 4.1). Each
  names what it means, and the compiler or the walk finds it. So
  inserting a declaration adds its own lines and changes only those of
  the declarations that name it, and a family-wide list gains one entry
  a line. When paths and tables were positions, one new record
  renumbered every later declaration's code, 1,718 lines once, and two
  branches that each added one conflicted everywhere after it.

```rust
// skein_codec; Path and Problem are Clone, Copy, Debug, equality and
// Hash whatever L is: L, the family's Limits, is a marker, no bytes
pub struct Path<L> { /* its name; L */ }
pub struct Problem<L> { pub path: Path<L>, pub reason: Reason }
impl<L> Path<L> {
    pub const fn new(name: &'static str) -> Self;   // as the schema spells it
    pub const fn name(self) -> &'static str;
    pub const fn problem(self, reason: Reason) -> Problem<L>;
}
// skein_codec::bounds: one declaration's shape, as DECLARATIONS holds it
pub struct Declaration { pub name: &'static str, pub nodes: &'static [Node] }
pub trait Record: Sized {
    type Limits;                       // the family's
    fn measure(&self, limits: &Self::Limits) -> Result<u32, Problem<Self::Limits>>;
    fn encode(&self, limits: &Self::Limits) -> Result<Box<[u8]>, Problem<Self::Limits>>;
    fn decode(limits: &Self::Limits, reader: &mut Reader<'_>)
        -> Result<Self, Problem<Self::Limits>>;
    fn encoded_bound(limits: &Self::Limits) -> Option<u64>;
    fn decoded_bound(limits: &Self::Limits) -> Option<u64>;
    // and what a nested value and a frame's writer call, after measuring
}
```

`decode` reads the whole of its reader, refusing bytes left over.
`Record` is implemented by generated code alone: hand-written code
calls it, and makes bytes with `encode`, or with `measure` before a
frame's writer. The helpers' own trait, `Field`, one implementation per
kind of field, is sealed by a private supertrait, so no crate but
`skein-codec` implements it. A boundary test refuses an implementation
of `Record` or `Field` in any file but `skein-codec`'s and a generated
one.

`generate(schema)` writes one schema version's records' module, and
nothing else: no tests (`golden.md`, section 1). The module holds no bound
arithmetic and no split function: a function of one line per field or
limit too long for the lint carries `golden.md`, section 1's expectation. What tests
take from the generator, its fixed-rule values and the constructs a
text uses, is `golden.md`, section 1's.

- **Its file:** `src/generated/<stem>.rs`, in the crate that holds the
  family's records. `<stem>` is the family's current schema file's name
  less its `-v<N>.schema`, each `-` an `_`: `tasks-v13.schema` makes
  `tasks.rs`, `worker-wire-v17.schema` `worker_wire.rs`. A split schema
  (`client-v34.schema` with `client-v34-part1.schema`) makes one module,
  named for its first part. Two families of one crate never share a
  stem. The crate includes the file under the Rust name its code
  already uses (`#[path = "generated/tasks.rs"] pub mod tasks_v13;`).
  skein's codec fixtures outside `src/generated/` (`skein/tests/codec/src/every/`,
  `lints/`, `scalars/`, `secrets/`) hold several versions of one family
  side by side, and keep their versions in their names (`command.md`, 1).
- **The command** (`command.md`, 8) writes this module to
  `<out-dir>/<stem>.rs`, the first schema's stem. Its module is the one
  the family's drift test writes; the merge job runs it
  (`temper/docs/design/protocol/tooling.md`, 4.1).

```rust
pub struct Output {
    pub records: String,      // the records' module: src/generated/<stem>.rs
}
pub fn generate(schema: &Schema) -> Output;
```

`Output` derives `Clone`, `Debug` and equality.

## 5. Versions

- **Every family's version is frozen** at its header's: a change of
  layout is made in the family's current text alone and moves no
  version, whether it is stored, sent or both (temper's
  `draft/archive/layouts-both-ways.md`, 3). The store's registered families
  (jig's `protocol/store/layouts.md`, 1), the worker wires (temper's
  `protocol/wires.md`, 3.1) and the client wires (jig's
  `protocol/client/versions.md`, 1) each say their numbers. So two
  goals that change one family's layout never collide on its version.
- **A layout is its fingerprint** (`fingerprints.md`, section 1): a
  family's names a build's layout of it, and a value type's a value's
  (`fingerprints.md`, 1.4). Two builds whose fingerprints are equal lay
  the family out alike, and nothing converts between them.
- **A reader reads one layout,** its current text's, with its generated
  code; a versioned record's head holds the header's version, which the
  reader reads first, with `skein-codec`, refusing another with the
  problem *another version*. A writer writes its current layout only.
  A value at another layout reaches generated code only once a plan has
  converted it (`layouts.md`).
- **The other layout's text travels with the data or with the peer,**
  never in the repository:
  - **a store's registry** holds the text of every layout a value is
    at, its transcripts' frames among them, and converts each value as
    it is read; its generated code reads and writes the version at a
    value's head, which the store maps to and from the value's layout id
    (jig's `protocol/store/layouts.md`);
  - **a worker** sends its wires' texts at its opening, and the engine
    converts each message both ways (jig's `protocol/channel/layouts.md`);
  - **a client** whose layouts differ from the engine's is refused by
    name at its hello and reloads; nothing converts for it (jig's
    `protocol/client/versions.md`, 1).

  A crate keeps its families' current texts alone (`command.md`, 1).
  The one older text kept is `smith-transcript`'s version 7, the
  store's seed for frames written before its registry, read by the
  store alone (temper's `protocol/wires.md`, 3.1).
- **What no plan derives** is its owner's: a store's written job (jig's
  `protocol/store/jobs.md`, 4), or a wire's written converter (jig's
  `protocol/channel/layouts.md`, 6). A dropped field keeps a tombstone
  (section 3). Each is kept only while something dev.fabrig.ai runs is
  at the layout that needs it (temper's `draft/archive/layouts-both-ways.md`,
  7.4).
- **Formats outside the registry are fixed:** the component's own rows
  and the secrets' store. A change designs and brings, in its own
  landing, how the deployed store gets there (temper's
  `draft/archive/store-floor.md`, section 5). A store below its floor is
  refused (jig's `protocol/store/opening.md`, 1.7 and 1.8).
- **On a channel,** the framing's version is agreed at its opening
  (../../net/channel.md, section 5.2), and the bodies carry no version of
  their own: their layouts are their owner's to judge and convert, as
  jig's engine does with a worker's (jig's `protocol/channel/wires.md`,
  2 and 4).
- **No family keeps golden files** but the `every` fixture (`golden.md`,
  section 1). A test of another layout builds that layout's values from
  its text.

## 10. Open questions

- **A piece's whole loop:** `piece::split_within` (3) holds the split, the
  charge and the bound; each owner loops and decodes each item. Whether a
  helper over `Record` should own that loop and report both framing failures
  and the item's `Problem` remains open.
- **A held type renamed in its own family** has no plan in its holder:
  the holder's `holds` clause is compared as it is spelled
  (`layouts.md`, 2), not through the held family's former names. A plan
  could match it through the held texts it is given.
- **Maps and sets:** sorted lists whose keys are checked unique, when a
  record needs one.
- **Smaller integers:** variable-width encodings, if a payload shows that
  fixed widths cost too much.
- **JSON from schemas:** the same generator writing JSON codecs over
  `skein-json` (../json.md, section 8).
- **Other languages:** a generator for a peer not built on skein, if one
  appears. The schema is plain enough for it.
