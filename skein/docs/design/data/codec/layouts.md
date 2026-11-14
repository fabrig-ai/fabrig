# Layouts and derived plans

How a value written at one layout of its value type is read at another,
with no patch file and no generic value: a plan derived from the two
layouts' texts, run as a pass over the value's bytes. A store that names
each value's layout (jig's `protocol/store/layouts.md`) builds one plan
per layout it holds, at open, and runs it on each value read at that
layout. The patch machine (`patches.md`) stays for the families a store
still patches and for the written jobs that need a generic value.

- **A value type's layout** (`README.md`, 9.4): a root declaration, at
  one variant where the root is broad, its fingerprint without the
  family's version and with what it holds resolved.
- **Derived from the two texts** (2): the old layout's text and the
  new one's, each with the texts of the families its values hold. The
  matching is the diff's (`diff.md`, 1), former names its confirmed
  renames; each change met in the root's reach is one of `diff.md`, 3's
  kinds, and a plan exists when none is written and every field added
  has a default.
- **A pass over the bytes** (3): copy, insert a default's bytes, skip a
  dropped field, widen an integer, write a moved tag or a new head,
  convert a held record or a piece in its length. No generic value, no
  index of nodes, no machine: one pass, its stack as deep as the
  layouts' nesting.
- **Step code,** in `skein-codec`'s `layout` module, bounded by its
  texts and the value's room (5).

## 1. Two layouts of one value type

A plan converts from an old root, `Root { decl, variant }` of the old
text, to a new root of the new text (`README.md`, 9.4). Its caller says
which: the root it stores beside each layout and the root its binary's
text names, both by its own table of value types. The two roots match
as the diff matches declarations (2): by name or a former name, the new
variant's former names among them; roots that do not match have no
plan (`Underived::Root`).

Each text comes with the texts of the families it holds (`README.md`,
9.4: `held`), the old with those its layout was made with, the new with
the binary's. A held record whose family is given on both sides is
converted by its own plan (2); one given on neither is copied as it is;
one given on one side alone has no plan (`Underived::Held`).

## 2. The plan

- **Matching:** declarations, then the fields of matched records and the
  variants of matched enumerations, as `diff.md`, 1, each part's former
  names (`README.md`, section 3) its confirmed renames. Only what the
  root's reach holds is matched and judged: a sibling variant of a broad
  root, and the declarations no value of the root can hold, are not, so
  a variant dropped beside the value type's own is no change of it.
- **Derived, and planned:** each change `diff.md`, 3 calls none needed or
  derived: a field added (its bytes the new text's default, `README.md`,
  7.3: `default`), dropped, or moved; a variant added after every kept
  one, or before one (its tags move); a bound widened; an integer widened
  (its bytes move); an `option` wrapped around a type (its presence put
  before it); a record made `versioned` (its head put before it) or no
  longer (its head taken away); a `bytes` clause added or taken away, as
  a `widen`.
- **Not derived,** and so not planned (`Underived`, naming the part by
  its new text's names, or its old text's for what it drops): a field
  added with no default (`NoDefault`), a `retype`, a `reshape`, a
  variant of the root's reach dropped or its record changed for another
  (`payload`), a held record's family given on one side alone, a pieces'
  item type no plan reaches. These are written changes: a store converts
  them by its owner's written job (jig's `protocol/store/jobs.md`, 4),
  never at read.
- **What a plan keeps** for its pass, per old declaration of the reach:
  its new declaration; per new field, its old field and that field's
  conversion, or the default's bytes; per old variant, its new tag; per
  type, its conversion level by level (as it is, widened, wrapped,
  named, held, pieces); and whether the declaration's conversion
  **keeps**: every old value of it is, byte for byte, a value of the new
  one, with no head moved (`patches.md`, 3.1's `keeps`, nested heads
  included). A root that keeps converts by its caller's head alone.
- **Pieces** are copied as their bytes where their item's conversion
  keeps, and opened otherwise, each piece converted as its item type
  and written with its new length, as `patches.md`, 4 opens them.
- **Held records** are converted by the held text's plan from the old
  held text to the new one, at the held type's own root, and keep only
  where that plan keeps; the held record's versioned head is written at
  the new held text's version.

## 3. The pass

`convert` reads the old value's bytes once, front to back, and writes
the new value's into the caller's buffer:

- **The root's head:** a versioned root's first two bytes are passed
  over, and the new text's version written in their place; the caller
  maps that head to its own (jig's `protocol/store/layouts.md`, 3). A
  nested versioned record's head is the new text's version too.
- **A record:** each old field's span is found in order (its bytes read
  as its type's decoder would, lengths and counts checked against what
  is left), then the new fields written in the new order, each from its
  old field's span by its conversion or as its default's bytes; a
  dropped field's span is skipped. A record's spans sit in a table the
  plan sizes by the most fields any record of the reach has, one table
  per level of the stack.
