# Patches

- **From the baseline on,** a landing that changes the layout of a
  stored family brings a patch beside it, and a store behind its
  binary applies the patches between, in order. Until the baseline,
  nothing asks for one.
- **Files, in the owner's crate beside its schemas:**
  - `schema/kept/<family>-v<N>.schema`: every version's text from the
    baseline on, never edited once landed but by the command's refresh
    of names, while no landed patch names it (`command.md`); the latest
    has the current layout;
  - `patches/NNNN.patch`: the patch's text (1), numbered per owner
    from the baseline, its number the owner's level once it applies;
  - `patches/pNNNN.rs`: its author's written functions, if any (5);
  - `patches/context.rs`: the owner's `Context`, what its driver passes
    each written function (5), written once by the command, empty,
    and its owner's to fill and keep from then on;
  - `patches/mod.rs`, generated and drift-tested: the patches' texts,
    the dispatch of their written sites, and constants naming each kept
    version's fields and variants and each patch's refusals, so that a
    written patch that names one wrongly does not build. Its drift test
    regenerates it alone from the kept texts and the patches: the
    author's `pNNNN.rs` and `context.rs` are outside it;
  - `patches/golden/NNNN/`: the old version's goldens through the patch,
    reviewed once and kept.
- **The text is data,** read at run time as step code, so jig runs
  temper's derived patches from temper's texts without naming temper:
  its language's version; the owner's level and its upstreams' levels
  when written; marks for the caller; then for each family it changes,
  its versions, `diff.md`, section 2's lines as the diff prints them, the
  values its author gives added fields, and its written sites with the
  refusals each may give (1, 2).
- **Planned before it runs.** `plan` checks that a family's lines turn
  its old layout into exactly the new one, or names the line that does
  not (3), and keeps what the machine needs: which old declaration
  becomes which, each field's source, each value's encoding.
- **A machine that stops, never calls back.** A family's `Patcher`
  holds its kept schemas and planned patches, decodes an old record into
  a generic value, and walks it, building the new value in preorder in
  lib's reusable `Buffer` (../../runtime/lib.md, section 6). It stops where its caller
  must act, at a written site or a held field, and resumes on the
  caller's answer: no function pointers, and jig runs temper's patches
  without naming temper (4).
- **Written sites are types,** not only roots, so a patch of a type
  reaches it in lists, variants and options; or one field of a record.
  A written function reads the old value by `README.md`, section 8.1's accessors and
  builds the new by putting values, opening and closing nested ones, and
  carrying an untouched part through its derived conversion. It may
  refuse a record with a reason its patch's text declares; its caller
  names the record by its key, and the start stops (5).
- **Held fields:** at a `bytes` field that holds another family's
  record, the machine stops for its caller, which patches the held bytes
  by that family's machine, in the order the levels give, and hands the
  result back, or carries them as they are. A plan that changes nothing
  of its own family, an identity plan, reaches held fields alone.
- **Versions inside:** every versioned record a patch reaches,
  root or nested, has its version rewritten.
- **Bounded:** a machine's heap is a worst case over its capacity (its
  versions, patches, texts, a record's bytes and nodes). A value made
  with `with_room` grows within its entry cap; a patch's machine grows its own
  run's tables and keeps their room (7).

## 1. The text

```
0041.patch:

patch 1                                   # the language's version
owner jig 41 after smith 7 skein 3        # the owner's level, its upstreams'
mark fresh-windows                        # for the caller

family jig-store-tasks 9 to 10
rename Task.owner assignee
widen Task.attempts u8 u32
add Task.priority u8 0
add Task.note option text 64 none
add Task.score u32 written                # a site gives it
order Task id title assignee state attempts
written Task.score                        # site 0
refuses unscored "a done task without attempts"   # refusal 0
add State.paused
```

```
patch     = "patch" language NL owner NL mark* block*
owner     = "owner" word level ["after" (word level)+]
mark      = "mark" word NL
block     = "family" family version "to" version NL (line NL)*
line      = operation | site | refusal
operation = "add" name | "drop" name | "rename" name name
          | "reshape" name | "versioned" name | "unversioned" name
          | "add" part [type value] | "drop" part | "rename" part name
          | "widen" part type type | "wrap" part type type
          | "retype" part type type
          | "order" name name+ | "payload" part record record
site      = "written" name | "written" part
refusal   = "refuses" name text
part      = name "." name
record    = name | "-"
value     = "?" | "written" | "none" | "some" value | "[]"
          | word | text
```

