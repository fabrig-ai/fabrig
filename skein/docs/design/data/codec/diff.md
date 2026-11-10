# The diff

- **Two versions of a family into their changes,** in skein-codegen's
  `diff`, ordinary Rust: the latest kept text and the current one, both
  read by `schema` (`README.md`, section 7). Each change names its declaration and
  its field or variant, has a kind (3), and is one line of text
  (2), the operation a patch spells (`patches.md`) and the command
  writes (`command.md`).
- **The plan's matching and change list:** the diff matches, compares
  and lists through `skein_codec::patch` (`patches.md`, 6), so one definition says
  what a compatible change is. The diff spells that list, places the
  renames among its lines (2) and proposes renames (1).
- **Matched by name** (1): declarations, then the fields of matched
  records and the variants of matched enumerations, the renames its
  author confirmed first. A rename is proposed where one leaves and
  another of the same layout takes its place; until its author confirms
  it, the diff holds it for a field dropped and another added.
- **Changes through a type are carried.** A declaration whose only
  change is in a type it names, in a field, a list, an option, a
  variant or pieces, has no change of its own: the named type's
  conversion reaches it wherever it is held (`patches.md`).
- **Every change is listed,** of every kind, so that a patch's
  operations turn the old layout into exactly the new one (`patches.md`),
  and the command learns from the kinds whether a patch is needed at
  all (`command.md`).
- **Not changes:** `boxed`, comments and the text's layout, which are
  not in the layout, and the declarations' order, which no byte depends
  on. A text whose declarations only moved has another family
  fingerprint and no change: the command refreshes the kept text, as
  for a rename.

## 1. Matching

- **Confirmed renames first,** each pairing a kept declaration, field
  or variant with a current one; then what is left pairs by name.
  A rename names the old declaration and the new; a field's or a
  variant's, its declaration by its current name, its old name and its
  new. A rename is refused that names what its text lacks, a name a
  second time, or a part of a declaration not matched.
- **Unmatched are added and dropped:** a current declaration, field or
  variant matched to none is added, and a kept one matched to none is
  dropped.
- **Proposals:** the unmatched kept and the unmatched current, each in
  its text's order, pair one to one while both have one left: a pair of
  the same layout is a proposed rename, for its author to confirm. Two
  fields have the same layout when their types compare equal (3),
  two variants when their records match or both have none, two
  declarations when they are of one shape, `versioned` alike, with as
  many fields or variants, pair for pair of the same layout. A field's
  or a variant's rename is proposed in a matched declaration only, so
  declarations' renames are proposed, and confirmed, first.

## 2. The changes

| Line | The change | Kind |
|---|---|---|
| `add D` | declaration `D` added | none needed |
| `drop D` | declaration `D` dropped | none needed |
| `rename D E` | declaration `D` renamed `E` | none needed |
| `reshape D` | `D` a record in one text and an enumeration in the other | written |
| `versioned D`, `unversioned D` | record `D` made `versioned`, or no longer | derived |
| `add D.f T v` | field `f` of type `T` added, every old record taking `v` | derived |
| `drop D.f` | field `f` dropped | derived |
| `rename D.a b` | field or variant `a` renamed `b` | none needed |
| `widen D.f T U` | field `f`'s type `T` widened to `U` | none needed, or derived when an integer is wider |
| `wrap D.f T U` | field `f`'s type `T` wrapped in `option` to `U` | derived |
| `retype D.f T U` | field `f`'s type changed otherwise | written |
| `order D a b …` | the fields or variants kept, now in this order | derived |
| `add D.v` | variant `v` added | none needed at the end, derived before a kept one |
| `drop D.v` | variant `v` dropped | written |
| `payload D.v P Q` | variant `v`'s record `P` made `Q`, `-` for none | written |

- **Names:** each line names a declaration, field or variant by its
  current name; a rename's first name and a drop's are the kept ones.
  `D.f` is a field or variant `f` of declaration `D`.
- **Types** are spelled as the schema spells them (`README.md`, 7.1), outermost
  first, a named type and a held record by their names: `T` with the
  kept text's names and `U` with the current's. A type ends where the
  grammar says, so a line needs no other marks.
