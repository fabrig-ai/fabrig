# Codecs from schemas

Provisional, 2026-10-06, revised 2026-10-26. How an application protocol
built on skein encodes its records, and how its kept records follow its
schemas from one version to the next. A schema says each record's fields
and their bounds. skein's generator turns it into plain Rust codecs,
limits and bounds, and `skein-codec` holds the little
the generated code shares, and reads schemas and their records at run
time. Codecs carry the bodies of a framed channel (../../net/channel.md), and
records kept in files or stores.

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
  - for tests, each record's fixed-rule values (section 6).
- **The subset's Rust, calling generic helpers** (../../../foundation/programming-model.md,
  section 10): each declaration implements `skein-codec`'s `Record`,
  one helper call per field. The helpers are `skein-codec`'s, under
  lib's exception (10.2), written once and tested hard; the generated
  code defines no generic item, closure or macro. The output is kept in
  the consumer's repository. A test fails when the code and the schema
  drift.
- **One pin on the bytes** (section 6). The `every` fixture uses every
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
  code (section 7), and decodes the bytes of any of its records into a
  generic value, an index over those bytes, with the generated code's
  checks (section 8). The generator reads schemas through the same
  parser, so there is one language.
- **Upgrades by patches, or by plans at read.** A layout's fingerprint
  (section 9), a diff of two versions into the kinds of change
  (`diff.md`), patches over generic values, run at run time without
  generated code for old versions (`patches.md`), and the command that
  writes them, whose check a gate calls (`command.md`); for a registered
  family, a value type's layout fingerprint (9.4) and a derived plan from
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
  - `schema`: the parser and the layout checks (section 7);
  - `value`: generic values (section 8);
  - `patch`: patches and the machine that runs them (`patches.md`);
  - `layout`: derived plans from one layout of a value type to another,
    and their pass over a value's bytes (`layouts.md`);
  - `bridge`: a wire's reverses, its floor, and the bridge that
    translates a record between the versions it speaks (`bridges.md`).
- **`skein-codegen`** is the generator, and the tooling of upgrades: the
  diff (`diff.md`) and the command (`command.md`). Ordinary Rust, not
  step code. It reads schemas through `skein-codec`'s parser and adds
  the checks only Rust needs (`README.md`, section 7). A consumer depends on it in its
  tests and tools only.
- **A consumer's codec crate** holds a family's generated record code,
  `src/generated/<stem>.rs` (section 4), and depends
  on lib and `skein-codec` alone; its dev-dependencies are free,
  `skein-codegen` among them. The family's **owner**, the crate that
  keeps it, holds the rest: its schemas, kept texts and patches where
  the family uses them (`command.md`, 1), its generated glue, its drift
  test (section 6), and the translation between its records and a
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
  the field's type and encoded once, as the text is read (7.3): a
  decimal word for an integer or a duration's nanoseconds, `true` or
  `false`, `0x` and hex for `fixed` and `bytes`, a quoted text, `[]`,
  `none` or `some` and a value, a variant without a record by its name.
  A record, a variant with one, a `secret` and a `bytes` that holds a
  record take none: a field of those an older layout lacks is a written
  change (`layouts.md`, 2). A default changes neither the bytes, the
  generated code, the golden bytes nor any fingerprint (section 9): it
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
  the workspace's lints report it (section 6): only the compiler knows a
  type's size (section 4), so the generator never guesses one, and every
  other variant's record stays inline, costing no allocation. Like
  `secret`, the word changes the Rust type and not the bytes, so marking
  a variant or unmarking it moves no golden bytes and no version
  (section 5).
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
  lib's `Secret` (../../runtime/lib.md, section 6), shows only its length in `Debug`, so
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
  computed at run time with these limits (7.3, `most_bytes_within`). A
  limit's name is the rule's `schema::limit_name` gives (7.3), the one
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
  with `skein-codec`'s reason; the module re-exports `skein-codec`'s
  `Path`, `Problem` and `Record`.

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

```rust
// skein_codec; Path and Problem derive Clone, Copy, Debug, equality, Hash
pub struct Path { pub declaration: u16, pub part: u16 }
pub struct Problem { pub path: Path, pub reason: Reason }
pub trait Record: Sized {
    type Limits;                       // the family's
    fn measure(&self, limits: &Self::Limits) -> Result<u32, Problem>;
    fn encode(&self, limits: &Self::Limits) -> Result<Box<[u8]>, Problem>;
    fn decode(limits: &Self::Limits, reader: &mut Reader<'_>) -> Result<Self, Problem>;
    fn encoded_bound(limits: &Self::Limits) -> Option<u64>;
    fn decoded_bound(limits: &Self::Limits) -> Option<u64>;
    // and what a nested value and a frame's writer call, after measuring
}
```

`decode` reads the whole of its reader, refusing bytes left over.
`Record` is implemented by generated code alone: hand-written code
calls it, and makes bytes with `encode`, or with `measure` before a
frame's writer.

