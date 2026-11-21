# Translation from maps files

2026-11-21, from goal K5 of `temper/docs/design/draft/codec-shape.md`, 4.
How a family's generated records are translated to the domain's types
and back. A maps file beside the schema says where the domain's side
differs from the record's; `skein-codegen` writes the translation from
the schema and the maps file into the owner's translation crate; the
owner writes by hand only the shapes that are its own. The bytes are
the schema's alone ([README.md](README.md), section 4): a map moves no
byte, bound, fingerprint or layout, and the schema's parser never reads
it.

## 1. In one page

- **The record stays its own type,** the schema's and nothing else's,
  and the codec crates depend on nothing new
  (`temper/docs/design/draft/codec-shape.md`, 4.4). The translation
  lives in the owner's translation crate, which already names both the
  domain and the records.
- **One maps file per family that has something to generate:**
  `schema/<family>.maps`, beside the family's schema texts, kept by the
  owner that translates. A family whose every record is the owner's own
  shape keeps none.
- **Generated where the two sides match,** field by field and variant
  by variant, through nested declarations, options, lists and boxes; a
  **converter** where a scalar's meaning changes; **by hand** where the
  shape is the owner's own (3.3).
- **Both directions, exhaustive,** as plain functions in the translation
  crate's generated module (4). Either direction may refuse, naming the
  record's field by its path.
- **Mixed families:** generated code calls the owner's translation of a
  `by hand` declaration, and the owner's code calls the generated
  functions, under one naming rule and one signature (4.1, 4.2).
- **Checked and merged as the records are:** the family's drift test
  runs the generator on the schema and the maps file, and the merge job
  makes a conflicted translation module again from them (7).
- **Its users:** jig's store, its translation crates
  `jig-protocol-store-records` and `jig-protocol-store`, and temper's
  store, `temper-protocol-store`. The wires follow (9).

## 2. The maps file

A text of lines. `#` starts a comment that runs to the end of its line;
blank lines count for nothing. A *name* is the schema's, as the schema
spells it: a declaration's, a field's or a variant's. A *path* is a Rust
path as the translation crate names it, and may start with an alias.

**The header,** before the first declaration line:

- `records PATH`: the family's generated record module
  ([README.md](README.md), section 4) as the translation crate names
  it. Once, required.
- `use ALIAS = PATH`: an alias for the rest of the file. Any number.
- `by hand PATH`: the module holding the owner's translations that
  generated code calls (4.2). Once, required where one is called.
- `context TYPE`: the type of the context every translation function
  takes and passes on (4.1). Once; `()` when absent.

**Declaration lines,** one for each declaration the file maps and each
the owner translates by hand, in the schema's order:

- `record NAME = PATH`, `enum NAME = PATH`: the declaration mirrors the
  domain's struct or enumeration at `PATH`.
- `record NAME stands for FIELD [via PATH]`: a record of one field that
  the domain holds as that field's value, wherever it holds the record:
  a list of `Reference { task }` as a list of task numbers, a variant's
  payload `Words { words }` as the variant's bytes. It has no domain
  type and no functions of its own.
- `record NAME`: a variant's payload with no domain type, given a line
  only for its block (3.2).
- `record NAME by hand`, `enum NAME by hand`: the owner's own shape.

**A block** may follow `record NAME = PATH`, `record NAME` or
`enum NAME = PATH`: its items between `{` and `}`, one to a line or
separated by commas, in the schema's order.

- In a record: `FIELD [= NAME] [via PATH] [slice | boxed]`: the
  domain's field of that name, a converter for its value (5), and the
  domain's shape where it isn't the record's (3.1).
- In an enumeration: `VARIANT [= NAME] [tuple [boxed]] [{ ... }]`: the
  domain's variant of that name, its payload held as one value (3.2),
  in a `Box`, and a block of the payload's fields for this variant
  alone.

**The domain's shape** is the record's unless a mark says otherwise:
`slice`, the domain holds a list as a boxed slice where the record holds
a `skein_lib::List`; `boxed`, the domain holds a declaration's value, or
a `tuple` payload, in a `Box`, whether or not the schema's `boxed` puts
one on the record's side. The generator reads nothing of the domain's
types, so the file says what the generated code converts.

What the file doesn't say matches by name (3). jig's notes family, for
its shape:

```
records crate::notes_v1
use n = jig_domain_notes
use s = skein_codec::convert
by hand crate::codec::notes
context crate::notes_v1::Limits

record Segment stands for bytes
enum Last = n::Last { exact tuple, open tuple }
record Pattern by hand
enum Scope = n::Scope
enum Author = n::Author
record Reference stands for task
record Entry = n::Entry { changed via s::wall }
record Line = n::Line
enum Value = n::Record { entry tuple, line tuple }
record Row by hand
```