- **An added field's value** is `none` for an `option`, which needs no
  more; otherwise `?`, asked of its author: no value is invented
  (temper's `draft/upgrades.md`, 4.3). The other forms of a value are
  the patch text's (`patches.md`).
- **Order:** the changes of each declaration in the current text's
  order, then the declarations dropped, in the kept text's. Within a
  declaration: its rename; `reshape`, after which its parts are not
  compared, or `versioned`; then each current field or variant in
  order, with its rename, its change of type or payload, or its `add`;
  then the kept ones dropped, in order; then `order`. Proposals follow
  the same order.
- **`order`** lists the fields, or the variants, matched in both texts,
  by their current names in the current order, when their order differs
  from the kept text's; an added one takes its place from the current
  text, and needs no `order`.

## 3. The kinds

- **None needed:** every old value is a new value, byte for byte, and
  means what it did. A declaration added, dropped or renamed; a field
  or variant renamed; a variant added after every kept one; a `widen`
  with no integer wider. A declaration dropped this way leaves its old
  records to their store: a root's records are the store's to remove, a
  change of keys, which its owner writes (temper's `draft/upgrades.md`,
  4.3). The family keeps its version, and the command refreshes the
  kept text (`command.md`).
- **Derived:** every old value becomes exactly one new value from the
  change alone, its author giving at most an added field's value: a
  field added or dropped, fields or variants reordered, a variant added
  before a kept one (later tags move), a `widen` with an integer wider
  (its bytes move), a `wrap`, a record made `versioned` (the family's
  version put before it) or no longer (that version taken away).
- **Written:** anything else; a written function, its author's, builds
  the new value (`patches.md`). A declaration with a written change is a
  written site, `written D` in a patch: its function builds the whole
  of its new value, and its other changes are listed beside it for that
  function's author. A `retype` may instead be its field's site,
  `written D.f` (`patches.md`, 3).
- **Types compare level by level,** outermost first, a named type equal
  to the one it matches (1) whatever that one's own changes:
  - **equal,** no change: the same levels, bounds and end;
  - **`widen`:** `U` holds every value of `T` once its integers are
    written at their new widths. The levels are alike (a `list` for a
    `list`, an `option` for an `option`), each bound the same or
    larger, and at the end an integer the same or wider, `text` made
    `bytes` or `secret`, `bytes` and `secret` made one another, a
    `bytes` clause (`holds` or `pieces`) added or taken away, or a
    named type matched;
  - **`wrap`:** `U` is `T` with an `option` at one or more levels where
    `T` has none, and is otherwise a `widen` of it. An `option` of `T`
    stands for the outermost `option` of `U` it can, so the one added
    is the one `T` has no level for;
  - **`retype`,** anything else, written: a bound narrowed, an `option`
    taken away, a kind changed (an integer made a `duration` or a
    `bool`, or back), a `fixed` length changed, `bytes` made `text`
    (old bytes need not be UTF-8), a `bytes` clause naming another
    family, type or declaration, a named type matched to another.

## 4. The version

The kinds decide the family's version, and the diff says whether the
current text's header follows:

- **No patch needed,** when every change is none needed: the family
  keeps the kept text's version.
- **A patch needed,** when a change is derived or written: the family
  moves to the kept version plus one, once per landing (temper's
  `draft/upgrades.md`, 4.2).
- **The verdict:** `Kept` (no patch, the version kept), `Moved` (a
  patch, the next version), `Unmoved` (a patch, the version kept),
  `Needless` (no patch, the next version), `Wrong` (any other). The
  command acts on it (`command.md`), deciding whether a patch is needed
  by `command.md`, 2's third step: `version()` alone holds where the kept text is
  not frozen, since under a frozen one a change none needed may need
  a patch.

A `versioned` record's layouts are compared without the version, which
is the family's: a version moved changes no record's line.

## 5. The interface

```rust
pub mod diff;  // skein_codegen::diff

pub fn diff(kept: &Schema, current: &Schema, renames: &[Rename])
    -> Result<Diff, DiffError>;          // skein_codec::schema::Schema

pub struct Diff {
    pub family: String,
    pub kept: u16,                       // the kept text's version
    pub current: u16,                    // the current text's version
    pub changes: Vec<Change>,            // in 10.2's order
    pub proposed: Vec<Rename>,           // proposed, not confirmed
}
pub struct Change { pub kind: Kind, pub op: Op }
pub enum Kind { NoneNeeded, Derived, Written }  // ordered so
pub enum Op {
    AddDeclaration { decl: String },
    DropDeclaration { decl: String },
    RenameDeclaration { old: String, new: String },
    Reshape { decl: String },
    Versioned { decl: String, versioned: bool },
    AddField { decl: String, field: String, ty: String, value: Fill },
    DropField { decl: String, field: String },
    RenamePart { decl: String, old: String, new: String },
    Widen { decl: String, field: String, kept: String, current: String },
    Wrap { decl: String, field: String, kept: String, current: String },
    Retype { decl: String, field: String, kept: String, current: String },
    Order { decl: String, parts: Vec<String> },
    AddVariant { decl: String, variant: String },
    DropVariant { decl: String, variant: String },
    Payload { decl: String, variant: String,
              kept: Option<String>, current: Option<String> },
}
pub enum Fill { Absent, Asked }          // `none`, `?`
pub enum Rename {
    Declaration { old: String, new: String },
    Part { decl: String, old: String, new: String },
}
pub enum Version { Kept, Moved, Unmoved, Needless, Wrong }
pub enum DiffError { Family, Rename(usize) }  // which rename

impl Diff {
    pub fn kind(&self) -> Option<Kind>;  // the most a change needs; None with none
    pub fn version(&self) -> Version;
    pub fn written(&self) -> Vec<String>; // the written sites, in order
}
```

- `Change`, `Kind`, `Fill` and `Rename` display as their lines and
  words: a change as its line (2), a kind as `none`, `derived` or
  `written`, a rename as its `rename` line.
- `Diff` displays as `family <family> <kept> to <current>`, then a
  line `<kind> <change>` per change and `proposed <rename>` per
  proposal.
- The types derive `Clone`, `Debug` and equality, and `Kind` its
  order. `DiffError::Family` is two texts of different families.

## 6. Tests

- **Step tests,** in skein-codegen, from both sides: for each line of
  2 and each kind it may have, a small pair of texts with that
  change beside the nearest that is not it (a variant appended and one
  inserted, a bound widened and one narrowed, a `wrap` and a `retype`),
  the whole output compared as text, so each line's form is pinned;
  proposals made and confirmed; each refused rename.
- **The story,** in `tests/codec`: a pair of fixture texts per kind,
  each holding every change of its kind and none of a higher one, and a
  pair with all three, their diffs kept as text beside them and naming
  each field.
- **Against the fingerprints:** every skein schema's diff against
  itself lists nothing; against its text with every name changed, given
  those renames, the renames alone, and without them, proposals for its
  declarations.