`generate(schema)` writes one schema version's records' module, and
nothing else: no tests (section 6). The module holds no bound
arithmetic and no split function: a function of one line per field or
limit too long for the lint carries section 6's expectation. What tests
take from the generator, its fixed-rule values and the constructs a
text uses, is section 6's.

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
  (section 9) before anything else crosses (`command.md`, 2, the wire
  rule).
- **A patch moves its family's version:** a new schema, whose generated
  code replaces the old (`command.md`, 2). There is one generated
  version at a time. This is not the rule for registered families.
- **A reader reads one version.** It reads a versioned record's version
  first, with `skein-codec`, and refuses another with the problem
  *another version*. Its generated code reads the current version only.
- **A patched family keeps its versions, from the baseline on.** Its
  owner keeps each version's schema as text, and a kept record
  of an older version is read as a generic value (section 8) and patched
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
  type it reaches (9.4), derived by a plan (`layouts.md`) or converted
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
- **No family keeps golden files** but the `every` fixture (section 6).
  A patched family's kept versions are represented by its texts and
  patch goldens (`command.md`, 3), and a test of an older version builds
  that version's values from its kept text.

## 6. Golden bytes and tests

- **One pin on the bytes.** A record's bytes are its fields' bytes in
  schema order, and a field's bytes follow from its kind alone, never
  from the family it is in. So one schema that uses every construct of
  the language (below), nested, pins every family's bytes, and each
  family's drift test proves that its committed code is the generator's
  output. That schema is skein's `every` fixture (`skein/tests/codec`):
  its current version holds every construct, and its earlier versions
  stay for the patch story (`patches.md`, 8). Its golden files, and its
  family digests' literals (9.3), are the only committed bytes of any
  family: a change to the generator or to `skein-codec` that moves a byte
  or a fingerprint fails there.
- **Fixed-rule values,** so the generator can change without moving
  them. For each record, and each variant of each enumeration, there are
  two:
  - **the smallest:** integers zero, bytes, text and lists empty, options
    absent, each enumeration at its first variant;
  - **a full one:** integers at their maximum, `fixed` bytes `0xa5`,
    bytes and text a pattern of up to three bytes, each list with one
    full item, options present.

  They are named `record_<record>_<smallest|full>.bin` and
  `enum_<enum>_<variant>_<smallest|full>.bin`. The generator computes
  their bytes from the schema alone, not through the code it writes, so
  `every`'s golden files are the rule written down, and any test builds
  a schema's values at run time from its text, a kept text's as a
  current one's.
- **`every`'s own tests:** its drift test compares its records' modules
  and its golden files with the generator's output, and each golden
  decodes by the generated code at the ceilings, measures its length,
  encodes back to its bytes, and decodes generically to the same bytes
  (8.3).
- **The constructs,** the list the pin holds:
  - each kind (`u8`, `u16`, `u32`, `u64`, `bool`, `duration`, `fixed`,
    `bytes`, `secret`, `text`, `list`, `option`, a record, an
    enumeration) in each place: a record's field, a list's item, an
    option's value;
  - an empty record and a `versioned` one; a variant without a record,
    one with a record, and a `boxed` one;
  - a `bytes` and a `secret` that hold another family's type, and a
    `bytes` and a `secret` of pieces;
  - a field with a default, and a former name;
  - the output's shapes: a record whose booleans the generator groups
    (more than three) and a generated function long enough to split.

  A test of the generator fails, naming each, when `every`'s current
  text lacks a construct of the list. Each family's drift test fails,
  naming each, when its schema uses a construct the list lacks, so a new
  construct lands only with its pin. Whether a held type is a record or
  an enumeration is its family's, which the text can't tell (7.2), so
  the list counts a hold once per carrier; `every` holds both, of its
  fixture family `every-other`.
- **What each family keeps,** none of it reading a golden file:
  - **drift:** the generator run on the schema equals the committed
    records' module, and the schema uses no construct the list lacks.
    On a difference it fails, saying how to write the code again, which
    the same test does when `SKEIN_CODEGEN_WRITE=1` asks it to. The test
    is the owner's: it reads the schema in the owner and the code in the
    family's codec crate (section 2), and writes the code where it reads
    it;
  - **fuzzing,** in the fuzzy suite, where the family has it: each
    top-level record's decoder over arbitrary bytes, and over the
    fixed-rule values with bytes changed, cut or grown. It never panics,
    and what decodes encodes back to the same bytes;
  - **memory,** one test per owner over its roots: each root's largest
    value (`largest`) at the ceilings, or under the test's cap at lower
    limits with its bounds at the ceilings pinned, decoded and measured
    with the counting allocator against its decoded bound (section 4).

  Their values are built at run time: the fixed-rule values of the
  schema's text, which the test embeds, the largest values given limits
  allow, which `largest` builds from it, or values a test writes. A
  test of an older version builds that
  version's values from its kept text (`command.md`, 1).
- **No generated tests.** The generator writes no tests and no crate
  keeps a `tests/generated` target: a field's round trip, bounds and
  refusals are its kind's, which `every` and the generic decoder's
  comparison with the generated code (8.3) hold for every family. A
  hand-written test pins what is its own, a projection or a
  translation, with values it builds.