- **Types** are `README.md`, section 7.1's `type`, ending where its grammar says.
- **Tokens:** schema words (`README.md`, 7.1), and `.` `?` `[` `]` as marks; a
  text, `"` then bytes of UTF-8 other than `"`, `\` and control bytes,
  then `"`, a `#` among them one of its bytes; `#` elsewhere starts a
  comment to the end of the line. Spaces and tabs separate tokens; a
  line's end ends a statement, and blank lines are none. Any other byte
  outside a comment is refused, before the grammar is read. `[]` is `[`
  then `]`.
- **Numbers:** a `language`, `version` and `level` are decimal words,
  a `u16`, `u16` and `u32`. The language is 1.
- **Lines** count from 1 as in a schema (`README.md`, 7.1); a refusal names the line
  of its token, and a change a block lacks (3) names the block's
  `family` line.
- **The header:** the owner's level is the patch's number; `after`
  lists each upstream's level when the patch was written (temper's
  `draft/upgrades.md`, 4.4), none for an owner without upstreams. A
  mark is a word the language gives no meaning: its caller's, such as
  smith's `fresh-windows` (temper's `draft/upgrades.md`, 7.2).
  `header` reads it alone (6), so that a caller reads the levels and
  marks of a patch that changes none of its families.
- **A block per family the patch changes,** once each, its versions
  kept ones, the second the first plus one. Its operations are `diff.md`, section 2
  's lines as the diff prints them without their kind, and a type
  is spelled as there: with the old text's names in `widen`, `wrap` and
  `retype`'s first type and `payload`'s first record, the new text's
  elsewhere, an added field's among them. `add part` and `drop part`
  are a field's when the part is a record's, a variant's when an
  enumeration's.
- **Sites and refusals are numbered** each in text order across the
  whole text, from 0, so a patch's code names them by its number and
  theirs (5). A `refuses` line belongs to the site line it follows;
  a site may give several, and each refusal's name is the patch's once.
- **Other families' blocks** are read for their grammar alone: each
  family's machine plans its own block (3).

## 2. Values

An added field's value is the diff's `?` answered by its author, or an
option's `none` kept or overridden. Each value is a form its type
takes, checked as the new schema's decoder checks it, and encoded once,
when the patch is planned:

| Type | Its value |
|---|---|
| `u8`, `u16`, `u32`, `u64`, `duration` | a decimal word within its width; a duration's nanoseconds |
| `bool` | `true` or `false` |
| `fixed N` | `0x` and exactly 2N lowercase hex digits |
| `bytes N` | `0x` and at most 2N lowercase hex digits, in pairs; `0x` alone is empty |
| `bytes N pieces T` | `0x` alone, no pieces |
| `bytes N holds F T` | none: a written site |
| `text N` | a text of at most N bytes |
| `secret N` | none: a written site |
| `list N T` | `[]`, the empty list |
| `option T` | `none`, or `some` and a value of `T` |
| an enumeration | a variant without a record, by its name |
| a record, or a variant with one | none: a written site |

- **`?` is refused** when planned: no value is invented (temper's
  `draft/upgrades.md`, 4.3). A field old records hold no evidence for
  is an `option`, and `none` for them.
- **`written`** says a site gives the value: `written D.f`, or
  `written D` when the whole declaration is written. So a value read
  per record from other data, or an option whose `none` would be wrong,
  is written.
- **The words `?`, `written`, `none` and `some`** are read as such
  whatever the type, so a variant so named is given by a site.
- **No secret in a text:** a secret's value is always written, so no
  credential is kept in a patch.

## 3. Planning

`plan` reads the text and plans its block for the machine's family,
against the two kept texts its versions name.

- **Matching,** as the diff's (`diff.md`, 1), with the block's renames as the
  confirmed ones: declarations, then the fields of matched records and
  the variants of matched enumerations.
