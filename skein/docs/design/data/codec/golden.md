# Codecs from schemas: golden bytes and tests

## 1. Golden bytes and tests

- **One pin on the bytes.** A record's bytes are its fields' bytes in
  schema order, and a field's bytes follow from its kind alone, never
  from the family it is in. So one schema that uses every construct of
  the language (below), nested, pins every family's bytes, and each
  family's drift test proves that its committed code is the generator's
  output. That schema is skein's `every` fixture (`skein/tests/codec`):
  its current version holds every construct, and its earlier versions
  stay as the layout plans' pairs (`layouts.md`, 6). Its golden files and the
  `every` and `secrets` digest literals (`fingerprints.md`, 1.3) pin the bytes and fingerprints: a generator or `skein-codec`
  change that moves either fails there.
- **Fixed-rule values,** so the generator can change without moving
  them. For each record, and each variant of each enumeration, there are
  two:
  - **the smallest:** integers zero, bytes, text and lists empty, options
    absent, each enumeration at its first variant;
  - **a full one:** integers at their maximum, `fixed` bytes `0xa5`,
    bytes and text a pattern of up to three bytes, each list with one
    full item, options present.

  They are named `record_<record>_<smallest|full>.bin` and
  `enum_<enum>_<variant>_<smallest|full>.bin`. Their bytes come
  from the schema alone, not the generated code, so
  `every`'s golden files are the rule written down, and any test builds
  a schema's values at run time from its text, another layout's text as
  the current one's.
- **`every`'s own tests:** its drift test compares its records' modules
  and its golden files with the generator's output, and each golden
  decodes by the generated code at the ceilings, measures its length,
  encodes back to its bytes, and decodes generically to the same bytes
  (`run-time.md`, 2.3).
- **The constructs,** the list the pin holds:
  - each kind (`u8`, `u16`, `u32`, `u64`, `bool`, `duration`, `fixed`,
    `bytes`, `secret`, `text`, `list`, `option`, a record, an
    enumeration) in each place: a record's field, a list's item, an
    option's value;
  - an empty record and a `versioned` one; a variant without a record,
    one with a record, and a `boxed` one;
  - a `bytes` and a `secret` that hold another family's type, and a
    `bytes` and a `secret` of pieces;
  - a field with a default, and a former name.

  The pin's check fails, naming each construct the list has that
  `every-v3` lacks. No family needs an unlisted-construct check: the
  list is the classifier's whole range, its matches over `Type`,
  `Shape` and `Held` tables have no rest arm, and the generator's tests
  hold each construct listed once. A new table variant fails
  skein-codegen's build until listed, then the pin's check until
  `every-v3` holds it. A held type's record or enumeration is its
  family's, unknown to the text (`run-time.md`, 1.2), so the list counts a hold per
  carrier; `every` holds both from `every-other`.
- **What each family keeps,** none of it reading a golden file of bytes:
  - **drift:** the generator run on the schema equals the committed
    records' module, and the family's **bounds file** equals the bounds
    its generated code computes.
    On a difference it fails, saying how to write both again, which
    the same test does when `SKEIN_CODEGEN_WRITE=1` asks it to. The test
    reads the owner's schema and the family's codec crate (`README.md`, section 2),
    writing the code where it reads it;
  - **the bounds file,** `bounds/<stem>.bounds` in the directory of the
    test world whose drift test checks the family
    (`skein/tests/codec/bounds/sample.bounds`; jig's shared
    `jig/tests/codec-families.rs` keeps its families' in
    `jig/tests/bounds/`; skein's fixtures that keep versions in their
    names, `every-v3.bounds`), is
    `bounds_text`'s (below): for each declaration of `DECLARATIONS`, in
    its order, one line with its encoded and decoded bounds at the
    ceilings and at each limit lowered to 3, then the family's, on the
    64-bit target the suites run on. Decoded bounds hold the generated
    types' sizes, which only the compiler knows, so the drift test writes
    the file, not the generator, and the merge job doesn't make it: a
    conflict in it is settled by the drift test in write mode. No test
    spells a bound, a type's size or a limit's name that the generated
    code or this file holds, and a hand edit of the file fails as one of
    the code does. A test keeps the bounds it pins at its owner's own
    limits, which are its owner's figures;
  - **fuzzing,** in the fuzzy suite, where the family has it: each
    top-level record's decoder over arbitrary bytes, and over the
    fixed-rule values with bytes changed, cut or grown. It never panics,
    and what decodes encodes back to the same bytes;
  - **memory:** each owner's suite includes `skein/testing/codec-memory.rs`
    by relative path, test code outside any crate. This harness holds
    the choice of limits under the test's cap, the largest clamp
    `Limits::at_most` (`README.md`, section 4) fits under it, the family
    check over all roots, and the round trip; a root's bounds at the
    ceilings are its bounds file's.
    It decodes each root's `largest` value, each enum variant, measuring
    the allocator's peak against the decoded bound (`README.md`, section 4).

  Values are built at run time from schema text (`values`), at given
  limits (`largest`), or by the test. Another layout's values are built
  from its text the same way.
