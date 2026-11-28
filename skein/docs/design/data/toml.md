# TOML

Provisional, 2026-11-28. The design of `skein-toml`: a bounded reader of
one whole TOML document into a compact document of tables, keys and
values, each with its line, refusing by line what isn't TOML. It is step
code under the strict subset (`../../foundation/programming-model.md`,
10) and depends on lib only, so that a service's steps read the files a
person writes, where no crate from outside skein may run. Its first
reader is temper's, for a repository's `.temper/`
(`temper/docs/design/protocol/temper-dir.md`).

## 1. In one page

- **TOML 1.0, whole** (2): every form the TOML 1.0 specification
  defines reads, and anything else is refused. No subset, and no
  extension.
- **One shot** (3): one whole borrowed input in, a `Document` or one
  `Refusal` out, nothing partial.
- **Values as the specification types them** (4): strings decoded,
  integers as `i64`, booleans; floats and dates or times as their checked
  text, which a reader that wants one parses, as `skein-json` hands up
  its numbers (`json.md`, 3.5).
- **Lines** (5): each key and each value knows the line it starts on, so
  a reader says `FILE:LINE: why` for what it refuses in a value it read.
- **Bounded** (6): the input's bytes, nesting, entries and a key's
  segments are under the caller's limits; past one is a refusal, with
  its line.
- **Decoding is the reader's** (7): the document says what the file
  says; which keys a file may hold, and what each means, is its
  reader's.

## 2. What it reads

- **The specification is TOML 1.0** (toml.io, v1.0.0): UTF-8 text; LF or
  CRLF line ends; comments; bare, quoted and dotted keys; basic,
  literal and both multi-line strings with every escape; decimal,
  hexadecimal, octal and binary integers with underscores; floats with
  `inf` and `nan`; booleans; offset and local date-times, local dates
  and local times; arrays, mixed and multi-line, with trailing commas;
  inline tables; tables; arrays of tables.
- **Its rules are refusals:** a key defined twice, directly or through
  dotted keys; a table header given twice, or naming a table that dotted
  keys or an inline table already defined; an inline table or a static
  array extended later; an array of tables over a key that holds another
  value; a control character in a comment or a string where the
  specification forbids one; an integer past `i64`; a date that doesn't
  exist (a 30 February, a month 13, a second past 60).
- **One leading byte order mark** (U+FEFF) is skipped, as editors write
  one; anywhere else it is a character like another.
- **Nothing is read in part:** a document with one fault anywhere is
  refused whole, at its first fault in the input's order.

## 3. The one-shot read

```rust
skein_toml::parse(input: &[u8], limits: &Limits) -> Result<Document, Refusal>
```

- **The `Document`** is its root table. A table holds its entries in
  the order they first appear in the input, each a key (one segment,
  decoded), the line it starts on and its value; a table reached
  through a header or a dotted key is an entry of its parent as an
  inline one is, so a reader walks one tree whatever the file's
  spelling. A reader looks an entry up by its key or walks them in
  order; an array's values are read by index, in order.
- **Compact:** one buffer of decoded text, and fixed-size records of
  tables, entries and values pointing into it, as `json.md`, 5.2 keeps a
  document: no box per value.
- **A `Refusal`** is the line, from 1, of its first fault and why, an
  enum: the input not UTF-8; a form the grammar doesn't take; a key
  redefined; a table redefined; a value past its form's range; a bound
  passed (6), naming which. The caller words it: skein says no file's
  name.
- **Bounded steps:** its work is bounded by the input's length and its
  limits; it recurses nowhere, nesting held in a `lib::Stack` of the
  configured depth.

## 4. Values

| TOML | In the document |
|---|---|
| string, of any of the four forms | its decoded bytes, UTF-8 |
| integer | `i64` |
| float | its text, checked, underscores kept |
| boolean | `bool` |
| offset date-time, local date-time, local date, local time | its kind and its text, checked |
| array | its values, in order |
| inline table, table | a table (3) |
| array of tables | an array of tables |

- **Kinds are kept apart:** a table from a header and one from an
  inline table read alike (3), and a reader that cares can ask which.
- **No float arithmetic:** skein parses no float, as `skein-json` doesn't
  (`json.md`, 7); a reader that takes one parses its text.

## 5. Lines

- **Each entry keeps the line its key starts on, and each value the line
  it starts on.** A table from a header keeps its header's line. A
  multi-line string or array starts where it opens.
- **A refusal's line** is the line where its fault is seen: the second
  definition of a key, the byte that breaks the grammar.

## 6. Limits

The caller's, each a whole number at least 1:

| Limit | Bounds | Past it |
|---|---|---|
| `input_bytes` | the input's length | refused at line 1 before anything is read |
| `depth` | arrays and inline tables nested, and a key's dotted segments, together | refused at the line that passes it |
| `entries` | entries and array values in the whole document | refused at the line that passes it |

- **Decoded text never outgrows the input:** every escape shrinks or
  keeps its bytes, so the text buffer is bounded by `input_bytes`.
- **A document's size** is bounded by its limits alone, so a caller
  sizes what it keeps before it reads.

## 7. Decoding is the reader's

- **skein reads TOML; a reader reads its file.** Which keys a file holds,
  which it refuses as unknown, its defaults and what a value means are
  the reader's, as a JSON reader's documents are its own (`json.md`, 5).
- **A reader's refusal** names the entry's line from the document, so
  every refusal a person meets names a line, whether skein or the
  reader made it.

## 8. Testing

Focused, with fuzzy where it says (`../testing/README.md`).

- **Every form reads:** a table of documents, one or more per form of
  section 2, each with the document it reads to: keys, values, kinds,
  order and lines.
- **Every rule refuses:** a table of documents, one per refusal of
  sections 2, 3 and 6, each with its line and its reason.
- **Round trip, fuzzy:** any document of a model within the limits,
  written as TOML in any of the spellings the specification allows for
  it (headers, dotted keys or inline tables; any string form; any
  integer base), reads to the model, its lines where the writer put
  them.
- **Any bytes, fuzzy:** any input within `input_bytes` reads or is
  refused, within the step bound, never past a limit.

## 9. Open questions

- **TOML 1.1** reads as 1.0 for now: a newer form (a newline in an inline
  table, `\e`) is refused until a reader needs it.