- **Every change has its line,** and every line its change, so the
  lines turn the old layout into exactly the new one:

  | The change | Its line |
  |---|---|
  | a declaration, field or variant unmatched in the new text, or in the old | `add`, or `drop` |
  | a variant added after every kept one | `add D.v`, or none |
  | a matched declaration of the other shape | `reshape D`, with `written D`; its parts are not compared |
  | a record made versioned, or no longer | `versioned D`, or `unversioned D` |
  | a matched field's type, by `diff.md`, 3 | none when equal; `widen`, or none when no integer is wider; `wrap`; `retype`, with `written D` or `written D.f` |
  | a variant's record not the match of its old one | `payload D.v P Q`, with `written D` |
  | a variant dropped | `drop D.v`, with `written D` |
  | matched parts in another order | `order D`, naming them in the new order |

  A line's names and types must spell what the two texts hold. The two
  implicit changes, a variant added after every kept one and a `widen`
  with no integer wider, may have their lines or not; every other
  change needs its own, renames included.
- **A site** names a declaration of the new text matched to an old one
  (`written D`), or a field of such a record (`written D.f`): an added
  field whose value is `written`, or a matched field, whose derived
  conversion the site replaces. A declaration that is a site has no
  field site; its function gives each field, an added one with a value
  among them (5). The diff names a declaration's site (`diff.md`, 3); a
  `retype` may have its field's instead.
- **What a plan keeps:**
  - each old declaration's new one, and each new one's old, if any;
  - per new field, its old field, its encoded value or its site; per
    old variant, its new one;
  - per old variant and field, whether it converts derived: not a
    variant dropped or whose `payload` changed, a `retype`'s field, a
    part of a reshaped declaration, nor a record with an added field
    whose value is `written`;
  - per declaration, whether a site or a held field is in its reach,
    and whether its conversion copies its bytes as they are: every part
    matched in place (none added, dropped or moved, no tag moved), every
    type equal or widened with no integer wider, and no versioned
    record, held field or site in reach;
  - each site's refusals.
- **Pieces are opened when their items convert:** a `pieces` field
  whose items' conversion copies is copied as its bytes; one whose
  items' conversion is not a copy is opened, each piece converted as
  its type (4). Pieces whose items' conversion stops, a site or a held
  field in reach, are refused (`Pieces`): the walk does not stop
  inside a piece (`README.md`, section 10).
- **An identity plan** is a kept version planned to itself: every
  declaration its own, nothing written, a versioned record's version
  unchanged. It copies a record but stops at its held fields, and
  refuses pieces whose type reaches one (`Pieces`).

`PatchError` names the line and the token (`at`, a span of the patch's
text) and what is wrong:

| `PatchWrong` | Refused |
|---|---|
| `Long` | a text past the capacity's `text` |
| `Character` | a byte outside a token, a text or a comment |
| `Expected(Expected)` | a token the grammar does not take next: the language, owner, level, family, version, `to`, a line, a name, a type, a value, a text, a record, the line's end |
| `Number` | a decimal past its width |
| `Language` | a language other than 1 |
| `Twice` | a block, the same operation on the same target, a site, or a refusal's name given again; a plan from a version already planned from |
| `Unkept` | a version the machine does not keep |
| `Version` | a second version other than the first plus one |
| `Unknown` | a name its text lacks: the old text's for `drop` and a rename's first name, the new text's otherwise |
| `Mismatch` | a line whose change the texts do not hold: types or records spelled otherwise, a `widen` that is not one by `diff.md`, 3, an `order` other than the new one, a shape or `versioned` unchanged, a part renamed in a declaration not matched |
| `Missing(Missing)` | a change without its line, or a written change without its site: the operation and the declaration, field or variant (in the old text for a drop, the new otherwise) |
| `Asked` | a `?` |
| `Value` | a value its type does not take (2), past its width or bound, or not UTF-8 |
| `Site` | a site naming an added declaration or a field of one, a field site in a written declaration, a `written` value without its site |
| `Stray` | a `refuses` line after no site |
| `Pieces` | pieces of a type whose conversion reaches a site or a held field, or, in an identity plan, that reaches a held field |
| `Room` | past the capacity's patches |

## 4. The machine

```
start(plan, root, record)       decode the old record by the plan's old text
next()                          walk the old value, building the new one
  -> Written { site, at }       the caller puts the site's one value, then next()
  -> Held { at, family, name }  the caller puts the field's one value, then next()
  -> Done                       the new record is whole
finish()                        its bytes, until the next start
next_pass(plan, root)           the next plan, over those bytes
```

