# Bridges

Provisional, 2026-11-13. How a family that crosses a wire is spoken in
several versions at once: each patch's reverse, which takes a record
down a version; the floor, the lowest version a build speaks; the
breaking declaration, for a change no reverse carries; and the bridge,
which takes a record from any version it reads to any it speaks. It
carries temper's `draft/redeploys.md`, sections 3.3, 4.1 and 4.5, for
skein's part. What crosses which wire, and which side translates, is
each application's (jig's `protocol/channel/wires.md`; temper's
`protocol/wires.md`).

## 1. In one page

- **Up, by the forward patch** (`patches.md`), as a store reads an old
  record. **Down, by its reverse:** each block of a wire's patch, from
  `k` to `k + 1`, has a reverse from `k + 1` to `k`, derived from its
  lines wherever they allow it (2), its author giving what they do
  not: the value a dropped field takes going down, or a written
  function (3).
- **Never altered, only refused.** A value the older version cannot
  hold is refused, naming where (`Halt::Unheld`), and every record
  taken down is taken back up and compared with what went down: one
  that does not come back as itself is refused too (`Halt::Altered`).
- **The floor** (4) is kept beside the wire's name in its crate's
  `schema/wires`: a build speaks every version from it to its current,
  and reads every kept version. A wire declared per-build speaks its
  current alone.
- **A change no reverse carries is declared breaking,** a line of its
  block (3), and raises the floor to its version. The check refuses a
  floor passed without one, a floor lowered, and a reverse a floor needs
  that does not plan (6).
- **The bridge** (5), `skein_codec::bridge::Bridge`, holds a family's
  kept texts, its forward plans and its reverses from the floor, and
  translates a record between any two versions it reads and speaks,
  in one call. Each owner's generated `patches/mod.rs` makes it (6).
- **Total:** a reverse is total for a root when no value of it can be
  refused (5.3), what a channel needs of what it sends to a run that
  cannot move.

## 2. The reverse

A block's reverse plans its family from the block's second kept version
to its first, from the block's plan, against the same two kept texts:
each change the plan holds (`patches.md`, 3) has its reverse, the two
implicit ones among them (a variant appended, a `widen` with no integer
wider), whether or not the block spells their lines. By the forward
line that spells each change:

| Forward line | Its reverse |
|---|---|
| `add D`, `drop D` (a declaration) | the declaration dropped, or kept back: a dropped one is reached only through parts whose own reverses give it |
| `rename` | the rename back |
| `add D.f T v`, a value | the field dropped; a record whose field holds other than `v` is refused (`Unheld`) |
| `add D.f T written` | the field dropped; the round trip (2.2) judges the record |
| `add D.v`, a variant | the other variants' tags mapped back; a value holding `v` is refused (`Unheld`) |
| `drop D.f` | the author's: `down D.f VALUE`, a down site, or none (3) |
| `drop D.v` | derived: every current variant has its old one |
| `widen D.f T U` | carried when the value is one of `T`: an integer within `T`'s width, a text, bytes, list or secret within `T`'s bound, bytes valid UTF-8 where `T` is a text, a `holds` or `pieces` clause dropped or added as the forward carried it; otherwise `Unheld` |
| `wrap D.f T U` | the added `option` taken away: a value at none there is `Unheld` |
| `order D …` | the old order |
| `versioned D`, `unversioned D` | the other |
| `reshape D`, `retype`, `payload` | the author's: a down site (3) |

- **A line with no derived reverse** needs its author's: a dropped
  field's `down` value, a down site of its declaration or field, or the
  block declared breaking. Without one the reverse does not plan
  (`PatchWrong::Irreversible`, naming the forward line).
- **A down site replaces the derived reverse** for its declaration, or
  its field, as a forward site does (`patches.md`, 3): its function
  builds the old value from the new, reading the new value by `Old` and
  building by `Build`, and may refuse with the reasons its `refuses`
  lines declare. A down site of a declaration with a field's down value
  beside it is refused (`Site`).
- **Forward sites have no part in the reverse:** a block whose forward
  site gives an added field (`add D.f T written`) still derives its
  reverse, which drops the field.
- **Held fields stop,** as forward: the caller carries their bytes, or
  translates them by their family's bridge (5).
- **Versions inside:** every versioned record the reverse reaches has
  its version rewritten, as forward.

### 2.1 What the older version assumes

- **An added field's value is what the older side assumed,** since every
  older record takes it going up (`patches.md`, 2). So a record whose
  added field holds that value goes down as if the field were not
  there, and one whose field holds another is refused: the older side
  would read it as the value it assumes, which is not what was sent.
