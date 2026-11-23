# Codecs from schemas

Provisional, 2026-10-06, revised 2026-10-26. How an application protocol
built on skein encodes its records, and how its kept records follow its
schemas from one version to the next. A schema says each record's fields
and their bounds. skein's generator turns it into plain Rust codecs,
limits and bounds, and `skein-codec` holds the little
the generated code shares, and reads schemas and their records at run
time. Codecs carry the bodies of a framed channel (../../net/channel.md), and
records kept in files or stores.

- [`golden.md`](golden.md): golden bytes and tests.
- [`run-time.md`](run-time.md): run-time schemas and generic values.
- [`fingerprints.md`](fingerprints.md): family and value-type fingerprints.

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
- **Versioned.** A schema is one version of a family, and a family has
  one current version. Generated code reads the current version and
  refuses another. Wires keep texts and patches for translation between
  versions (`patches.md`, `bridges.md`). A registered family's version
  stays where it is: its store names each value's layout and converts
  it at read through the texts its registry holds (section 5,
  `layouts.md`). Formats outside the registry are fixed at their
  versions; a change to one brings its own route for the deployed store.
- **Defaults and former names in the schema** (section 3): a field may
  give the value an older layout without it takes, and a declaration,
  field or variant the names it had, so that a layout converts to the
  next with no text beside the schema.
- **Read at run time too.** `skein-codec` reads a schema's text as step
  code (`run-time.md`, section 1), and decodes the bytes of any of its records into a
  generic value, an index over those bytes, with the generated code's
  checks (`run-time.md`, section 2). The generator reads schemas through the same
  parser, so there is one language.
- **Upgrades by patches, or by plans at read.** A layout's fingerprint
  (`fingerprints.md`, section 1), a diff of two versions into the kinds of change
  (`diff.md`), patches over generic values, run at run time without
  generated code for old versions (`patches.md`), and the command that
  writes them, whose check a gate calls (`command.md`); for a registered
  family, a value type's layout fingerprint (`fingerprints.md`, 1.4) and a derived plan from
  one layout to another, a pass over a value's bytes run as it is read
  (`layouts.md`).

## 2. In skein

- **`skein-codec`** depends on lib only, and builds for
  `wasm32-unknown-unknown` as lib does (../../runtime/lib.md, section 2), with the code
  generated for it. It holds what the generated code shares:
  - `Record`, the trait each generated declaration implements, and the
    generic helpers that check, measure, encode and decode each kind of
    field (section 4);
  - the walk that computes bounds from a module's table (section 4);
  - the reasons a decoding fails, and the problem naming its path;
  - bounded reads of lengths, counts and text;
  - reading a versioned record's version;
  - writing a piece of items and splitting one (section 3).

  And, as step code, what reads schemas and records at run time:
  - `schema`: the parser and the layout checks (`run-time.md`, section 1);
  - `value`: generic values (`run-time.md`, section 2);
  - `patch`: patches and the machine that runs them (`patches.md`);
  - `layout`: derived plans from one layout of a value type to another,
    and their pass over a value's bytes (`layouts.md`);
  - `bridge`: a wire's reverses, its floor, and the bridge that
    translates a record between the versions it speaks (`bridges.md`).
- **`skein-codegen`** is the generator, and the tooling of upgrades: the
  diff (`diff.md`), the command (`command.md`) and the translation from
  maps files (`maps.md`). Ordinary Rust, not
  step code. It reads schemas through `skein-codec`'s parser and adds
  the checks only Rust needs (`run-time.md`, section 1). A consumer depends on it in its
  tests and tools only.