- **The walk** builds the new value in preorder, each new part from its
  source in the old value, by its plan:
  - a record's fields in the new order, each from its matched old field
    by the conversion of its type, or its added value; the new version
    first when it is versioned, the old version passed over;
  - an enumeration's new tag, the match of its old variant, then the
    conversion of its record;
  - a list's count, then each item's conversion; an option's presence,
    then its value's;
  - an opened `pieces` field (3), piece by piece: each old piece
    decoded by the old text's item type, converted as that type, and
    written with its new length before it, as `piece::write` frames
    it; the field's new bytes then held to its new bound (`Built`);
  - a type's conversion by `diff.md`, 3, level by level: equal and widened
    levels as they are, a wrapped level's presence put before it, an
    integer at its new width, a named type by its declaration's
    conversion;
  - a declaration whose conversion copies, as its range of the old
    bytes, at once.
- **It stops** at each old value of a written declaration (`at` is
  that value), at each written field (`at` is the old record it
  belongs to), and at each held field, a `bytes` that holds the same
  type of another family in both texts (`at` is its old bytes; the
  family and type are spans of the old text, spelled by
  `Patcher::spelled`).
- **The caller's answer is one value,** of the new type the walk
  stands at, put before `next()`: a written function's (5); for a
  held field, the held record patched by its family's machine and put
  with `put_content`, or the old bytes carried, `carry(old, at)`, where
  that family does not move.
- **Halts end the record.** A `Halt` is the record's end, the start's
  to report by the record's key: refused by its old schema (`Decode`),
  by a written function with a declared reason (`Refused`) or another
  (`Undeclared`), an old node misread (`Misread`), a build the new type
  refuses (`Misbuilt`, `Built`), past the capacity (`Room`), a call out
  of turn (`Order`), a site its caller has no function for (`NoSite`).
  The next `start` begins afresh.

## 5. Written functions

```rust
// patches/p0041.rs: the command writes the skeleton, the author the body
use super::Context;
use super::kept::jig_store_tasks::p0041::{new, old};
use super::refusal::p0041::UNSCORED;
use skein_codec::patch::{Build, Halt, Old};
use skein_codec::value::Node;

/// Task.score, 9 to 10, a field (`written Task.score`): ten a try; a
/// done task without attempts refused.
pub fn task_score(old: &Old, new: &mut Build, at: Node, _cx: &mut Context) -> Result<(), Halt> {
    let attempts = old.uint(old.field(at, old::task::ATTEMPTS)?)?;
    let state = old.field(at, old::task::STATE)?;
    if attempts == 0 && old.tag(state)? == old::state::DONE {
        return Err(new.refuse(UNSCORED));
    }
    new.put_uint(attempts.saturating_mul(10))
}

/// Shape, 9 to 10, a declaration (`drop Shape.square`, `written Shape`):
/// a square made a rectangle, the other variants carried.
pub fn shape(old: &Old, new: &mut Build, at: Node, _cx: &mut Context) -> Result<(), Halt> {
    if old.tag(at)? != old::shape::SQUARE {
        return new.carry(old, at);
    }
    let square = old.payload(at)?.ok_or(Halt::Misread(at))?;
    let side = old.field(square, old::square::SIDE)?;
    new.open_variant(new::shape::RECTANGLE)?;
    new.carry(old, side)?;
    new.carry(old, side)?;
    new.close()
}
```

- **Two halves:** at a stop, `Patcher::parts()` gives the old value to
  read and the new one to build, `(&Old, &mut Build)`, borrowed apart so
  that a function reads one while it builds the other. Its owner's
  dispatch passes them with the stop's `at` and its `Context`. A
  function's own state is its arguments.
- **Beyond its site,** a function reads two ways:
  - **the same record,** outside the site's part, through `Old`: `root()`
    is the record's root, and `parent(node)` the node holding a node,
    none for the root, each a link the decoded value keeps. So a site
    nested in a list reads the record around it, and an enclosing
    declaration needs no site of its own for it;
  - **all else,** through `Context`, its owner's type: the record's key,
    which its driver sets before each `start`, and whatever other data
    the driver reads for it, such as another record's value. It holds
    no references, and a function changes nothing in it that a later
    record reads.
- **Reading the old:** `README.md`, section 8.1's accessors on `Old`, each misread a
  `Halt::Misread`: `field`, `items` and `next_item`, `count` (a list's
  items, so that a function walks them in a `for`), `some`, `tag`,
  `payload`, `uint`, `boolean`, `content`, `expose`; and `root` and
  `parent`.