- **No test target of a library that holds no other tests:** a codec
  crate whose library holds no tests sets `[lib] test = false`, so a
  test build compiles its generated code once, as a dependency
  (`tools/crates/design.md`, 3.4). A crate whose library tests need
  private items keeps them, as the wires and smith's charter, channel
  and transcript do.
- **The generator's cost grows with its schema,** not with its square:
  what it needs of a type or a field (its declaration by name, whether a
  type holds bounds, a field's path) is found once per generation. The
  limits are checked against the ceilings once per family, where their
  owner makes them, and once by each worst case asked of them; never in
  each type, nor in a constructor or a decoder (section 4).
- **Generated files** say so in their first line. They are left out of
  formatting, and pass the workspace's lints: the generator writes what
  the lints accept (functions short enough, no nesting to collapse), and
  an owner marks `boxed` an enum's variant the lints find too large
  (section 3). Two scoped expectations stand, emitted only where their
  lint fires: `too_many_lines` on a function of one line per field,
  limit or path (a long record's codec, `Limits::values`), and
  `struct_excessive_bools` on a record of more than three booleans,
  which it holds as named fields.
- **Nothing every change touches:** a generated module holds no array
  whose length counts the schema's items (a `fixed N` field's `[u8; N]`
  is its width), no item named by an ordinal, no version in its file's
  name and no committed fingerprint, and each declaration's code sits in
  its own place, in schema order, with family-wide lists one entry a
  line. So two branches that change different declarations of one
  family merge its generated module line by line, into the module of
  the merged schema; where they don't, the merge job writes it again
  from the merged texts (`temper/docs/design/protocol/tooling.md`, 4.1).
- **What tests read of the generator:**

  ```rust
  pub struct Golden { pub name: String, pub bytes: Vec<u8> }
  // the fixed-rule values, in declaration order, without writing code
  pub fn goldens(schema: &Schema) -> Vec<Golden>;
  pub enum Construct { /* one of the list; its Debug names it */ }
  pub const CONSTRUCTS: &[Construct];  // the list the pin holds
  // the constructs a schema text uses, each once
  pub fn constructs(text: &str) -> Result<Vec<Construct>, Error>;
  // those of them CONSTRUCTS lacks
  pub fn unpinned(text: &str) -> Result<Vec<Construct>, Error>;
  // those of CONSTRUCTS the text lacks
  pub fn missing(text: &str) -> Result<Vec<Construct>, Error>;
  // the declarations no other declaration names, in declaration order
  pub fn roots(schema: &Schema) -> Vec<String>;
  // the bytes of a declaration's largest value at these limits,
  // given in LIMIT_NAMES order
  pub fn largest(schema: &Schema, declaration: &str, limits: &[u32])
      -> Result<Vec<u8>, Error>;
  ```

  The largest value: each list at its count limit, each item, option
  and field at its largest, each bytes, secret and text at its length
  limit, integers at their maximum, each enumeration at the variant
  whose encoding there is longest, the first of equals. Its bytes are
  computed from the schema alone, as the fixed rule's are.

- **The generator's own tests:**
  - schemas it refuses: a type used before it is declared, a name used
    twice, a worst case too large;
  - small schemas' encodings, compared with byte strings written by hand;
  - the constructs small schemas use, each construct of the list found
    in one, and `every`'s current text using each; that text with a
    construct's uses taken out, `missing` naming it;
  - `largest`'s bytes for small schemas, compared with byte strings
    written by hand.

## 7. Schemas at run time

- **One grammar.** `skein-codec`'s `schema` module reads a schema's text
  into tables and checks its layout, as step code: bounded, no
  recursion, nothing allocated past the text's ceilings. skein-codegen's
  `parse` calls it, converts the tables into the generator's own tree,
  and adds the checks only Rust needs: names that are Rust identifiers
  of the right case and no keyword, fields not named as the generated
  methods, declarations and their parts (fields, a version, a tail)
  numbered within a path's `u16`, and each generated name (types,
  paths, limits) used once. A family written lean also names no type as
  an item a lean module re-exports (`Path`, `Problem`, `Record`): that
  check is the form's, since a sealed module re-exports none of them.
- **Refusals as before.** A text with one fault is refused at the same
  line with the same message as before the parser moved; a text with
  several gives the first by pass: characters, then the grammar, then
  the layout, then Rust's names.
- **Every version is read so.** The same parser reads a kept version's
  text at run time (`patches.md`) and the current one in the generator,
  so a kept text means what it meant when it was current.

### 7.1 The grammar

```
schema      = "family" family version declaration*
declaration = ["versioned"] "record" name former* "{" field* "}"
            | "enum" name former* "{" variant* "}"
field       = name former* ":" type ["=" default] [","]
variant     = name former* [":" ["boxed"] name] [","]
former      = "was" name
default     = "none" | "some" default | "[" "]" | word | text
type        = "u8" | "u16" | "u32" | "u64" | "bool" | "duration"
            | "fixed" bound | "secret" bound | "text" bound
            | "bytes" bound [held]
            | "list" bound type | "option" type
            | word                        # a declaration declared before
held        = "holds" family word | "pieces" word   ; after a bytes or a secret bound
```

