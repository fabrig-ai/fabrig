# Codecs from schemas

Provisional, 2026-10-06, revised 2026-10-26. How an application protocol
built on skein encodes its records, and how its kept records follow its
schemas from one version to the next. A schema says each record's fields
and their bounds. skein's generator turns it into plain Rust codecs,
limits, bounds and golden bytes, and `skein-codec` holds the little
the generated code shares, and reads schemas and their records at run
time. Codecs carry the bodies of a framed channel (../../net/channel.md), and
records kept in files or stores.

## 1. In one page

- **The schema is the specification.** A small file per family and version
  gives each record's fields, their types and their bounds. Design
  documents name records and bounds; the schema gives the bytes.
- **Generated, not written.** A generator, ordinary Rust in skein, turns a
  schema into:
  - the records' types, with their bounds sealed by constructors;
  - limits, whose ceilings are the schema's bounds;
  - measured encoding;
  - bounded decoding with typed problems;
  - each record's encoded and decoded bounds;
  - golden bytes.
- **Plain Rust of the subset** (../../../foundation/programming-model.md, section 10):
  structs, enums, functions and `match`; no traits beyond derives, no
  generics, closures or macros. The output is kept in the consumer's
  repository and reviewed like any other code. A test fails when the code
  and the schema drift.
- **Sized** (../../../foundation/programming-model.md, section 8). Every length comes before
  its bytes, and is checked against its limit and what is left before
  anything is set aside. Types form a tree of fixed depth, so decoding
  never recurses on a peer's choice.
- **Canonical.** One value has one encoding, and decoding refuses any
  other, so what decodes encodes back to the same bytes.
- **Versioned.** A schema is one version of a family, and a family has
  one current version. A record kept beyond a channel carries its
  version. A reader reads the current version and refuses another. A
  family's baseline is its owner's (section 5): until it, no version moves
  and nothing is translated; from it on, kept versions are read as
  generic values and patched to the current one (`patches.md`).
- **Read at run time too.** `skein-codec` reads a schema's text as step
  code (section 7), and decodes the bytes of any of its records into a
  generic value, an index over those bytes, with the generated code's
  checks (section 8). The generator reads schemas through the same
  parser, so there is one language.
- **Upgrades by patches.** A layout's fingerprint (section 9), a diff of
  two versions into the kinds of change (`diff.md`), patches over
  generic values, run at run time without generated code for old
  versions (`patches.md`), and the command that writes them, whose check a
  gate calls (`command.md`).

## 2. In skein

- **`skein-codec`** depends on lib only, and builds for
  `wasm32-unknown-unknown` as lib does (../../runtime/lib.md, section 2), with the code
  generated for it. It holds what the generated code shares:
  - the reasons a decoding fails;
  - bounded reads of lengths, counts and text;
  - reading a versioned record's version;
  - writing a piece of items and splitting one (section 3).

  And, as step code, what reads schemas and records at run time:
  - `schema`: the parser and the layout checks (section 7);
  - `value`: generic values (section 8);
  - `patch`: patches and the machine that runs them (`patches.md`).
- **`skein-codegen`** is the generator, and the tooling of upgrades: the
  diff (`diff.md`) and the command (`command.md`). Ordinary Rust, not
  step code. It reads schemas through `skein-codec`'s parser and adds
  the checks only Rust needs (`README.md`, section 7). A consumer depends on it in its
  tests and tools only.
- **A consumer's codec crate** holds a family's generated record code,
  `src/generated/<family>_v<N>.rs` or `v<N>.rs` (section 4), and depends
  on lib and `skein-codec` alone; its dev-dependencies are free,
  `skein-codegen` among them. The family's **owner**, the crate that
  keeps it, holds the rest: its schemas, kept texts and patches
  (`command.md`, 1), its generated glue, its golden bytes and its drift
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
  `jig-protocol-store-people`). Each generated module keeps its name at
  the codec crate's root (`v3`, `cursor_v1`), and the owner names it
  where it named it before: a public module is re-exported at the
  owner's same path, which callers keep, and either path names it.
- **An owner with no domain to translate is its own codec crate,**
  holding schemas, code, golden bytes and drift test together: smith's
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
- **Every field is present.** An absent value is an explicit `option`;
  there are no defaults.
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
  patch of the held type reach the field wherever it is kept (`patches.md`
  ). From the baseline on (`patches.md`), a field of a stored family
  whose bytes are a record or pieces says so. A `secret N` may say so
  too, `secret N holds skein-oauth SavedToken`, and stays a secret: never
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
  once, where it makes its limits. Constructors and decoders check
  values only, never the limits: each value against its limit no higher
  than its ceiling, so no value past a ceiling exists or decodes, whatever
  limits they are given, and a call costs what its value holds, not the
  family's size.
- **Its problems.** An enumeration of the family's fields, and a problem
  that names one of them with `skein-codec`'s reason.

For each record and enumeration:

- **Its type.** Fields are private:
  - a constructor checks every field against the limits it is given and
    refuses with the field that broke one, so any value that exists fits
    them;
  - accessors read the fields, and a function that consumes the value
    hands its fields back, so bytes move and are not copied;
  - the derives are `Debug`, `Clone`, equality and `Hash`; a `secret`
    field's `Debug` is its length (section 3).
- **Measuring:** its encoded length. This cannot overflow, since the
  ceilings bound it.
- **Encoding** into a `lib::Writer` with that much room left: the rest of a
  frame's writer (../../net/channel.md, section 7), or a box of its own.
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

  Each is `None` for limits past a ceiling. It checks the family's
  limits once, where it is asked, then reaches the types its fields name
  without checking them again: a bound costs the types it reaches,
  not their number times the family's limits.

`generate(schema)` writes a module for one schema version. It emits
`encoded_bound` and `decoded_bound`, their `_within` forms and numbered
helpers, and uses `List::price`. The CLI's generation invocation takes
`<schema> <out-dir>`; any extra argument is a usage error. The bound names
and `worst_case_bytes`/`worst_case_heap` (and their `_within` forms) stay
reserved for fields.

## 5. Versions

- **A family's baseline:** for the families a temper deployment stores,
  the store temper's `main` wrote at `bc23255eb7` (temper's
  `protocol/baseline.md`; the user, 2026-11-11); for every other family
  a build stores, sends or embeds, the first release. Until it, a family's version
  keeps its number: a change a reader would notice changes the schema,
  its generated code and its golden bytes in place. A record an older
  build kept is then not told apart by its version, so nothing is kept
  across builds that change its family: a store from an older build is
  started afresh. From it, a version moves in the landing whose patch
  moves it (`command.md`, 2).
- **A wire's family moves with its layout.** A family that crosses
  between two builds of its owner's (a channel's bodies, what one side
  writes and the other reads) is marked a wire in its crate
  (`command.md`, 1). From its baseline, its owner's first `--baseline`
  run, its version moves whenever its layout does, an old value
  surviving or not, so that two builds that name one version lay it out
  alike and a peer can be judged by its version and its fingerprint
  (section 9) before anything else crosses (`command.md`, 2, the wire
  rule).
- **From the baseline on, any change a reader would notice is a new
  version:** a new schema, whose generated code replaces the old. There
  is one version at a time, and no promise of stability to anyone
  outside the family's owner.
- **A reader reads one version.** It reads a versioned record's version
  first, with `skein-codec`, and refuses another with the problem
  *another version*. Its generated code reads the current version only.
- **Kept versions are patched, from the baseline on.** From it, a
  family's owner keeps each version's schema as text, and a kept record
  of an older version is read as a generic value (section 8) and patched
  to the current version (`patches.md`) before the generated code reads
  it. A version then names one layout. Until the baseline, nothing is
  translated. What a channel's sides speak across two builds is agreed
  at its opening (../../net/channel.md, 5.2), each side judging the
  other's wires by their versions and fingerprints; translating between
  them is its owner's.
- **A writer writes its current version only.**
- **On a channel,** the agreed version picks the code for its bodies
  (../../net/channel.md, section 5.2). Bodies carry no version of their own.
- **Golden bytes:** until the baseline, a family has one set, its
  current schema's, which a change moves in place. From it on, a
  version's golden bytes stand for it, and a change that moves them is a
  new version.

## 6. Golden bytes and tests

- **Golden values follow a fixed rule,** so the generator can change
  without moving them. For each record, and each variant of each
  enumeration, there are two values:
  - **the smallest:** integers zero, bytes, text and lists empty, options
    absent, each enumeration at its first variant;
  - **a full one:** integers at their maximum, bytes and text a fixed
    pattern of a few bytes, each list with one full item, options present.

  Their encodings sit in a file beside the code, and each type's values
  must encode and decode as recorded.
- **Bounds:** generated tests decode a field at its limit, and one past it,
  without stored files. The limit is a test's own, a few bytes or items,
  with every other limit at its ceiling: decoding checks the limit it is
  given, so a small one checks the same code as the ceiling, which the
  worst case (section 4) and the memory test check.
- **Table tests:** each generated test checks a group of cases, up to
  1,024 golden values or 64 bounds, and names every case that fails. A
  test process costs more to start than one case costs to check
  (../../../foundation/testing-strategy.md, section 8).
- **Drift:** a test runs the generator on the schema and compares the
  result with the committed code and golden bytes. On a difference it
  fails, saying how to write them again, which the same test does when an
  environment variable asks it to. The test is the owner's: it reads the
  schema and the golden bytes in the owner, and the code in the family's
  codec crate (section 2), and writes each where it reads it.
- **One family to a golden directory:** a crate that holds several
  families gives each version of each a directory of its own, as skein's
  fixtures do with `golden/<family>-v<N>`, and a directory holds that
  version's goldens and nothing else. The command writes `golden/v<N>`, for
  a crate of one family (`command.md`).