- **Building the new,** in preorder, each step checked against the
  new type the build stands at, as the new schema's decoder checks it:
  - `put_uint`, `put_bool`; `put_content` for `fixed`, `bytes`, `text`
    and `secret` bytes, and `put_content_of` for an old node's own
    (checked as the new type's: width, bound, UTF-8), never a held
    field's (`Misbuilt`: `carry` it at its stop);
  - `put_none`; `put_variant`, a variant without a record;
  - `open`: a record (its version put when versioned), a list (its
    count put at its `close`), an option's value; `open_variant`: a
    variant and its record at once; `close`: the innermost, with every
    field given, a list's count, an option's one value;
  - `carry(old, node)`: one value, the derived conversion of an old
    part, when its type and the new one compare equal, widened or
    wrapped (`diff.md`, 3), named types matched by the plan.
- **What carries:** a part that converts derived (3), as the
  variant each record holds decides; `carry` of one that does not halts
  `Misbuilt` with the path of its first such part. A carried part
  converts in place, at once, so one whose type has a site or a held
  field in reach, through any variant, halts `Misbuilt` too, at its own
  path and before it puts anything, whichever variant the record holds:
  its author learns it from any record, and builds that part, until
  carried parts stop in their turn (`README.md`, section 10). A part that does not
  convert derived is a site or lies in a site's declaration (3), so
  the first rule meets one only where a declaration's site carries its
  own value. At a stop, `carry(old, at)` is the stop's own: at a
  declaration's site, the derived conversion of the variant at hand,
  never the site again; at a held field, its bytes as they are. At a
  field's site `at` is its record, and the old field `old.field(at,
  …)`.
- **One value per stop:** a field site's type, or the declaration's;
  `next()` refuses a stop answered with more or less (`Order`).
- **Refusals** are a site's declared reasons: `new.refuse(r)` gives the
  `Halt::Refused(r)` to return, or `Undeclared(r)` for a reason its site
  does not declare, and ends the record; `Patcher::refusal(r)` spells
  its text for the caller's report.
- **Names by constants,** never by text: `patches/mod.rs` (`command.md`)
  generates, per family and kept version, a module per declaration of
  its `FieldIx` and `VariantIx` constants, its parts' names in upper
  case (`kept::jig_store_tasks::v9::task::ATTEMPTS`), per block its
  versions' aliases (`kept::jig_store_tasks::p0041::{old, new}`, so
  written code names no version), and per patch its refusals'
  (`refusal::p0041::UNSCORED`), so a misspelt name fails to
  build; the generator refuses two names that spell one constant. The
  dispatch matches a site's patch, then its index, in nested `match`es,
  each with a last arm `Err(Halt::NoSite(site))`:

  ```rust
  match site.patch() {
      41 => match site.index() {
          0 => p0041::task_score(old, new, at, cx),
          _ => Err(Halt::NoSite(site)),
      },
      _ => Err(Halt::NoSite(site)),
  }
  ```

## 6. The interface

```rust
// skein-lib (../../runtime/lib.md, section 6)
pub struct Buffer { /* bytes, reused, bounded by a capacity */ }

// skein_codec::value, beside 8.1's readers
impl Value {
    pub fn count(&self, list: Node) -> Result<u32, Misread>;           // a list's items
    pub fn parent(&self, node: Node) -> Result<Option<Node>, Misread>; // none for the root
}

// skein_codec::patch
pub const LANGUAGE: u16 = 1;

pub struct Capacity {
    pub versions: u32, // kept texts
    pub patches: u32,  // plans, identity plans among them
    pub text: u32,     // the longest kept or patch text, at most MAX_TEXT
    pub bytes: u32,    // a record's bytes, old or new
    pub nodes: u32,    // a record's nodes, old or new
}
pub struct Patcher { /* an Old and a Build */ }
pub struct Old { /* the old value */ }
pub struct Build { /* the kept schemas, the plans, the new value */ }
pub struct PlanIx(u32);
pub struct Versions { pub from: u16, pub to: u16 }
pub struct Site { /* the patch's number, the site's index */ }
impl Site {
    pub const fn new(patch: u32, index: u32) -> Site;
    pub const fn patch(self) -> u32;
    pub const fn index(self) -> u32;
}
pub struct RefusalIx(u32);           // in the patch's text order
impl RefusalIx {
    pub const fn new(index: u32) -> RefusalIx;
    pub const fn index(self) -> u32;
}
pub struct Header { /* spans of its text */ }