- **Tokens:** words of ASCII letters, digits, `_` and `-`; the marks
  `{`, `}`, `:`, `,`, `=`, `[` and `]`; a text, `"` then bytes of UTF-8
  other than `"`, `\` and control bytes, then `"`, a `#` among them one
  of its bytes, as a patch's (`patches.md`, 1); whitespace between them.
  `#` elsewhere starts a comment that runs to the end of its line. Any
  other byte outside a comment or a text is refused, before the grammar
  is read.
- **`was`** after a declaration's, a field's or a variant's name, or
  after a former name, starts a former name, one name each: `paused was
  suspended was halted`, so that a variant's former names never run
  into the next variant's name. A field named `was` follows its
  record's `{` or the field before it, as any field does; a variant
  named `was` after another variant follows a comma.
- **A default** follows its field's type and its `=`: one word, one
  text, `[]`, or `none` or `some` and a default, so it ends where the
  next field's name starts.
- **Words:** a `name` starts with a letter or `_` and holds letters,
  digits and `_`; a `family` holds letters, digits, `_` and `-`; a
  `bound` is a decimal `u32` and a `version` a decimal `u16`.
- **`holds` and `pieces`** after a `bytes` bound start its `held` clause,
  unless a `:` follows them: then they name the next field, as
  `holds: list 8 ChangeHold` does.
- **Lines** count from 1, one more after each `\n`. Each declaration,
  field and variant keeps the line of its name. A refusal of the
  grammar has the line of its token, or the text's last line at its
  end, which a final `\n` ends without starting another; a refusal of
  the layout, the line of the field, variant or declaration it names.
- **Bounded:** a text of at most `MAX_TEXT` bytes and `MAX_DECLARATIONS`
  declarations, refused past either. Each table is allocated once, at a
  size the text's length bounds. A type's chain of `list` and `option`
  is read in a loop, outermost first, and a named type is looked up in
  the declarations already read, so neither the parser nor the checks
  recurse.

### 7.2 The layout checks

Each declaration, in order, as the generator checked them (section 6):

- a record: for each field in order, its name not used before in the
  record, each name its type reaches declared before the record, and its
  worst case, and the record's up to it, within `u64`;
- an enumeration: 1 to 256 variants; for each in order, its name not
  used before in the enumeration, and its payload, if any, a record
  declared before; then its worst case within `u64`;
- the declaration's name not used before in the schema;
- its encoded worst case (section 4) within `u32`.

Then, once every declaration is read, the names and defaults:

- each former name none of its scope's names nor another former name
  there (`Former`): a declaration's among every declaration's name and
  former name, a field's among its record's, a variant's among its
  enumeration's;
- each default a value its field's type takes (`Default`), as a
  patch's value is checked (`patches.md`, 2): within its width or bound,
  UTF-8 for `text`, a variant without a record of a named enumeration,
  `some` only under `option`, `[]` only for `list`, `0x` alone for
  `pieces`; none for a named record, a `secret` or a `bytes` that holds
  a record. Its encoding, as the type's decoder would read it, is kept
  in the tables (7.3).

`pieces T` names a declaration declared before; `holds` names another
family's type, which the schema cannot check.

### 7.3 The tables

```rust
pub const MAX_TEXT: u32 = 1_048_576;
pub const MAX_DECLARATIONS: u32 = 4_096;

pub fn parse(text: &[u8]) -> Result<Schema, SchemaError>; // copies the text

pub struct Schema { /* the text, the family, the version, the tables */ }
pub struct DeclIx(u32);    // and FieldIx, VariantIx, TypeIx: indexes in
                           // the schema's tables, in the text's order
impl DeclIx {
    pub const fn new(index: u32) -> DeclIx;
    pub const fn index(self) -> u32;
}
pub struct Span { pub start: u32, pub end: u32 }  // a range of the text

pub struct Declaration { pub name: Span, pub line: u32, pub shape: Shape }
pub enum Shape {
    Record { versioned: bool, first: FieldIx, count: u32 },
    Enum { first: VariantIx, count: u32 },
}
pub struct Field { pub name: Span, pub line: u32, pub ty: TypeIx }
pub struct Variant { pub name: Span, pub line: u32, pub record: Option<DeclIx>, pub boxed: bool }
pub enum Type {
    U8, U16, U32, U64, Bool, Duration,
    Fixed(u32), Secret(u32), Text(u32),
    Bytes { bound: u32, held: Held },
    List { bound: u32, item: TypeIx },
    Option(TypeIx),
    Named(DeclIx),
}
pub enum Held { Opaque, Record { family: Span, name: Span }, Pieces(DeclIx) }

impl Schema {
    pub fn text(&self) -> &[u8];
    pub fn spelled(&self, span: Span) -> &[u8];
    pub fn family(&self) -> Span;
    pub fn version(&self) -> u16;
    pub fn declarations(&self) -> u32;
    pub fn declaration(&self, at: DeclIx) -> Declaration;
    pub fn field(&self, at: FieldIx) -> Field;
    pub fn variant(&self, at: VariantIx) -> Variant;
    pub fn ty(&self, at: TypeIx) -> Type;
    pub fn find(&self, name: &[u8]) -> Option<DeclIx>;
    pub fn fewest_bytes(&self, at: TypeIx) -> u32;
    pub fn most_bytes(&self, at: DeclIx) -> u32;  // the encoded worst case
    pub fn most_bytes_within(&self, at: DeclIx, names: &[&str], values: &[u32]) -> Option<u64>;
    pub fn default(&self, at: FieldIx) -> Option<&[u8]>; // its default's encoding; an option's none without one
    pub fn was(&self, named: Named) -> &[Span];          // its former names, in the text's order
}
pub enum Named { Declaration(DeclIx), Field(FieldIx), Variant(VariantIx) }

pub struct SchemaError { pub line: u32, pub at: Span, pub wrong: Wrong }
pub enum Wrong {
    Long, Many,                                       // past a ceiling
    Character, End, Expected(Expected), Name, Bound, Family, Version,
    Undeclared, NotRecord, NoVariants, ManyVariants,  // the layout
    DuplicateType, DuplicateField, DuplicateVariant, Overflow, Large,
    Former, Default,                                  // the names and defaults
}
pub enum Expected { Family, Open, Colon, Record, Type, Declaration, Name, Default }
```