- **The generator's cost grows with its schema,** not with its square:
  what it needs of a type or a field (its declaration by name, whether a
  type holds bounds, a field's path) is found once per generation. The
  limits are checked against the ceilings once per family, where their
  owner makes them, and once by each worst case asked of them; never in
  each type, nor in a constructor or a decoder (section 4).
- **Generated files** say so in their first line. They are left out of
  formatting, and pass the workspace's lints with no allowances: the
  generator writes what the lints accept (functions short enough, no
  nesting to collapse), and an owner marks `boxed` an enum's variant the
  lints find too large (section 3).
- **Fuzzing,** in the fuzzy suite: each top-level record's decoder over
  arbitrary bytes. It never panics, and what decodes encodes back to the
  same bytes.
- **Memory:** decoding a full value at the ceilings, measured with the
  counting allocator against its heap's worst case.
- **The generator's own tests:**
  - schemas it refuses: a type used before it is declared, a name used
    twice, a worst case too large;
  - small schemas' encodings, compared with byte strings written by hand.

## 7. Schemas at run time

- **One grammar.** `skein-codec`'s `schema` module reads a schema's text
  into tables and checks its layout, as step code: bounded, no
  recursion, nothing allocated past the text's ceilings. skein-codegen's
  `parse` calls it, converts the tables into the generator's own tree,
  and adds the checks only Rust needs: names that are Rust identifiers
  of the right case and no keyword, fields not named as the generated
  methods, and each generated name (types, paths, limits) used once.
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
declaration = ["versioned"] "record" name "{" field* "}"
            | "enum" name "{" variant* "}"
field       = name ":" type [","]
variant     = name [":" ["boxed"] name] [","]
type        = "u8" | "u16" | "u32" | "u64" | "bool" | "duration"
            | "fixed" bound | "secret" bound | "text" bound
            | "bytes" bound [held]
            | "list" bound type | "option" type
            | word                        # a declaration declared before
held        = "holds" family word | "pieces" word   ; after a bytes or a secret bound
```

- **Tokens:** words of ASCII letters, digits, `_` and `-`; the marks
  `{`, `}`, `:` and `,`; whitespace between them. `#` starts a comment
  that runs to the end of its line. Any other byte outside a comment is
  refused, before the grammar is read.
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
}

pub struct SchemaError { pub line: u32, pub at: Span, pub wrong: Wrong }
pub enum Wrong {
    Long, Many,                                       // past a ceiling
    Character, End, Expected(Expected), Name, Bound, Family, Version,
    Undeclared, NotRecord, NoVariants, ManyVariants,  // the layout
    DuplicateType, DuplicateField, DuplicateVariant, Overflow, Large,
}
pub enum Expected { Family, Open, Colon, Record, Type, Declaration }
```

The plain types derive `Clone`, `Copy` where small, `Debug`, equality
and `Hash`; `Schema` derives `Debug` alone, as lib's `Map` holding its
names has neither `Clone` nor equality. A refusal names its token by `at`; skein-codegen writes the
message from it and the text. A type's fewest bytes are the generator's
(a list's precheck, section 8.2): a scalar's width, `fixed N`'s N, 4 for
a length or a count, 1 for an `option`, a record's fields' sum (2 more
if `versioned`), and an enumeration's 1 more than its smallest
variant's.

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
  form the generated types' `Debug` takes, and compared with it.
- **Goldens:** every golden of every skein schema decodes generically
  to its own bytes.
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
  own fingerprint. Names, comments, the text's layout and `boxed` are
  not in it. A record held from another family is named by its family
  and type, since its layout is that family's.
- **A family's fingerprint** is over all its declarations', in the
  text's order, including those no other declaration names: the kept
  text is the unit, and any declaration may be a stored root, which only its
  owner's keys say. A declaration added, dropped or moved moves it, as
  a declaration's own change does.
- **Step code,** in `schema`: one pass over the declarations in order,
  each named type's fingerprint read from the entries already made, so
  nothing recurses. It allocates that table alone, a fingerprint per
  declaration, at most `MAX_DECLARATIONS`.
- **A rename moves no fingerprint,** and the check (`command.md`) also
  asks the latest kept text to follow the current names, so that a
  later diff never mistakes a rename for a field dropped and another
  added.
- **A family's fingerprint moved** means its kept text is replaced,
  not that a patch is owed: the check decides by the diff (`command.md`
  ), never by the family's fingerprint alone, and nothing at run
  time takes its move for a change of layout. What must decide one
  uses the declarations' fingerprints.

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
  beside it, and the fixture that holds every type's, so that a change
  of the encoding is a reviewed diff.
- **Every skein schema,** at run time: its fingerprints equal those of
  its text with every name changed, the comments gone, the layout
  changed and `boxed` turned over.

## 10. Open questions

- **Shrinking:** a patcher keeps the largest allocation it reached, so
  one large record keeps it at that size until the patcher goes.

- **Upstreams' levels:** the command writes a patch's `after` as its
  author gives it (`command.md`, 3); jig, which orders patches across
  owners, reads them from the pins.
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
  texts alone, so it cannot see the held family's renames. jig,
  which patches held fields across families, gives the diff
  those renames.
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