pub enum Step {
    Written { site: Site, at: Node },
    Held { at: Node, family: Span, name: Span },
    Done,
}
pub enum Halt {
    Decode(Refusal),                 // the old record, by its schema (8.1)
    Refused(RefusalIx),              // a written function's declared reason
    Undeclared(RefusalIx),           // a reason its site does not declare
    Misread(Node),                   // an old node read as what it is not
    Misbuilt(Path),                  // a build step the new value there is not
    Built(Problem),                  // a value its new type's decoder refuses
    Room,                            // past the capacity's bytes or nodes
    Order,                           // out of turn: no run, the stop's one value not put
    NoSite(Site),                    // the caller has no function for the site
}
pub enum KeepError { Schema(SchemaError), Family, Twice, Room }
pub struct PatchError { pub line: u32, pub at: Span, pub wrong: PatchWrong }
pub enum PatchWrong { /* 11.3's table */ }
pub enum Expected {
    Language, Owner, Level, Family, Version, To, Line,
    Name, Type, Value, Text, Record, End,
}
pub struct Missing { pub op: Op, pub target: Target }
pub enum Op {
    Add, Drop, Reshape, Versioned, Unversioned, Widen, Wrap, Retype,
    Order, Payload, Written,
}
pub enum Target { Declaration(DeclIx), Field(FieldIx), Variant(VariantIx) }

// The matching and the change list a plan computes, which the diff reads (section 10).
// A rename check says what is wrong; a plan's own refusals stay the plan's.
pub struct Matching { /* each current declaration's, field's and variant's kept one */ }
pub enum Comparison { Equal, Widen { integer: bool }, Wrap, Retype }
pub struct Change {
    pub op: Op,
    pub target: Target,
    pub implicit: bool,
    pub written: Option<Target>,
    /* whether a plan's line has met it */
}
pub enum RenameRefused { OldUnknown, NewUnknown, DeclarationUnknown, NotMatched, Twice }
pub fn changes(old: &Schema, new: &Schema, matching: &Matching) -> List<Change>;
impl Matching {
    pub fn new(new: &Schema) -> Matching; // nothing paired
    pub fn old(&self, new: DeclIx) -> Option<DeclIx>;
    pub fn part(&self, new: Target) -> Option<Target>;
    pub fn used(&self, old: Target) -> bool;
    pub fn pair(&mut self, new: Target, old: Target);
    pub fn declaration_rename(&self, old: &Schema, new: &Schema, from: &[u8], to: &[u8])
        -> Result<(Target, Target), RenameRefused>;
    pub fn part_rename(&self, old: &Schema, new: &Schema, declaration: &[u8], from: &[u8], to: &[u8])
        -> Result<(Target, Target), RenameRefused>;
    pub fn pair_declarations(&mut self, old: &Schema, new: &Schema); // what is left, by name
    pub fn pair_parts(&mut self, old: &Schema, new: &Schema);        // what is left, by name
    pub fn compare(&self, old: &Schema, new: &Schema, kept: TypeIx, current: TypeIx) -> Comparison;
    pub fn payload(&self, old: Option<DeclIx>, new: Option<DeclIx>) -> bool;
}

pub fn header(patch: &[u8]) -> Result<Header, PatchError>; // its header lines alone
impl Header {
    pub fn owner(&self) -> Span;
    pub fn level(&self) -> u32;
    pub fn after(&self, patch: &[u8], upstream: &[u8]) -> Option<u32>;
    pub fn marked(&self, patch: &[u8], mark: &[u8]) -> bool;
}

impl Patcher {
    pub fn new(capacity: Capacity) -> Patcher;
    pub fn workspace_bound(capacity: Capacity) -> Option<u64>;
    pub fn keep(&mut self, text: &[u8]) -> Result<u16, KeepError>; // its version
    pub fn schema(&self, version: u16) -> Option<&Schema>;
    pub fn plan(&mut self, patch: &[u8]) -> Result<Option<PlanIx>, PatchError>; // none: no block of its family
    pub fn identity(&mut self, version: u16) -> Result<PlanIx, PatchWrong>; // Unkept, Twice, Pieces, Room
    pub fn plan_from(&self, version: u16) -> Option<PlanIx>; // a patch's, not an identity
    pub fn versions(&self, plan: PlanIx) -> Versions;
    pub fn counterpart(&self, plan: PlanIx, old: DeclIx) -> Option<DeclIx>;
    pub fn source(&self, plan: PlanIx, new: DeclIx) -> Option<DeclIx>;
    pub fn copies(&self, plan: PlanIx, old: DeclIx) -> bool; // its conversion copies (5)

