# Codecs from schemas: run-time schemas and generic values

## 1. Schemas at run time

- **One grammar.** `skein-codec`'s `schema` module reads a schema's text
  into tables and checks its layout, as step code: bounded, no
  recursion, nothing allocated past the text's ceilings. skein-codegen's
  `parse` calls it, converts the tables into the generator's own tree,
  and adds the checks only Rust needs: names that are Rust identifiers
  of the right case and no keyword, fields not named as the generated
  methods, declarations and their parts (fields, a version, a tail)
  numbered within a path's `u16`, and each generated name (types,
  paths, limits) used once, and no type named `Path`, `Problem` or
  `Record`, which the module names (`README.md`, section 4).
- **Refusals as before.** A text with one fault is refused at the same
  line with the same message as before the parser moved; a text with
  several gives the first by pass: characters, then the grammar, then
  the layout, then Rust's names.
- **Every version is read so.** The same parser reads a kept version's
  text at run time (`patches.md`) and the current one in the generator,
  so a kept text means what it meant when it was current.

### 1.1 The grammar

```
schema      = "family" family version declaration*
declaration = ["versioned"] "record" name former* "{" field* "}"
            | "enum" name former* "{" variant* "}"
field       = name former* ":" type ["=" default] [","]
variant     = name former* [":" ["boxed"] name] [","]
former      = "was" name
default     = "none" | "some" default | "[" "]" | word | text
type        = "u8" | "u16" | "u32" | "u64" | "bool" | "duration"
            | "fixed" bound | "secret" bound [held] | "text" bound
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
- **`holds` and `pieces`** after a `bytes` or `secret` bound start its
  `held` clause, unless a `:` follows them: then they name the next field,
  as `holds: list 8 ChangeHold` does.
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

### 1.2 The layout checks

Each declaration, in order, as the generator checked them (`golden.md`, section 1):

- a record: for each field in order, its name not used before in the
  record, each name its type reaches declared before the record, and its
  worst case, and the record's up to it, within `u64`;
- an enumeration: 1 to 256 variants; for each in order, its name not
  used before in the enumeration, and its payload, if any, a record
  declared before; then its worst case within `u64`;
- the declaration's name not used before in the schema;
- its encoded worst case (`README.md`, section 4) within `u32`.

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
  in the tables (1.3).

`pieces T` names a declaration declared before; `holds` names another
family's type, which the schema cannot check.

### 1.3 The tables

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
    Fixed(u32),
    Secret { bound: u32, held: Held },
    Text(u32), Bytes { bound: u32, held: Held },
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
and `Hash`; `Schema` derives `Debug` and `Clone` alone: a copy is its
text and tables, for a holder that keeps schemas of its own (a
`Patcher`'s `keep_schema`, `patches.md`) from one already parsed, at a
fraction of a parse; two schemas are told apart by their texts, not by
an equality. A refusal names its token by `at`; skein-codegen writes the
message from it and the text. A type's fewest bytes are the generator's
(a list's precheck, section 2.2): a scalar's width, `fixed N`'s N, 4 for
a length or a count, 1 for an `option`, a record's fields' sum (2 more
if `versioned`), and an enumeration's 1 more than its smallest
variant's.

- **A kept version's bounds under a side's limits:**
  `most_bytes_within(at, names, values)` is the most bytes `at` encodes
  to, as `most_bytes` computes it, with each bound of the text replaced
  by the value of the limit of the same name, when `names` holds it and
  the value is no larger than the bound, and the text's own bound
  otherwise. Names follow the generator's rule (`README.md`, section 4, *Its limits
  by name*): the record's name in snake case, an underscore and the
  field's, `_some` for an option's value and `_item` for a list's item,
  level by level. The rule lives in `skein-codec` beside the parser,
  and the generator names its limits by it, so the two never differ.
  `None` when `names` and `values` differ in length or the sum
  overflows. So a channel that speaks an older version states and checks
  its terms from that version's kept text with its current limits
  (jig's `protocol/channel/wires.md`): an appended field, absent
  there, counts nothing.

## 2. Generic values

- **An index over the bytes.** A `Value` holds a record's bytes, moved
  in, and an index of entries, one per node, in preorder: the record,
  each field's value, each item of a list, an option's value, a
  variant's record. An entry names its declaration or type, where its
  bytes start and end, its count, tag or presence, its parent, and the
  entry after its subtree. Nothing is copied out of the bytes.
- **Encoding is the bytes.** The encoding is canonical (`README.md`, section 1), so
  a decoded value's encoding is the bytes it was decoded from, two
  values of one type are equal when their bytes are, and a subtree's
  encoding is its range, which a patch carries by copying it (`patches.md`
  ).
- **Decoded at the schema's ceilings,** with the generated code's
  checks in its order (2.2), so a value within its room decodes
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

### 2.1 The value

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

A `Path` names what the generated `Path` (`README.md`, section 4) names, by the
same positions: `Report`'s field `title` (`paths::REPORT_TITLE`), the
tag of `Effect`, the bytes after `Field`, the version of `Report`. Reading a node as what it is not is a `Misread`, a value
the caller handles, never a panic.

### 2.2 The order of the checks

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

### 2.3 Tests

- **Step tests** of the parser and the checks: each refusal from both
  sides, a text that passes beside one that does not; `holds` and
  `pieces`, and the fields so named; the ceilings at and past their
  bound.
- **Generic against generated,** in the fuzzy suite, over skein's
  schemas and a fixture that holds every type of the language: values
  drawn from a schema by a seeded generator decode by both to the same
  tree and encode back to the same bytes; arbitrary bytes, and fixed-rule
  values with a byte changed, cut or grown, are refused by both with the
  same path and reason, or decode by both alike. The trees are compared
  without changing the generated code: the generic one is printed in the
  form the generated types' `Debug` takes, and compared with it. The test
  names each case's family, root and seed on stderr before both decoders
  run it, and a disagreement is a finding the comparison returns, with
  the bytes, which the test fails on; nothing catches a decoder's panic
  (../../../foundation/programming-model.md, 10.4).
- **Fixed-rule values:** every fixed-rule value of every skein schema
  (`golden.md`, section 1) decodes generically to its own bytes.
- **Memory:** the decoder entry tables stay within `decoder_bound`,
  growing only past the room earlier decodes reached, counted by the
  counting allocator.