- **A consumer's codec crate** holds a family's generated record code,
  `src/generated/<stem>.rs` (section 4), and depends
  on lib and `skein-codec` alone; its dev-dependencies are free,
  `skein-codegen` among them. The family's **owner**, the crate that
  keeps it, holds the rest: its schemas, kept texts and patches where
  the family uses them (`command.md`, 1), its generated glue, its drift
  test (`golden.md`, section 1), and the translation between its records and a
  domain, with the domain's crates as its dependencies. So the generated
  code compiles beside the domains rather than after them, and only a
  change of schema recompiles it (temper's `draft/crate-graph.md`, 4.1).
  An owner's patches may name its domain's types, so they stay with it.
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
- **A default is the value an older layout's records take** for a field
  their layout lacks (`layouts.md`, 2), written after the type:
  `infrastructure: u32 = 0`. An `option` without one defaults to none.
  Its forms are a patch's values (`patches.md`, 2), each checked against
  the field's type and encoded once, as the text is read (`run-time.md`, 1.3): a
  decimal word for an integer or a duration's nanoseconds, `true` or
  `false`, `0x` and hex for `fixed` and `bytes`, a quoted text, `[]`,
  `none` or `some` and a value, a variant without a record by its name.
  A record, a variant with one, a `secret` and a `bytes` that holds a
  record take none: a field of those an older layout lacks is a written
  change (`layouts.md`, 2). A default changes neither the bytes, the
  generated code, the golden bytes nor any fingerprint (`fingerprints.md`, section 1): it
  speaks only to a layout without its field.
- **Former names:** a declaration, a field or a variant may name what it
  was called, `was` and one or more names after its own:
  `record Task was Job {`, `assignee was owner: u32`, `paused was
  suspended`. A plan between two layouts takes them as renames
  confirmed (`layouts.md`, 2; `diff.md`, 1), so a text renames a part
  and changes another in the same landing with nothing beside it. A
  former name is none of its scope's current names (the declarations',
  a record's fields', an enumeration's variants') and none of its other
  former names; it stays as long as a store may hold a layout that
  spells it. Like names, former names are in no fingerprint.
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
  patch of the held type reach the field wherever it is kept (`patches.md`),
  and a layout plan convert it with its holder (`layouts.md`, 2). A field
  of a stored family whose bytes are a record or pieces says so. A
  `secret N` may say so too, `secret N holds skein-oauth SavedToken`,
  and stays a secret: never
  printed, its bytes reaching a patch as a held field's do.
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
  and a constant holding the schema's bounds as ceilings. A side's limits
  may be smaller than the ceilings, never larger (../../../foundation/programming-model.md,
  section 7). One generated function checks a family's limits against
  its ceilings, naming the first past one; the family's owner calls it
  once, where it makes its limits. Encoding and decoding check values
  only, never the limits: each value against its limit no higher than
  its ceiling, so no value past a ceiling becomes bytes or decodes,
  whatever limits they are given, and a call costs what its value holds,
  not the family's size.
- **Its version.** `pub const VERSION: u16`, the schema's version, which
  its versioned records' encoding and decoding name; a hand-written
  constant that restates it is defined by it.
- **Its limits by name.** `LIMIT_NAMES: &[&str]`, each limit's field
  name in the struct's order, and `Limits::values(&self)`, their values
  in the same order, read as a `&[u32]`, so that a kept version's bounds can be
  computed at run time with these limits (`run-time.md`, 1.3, `most_bytes_within`). A
  limit's name is the rule's `schema::limit_name` gives (`run-time.md`, 1.3), the one
  the generator names the field by.
- **Its paths, as data.** A path is a declaration and a part of it, each
  by its position in the schema: a record's fields, its version and its
  trailing bytes, an enumeration's tag. The module's `paths` names each
  one a constant, its schema name in capitals (`paths::SPEC_WORDS`,
  `paths::ROW_VERSION`, `paths::ROW_TAIL`, `paths::PART_KIND_TAG`), and
  `name(path)` reads its name from one table, as the schema spells it:
  `Record.field`, `Record (version)`, `Record (tail)` and `Enum (tag)`,
  so a step can name a refusal's field in its words without formatting
  (`../../../foundation/programming-model.md`, 3). A problem is a path
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
  holds one table of its declarations, their shapes and fields, in
  schema order, one entry a line, with the size of each type a list or a
  `boxed` variant holds, and one walk in `skein-codec` computes every
  bound from it and the limits' values, with checked arithmetic, no
  recursion and memory bounded by the table. It checks the family's
  limits once, where it is asked: a bound costs the types it reaches,
  not their number times the family's limits. The family's bounds are
  the most of its declarations'.
  The generator also computes each record's encoded bound at zero limits
  from the same table and emits it as `ENCODED_AT_ZERO`.