## 3. Matching

### 3.1 Records

- **Fields by name,** every field of both sides named, with no rest
  pattern: a field on either side with no counterpart fails the build of
  the generated module, at the function that a comment ties to the maps
  file's line. `= NAME` names the domain's field.
- **A field's value,** by its type in the schema:
  - an integer, a boolean, bytes, text, a fixed array, a duration: the
    same value, cloned from the borrowed domain value on the way to the
    record;
  - a declaration: through that declaration's map: its generated
    functions, the owner's by-hand pair, or, for a record that stands
    for its field, that field's value;
  - an option: an `Option` on the domain's side, its value by the same
    rule;
  - a list: on the domain's side a `skein_lib::List`, or a boxed slice
    where the item says `slice`, each item by the same rule; a list
    whose count passes a `u32` refuses with `Bound` at the field;
  - a field of pieces: none; its record is the owner's (3.3).
- **Boxes** are put on or taken off as the two sides have them: the
  schema's `boxed` on the record's, a `boxed` mark on the domain's, at a
  field or a `tuple` payload. Where both sides hold the same container
  or box, the generated code writes no conversion between them.
- **`via PATH`** passes the field's value, each value inside an option
  or a list, through the converter (5) instead.

### 3.2 Enumerations

- **Variants by name:** the schema's `move_task` is the domain's
  `MoveTask`, the PascalCase the records' own variants take, unless
  `= NAME` names another. Every variant of both sides is named: one on
  either side with no counterpart fails the build.
- **A unit variant** is a unit variant.
- **A payload, inline by default:** the domain's variant is a struct
  variant holding the payload record's fields, matched as a record's
  (3.1). A block on the variant renames or converts them for that
  variant alone, over the payload record's own block, so that one
  payload may serve several variants.
- **`tuple`:** the domain's variant holds one value: the payload's
  mapped domain type, or, for a payload that stands for its field, that
  field's value.

### 3.3 What stays by hand

The owner's own shape (`temper/docs/design/draft/codec-shape.md`, 4.1,
the third kind), each declaration marked `by hand`:

- a record split, joined or flattened otherwise than the domain's; a
  positional tuple; a domain type with private fields or a checked
  constructor;
- a field with no counterpart: a default, a value taken from a key or
  from the context, a variant one side lacks;
- a list framed as pieces, or bounded by a budget of the owner's;
- a lookup, a remap, a check of the owner's or across fields.

A declaration the generator can't express is by hand; the grammar grows
only for a shape that recurs (9).

## 4. The generated module

### 4.1 Its functions

- **Where:** `src/generated/<module>.rs` of the translation crate, or a
  directory below its `src/generated/` where the crate holds the
  family's records too; `<module>` the records' module name
  ([README.md](README.md), section 4). Its first line reads
  `// Generated by skein-codegen from <family> <version> and its maps. Do not edit.`
- **A pair for each mapped declaration** with a domain type, in the
  schema's order:
  - `<decl>_to_record(value: &D, context: &C) -> Result<R, Problem>`;
  - `<decl>_to_domain(record: R, context: &C) -> Result<D, Problem>`;

  `<decl>` the declaration's name in snake case, `R` its record, `D` the
  domain's type, `C` the header's context and `Problem` the family's
  generated problem, a path and a reason.
- **Refusals:** a converter's reason at the field's path, as the
  records' `paths` module names it; a count past `u32` as `Bound` at its
  field; a by-hand or nested refusal unchanged. A value past its limits
  is the encoder's to refuse: translation checks no limit.
- **The context** reaches every call to a by-hand pair; no generated
  line reads it.
- **Plain step code** under the workspace's lints: matches on both
  sides, named fields, the converters, the by-hand pairs and its own
  functions. Its one allowance is the generated files' own
  ([README.md](README.md), section 4): a pair of one line per field or
  variant whose body passes the lint's line bound carries
  `#[expect(clippy::too_many_lines, reason = "...")]`, emitted only
  there, as the records' emitter does. It defines and calls no generic helper; a
  domain's choice of container or box, which the maps file states (2),
  is bridged by `Box::new`, a move out of the box and `skein-lib`'s
  conversions between a `List` and a boxed slice, each written only
  where the two sides differ, so that no conversion is to its own type;
  `skein-codec`'s exception
  (`skein/docs/foundation/programming-model.md`, 10.2) stays its
  codecs'.

### 4.2 By hand

