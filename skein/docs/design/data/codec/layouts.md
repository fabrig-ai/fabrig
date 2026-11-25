# Layouts and derived plans

How a value written at one layout of its value type is read or kept at
another, with no generic value: a plan derived from the two layouts'
texts, in one of two modes, run as a pass over the value's bytes that
gives the other layout's bytes or refuses that value. It is the one
conversion between layouts, for the store and every wire. A plan has no
notion of older or newer: its old text is the **source**, the layout
the bytes are at, and its new text the **target**, the layout its
caller wants. A store that names each value's layout (jig's
`protocol/store/layouts.md`) builds one plan per layout it holds, at
open, and runs it on each value read at that layout, its transcripts'
frames among them. An engine whose worker speaks another layout of a
wire builds a plan for each value type the wire carries, each way, at
the connection's opening, and runs one on each message (jig's
`protocol/channel/layouts.md`). What no plan derives is its owner's: a
store's written job (jig's `protocol/store/jobs.md`, 4), or a wire's
written converter (jig's `protocol/channel/layouts.md`, 6).

- **A value type's layout** (`fingerprints.md`, 1.4): a root declaration, at
  one variant where the root is broad and its keys tell its variants
  apart, its fingerprint without the
  family's version and with what it holds resolved.
- **Derived from the two texts** (2): the source's text and the
  target's, each with the texts of the families its values hold. Parts
  match by their names and former names (2); each change met in the
  root's reach is judged by 2's rules, and either planned, planned with
  a refusal per value, or not derived.
- **Two modes** (2): **Read**, where the target uses the value as its
  own and skips what it lacks; **Keep**, where the value must mean the
  same at the target, so what the target lacks is dropped only at its
  default or where the source marks it `droppable`. The store's plans
  are the Read mode's that refuse no value.
- **Refused per value** (3): which values a plan refuses is known only
  by running it. The pass refuses a value naming its declaration and
  part by the source text's names, as a decoder's refusal does.
- **A pass over the bytes** (3): copy, insert a default's bytes, skip a
  dropped field, widen or narrow an integer, write a moved tag or a new
  head, convert a held record or a piece in its length. No generic
  value, no index of nodes, no machine: one pass, as deep as the
  layouts' nesting.
- **Step code,** in `skein-codec`'s `layout` module, bounded by its
  texts and the value's room (5).

## 1. Two layouts of one value type

A plan converts from a source root, `Root { decl, variant }` of the old
text, to a target root of the new text (`fingerprints.md`, 1.4). Its caller says
which: the root it stores beside each layout and the root its binary's
text names, both by its own table of value types. The two roots match
as 2 matches declarations: by name or a former name, the new
variant's former names among them; roots that do not match have no
plan (`Underived::Root`). A broad root without a variant (`fingerprints.md`,
1.4) matches only another without one, and is planned as the
declaration whole, every variant of its reach judged; one at a variant
and one without never match.

Each text comes with the texts of the families it holds (`fingerprints.md`,
1.4: `held`), the source with those its layout was made with, the target with
the caller's. A held record or enumeration (2: Held records) whose family is given on both sides is
converted by its own plan (2); one given on neither is copied as it is;
one given on one side alone has no plan (`Underived::Held`).

The plan depends on the two texts, not their fingerprints alone:
defaults, former names, tombstones and `droppable` marks are in no
fingerprint (`README.md`, section 3; `fingerprints.md`, section 1), so a
holder that keeps plans for reuse keys them by the two texts.

## 2. The plan

- **Matching:** declarations, then the fields of matched records and the
  variants of matched enumerations. A part matches the other text's
  part of its own name; failing that, the part one of its former names
  (`README.md`, section 3) names; and, in `between` (4), failing that,
  the other text's part whose former names name it, so that a plan from
  a text that renamed a part back to one that did not matches as the
  plan the other way. In `between`, a pairing found through a former
  name holds only where matching from the other side finds the same
  pair, so a part with several names in its history pairs one way or
  not at all. `derive` reads the target's former names alone. A part
  matched to none is added, or dropped. A field of
  either text that no field of the other matches is matched, the same
  way, against the other record's tombstones (`README.md`, section 3),
  in both modes alike, so the two modes plan the same pairs. Each pair
  of texts a plan reaches has every target declaration matched by
  name; only what the root's reach holds has its parts matched, and
  is judged: a sibling variant of a broad root, and the declarations no
  value of the root can hold, are not, so a variant dropped beside the
  value type's own is no change of it. Comments, `boxed`, the text's layout and the declarations'
  order are in no layout and change nothing.
- **Types compare level by level,** outermost first, a named type equal
  to the one it matches whatever that one's own changes:
  - **equal:** the same levels, bounds and end;
  - **widened:** the levels alike (a `list` for a `list`, an `option` for
    an `option`), each bound the same or larger, and at the end an
    integer the same or wider, `text` made `bytes` or `secret`, `bytes`
    and `secret` made one another, a `bytes` clause (`holds` or
    `pieces`) added or taken away, or a named type matched;
  - **wrapped:** an `option` at one or more levels where the source has
    none, and otherwise widened; a source `option` stands for the
    outermost target `option` it can, so the one added is the one the
    source has no level for;
  - **retyped,** anything else: a bound or an integer narrowed and an
    `option` taken away, which the table below plans with a refusal per
    value; and a kind changed (an integer made a `duration` or a `bool`,
    or back), a `fixed` length changed, `bytes` made `text` (bytes need
    not be UTF-8), a `bytes` clause naming another family, type or
    declaration, or a named type matched to another, which no plan
    derives.
- **Planned in both modes,** every source value becoming one target
  value from the change alone: a field added, dropped, or moved; a
  variant added after every kept one, or before one (its tags move); a
  bound widened; an
  integer widened (its bytes move); an `option` wrapped around a type
  (its presence put before it); a record made `versioned` (its head put
  before it) or no longer (its head taken away); a `bytes` clause added
  or taken away, as a widening.
- **Each change between source and target, by mode:**

| Between source and target | Read | Keep |
|---|---|---|
| A field only the source has | skipped | dropped where its bytes are its compared default's, or where the source marks it `droppable`; otherwise the value is refused (`Refused::Field`) |
| A field only the target has | the target's default, else the source's tombstone's default converted to the target's layout (below); with neither, no plan (`NoDefault`) | the same |
| An integer narrower at the target, or a bound narrower (a list's count, a `text`'s, a `bytes`' or a `secret`'s length) | converted where the value fits; otherwise the value is refused (`Refused::Narrowed`) | the same |
| A variant of the reach only the source has | a value holding it is refused (`Refused::Variant`) | the same |
| A variant only the target has | its tags mapped by name | the same |
| An `option` at the source, its type at the target | a present value converted; an absent one takes the target field's default where the `option` is the field's outermost level and the field has one, and is refused otherwise (`Refused::Absent`) | a present value converted; an absent one refused (`Refused::Absent`) |
| A former name | a rename confirmed | the same |
| Any other retyping (above); a declaration a record in one text and an enumeration in the other (`Reshaped`); a variant's record changed for another (`Payload`); a held record's family given on one side alone (`Held`); a pieces' item type no plan reaches (`Pieces`) | no plan (`Underived`) | the same |

- **A field's compared default,** in Keep, is the value the way back
  writes for it: the source field's own default, else the target
  record's tombstone's default converted to the source's layout, as
  the Read plan from the target to the source converts it. A source
  field the target lacks, with no compared default and no `droppable`
  mark, refuses every value in Keep, and is still a plan. `droppable`
  is the source's mark alone, read in Keep alone.
- **A tombstone's default is at its own text's layout,** encoded as
  that text is read (`README.md`, section 3), so a plan converts it to
  the other text's layout before it uses it: a source tombstone's,
  written for a field only the target has, by the field type's
  conversion from the source to the target in the plan's mode; a
  target tombstone's, compared in Keep, from the target to the source
  as the way back's Read plan converts it. It converts as one value of
  its type: a part of the type the default does not hold (a sibling
  variant, an `option`'s value where it is `none`) refuses nothing, in
  `derive` too. A tombstone stands for a field only where its type is
  the field's, level for level and bound for bound, and its default
  converts. Matched to a field of another type, it gives no plan in
  either mode; where its default does not convert (it holds a variant
  the other text lacks), it gives no plan where its default is used:
  for a field only the target has in both modes, and for a compared
  default in Keep. Either is `Retyped`, naming the field.
- **A round trip holds:** a value the Keep plan from a source to a
  target converts, converted back by the Read plan from that target to
  the source, is the value itself, but for the fields the source marks
  `droppable`, which come back at the value the way back writes. The
  compared default and Keep's refusal of an absent `option` the target
  takes away follow from it: writing the target's default there would
  come back present.
- **Not derived,** and so not planned (`Underived`, naming the part by
  the target text's names, or the source's for what it drops): the last
  row above. These are written changes: a store converts them by its
  owner's written job (jig's `protocol/store/jobs.md`, 4), never at read.
- **The store's derivation** (`derive`, 4) is the Read mode's, with
  every part a value could be refused at refused as the plan's instead:
  a variant of the reach the target drops is `VariantDropped`, a
  narrowed integer or bound and an `option` taken away are `Retyped`,
  each naming the part as before. Where it plans, its plan is the Read
  mode's, equal to it, and refuses no value. A store reads with it
  (jig's `protocol/store/layouts.md`), since a stored value no plan
  can convert is its owner's written job.
- **A declaration's conversion keeps** where every source value of it
  is, byte for byte, a value of the target, with no head moved, nested
  heads included. A root that keeps converts by its caller's head alone
  (`keeps`, 4). A declaration whose conversion can refuse a value does
  not keep, and neither does any declaration enclosing it in the reach,
  so no copy of a span passes over it.
- **Pieces** are copied as their bytes where their item's conversion
  keeps, and opened otherwise, each piece converted as its item type
  and written with its new length (3).
- **Held records** are converted by the held text's plan from the source
  held text to the target one, in the same mode, at the held type's own
  root, and keep only where that plan keeps; the held record's versioned
  head is written at the target held text's version. A held type may be
  an enumeration (`refusal: bytes 517 holds jig-store-people Refusal`):
  it is planned at its declaration whole, as a broad root without a
  variant (1), and has no head. A held type that is neither a record nor
  an enumeration has no plan (`Underived::Held`).

## 3. The pass

`convert` reads the source value's bytes once, front to back, and writes
the target value's into the caller's buffer:

- **The root's head:** a versioned root's first two bytes are passed
  over, and the target text's version written in their place; the caller
  maps that head to its own (jig's `protocol/store/layouts.md`, 4). A
  nested versioned record's head is the target text's version too.
- **A record:** each source field's span is found in order (its bytes read
  as its type's decoder would, lengths and counts checked against what
  is left), then, in Keep, each field the target lacks compared with its
  compared default's bytes, unless it is `droppable`, before anything
  of the record is written; then the target fields written in the
  target order, each from its source field's span by its conversion or
  as its default's bytes; a field the target lacks is skipped.
- **An enumeration:** the source tag read, its target tag written, then its
  record converted; a tag the source text lacks is `Misconverted::Tag`; a
  variant the target lacks is `Misconverted::Refused`, read before
  anything of it is written.
- **A list:** its count, then each item converted; an option: its
  presence (0 or 1, else `Tag`), then its value. An `option` the target
  takes away writes its value alone, or its field's default's bytes,
  as 2 says.
- **A widened or narrowed integer** is read at its source width and
  written at its target one, big-endian; one past the target width's
  largest value is refused before it is written.
- **A narrowed bound:** a list's count, or a `text`'s, a `bytes`' or a
  `secret`'s length, past the target's bound is refused as it is read,
  before its items or bytes.
- **A held field or opened pieces:** a placeholder for the length, the
  content converted, then the length written back and held to the
  target bound (`Misconverted::Bound`), narrowed or not.
- **A refusal** is `Misconverted::Refused(index)`, the plan's index of
  what refused it (`Plan::refusal`); the buffer holds nothing the caller
  may use.
- **A part whose conversion keeps** is copied as its span at once.
- **The end:** bytes left over after the root are `Trailing`; a
  value shorter than a read is `Short`; a `bool` past 1 is `Bool`. The
  pass reads lengths, counts, tags and presences; it checks no text's
  UTF-8 and no content against its bound beyond what finding the span
  and a narrowed bound need, since the target value's decoder checks
  every field of what the pass writes.

Nothing recurses, and a reach nested past `MAX_DEPTH` is refused when
planned (`Deep`). What the pass needs beside the plan is the caller's
`Workspace`, fitted to the plan before the pass, so the plan stays
shared and unchanged by a pass and the pass allocates nothing; a
workspace too small for the plan is refused at once (`Room`), before a
byte is read.

## 4. The interface

```rust
pub mod layout;  // skein_codec::layout

pub const MAX_DEPTH: u32 = 64;
pub struct Plan { /* owned tables, built from the texts, holding no reference to them */ }
pub enum Mode { Read, Keep }
pub struct Underived { pub why: Why, pub decl: Span, pub part: Option<Span>, pub old: bool } // a span of the target text, or of the source when `old`
pub enum Why { Root, NoDefault, Retyped, Reshaped, VariantDropped, Payload, Held, Pieces, Deep }
pub enum Refused { Variant, Field, Narrowed, Absent }  // 2's table: why a value is refused
pub enum Misconverted { Short, Tag, Bool, Bound, Trailing, Room, Refused(u16) } // Refused: the plan's index of its refusal

impl Plan {
    pub fn derive(                                     // 2: the store's, the Read mode refusing no value
        old: &Schema, old_root: Root, old_held: &[&Schema],
        new: &Schema, new_root: Root, new_held: &[&Schema],
    ) -> Result<Plan, Underived>;
    pub fn between(                                    // 2: either mode, refusing per value
        mode: Mode,
        old: &Schema, old_root: Root, old_held: &[&Schema],
        new: &Schema, new_root: Root, new_held: &[&Schema],
    ) -> Result<Plan, Underived>;
    pub fn refuses(&self) -> bool;                     // it may refuse a value
    pub fn refusal(&self, index: u16) -> Option<(Refused, &str, &str)>; // its kind, declaration and part, by the source text's names
    pub fn keeps(&self) -> bool;                       // the root keeps: its head alone moves
    pub fn convert(&self, old: &[u8], out: &mut Buffer, work: &mut Workspace)
        -> Result<(), Misconverted>;                   // lib's Buffer, cleared first; Room if `work` does not fit
    pub fn variant_tag(&self, old: u8) -> Result<u8, Misconverted>; // a whole enumeration's: a source tag's target tag
    pub fn convert_variant(&self, old: u8, payload: &[u8], out: &mut Buffer, work: &mut Workspace)
        -> Result<u8, Misconverted>;                   // a whole enumeration's: the payload alone into `out`, its target tag returned
    pub fn workspace_bound(&self) -> Option<u64>;      // what a workspace fitted to it holds, in bytes
}

pub struct Workspace { /* what the pass needs beside the plan, reused from pass to pass */ }
impl Workspace {
    pub fn new() -> Workspace;                         // empty: allocates nothing
    pub fn fit(&mut self, plan: &Plan) -> bool;        // grows to the plan's need, never shrinks; false: its bound overflows
    pub fn fits(&self, plan: &Plan) -> bool;
}
```

- A refusal's part is a field's name for `Field`, `Narrowed` and
  `Absent`, the field holding the level refused, and a variant's for
  `Variant`; its declaration is the record or enumeration holding it.
- A workspace is the caller's, beside its `Buffer`: one fitted to each
  of several plans in turn serves them all, so a store fits one to every
  plan it holds at open and allocates nothing per read. A pass leaves
  nothing in it the next pass reads.

- `derive` and `between` judge what both roots reach and nothing else.
  A plan's cost is what its derivation matches: every target
  declaration of each pair of texts it reaches, and the parts of the
  reach (2). The plan holds no reference to either text, and records no
  mode. The two modes' plans for a pair differ where Keep checks a
  source field the target lacks, and where Read takes the outer default
  for an absent `option` the target takes away and Keep refuses it;
  Keep has no plan where a target tombstone's default does not convert
  back (2). Elsewhere they are equal.
- `convert` writes into lib's reusable `Buffer`
  (../../runtime/lib-bytes.md, section 1), whose capacity bounds the new
  value (`Room` past it): its caller gives the most bytes it admits for
  a value, so a converted value past its store's bound is refused, not
  written past it.
- **A whole enumeration's tag apart from its payload:** for a plan whose
  roots are whole enumerations (`Root { decl, variant: None }`), where a
  frame carries the tag as its kind and the payload as its body,
  `variant_tag` maps a source tag as `convert` would: `Tag` for one the
  source text lacks, `Refused(index)` for a variant the target lacks.
  `convert_variant` is the same pass as `convert` over the tag followed
  by the payload, its tag returned rather than written: `out` holds what
  `convert` writes after the tag, bounded by `out`'s capacity alone, and
  the errors are the same. A plan whose root is no whole enumeration
  answers `Tag` to both.
- A plan with the same two layouts on both sides is the identity,
  keeps, and converts by the head alone.

## 5. Bounds

- **A plan's tables** are per declaration and part of the reach, at most
  `MAX_DECLARATIONS` per text it reaches, and the bytes of the defaults
  it writes and compares (2).
- **Its refusals** are at most `MAX_DECLARATIONS`, each named by a `u16`
  index; a reach with more is refused when planned (`Deep`).
- **A workspace** is the caller's, `workspace_bound` bytes for its plan,
  allocated by `fit` outside any pass.
- **A pass's output** is the caller's `Buffer`, within its capacity; it
  allocates nothing else, in the buffer, the workspace or elsewhere.

## 6. Tests

Which cases the tests take is the code's. The step tests in
`skein-codec` convert to bytes written by hand and pin each
`Underived`, each `Misconverted` and each row of 2's table in each mode;
the fuzzy suite of `skein-codec-tests` holds 2's round trip over the
ordered pairs of `every`'s texts (`golden.md`, section 1), which keep
their older versions as its pairs, and over pairs derived from them;
`temper-codecs-world` derives every registered family's texts at the
builds its deployed directories keep to the binary's.

## 7. Open questions

- **Written changes at read:** a written change converts by its owner's
  job at open (jig's `protocol/store/jobs.md`, 4), since step code calls
  no function an owner hands in; a later design may run a written site
  at read where its owner reads the value itself.
- **A store reading by plans that refuse:** the store reads an older
  layout by `derive`, so a value type whose Read plan would refuse a
  value still needs its owner's written job; it plans a later layout
  by `between` and checks every value at it at open (jig's
  `protocol/store/layouts.md`, 5.6). Whether an upgrade could do the
  same for an older layout, in place of a written job, is open.