```rust
// skein_codec; Path and Problem are Clone, Copy, Debug, equality and
// Hash whatever L is: L, the family's Limits, is a marker, no bytes
pub struct Path<L> { pub declaration: u16, pub part: u16, /* L */ }
pub struct Problem<L> { pub path: Path<L>, pub reason: Reason }
impl<L> Path<L> {
    pub const fn new(declaration: u16, part: u16) -> Self;
    pub const fn problem(self, reason: Reason) -> Problem<L>;
}
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
- **The command:** `skein-codegen <schema>... <out-dir>` reads the
  schemas' texts, concatenated in the order given with nothing between
  them, as one schema, and writes its records' module to
  `<out-dir>/<stem>.rs`, the first schema's stem, making `<out-dir>`
  where absent, and nothing else. A schema that does not read or parse
  exits nonzero, saying why on stderr, and writes nothing. Its module
  is the one the family's drift test writes; the merge job runs it
  (`temper/docs/design/protocol/tooling.md`, 4.1). `patch` as the first
  argument is the patch command (`command.md`, 8).

```rust
pub struct Output {
    pub records: String,      // the records' module: src/generated/<stem>.rs
}
pub fn generate(schema: &Schema) -> Output;
```

`Output` derives `Clone`, `Debug` and equality.

## 5. Versions

- **Three rules:** wires keep versioned texts and patches; registered
  store families keep their version and convert by layout; formats
  outside the registry are fixed at their versions. A store below its
  floor is refused, not walked through patches (jig's
  `protocol/store/opening.md`, 1.7 and 1.8).
- **A patched family's baseline is its owner's first kept text,** made
  by `--baseline` (`command.md`, 1). Before it, no version moves and
  nothing is translated. The command knows the crate's files, not a
  deployment or a release.
- **A wire's family moves with its layout.** A family that crosses
  between two builds of its owner's (a channel's bodies, what one side
  writes and the other reads) is marked a wire in its crate
  (`command.md`, 1). From its baseline, its owner's first `--baseline`
  run, its version moves whenever its layout does, an old value
  surviving or not, so that two builds that name one version lay it out
  alike and a peer can be judged by its version and its fingerprint
  (`fingerprints.md`, section 1) before anything else crosses (`command.md`, 2, the wire
  rule).
- **A patch moves its family's version:** a new schema, whose generated
  code replaces the old (`command.md`, 2). There is one generated
  version at a time. This is not the rule for registered families.
- **A reader reads one version.** It reads a versioned record's version
  first, with `skein-codec`, and refuses another with the problem
  *another version*. Its generated code reads the current version only.
- **A patched family keeps its versions, from the baseline on.** Its
  owner keeps each version's schema as text, and a kept record
  of an older version is read as a generic value (`run-time.md`, section 2) and patched
  to the current version (`patches.md`) before the generated code reads
  it. A version then names one layout. Until the baseline, nothing is
  translated. What a channel's sides speak across two builds is agreed
  at its opening (../../net/channel.md, 5.2), each side judging the
  other's wires by their versions and fingerprints; translating between
  them is its owner's.
- **A registered family's version stays.** A stored family its owner
  marks registered (`command.md`, 1) is kept by a store that names each
  value's layout and converts it as it is read (jig's
  `protocol/store/layouts.md`): its crate keeps no texts or patches for
  older layouts. A change of its layout is a new layout of each value
  type it reaches (`fingerprints.md`, 1.4), derived by a plan (`layouts.md`) or converted
  by its owner's written job. The registry holds the layouts and texts
  values may still use. Its generated code reads and writes its version
  at a value's head, as any family's; the store maps that head to and
  from the value's layout id.
- **Formats outside the registry are fixed:** the component's own rows,
  the secrets' store and the transcripts' frames. A change designs and
  brings, in its own landing, how the deployed store gets there
  (temper's `draft/store-floor.md`, section 5). The frame patcher stays:
  a frame behind the binary's versions is patched as it is read (jig's
  `protocol/store/transcripts.md`, 6).
- **A writer writes its current version only.**
- **On a channel,** the agreed version picks the code for its bodies
  (../../net/channel.md, section 5.2). Bodies carry no version of their own.
- **No family keeps golden files** but the `every` fixture (`golden.md`, section 1).
  A patched family's kept versions are represented by its texts and
  patch goldens (`command.md`, 3), and a test of an older version builds
  that version's values from its kept text.

## 6. Golden bytes and tests

Moved to [`golden.md`](golden.md), section 1.

## 7. Schemas at run time

Moved to [`run-time.md`](run-time.md), section 1.

## 8. Generic values

Moved to [`run-time.md`](run-time.md), section 2.

## 9. Fingerprints

Moved to [`fingerprints.md`](fingerprints.md), section 1.

## 10. Open questions

- **Shrinking:** a patcher keeps the largest allocation it reached, so
  one large record keeps it at that size until the patcher goes.

- **Upstreams' levels:** the command writes a patch's `after` as its
  author gives it (`command.md`, 3); it does not discover upstreams'
  levels for the author.
- **Proposals to a fixpoint:** the command runs the diff in rounds to
  propose renames a level of references down (`command.md`, 2); the diff itself
  could take candidate pairs as matched until none fails, and propose
  them all in one pass.
- **A rename to its own name:** a plan refuses `rename D D` (`Mismatch`,
  `patches.md`, 3), and the diff takes it, pairing the two and writing no line
  (`diff.md`, 1). Default: each stays as it is, since the diff never spells that
  line.
- **A piece's whole loop:** `piece::split_within` (3) holds the split, the
  charge and the bound; each owner loops and decodes each item. Whether a
  helper over `Record` should own that loop and report both framing failures
  and the item's `Problem` remains open.
- **Pieces whose items stop:** pieces whose items' conversion reaches a
  site or a held field, or an identity plan whose pieces reach a held
  field, are refused when planned (`patches.md`, 3); pieces whose items
  convert without stopping are opened by the walk (`patches.md`, 4).
  Stopping inside a piece would lift it: the stop's `at` a node of the
  decoded piece, the piece's length written once its value is whole.
- **Carried parts that stop:** a written function's `carry` converts in
  place, so a part with a site or a held field in reach is refused
  (`patches.md`, 5). Drafts would lift it behind the same calls: the function
  writes into a draft, a stack of them in one arena, and a carried
  part is a slot the machine fills as it emits the draft, stopping at
  each site and held field in its turn.
- **Richer builders:** carrying a record's fields up to one, or to its
  end, should written functions that build records by hand be common.
- **Richer values:** a record's value, or a list's with items, in a
  patch's text, should authors write the same function often (`patches.md`, 2).
- **A held type renamed in its own family** is a `retype` in its
  holder's diff, written: the holder's diff reads its own family's two
  texts alone, so it cannot see the held family's renames. A caller
  could give the diff those renames.
- **Composing patches:** a record k versions behind is patched in k
  passes; plans may compose later, behind the same interface, and so
  remove the box each pass after the first allocates (`patches.md`, 7).
- **Maps and sets:** sorted lists whose keys are checked unique, when a
  record needs one.
- **Smaller integers:** variable-width encodings, if a payload shows that
  fixed widths cost too much.
- **JSON from schemas:** the same generator writing JSON codecs over
  `skein-json` (../json.md, section 8).
- **Other languages:** a generator for a peer not built on skein, if one
  appears. The schema is plain enough for it.