- **Called by generated code:** a `by hand` declaration that a mapped
  one reaches is translated by the owner's pair in the header's `by
  hand` module, with the names and signature of 4.1.
- **Calling generated code:** the owner's translation calls the
  generated pairs for what they cover. A family's root, its routing by
  key and its framing into bytes stay the owner's.
- **Otherwise free:** a by-hand declaration no generated function
  reaches keeps whatever shape its owner gives it.

## 5. Converters

- **A pair of functions,** named by a path's stem: `via s::wall` calls
  `s::wall_to_record`, from the domain's value by reference to the
  record's, and `s::wall_to_domain`, from the record's value to the
  domain's. Each returns `Result<_, skein_codec::Reason>`; the generated
  code adds the field's path. A converter takes no context.
- **Shared, in `skein_codec::convert`,** for `skein-lib`'s types and
  core's:
  - `wall`: `Wall` and its nanoseconds, a `u64`;
  - `time`: `Time` and its nanoseconds;
  - `duration`: `Duration` and its nanoseconds;
  - `token`: `Token` and its raw `u64`;
  - `text`: `Box<str>` and its bytes, refusing bytes that aren't UTF-8
    with `Utf8`.

  Only `text` refuses.
- **An owner's,** in its translation crate: a bit set and its integer, a
  newtype of the owner's and its number, a signed number and its bits.

## 6. The generator's refusals

The generator writes nothing, and names the maps file's line and the
schema's name, for:

- a name the schema doesn't hold, or a declaration or item named twice;
- a declaration line or an item out of the schema's order;
- a header without `records`, or a by-hand pair to call with no `by
  hand` module;
- a mapped declaration that reaches, through a field or a `tuple`
  payload, a declaration with no line, or a payload record with no
  domain type;
- `stands for` on a record of more than one field, or naming a field it
  doesn't hold;
- `tuple` or a block on a unit variant; `via` on a field whose value is
  a declaration;
- `slice` on a field whose value, or an option's, isn't a list; `boxed`
  on a field whose value, or an option's, isn't a mapped declaration's
  domain type, or on a variant without `tuple`;
- a mapped record with a field of pieces.

## 7. Command, drift and merges

- **The command:** `skein-codegen <schema>... <maps> <out-dir>`. A last
  input whose name ends in `.maps` makes it write the translation module,
  `<out-dir>/<module>.rs`, in place of the records. The library's
  `translate(&Schema, &str)` returns that module's text from the parsed
  schema and the maps file's text, or the refusal of 6.
- **Drift:** each family with a maps file has its drift test
  ([README.md](README.md), section 6) compare the translation module
  with the generator's output, `SKEIN_CODEGEN_WRITE=1` writing it.
- **Merges:** `tools/merge/codecs.txt` gives a translation module a row
  of its own: its path, the schema texts in command order, the maps file
  last. The merge job makes it again where they and the generator merged
  clean (`temper/docs/design/protocol/tooling.md`, 4.1). The generator
  is `skein-codegen`'s crate, the converters `skein-codec`'s, both
  already the job's generator. A conflicted maps file is a conflicted
  source, left with the generated module for the resolution
  (`temper/docs/design/draft/codec-shape.md`, 6.2).
- **The list's check:** `fabrig-boundary`'s `merge_codecs` reproduces a
  row whose last input is a maps file through `translate`.

## 8. Tests

- **The fixture,** in skein's codec world (`skein/tests/codec`): a
  family `mapped`, with its records, a domain of its own, a maps file
  using every form of 2 and 3, a by-hand declaration with a context, and
  a converter of its own beside the shared ones. Its translation module
  sits in `src/generated/maps/`, beside the records'. Its drift test
  checks the module; a domain value taken to its record, to bytes and
  back is the value it was; a converter's refusal names its field.
- **The generator's tests:** each refusal of 6, on a maps text.
- **The merge job's test** with the real generator: two clean edits, to
  the fixture's schema and to its maps file, conflict in the translation
  module, and the job settles it (`tools/merge/README.md`).
- **Each owner's tests** of its translation stand: the same bytes, the
  same domain values, the same refusals at its entries.

## 9. Later and open questions

- **The wires' translation** takes this format once their versions
  freeze (`temper/docs/design/draft/layouts-both-ways.md`), each end
  with its own maps file.
- **The grammar grows by count:** a by-hand shape that recurs across
  owners (a positional tuple, a split variant) earns a form of 2 when
  its count says so, not before.
- **Domain types that copy a record field for field** with no rule of
  their own, such as jig's component manifests in its records crate,
  could be the records themselves; each owner judges its own.