    pub fn start(&mut self, plan: PlanIx, root: DeclIx, record: Box<[u8]>) -> Result<(), Halt>;
    pub fn next(&mut self) -> Result<Step, Halt>;
    pub fn parts(&mut self) -> (&Old, &mut Build);
    pub fn finish(&mut self) -> Result<&[u8], Halt>;   // until the next start
    pub fn next_pass(&mut self, plan: PlanIx, root: DeclIx) -> Result<(), Halt>;
    pub fn spelled(&self, span: Span) -> &[u8];        // the running plan's old text
    pub fn refusal(&self, refusal: RefusalIx) -> Option<&[u8]>; // its text, in the running plan
}

impl Old {                           // 8.1's readers, each Misread a Halt::Misread
    pub fn root(&self) -> Node;
    pub fn parent(&self, node: Node) -> Result<Option<Node>, Halt>;
    pub fn field(&self, record: Node, field: FieldIx) -> Result<Node, Halt>;
    pub fn items(&self, list: Node) -> Result<Items, Halt>;
    pub fn next_item(&self, items: &mut Items) -> Option<Node>;
    pub fn count(&self, list: Node) -> Result<u32, Halt>;
    pub fn some(&self, option: Node) -> Result<Option<Node>, Halt>;
    pub fn tag(&self, enumeration: Node) -> Result<VariantIx, Halt>;
    pub fn payload(&self, enumeration: Node) -> Result<Option<Node>, Halt>;
    pub fn uint(&self, node: Node) -> Result<u64, Halt>;
    pub fn boolean(&self, node: Node) -> Result<bool, Halt>;
    pub fn content(&self, node: Node) -> Result<&[u8], Halt>;
    pub fn expose(&self, node: Node) -> Result<&[u8], Halt>;
}