- **Each text once.** An owner exports each family's current text as a
  public constant, `SCHEMA`, or `<STEM>_SCHEMA` where it owns several,
  a split schema's parts concatenated; jig's and temper's stores export
  theirs in `STORE_FAMILIES` and `OWNER_SET`. Tests name a text by it,
  never by its file.
- **No generated tests.** The generator writes no tests and no crate
  keeps a `tests/generated` target: a field's round trip, bounds and
  refusals are its kind's, which `every` and the generic decoder's
  comparison with the generated code (`run-time.md`, 2.3) hold for every family. A
  hand-written test pins what is its own, a projection or a
  translation, with values it builds.
- **No test target of a library that holds no other tests:** such a
  codec crate sets `[lib] test = false`, so a test build compiles its
  generated code once (`tools/crates/design.md`, 3.4); one whose tests
  need private items keeps them.
- **The generator's cost grows with its schema,** not with its square:
  what it needs of a type or a field (its declaration by name, whether a
  type holds bounds, a field's path) is found once per generation. The
  limits are checked against the ceilings once per family, where their
  owner makes them, and once by each worst case asked of them; never in
  each type or decoder (`README.md`, section 4).
- **Generated files** say so in their first line. They are left out of
  formatting, and pass the workspace's lints: the generator writes what
  the lints accept (functions short enough, no nesting to collapse), and
  an owner marks `boxed` an enum's variant the lints find too large
  (`README.md`, section 3). Two scoped expectations stand, emitted only where their
  lint fires: `too_many_lines` on a function of one line per field,
  limit or path (a long record's codec, `Limits::values`), and
  `struct_excessive_bools` on a record of more than three booleans,
  which it holds as named fields.
- **Nothing every change touches:** a generated module holds no array
  whose length counts the schema's items (a `fixed N` field's `[u8; N]`
  is its width), no item named by an ordinal, no position
  (`README.md`, section 4, "Stable under insertion") and no version in its
  file's name, and each declaration's code sits in its own place, with
  family-wide lists one entry a line. Its one line that every change of
  layout moves is `LAYOUT`, the family's fingerprint (`README.md`,
  section 4). So two branches that change different declarations of one
  family merge its generated module line by line but for that line; the
  merge job writes the module again from the merged texts
  (`temper/docs/design/protocol/tooling.md`, 4.1).
- **What tests read of the generator:**

  ```rust
  pub struct FixedValue { pub name: String, pub bytes: Vec<u8> }
  // the fixed-rule values, in declaration order, without writing code
  pub fn fixed_values(schema: &Schema) -> Vec<FixedValue>;
  pub enum Construct { /* one of the list; its Debug names it */ }
  pub const CONSTRUCTS: &[Construct];  // the list the pin holds
  // the constructs a schema text uses, each once
  pub fn constructs(text: &str) -> Result<Vec<Construct>, Error>;
  // those of CONSTRUCTS the text lacks
  pub fn missing(text: &str) -> Result<Vec<Construct>, Error>;
  // the declarations no other declaration names, in declaration order
  pub fn roots(schema: &Schema) -> Vec<String>;
  // the bytes of a declaration's largest value at these limits,
  // given in LIMIT_NAMES order
  pub fn largest(schema: &Schema, declaration: &str, limits: &[u32])
      -> Result<Vec<u8>, Error>;
  // an enumeration's, once per variant in order; a record's, once
  pub fn largest_variants(schema: &Schema, declaration: &str, limits: &[u32])
      -> Result<Vec<Vec<u8>>, Error>;
  // made once per text for the process; a text that fails panics
  pub struct Values { pub schema: Schema, pub fixed: Vec<FixedValue> }
  pub fn values(text: &'static str) -> &'static Values;
  // a family's bounds file, from its generated DECLARATIONS and
  // CEILINGS.values(); a bound the walk can't compute panics
  pub fn bounds_text(declarations: &[skein_codec::bounds::Declaration], ceilings: &[u32])
      -> String;
  ```

  The largest value: each list at its count limit, each item, option
  and field at its largest, each bytes, secret and text at its length
  limit, integers at their maximum, each enumeration at the variant
  whose encoding there is longest, the first of equals. Its bytes are
  computed from the schema alone, as the fixed rule's are.

- **The generator's own tests:** the schemas it refuses (a type used
  before it is declared, a name used twice, a worst case too large);
  small schemas' encodings and largest values against bytes written by
  hand; each construct found in a small schema and in `every`'s text,
  and `missing` naming one taken out of it; a declaration inserted into
  a schema, and its line into a maps file, leaving every line of the
  modules generated before in the new ones, in order, the lines added
  being the inserted declaration's (`README.md`, section 4, "Stable
  under insertion").