- **A dropped field goes down with its author's value:** the value every
  newer record stands for in the older version, `down D.f VALUE`, typed
  as the field was in the older text, as an added field's value is
  (`patches.md`, 2). A secret's, a record's or a held field's value is a
  down site's.

### 2.2 The round trip

- **Every record taken down is taken back up** from the version it
  reached to the one it left, and its bytes compared with those that
  went down. Unequal, it is refused with `Halt::Altered`, and nothing
  of it is handed out. So a derived reverse that drops a field holding
  what the older side would not assume, a down site that loses
  something, or a forward site that does not invert its reverse, each
  refuses rather than alters.
- **Its cost** is one forward pass per version crossed, over a record
  already bounded; a bridge does it on every record it takes down.

## 3. The text's additions

To `patches.md`, 1's grammar, within a block:

```
line      = operation | site | refusal | down | breaking
down      = "down" part value          ; a dropped field's value going down
          | "down" "written" name      ; a down site, the declaration
          | "down" "written" part      ; a down site, the field
breaking  = "breaking" text            ; no reverse: the floor rises to this version
```

```
family jig-store-tasks 9 to 10
drop Task.note
down Task.note none                       # the older side assumes no note
add Task.score u32 written
written Task.score                        # site 0, forward
widen Task.attempts u8 u32
add State.paused
retype Task.title text 64 bytes 64
down written Task                         # site 1, down
refuses unprintable "a title that is not UTF-8"   # refusal 0, of site 1
```

- **Names:** a `down` value's part is the older text's, the field the
  block drops; a down site's is the newer text's declaration matched to
  an older one, or a field of such a record, as a forward site's is the
  newer text's.
- **Sites and refusals are numbered** in text order across the whole
  text, down sites among them (`patches.md`, 1), so one dispatch, the
  owner's `written`, serves both directions, and a down site's function
  is told its direction by its number alone.
- **`breaking`** holds the reason, which the command's report and the
  check print: what makes the change one no reverse carries. A breaking
  block has no reverse; its down lines are refused (`Twice`, a `down`
  beside `breaking`). Once a block is landed, its `breaking` line, like
  the rest of it, never changes.
- **Landed patches stay as they are:** the grammar only adds, so every
  landed text reads as before, and a landed block without down lines
  derives what it can. Its reverse is needed only for versions above a
  floor (4).
- **`PatchWrong` gains** `Irreversible` (a forward line whose reverse is
  neither derived nor given, at that line) and `Breaking` (a reverse
  asked of a breaking block); `Missing` and `Site` cover a down site
  missing or misplaced as they do a forward one. A down line in a
  family's block that is not a wire's plans as any line: the store
  never asks for its reverse.

## 4. The floor

- **Kept in `schema/wires`,** one line per wire family (`command.md`,
  1):

  ```
  jig-worker-wire 11
  smith-channel build
  ```

  `NAME FLOOR`: the build speaks versions `FLOOR` to the current;
  `NAME build`: a per-build wire, which speaks its current alone, needs
  no reverse and no breaking declaration when it moves (temper's
  `draft/redeploys.md`, 4.4), its floor its current. A line that is
  neither, or names no family of the crate, is the command's error
  (`Error::Wire`, as before).
- **What it bounds:** a build speaks, negotiates and translates down to
  versions from the floor; it reads, and translates up from, every kept
  version, since kept texts and forward patches stay for good
  (`command.md`, section 7).