impl Build {                         // the new value, in preorder
    pub fn put_uint(&mut self, value: u64) -> Result<(), Halt>;
    pub fn put_bool(&mut self, value: bool) -> Result<(), Halt>;
    pub fn put_content(&mut self, bytes: &[u8]) -> Result<(), Halt>;
    pub fn put_content_of(&mut self, old: &Old, node: Node) -> Result<(), Halt>;
    pub fn put_none(&mut self) -> Result<(), Halt>;
    pub fn put_variant(&mut self, variant: VariantIx) -> Result<(), Halt>;
    pub fn open(&mut self) -> Result<(), Halt>;
    pub fn open_variant(&mut self, variant: VariantIx) -> Result<(), Halt>;
    pub fn close(&mut self) -> Result<(), Halt>;
    pub fn carry(&mut self, old: &Old, node: Node) -> Result<(), Halt>;
    pub fn refuse(&mut self, refusal: RefusalIx) -> Halt;
}
```

- **Indexes** are each version's own: `start`'s root, `Step`'s `at`
  and `Old`'s fields and variants are the plan's old text's; `Build`'s
  variants its new text's. A record k versions behind runs k plans:
  `plan_from` finds each, `source` names the root in the one before
  (and `counterpart` in the one after), and `next_pass` starts each
  after the first over the last `finish`'s bytes.
- **`copies`** reads the plan's own flag for an old declaration, the
  one the walk reads to copy pieces or open them: a reader that must
  tell opened pieces from copied ones (the patch goldens, `skein-codegen`)
  asks it rather than repeat the planning.
- **A machine per family.** `keep` refuses another family's text
  (`Family`), a version kept already (`Twice`), and past the
  capacity's versions or text (`Room`). Plans name kept versions, so
  texts are kept before the patches that use them are planned. `plan`
  reads the whole text's grammar and plans its family's block, if the
  text has one.
- The plain types derive as `README.md`, section 7.3's; `Patcher`, `Old` and `Build`
  derive `Debug` alone, and print no record's bytes.

## 7. Bounds

- **Kept and growing tables:** `new` allocates the kept schemas' and
  plans' slots once; `keep` a schema's tables, `plan` a plan's, each
  once. The run's tables (the old value's entries, the output, the
  walk's and the build's stacks) start at a small floor independent
  of the capacity, grow as a record needs them, never past the
  capacity's bytes and nodes, and keep their largest room for the
  records after. Before any record, together they hold at most the
  smaller of their capacity's price and 64 KiB. While the most nodes
  and bytes reached by any started record, a refused one included,
  are each at most a sixteenth of their capacity, their held heap is
  at most the larger of that floor and twice the run's tables' price
  at those needs.
  Small tables double, up to a sixteenth of their ceiling; the next
  growth takes the ceiling. A builder that outgrows the old index
  reserves its replacement separately until the next start can use
  it, sharing the small entries' allowance with the occupied index.
  Growth copies at most the capacity's nodes or bytes.
- **Per record:** the old record's box is its caller's, moved in by
  `start` and freed by the next. A record no larger in nodes or bytes
  than the most reached before it allocates nothing beyond its pass's
  box; each further pass allocates that box, the last `finish`'s bytes
  at their exact length, and a held field's bytes are its caller's
  box for its family's machine.
- **The workspace bound:** `workspace_bound(capacity)` is the run's tables,
  two records' bytes, and the capacity's versions of kept texts and
  patches of plans, each at the capacity's `text`, parsing's and
  planning's scratch included, and what growth holds while it moves,
  at most a sixteenth of the run's tables, rounded up, in checked
  arithmetic. jig's budget for patching is its map's twice plus the
  machine of each family it keeps: a held field runs its family's
  machine while its holder's waits.
- **Bounded walks:** each `next` turns at most a bound the capacity's
  nodes give, and no step recurses: the walk and the build, `carry`'s
  among them, are stacks bounded by the record's nodes. A record past
  the capacity's bytes or nodes, old or new, halts with `Room`, or with
  `Decode(Refusal::Room)` for the old value's nodes.

## 8. Tests

Step tests, in skein-lib and skein-codec, and the story in
`tests/codec`; no world.

- **The buffer** (../../runtime/lib.md, section 10): back-patching, reuse after
  `clear` and `truncate`, a put past the capacity refused whole, the
  exact copy out.
- **The text:** each line's form pinned, and each refusal from both
  sides, naming its line and token; the header read alone.
- **Planning:** each change's check from both sides, the line that
  turns the layout beside the nearest that does not; each value form
  and its refusals; no invention (an added field that is not an option
  needs a value or a site; `?` refused), sites required by written
  changes, the implicit changes with and without their lines.
- **The machine:** each derived operation over generic values; a
  written declaration and a written field with their functions (reads,
  puts, carry, a carry refused for a part that does not convert derived
  and for one with a site in reach, a declared refusal and an
  undeclared one); versions rewritten in nested records; held fields
  answered both ways; an identity plan; a record two versions behind;
  each `Halt`. Every result decodes by the new schema as a generic
  value and equals its expected bytes; numbers are carried as they are.
- **A property,** in the fuzzy suite: values drawn from the `every`
  fixture's old schema (`README.md`, 8.3's seeded generator) through a derived patch
  decode by the new schema, each carried field equal to its old one;
  an identity plan, and a patch without lines between layouts equal but
  for the version, give back the bytes, but for the versions.
- **The story:** a fixture patch under `tests/codec` with every derived
  operation and a written site, run over the `every` fixture's goldens,
  its results kept as goldens beside it, the shape of `patches/golden/`,
  and decoded by the new schema's generated code.
- **Memory:** the heap the machine holds stays within
  `workspace_bound`, counted by the counting allocator through keeping,
  planning and many records. Large ceilings with small written and
  held records stay near the small records' price, through repeated
  reuse. Lists grow through several steps to the node and byte
  ceilings; exactly at each they fit, one past they halt with the
  same refusal, and the next fitting record reuses their room. Step
  tests check both stacks with nested and written values, zero and
  tiny capacities, a larger next pass, and reuse after refusal;
  capacities, and the allocations the counting allocator's span counts
  around each reused pass (../../testing/README.md, 5), show that reuse neither
  grows nor copies a table; no test reads a table's address. A stack's
  room is a list's cap, its buffer growing as the walk deepens
  (../../runtime/lib.md, 5), so a pass reuses its stacks once a pass as deep has
  run. The price includes growth's peak within a sixteenth of the
  run's tables.