`Declaration`, `Field` and `Variant` keep their fields: a former name
and a default live in tables of their own, read by `was` and `default`,
so no code that builds or matches the three changes. A field's default
is a span of the text for its error's `at`, and its encoding in a pool
the tables allocate at a size the text bounds: an integer at its width,
a `bool`'s byte, a length or a count before its bytes (0 for `[]`), an
option's presence byte before its value, a variant's tag. `default`
answers that encoding for a field with `=`, `[0]` for an `option`
without one, and `None` for any other field without one.

The plain types derive `Clone`, `Copy` where small, `Debug`, equality
and `Hash`; `Schema` derives `Debug` alone, as lib's `Map` holding its
names has neither `Clone` nor equality. A refusal names its token by `at`; skein-codegen writes the
message from it and the text. A type's fewest bytes are the generator's
(a list's precheck, section 8.2): a scalar's width, `fixed N`'s N, 4 for
a length or a count, 1 for an `option`, a record's fields' sum (2 more
if `versioned`), and an enumeration's 1 more than its smallest
variant's.

- **A kept version's bounds under a side's limits:**
  `most_bytes_within(at, names, values)` is the most bytes `at` encodes
  to, as `most_bytes` computes it, with each bound of the text replaced
  by the value of the limit of the same name, when `names` holds it and
  the value is no larger than the bound, and the text's own bound
  otherwise. Names follow the generator's rule (section 4, *Its limits
  by name*): the record's name in snake case, an underscore and the
  field's, `_some` for an option's value and `_item` for a list's item,
  level by level. The rule lives in `skein-codec` beside the parser,
  and the generator names its limits by it, so the two never differ.
  `None` when `names` and `values` differ in length or the sum
  overflows. So a channel that speaks an older version states and checks
  its terms from that version's kept text with its current limits
  (jig's `protocol/channel/wires.md`): an appended field, absent
  there, counts nothing.

## 8. Generic values

- **An index over the bytes.** A `Value` holds a record's bytes, moved
  in, and an index of entries, one per node, in preorder: the record,
  each field's value, each item of a list, an option's value, a
  variant's record. An entry names its declaration or type, where its
  bytes start and end, its count, tag or presence, its parent, and the
  entry after its subtree. Nothing is copied out of the bytes.
- **Encoding is the bytes.** The encoding is canonical (section 1), so
  a decoded value's encoding is the bytes it was decoded from, two
  values of one type are equal when their bytes are, and a subtree's
  encoding is its range, which a patch carries by copying it (`patches.md`
  ).
- **Decoded at the schema's ceilings,** with the generated code's
  checks in its order (8.2), so a value within its room decodes
  generically exactly when its generated decoder, given the ceilings as
  its limits, decodes it, and a refusal names the same field with the
  same reason. Limits below
  the ceilings stay the generated decoders'.
- **Its room.** A value made with `with_room` sets an entry cap and
  grows its entry tables as they fill, reusing their storage on later
  decodes. A patch's machine
  grows its own entries (`patches.md`, 7). A record that needs more nodes than
  the room is refused with `Refusal::Room`, since a node may take no
  bytes (an empty record), so the bytes alone do not bound the count.
- **Held fields stay opaque.** A `bytes` field that holds a record or
  pieces is read as its bytes, as the generated code reads it; a patch
  opens it (`patches.md`).
- **Printed without content.** A value's bytes are a `lib::Secret`, so
  its `Debug` shows their length and the index (types, offsets, counts,
  tags), never a byte of a field, a secret's or any other. They are
  given back as that `Secret`, moved, not copied.
- **Built by a patch.** A new value is built only by a patch's machine
  (`patches.md`), checked against its schema as it goes.

### 8.1 The value

```rust
pub struct Value { /* the bytes, as a Secret, and the entries */ }
pub struct Node(u32);                          // an entry, in preorder
pub struct Items { /* the next item and how many are left */ }
pub struct Path { pub decl: DeclIx, pub part: Part }
pub enum Part { Field(FieldIx), Tag, Tail, Version }
pub struct Problem { pub path: Path, pub reason: Reason }
pub enum Refusal { Problem(Problem), Room }
pub struct Misread { pub node: Node }          // the node is not of that kind

impl Value {
    pub fn with_room(nodes: u32) -> Value;
    pub fn decoder_bound(nodes: u32) -> Option<u64>; // the decoder entry tables
    pub fn decode(&mut self, schema: &Schema, root: DeclIx, bytes: Box<[u8]>)
        -> Result<Node, Refusal>;              // the root; the bytes are dropped on a refusal
    pub fn bytes(&self) -> &[u8];              // the encoding
    pub fn take(&mut self) -> Secret;          // the bytes back, moved; the value is empty
    pub fn span(&self, node: Node) -> Result<Span, Misread>;
    pub fn field(&self, record: Node, field: FieldIx) -> Result<Node, Misread>;
    pub fn items(&self, list: Node) -> Result<Items, Misread>;
    pub fn next_item(&self, items: &mut Items) -> Option<Node>;
    pub fn some(&self, option: Node) -> Result<Option<Node>, Misread>;
    pub fn tag(&self, enumeration: Node) -> Result<VariantIx, Misread>;
    pub fn payload(&self, enumeration: Node) -> Result<Option<Node>, Misread>;
    pub fn uint(&self, node: Node) -> Result<u64, Misread>;     // u8 to u64; a duration's nanoseconds
    pub fn boolean(&self, node: Node) -> Result<bool, Misread>;
    pub fn content(&self, node: Node) -> Result<&[u8], Misread>; // fixed, bytes, text
    pub fn expose(&self, node: Node) -> Result<&[u8], Misread>;  // a secret's
}
```

A `Path` is the generated `Path` (section 4) by its parts:
`ReportTitle` is `Report`'s field `title`, `EffectTag` the tag of
`Effect`, `FieldTail` the bytes after `Field`, `ReportVersion` the version
of `Report`. Reading a node as what it is not is a `Misread`, a value
the caller handles, never a panic.

### 8.2 The order of the checks

As the generated decoder's, with each bound at its ceiling:

- **A versioned record** reads its version first: too short is `Short`
  at its `Version`, another than the schema's is `Version` there. Then
  its fields in order.
- **An enumeration** reads its tag: too short is `Short` at its `Tag`,
  one past its variants `Tag` there. Then the variant's record, if it
  has one.
- **A field's value** is refused at the field's path, whatever list or
  option holds it; a named type's own checks name its own paths:
  - an integer, a duration or `fixed N`: too short is `Short`;
  - `bool`: too short is `Short`, a byte past 1 is `Bool`;
  - `bytes N`, `secret N` and `text N`: a length too short to read is
    `Short`, past N `Bound`, past the bytes left `Short`; text not UTF-8
    is `Utf8`;
  - `list N T`: a count too short to read is `Short`, past N `Bound`,
    and past the bytes left divided by T's fewest bytes `Short`, when
    those are at least 1; then each item;
  - `option T`: its byte too short is `Short`, 0 is none, 1 is T, any
    other `Tag`.
- **The root** refuses bytes left over with `Trailing`: at its `Tail` if
  a record, at its `Tag` if an enumeration.
- **The room** is checked as each node is entered, before its bytes are
  read.

### 8.3 Tests

- **Step tests** of the parser and the checks: each refusal from both
  sides, a text that passes beside one that does not; `holds` and
  `pieces`, and the fields so named; the ceilings at and past their
  bound.
- **The old parser as an oracle** until it goes: both give the same
  tree, or the same line and message, over every schema in skein and the
  generator's fixtures. The generated code stays byte-identical, which
  the drift tests hold.
- **Generic against generated,** in the fuzzy suite, over skein's
  schemas and a fixture that holds every type of the language: values
  drawn from a schema by a seeded generator decode by both to the same
  tree and encode back to the same bytes; arbitrary bytes, and goldens
  with a byte changed, cut or grown, are refused by both with the same
  path and reason, or decode by both alike. The trees are compared
  without changing the generated code: the generic one is printed in the
  form the generated types' `Debug` takes, and compared with it. The test
  names each case's family, root and seed on stderr before both decoders
  run it, and a disagreement is a finding the comparison returns, with
  the bytes, which the test fails on; nothing catches a decoder's panic
  (../../../foundation/programming-model.md, 10.4).
- **Fixed-rule values:** every fixed-rule value of every skein schema
  (section 6) decodes generically to its own bytes.
- **Memory:** the decoder entry tables stay within `decoder_bound`,
  growing only past the room earlier decodes reached, counted by the
  counting allocator.

## 9. Fingerprints

- **Of the layout, not the names.** A declaration's fingerprint is a
  SHA-256 (lib's) over a canonical encoding of what its bytes are
  (9.1): record or enumeration, `versioned` and the family's version,
  each field's type in order (its kind and width, a `fixed` length, a
  bound, `list` and `option`, text, bytes or secret, what a `bytes`
  holds), the variants' payloads in order, and each named type by its
  own fingerprint. Names, former names, defaults, comments, the text's
  layout and `boxed` are not in it. A record held from another family is
  named by its family and type, since its layout is that family's.
  These are the wires' and the patches' fingerprints, unchanged; a
  stored value's layout is 9.4's, which leaves the family's version out
  and resolves what a value holds.
- **A family's fingerprint** is over all its declarations', in the
  text's order, including those no other declaration names: the schema
  text is the unit, and any declaration may be a root, which only its
  owner's use says. A declaration added, dropped or moved moves it, as
  a declaration's own change does.
- **Step code,** in `schema`: one pass over the declarations in order,
  each named type's fingerprint read from the entries already made, so
  nothing recurses. It allocates that table alone, a fingerprint per
  declaration, at most `MAX_DECLARATIONS`.
- **A rename moves no fingerprint.** For a patched family, the command
  refreshes a text not frozen or keeps renames waiting beside a frozen
  one (`command.md`, 4), so that a later diff never mistakes a rename
  for a field dropped and another added. A registered family's former
  names are in its current schema (section 3).
- **A family's fingerprint is not a value type's layout.** The patch
  check judges changes by the diff and, for wires, the family
  fingerprint (`command.md`, 2). Registered values use 9.4's layout
  fingerprints, not kept versions of the family.

### 9.1 The encoding

A fingerprint is the SHA-256 of a byte string that starts with the
encoding's version, 1, so that a later encoding is never mistaken for
this one. Integers are big-endian; a fingerprint inside another is its
32 bytes.

```
fingerprint = SHA-256(0x01 · layout)
layout      = record | enumeration | family
record      = 0x01 · (0x00 | 0x01 · u16 version) · u32 count · type*
enumeration = 0x02 · u32 count · (0x00 | 0x01 · fingerprint)*
family      = 0x03 · u32 count · fingerprint*
type        = 0x01 u8 | 0x02 u16 | 0x03 u32 | 0x04 u64
            | 0x05 bool | 0x06 duration
            | 0x07 · u32 N               fixed N
            | 0x08 · u32 N               secret N
            | 0x09 · u32 N               text N
            | 0x0a · u32 N · held        bytes N
            | 0x0b · u32 N · type        list N, then its item
            | 0x0c · type                option, then its value
            | 0x0d · fingerprint         a named declaration
            | 0x0e · u32 N · held        secret N that says what it holds (held never 0x00)
held        = 0x00                       nothing said
            | 0x01 · u32 n · family · u32 n · name     holds
            | 0x02 · fingerprint         pieces of a declaration
```

- A record's types are its fields', in order; an enumeration's entries
  its variants', 0 for none and 1 with the record's fingerprint; a
  family's its declarations', in order.
- The `version` is the family's, in a `versioned` record only: a family
  with no versioned record has the same fingerprints at every version.
- A plain `secret N` keeps `0x08`, and one that says what it holds takes
  `0x0e` with its `held`, so no family's fingerprint moves for the word
  until it uses it.
- Each production starts with a byte of its own and ends where its
  grammar says, so two layouts' bytes are equal only when the layouts
  are.

### 9.2 The interface

```rust
pub struct Fingerprint(/* 32 bytes */);  // Clone, Copy, Debug, equality, order, Hash
pub struct Fingerprints { /* a fingerprint per declaration, and the family's */ }

impl Fingerprint {
    pub const fn digest(self) -> [u8; 32];
}
impl Schema {
    pub fn fingerprints(&self) -> Fingerprints;
}
impl Fingerprints {
    pub fn declaration(&self, at: DeclIx) -> Fingerprint;
    pub fn family(&self) -> Fingerprint;
}
```

`Fingerprints` derives `Debug` alone, as `Schema` does.

### 9.3 Tests

- **Step tests,** from both sides: each element the encoding covers
  moves a fingerprint, beside the nearest change that moves none (a
  rename, a comment, the text's layout, `boxed`); a named type's change
  moves its referrers', and its rename does not; a declaration nothing
  names moves the family's alone.
- **Golden digests:** a small schema's, its encoded bytes written out
  beside it, and the `every` fixture's, its first version's and its
  current one's, so that a change of the encoding is a reviewed diff.
- **Every skein schema,** at run time: its fingerprints equal those of
  its text with every name changed, the comments gone, the layout
  changed and `boxed` turned over.

### 9.4 A value type's layout

A store that names each value's layout (jig's
`protocol/store/layouts.md`) names it by the layout of the value's
**value type**: a root declaration of the family, and, where the root
is broad, the one variant of its leading enumeration that the value's
key says it holds. For example, a schema with `record Envelope {
value: Entry }` and `enum Entry { book: Book, note: Note }` has a broad
root: a book's layout is `Envelope` with `Entry` at `book` alone, so a
change to `Note` moves the book's layout not at all.

- **The root,** `Root { decl, variant }`: `variant` may be given only
  when `decl` is broad, a declaration with a leading enumeration as
  `patches.md`, 3.1 defines it (an enumeration itself, or a record whose
  first field's type is an enumeration with no `list` or `option`
  level), and names a variant of that enumeration. Any other root takes
  none. A broad root without one is the declaration whole, every variant
  of its enumeration, encoded as `value`: the root of a value type whose
  keys do not tell its variants apart (the settings' `ApplicationState`,
  whose `follows` leads it; a key that holds an older variant beside the
  one its owner writes).
- **Without the family's version.** A `versioned` record is encoded by
  its flag alone, so that a layout keeps its fingerprint while its
  family's version moves, and a value type whose bytes no change reached
  keeps its layout across versions.
- **What a value holds, resolved.** A `bytes N holds F T` (or a `secret`
  that says so) whose family `F` the caller gives (its text among
  `held`) is encoded by `T`'s own layout in that text, so a change to a
  held record moves the layout of every value type that holds it, and a
  plan converts it with its holder (`layouts.md`, 2). One the caller
  does not give is encoded as 9.1 names it, by family and type. A held
  text's own holds resolve among the same texts, to a depth of
  `MAX_HELD`, 4; deeper is refused (`LayoutError::Deep`).
- **Everything else as 9.1:** a layout's fingerprint equals another's
  only when every value of one is, byte for byte, a value of the other
  but for versioned heads.

```
layout      = SHA-256(0x02 · value)        # the encoding's version, 2
value       = record' | enumeration'        # the root's
record'     = 0x01 · (0x00 | 0x01) · u32 count · type'*   # versioned by its flag alone
enumeration' = 0x02 · u32 count · (0x00 | 0x01 · inner)*
restricted  = 0x04 · u32 index · (0x00 | 0x01 · inner)    # a broad root's leading enumeration at its variant
type'       = as 9.1's type, each named declaration 0x0d · inner, each held' in place of held
            | 0x0f · restricted                           # a broad record's first field, at its variant
held'       = held                                        # 9.1's, for a family not given, and pieces by inner
            | 0x03 · inner                                # holds F T, F given: T's layout
inner       = SHA-256(0x02 · record' | 0x02 · enumeration')  # a declaration's, by this encoding
```

A broad enumeration root at a variant is `restricted` in place of
`value`; a broad record root at a variant is `record'` whose first type
is `0x0f · restricted`; a broad root without one is `value`, as any
other root.
`index` is the variant's place in its enumeration, which its tag is,
so two layouts that put the variant at two tags differ.

```rust
pub const MAX_HELD: u32 = 4;
pub struct Root { pub decl: DeclIx, pub variant: Option<VariantIx> }
pub enum LayoutError { Root, Deep }     // a variant where none is taken, or one its enumeration lacks; held past MAX_HELD
impl Schema {
    pub fn root(&self, decl: &[u8], variant: Option<&[u8]>) -> Option<Root>; // by its text's own names
    pub fn layout(&self, root: Root, held: &[&Schema]) -> Result<Fingerprint, LayoutError>;
}
```

`Root` derives `Clone`, `Copy`, `Debug`, equality and `Hash`. `layout`
is step code as `fingerprints` is: a pass over the declarations the
root reaches, each held text's in turn, with no recursion and a table
per text it reaches, at most `MAX_DECLARATIONS` each. Its tests, beside
9.3's: a versioned root's layout equal across its family's versions; a
restricted variant's layout unmoved by a change to a sibling
variant's record and moved by its own; a broad root without a variant
moved by either; a held record's change moving
its holder's layout when its family is given and not when it is not;
defaults and former names moving nothing; golden digests for the
fixture that holds every type, as 9.3's.

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
- **The generator's statements as structure:** nothing parses or rewrites
  text an emitter wrote, and a function that may split is a value of
  statements, but each statement is text from `format!`, under a
  module-wide expect in every `emit_*` module, and `emit_tests.rs` cuts its
  own source template. A syntax tree, should a consumer need more.
- **A piece's whole loop:** `piece::split_within` (3) holds the split, the
  charge and the bound, and each owner writes its own `for`, decoding each
  item with its type's generated code, since a loop over any type would take
  a closure or a trait. The generator could write the loop, with the
  rest of a record family's translation.
- **A record's own `to_bytes`:** an owner makes a `Writer`, encodes into it
  and takes its bytes at each site. Generated per record, it changes the
  generator and every family's generated code.
- **What the generator prints for a widening:** the emitters write
  `usize::try_from(..).expect(..)` for a bound's `u32`, where step code writes
  `usize_of` (../../runtime/lib.md, 6). Printing `usize_of` regenerates every tree's codecs,
  with no reader's gain yet.
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