- **How it moves** (temper's `draft/redeploys.md`, 3.3), each held by the
  check (6):
  - **never down:** a branch's floor below the base's is refused;
  - **at most the current;**
  - **across a move, at least kept:** a branch whose patch moves the
    family from `k` to `k + 1` keeps its floor at or below `k`, unless
    the block is breaking, when the floor is `k + 1`;
  - **raised alone,** in a landing that moves nothing of the family, to
    any version at or below the current: the landing that says no
    deployment speaks the versions below. Its reviewer reads the line.
- **Every reverse from the floor plans:** for each version above the
  floor up to the current, the block that reached it has a reverse.
- **At its baseline** a wire's floor is its first kept version.

## 5. The bridge at run time

### 5.1 The interface

```rust
// skein_codec::bridge
pub type Sites = fn(Site, &Old, &mut Build, Node) -> Result<(), Halt>;

pub struct Bridge { /* a Patcher, its kept texts, forward plans and reverses */ }

pub enum BridgeError {
    Keep(KeepError),                                // a kept text refused
    Plan { patch: u32, error: PatchError },         // a forward block that does not plan
    Reverse { version: u16, error: PatchError },    // a reverse the floor needs that does not plan
    Floor,                                          // the floor past the current, or below the first kept
    Room,                                           // past the capacity
}

impl Bridge {
    pub fn new(capacity: Capacity, texts: &[&str], patches: &[&str], floor: u16, sites: Sites)
        -> Result<Bridge, BridgeError>;
    pub fn family(&self) -> &[u8];
    pub fn first(&self) -> u16;                     // the first kept version: what it reads
    pub fn floor(&self) -> u16;                     // what it speaks, from here
    pub fn current(&self) -> u16;
    pub fn layout(&self, version: u16) -> Option<[u8; 32]>; // the kept text's family fingerprint
    pub fn translate(&mut self, from: u16, to: u16, root: &[u8], record: Box<[u8]>)
        -> Result<Box<[u8]>, Halt>;
    pub fn total(&self, to: u16, root: &[u8]) -> Option<bool>;
}

/// Each version's family fingerprint, from `floor` to the last text's.
pub fn layouts(texts: &[&str], floor: u16) -> Result<List<[u8; 32]>, BridgeError>;

// skein_codec::patch, beside patches.md, 6
impl Patcher {
    pub fn reverse(&mut self, patch: &[u8]) -> Result<Option<PlanIx>, PatchError>; // none: no block of its family
    pub fn reverse_from(&self, version: u16) -> Option<PlanIx>;   // from `version` to `version - 1`
    pub fn total(&self, plan: PlanIx, root: DeclIx) -> bool;      // 5.3
}
pub enum Halt { /* patches.md, 6's, and: */
    Unheld(Path),                    // a value the older version cannot hold, where
    Altered,                         // a record taken down that did not come back up as itself
}
```

- **`new`** keeps `texts`, a family's kept texts in version order from
  its first (its generated `kept::<family>::TEXTS`), plans each of
  `patches` (its owner's `PATCHES`, `patches.md`, 1) that has a block
  of the family, and plans the reverse of each block whose second
  version is above `floor`. It refuses what does not plan, so a bridge
  that is made translates every pair it is asked. A test or a world may
  pass a floor below the generated one, where the reverses plan.
- **`translate`** takes `record`, the bytes of `root` as version `from`
  encodes it, to version `to`. `root` is the declaration's name in the
  current text; the bridge finds its counterpart in each version by the
  plans. `from` must be at least `first` and at most the current, `to`
  at least the floor and at most the current, else `Halt::Order`.
  Equal, the bytes come back as they are; `from` below `to`, up by each
  forward plan in turn; above, down by each reverse in turn, then the
  round trip (2.2). Written sites, forward or down, go to `sites` with
  the stop's `at`; held fields are carried as they are.
- **`total`** answers whether every reverse from the current down to
  `to` is total for `root` (5.3); none when `root` or `to` is not one
  the bridge speaks.
- **`layout`** is the kept text's `fingerprints().family().digest()`
  (`README.md`, section 9), as a wire's `layout()` is its current text's.

### 5.2 Bounds

- **A bridge's capacity** is its caller's: `versions` at least its kept
  texts, `patches` at least its forward plans and reverses, `bytes` and
  `nodes` its largest record's in any version it speaks, which its
  owner's limits give. `new` allocates as `Patcher::new` and its plans
  do (`patches.md`, section 7); `translate` allocates as a record's
  passes do there, plus one box for the round trip's comparison, and
  holds nothing between calls but the machine's kept room.
- **Step code:** no recursion, no panic, every arithmetic checked, as the
  machine.

### 5.3 Total

- **A reverse is total for a root** when no record of it can halt: in
  the root's reach, nothing that `Unheld` judges (an added field with a
  value or written, a variant added, a `widen` or `wrap`), and no down
  site that declares a refusal. A down site with none is trusted, and
  the round trip is its guard at run time.
- **Planned once:** the plan keeps it per declaration, as it keeps
  whether a site is in reach (`patches.md`, 3).
- **What it is for:** a channel's owner asks it of what it sends to a
  run that cannot move, and of what a newer side sends an older one
  (jig's `protocol/channel/wires.md`); a gate asks it of each
  version from the floor.

## 6. The command, the check and the generated module

- **`patches/mod.rs`** (`command.md`, 3) also holds, for each family
  `schema/wires` lists:
  - `kept::<family>::FLOOR: u16`, its floor, its current for a
    per-build wire;
  - `kept::<family>::bridge(capacity: Capacity) -> Result<Bridge,
    BridgeError>`, made from `TEXTS`, `PATCHES`, `FLOOR` and `sites`;

  and, once for the crate, `pub fn sites(site, old, new, at) ->
  Result<(), Halt>`, the dispatch `written` with a `Context::default()`.
  It is generated from the kept texts, the patches and `schema/wires`,
  and `current` holds it so, so the floor in code is the file's.
- **The skeleton** (`command.md`, 3) writes a function for each down site
  as for a forward one, named by its target in snake case with `_down`
  after it (`task_down`, `task_title_down`), its doc comment naming the
  site as a down site.
- **The run** (`command.md`, 2): a wire's block that the command writes
  is asked its reverse; a forward line without one is reported in
  `asked`, naming its three ways and the bridgeable alternative, such as
  "drop Task.note: give `down Task.note VALUE`, write `down written
  Task`, or declare the block breaking; bridgeable: keep Task.note
  beside its replacement until the floor passes it". The command never
  writes a `down` value, a down site or `breaking` itself: they are the
  author's, kept when the patch is derived again, as values and sites
  are.
- **The check** (`command.md`, 7) gains:

  | `Finding` | Found |
  |---|---|
  | `Floor { family, floor, wrong }` | `Above`: past the current; `Unkept`: below the first kept version; `Lowered`: below the base's; `Passed`: above the version the branch's patch moves the family from, its block not breaking; `Breaking`: a breaking block's family whose floor is not the block's second version |
  | `Unbridged { family, version, error }` | the reverse of the block that reached `version`, above the floor, that does not plan |

  `wrong` is a `FloorWrong`. A per-build wire is held to none of them.
- **The goldens down:** the golden test of patch `n` (`command.md`, 3)
  also takes, for each family whose block reaches a version above its
  floor, each golden of the newer version through the reverse, and holds
  the result against `patches/golden/NNNN/<family>/down/<golden>`, or,
  refused, its reason in `<golden>.refused`: `unheld PATH`, or a declared
  refusal's `refuses <name>`. A result is the round trip's (2.2), so a
  golden that does not come back as itself fails, naming it.
  `SKEIN_PATCH_WRITE=1` writes them, as it writes the goldens up. A
  landed patch's goldens down, once written, never change; a floor
  raised past a patch's versions leaves its goldens down as they are.

## 7. Tests

Step tests in skein-codec and skein-codegen, the story in
`tests/codec-patches`, a property in the fuzzy suite; no world.

- **The reverse:** each forward line's reverse over generic values, as
  the machine's tests run each derived operation (`patches.md`, 8): its
  result decodes by the older text and equals its expected bytes; each
  `Unheld` from both sides (an added field at its value and at another;
  an appended variant and a kept one; an integer and a text at the older
  bound and one past it; a wrapped option some and none); a dropped
  field going down with its value; a down site of a declaration and of
  a field, with a declared refusal and without; a block whose line has
  no reverse refused `Irreversible` at that line; a breaking block
  refused `Breaking`; versions rewritten in nested records.
- **The round trip:** a down site that loses a field refused `Altered`;
  a record two versions above `to` taken down by two reverses.
- **Total:** a root whose reach holds an appended variant, one holding
  an added field, one past a down site with a refusal, each not total;
  one whose reach changed only by renames and orders, total.
- **The bridge:** made from a family of three kept versions with a floor
  at the second: every pair it speaks translated, and the first version
  read up but not spoken (`Order`); a floor below a reverse that does
  not plan refused `Reverse`; `layout` for each version.
- **The command and the check:** each `FloorWrong` from both sides; an
  `Unbridged` and its absence; a per-build wire moving freely; the
  skeleton's down functions; `asked` naming the three ways; the goldens
  down written, checked and failing on a stale file.
- **A property,** in the fuzzy suite: values drawn from the `every`
  fixture's newest text (`README.md`, 8.3's seeded generator) through a
  derived reverse: each one either comes back up as itself or is refused
  with `Unheld`, never altered; values drawn from an older text taken up
  decode by the current text.

## 8. Open questions

- **A down site's context.** It reads only the record; one that needs
  other data, as a store's written function reads its `Context`, has
  none at a channel. Default: none, until a reverse needs it.
- **Bridging held fields.** A bridge carries a held field's bytes; a
  caller that bridges a family holding another's translates the held
  bytes itself. Default: no wire holds another's today.