- **An enumeration:** the old tag read, its new tag written, then its
  record converted; a tag the old text lacks is `Misconverted::Tag`.
- **A list:** its count, then each item converted; an option: its
  presence (0 or 1, else `Tag`), then its value.
- **A widened integer** is read at its old width and written at its new
  one, big-endian.
- **A held field or opened pieces:** a placeholder for the length, the
  content converted, then the length written back and held to the new
  bound (`Misconverted::Bound`).
- **A part whose conversion keeps** is copied as its span at once.
- **The end:** bytes left over after the root are `Trailing`; a
  value shorter than a read is `Short`; a `bool` past 1 is `Bool`. The
  pass reads lengths, counts, tags and presences; it checks no text's
  UTF-8 and no content against its bound beyond what finding the span
  needs, since the new value's decoder checks every field of what the
  pass writes.

Nothing recurses: a list's items, a record's fields and a held record's
reach are entered on an explicit stack whose depth the plan computes
from the reach's nesting, and refused past `MAX_DEPTH` when planned.

## 4. The interface

```rust
pub mod layout;  // skein_codec::layout

pub const MAX_DEPTH: u32 = 64;
pub struct Plan { /* owned tables, built from the texts, holding no reference to them */ }
pub struct Underived { pub why: Why, pub decl: Span, pub part: Option<Span>, pub old: bool } // a span of the new text, or of the old when `old`
pub enum Why { Root, NoDefault, Retyped, Reshaped, VariantDropped, Payload, Held, Pieces, Deep }
pub enum Misconverted { Short, Tag, Bool, Bound, Trailing, Room }

impl Plan {
    pub fn derive(
        old: &Schema, old_root: Root, old_held: &[&Schema],
        new: &Schema, new_root: Root, new_held: &[&Schema],
    ) -> Result<Plan, Underived>;
    pub fn keeps(&self) -> bool;                       // the root keeps: its head alone moves
    pub fn convert(&self, old: &[u8], out: &mut Buffer) -> Result<(), Misconverted>; // lib's Buffer, cleared first
    pub fn workspace_bound(&self) -> Option<u64>;      // its stack's tables
}
```

- `derive` judges what both roots reach and nothing else, so its cost
  is that reach's parts, not the texts'; the plan holds no reference to
  either text.
- `convert` writes into lib's reusable `Buffer`
  (../../runtime/lib.md, section 6), whose capacity bounds the new
  value (`Room` past it): its caller gives the most bytes it admits for
  a value, so a converted value past its store's bound is refused, not
  written past it.
- The types derive `Clone` and `Debug`; `Underived`, `Why` and
  `Misconverted` equality besides. A plan with the same two layouts on
  both sides is the identity, keeps, and converts by the head alone.

## 5. Bounds

- **A plan's tables** are per declaration and part of the reach, at most
  `MAX_DECLARATIONS` per text it reaches, and the defaults' bytes, the
  new text's pool's; its stack is `MAX_DEPTH` levels, each a record's
  span table at the reach's most fields.
- **A pass's output** is the caller's `Buffer`, within its capacity; it
  allocates nothing else.

## 6. Tests

- **Step tests,** in `skein-codec`, from both sides, one small pair of
  texts each, the converted bytes compared with bytes written by hand:
  each derived change of 2 beside the nearest that is not derived (a
  field added with a default and one without, a variant appended and one
  dropped, an integer widened and one narrowed, a `wrap` and a
  `retype`), a broad root whose sibling variant changed and keeps, a
  held record converted in its length, pieces copied and opened, former
  names matched, each `Underived` and each `Misconverted`.
- **The machine as oracle,** in the fuzzy suite: for each pair of
  `tests/codec-patches`' fixture texts whose patch is derived, values
  drawn from the old text by the seeded generator of `README.md`, 8.3,
  converted by the plan and by the patch machine, give the same bytes;
  arbitrary bytes and goldens with a byte changed, cut or grown are
  refused by the pass, or converted to bytes the new text's generic
  decoder takes or refuses as it would any.
- **Keeps:** a plan that keeps converts each drawn value to itself but
  for its head.
- **Memory:** a pass allocates nothing past its buffer, counted by the
  counting allocator.

## 7. Open questions

- **Written changes at read:** a written change converts by its owner's
  job at open (jig's `protocol/store/jobs.md`, 4), since step code calls
  no function an owner hands in; a later design may run a written site
  at read where its owner reads the value itself.
