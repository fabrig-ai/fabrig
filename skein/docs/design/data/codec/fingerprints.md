# Codecs from schemas: fingerprints

## 1. Fingerprints

- **Of the layout, not the names.** A declaration's fingerprint is a
  SHA-256 (lib's) over a canonical encoding of what its bytes are
  (1.1): record or enumeration, `versioned` and the family's version,
  each field's type in order (its kind and width, a `fixed` length, a
  bound, `list` and `option`, text, bytes or secret, what a `bytes`
  holds), the variants' payloads in order, and each named type by its
  own fingerprint. Names, former names, defaults, comments, the text's
  layout and `boxed` are not in it. A record held from another family is
  named by its family and type, since its layout is that family's.
  These are the wires' and the patches' fingerprints, unchanged; a
  stored value's layout is 1.4's, which leaves the family's version out
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
  names are in its current schema (`README.md`, section 3).
- **A family's fingerprint is not a value type's layout.** The patch
  check judges changes by the diff and, for wires, the family
  fingerprint (`command.md`, 2). Registered values use 1.4's layout
  fingerprints, not kept versions of the family.

### 1.1 The encoding

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

### 1.2 The interface

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

### 1.3 Tests

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

### 1.4 A value type's layout

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
  does not give is encoded as 1.1 names it, by family and type. A held
  text's own holds resolve among the same texts, to a depth of
  `MAX_HELD`, 4; deeper is refused (`LayoutError::Deep`).
- **Everything else as 1.1:** a layout's fingerprint equals another's
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
1.3's: a versioned root's layout equal across its family's versions; a
restricted variant's layout unmoved by a change to a sibling
variant's record and moved by its own; a broad root without a variant
moved by either; a held record's change moving
its holder's layout when its family is given and not when it is not;
defaults and former names moving nothing; golden digests for the
fixture that holds every type, as 1.3's.
