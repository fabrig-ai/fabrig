# ui-reaches: the contracts

The shapes that cross this plan's tasks, binding as written. The design behind them is `temper/docs/design/domain/forge.md`, 5.10 (the judgement) and `temper/docs/design/protocol/configuration.md`, 13.2 (kind 8 and the records). A later section of this note adds shapes; none changes one above it.

## 1. The kit: `temper-protocol-reaches`

A step crate at `temper/crates/temper-protocol-reaches`, `#![no_std]` with `alloc`, depending on `skein-lib` alone until `classes-value` adds `temper-client-protocol-wire` for section 3. Its lints profile `step` (`python3 -I -B tools/lints/lints.py --set step temper/crates/temper-protocol-reaches`). Every item below is `pub` at the crate's root (modules inside are the implementer's).

```rust
pub const ITEMS: usize = 20;          // items in a table
pub const WORDS_BYTES: usize = 120;   // an item's words
pub const SETS: usize = 6;            // path sets in a table, numbered 1 to SETS
pub const NAME_BYTES: usize = 40;     // a set's name
pub const PATTERNS: usize = 6;        // patterns in a set
pub const PATTERN_BYTES: usize = 80;  // a pattern
pub const PATHS: usize = 16;          // paths in one call
pub const PATH_BYTES: usize = 256;    // a path

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Kind { Wording, Retry, Implementation, Tests, Dependency, Departure, Money, Irreversible, Rules, Other }
impl Kind {
    pub const ALL: [Kind; 10];                 // in the order above
    pub const fn fixed(self) -> bool;          // Money, Irreversible, Rules, Other
    pub const fn name(self) -> &'static [u8];  // b"wording", b"retry", b"implementation", b"tests", b"dependency",
                                               // b"departure", b"money", b"irreversible", b"rules", b"other"
    pub fn named(name: &[u8]) -> Option<Kind>; // the inverse of name
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Class { Decide, Ask, AskAway, Wait }
impl Class {
    pub const fn strictness(self) -> u8;       // 0, 1, 2, 3
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Subject { Kind(Kind), Set(u8) }

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Item { pub subject: Subject, pub words: Box<[u8]>, pub class: Class }

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PathSet { pub number: u8, pub name: Box<[u8]>, pub patterns: Box<[Box<[u8]>]> }

#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct Classes { pub items: Box<[Item]>, pub sets: Box<[PathSet]> }

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Way { YouApprove, AgentsReview, DraftFirst }

/// The first fault `check` finds; `item` and `set` are an item's place from 1 and a set's place from 1.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Refusal {
    Items,                         // more than ITEMS
    Sets,                          // more than SETS
    Fixed { item: u8 },            // an item of a fixed kind
    Wait { item: u8 },             // an item of class Wait
    Words { item: u8 },            // words past WORDS_BYTES
    NoSet { item: u8 },            // names a set the table lacks
    Twice { item: u8 },            // a second item without words of its subject
    Number { set: u8 },            // a number outside 1 to SETS, or twice
    Name { set: u8 },              // empty or past NAME_BYTES
    Patterns { set: u8 },          // none, or more than PATTERNS
    Pattern { set: u8, pattern: u8 }, // empty, past PATTERN_BYTES or not well formed (pattern from 1)
}

impl Classes {
    pub fn check(&self) -> Result<(), Refusal>;
    pub fn stock(way: Way) -> Classes;
    /// Its own item's class (the item without words); AskAway with none; Wait for a fixed kind.
    pub fn class_of(&self, subject: Subject) -> Class;
    /// An item of class Ask is in the table.
    pub fn away_may_decide(&self) -> bool;
    pub fn set(&self, number: u8) -> Option<&PathSet>;
}

/// A pattern against one repository-relative path (forge.md, 5.10, *A pattern*); false for a pattern not well formed.
pub fn matches(pattern: &[u8], path: &[u8]) -> bool;
/// A well formed pattern, as `check` judges one.
pub fn well_formed(pattern: &[u8]) -> bool;
impl PathSet {
    /// Bit i set where patterns[i] matches one of the paths.
    pub fn matched(&self, paths: &[Box<[u8]>]) -> u8;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Ask<'a> { pub kind: Kind, pub paths: &'a [Box<[u8]>], pub under: u8, pub away: bool, pub always: bool }

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Reason { Kind, Set(u8), Always }

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Reach { pub kind: Kind, pub class: Class, pub reason: Reason, pub under: u8, pub sets: u8, pub matched: u8, pub away: bool }
impl Reach {
    /// class Decide, or class Ask with away.
    pub const fn decided(&self) -> bool;
}

pub fn judge(classes: &Classes, ask: &Ask<'_>) -> Reach;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Offer { Decide(Subject), Words(u8), Split(u8) }
/// Stop asking's offers for a reach, in the order Decide, Words, Split (forge.md, 5.10, *Stop asking*).
pub fn offers(classes: &Classes, reach: &Reach) -> Box<[Offer]>;
/// The table an offer makes; `words` are read by Offer::Words alone. Refused as `check` refuses.
pub fn narrowed(classes: &Classes, reach: &Reach, offer: Offer, words: &[u8]) -> Result<Classes, Refusal>;
```

`judge` takes the table as given, checked or not: a subject's own item is the first item without words of that subject; `under` is an item's place from 1.

## 2. The records: `ProjectClasses` and `Reach`

In `temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema`, after `ProjectWays`, under the comment `# What reaches a person (temper/docs/design/domain/forge.md, 5.10).`:

```
enum ReachKind { wording, retry, implementation, tests, dependency, departure, money, irreversible, rules, other }
enum ReachClass { decide, ask, ask_away, wait }
record ReachKindOf { kind: ReachKind }
record ReachSet { number: u8 }
enum ClassSubject { kind: ReachKindOf, set: ReachSet }
record ClassItem { subject: ClassSubject words: bytes 120 class: ReachClass }
record PathSet { number: u8 name: bytes 40 patterns: list 6 bytes 80 }
record ProjectClasses { items: list 20 ClassItem sets: list 6 PathSet }
enum ReachReason { kind, set: ReachSet, always }
record Reach { kind: ReachKind class: ReachClass reason: ReachReason under: u8 sets: u8 matched: u8 away: bool }
```

In `temper/crates/temper-protocol-store/schema/store-v11.schema`, beside its `ProjectWays`: the same `ReachKind`, `ReachClass`, `ReachKindOf`, `ReachSet`, `ClassSubject`, `ClassItem`, `PathSet` and `ProjectClasses`, and `SettingValue` gains `classes: ProjectClasses` as its last variant. The store's `Reach` comes with `decide-forge`.

The wire's `settings_kind` (`temper/crates/temper-client-protocol-wire/src/lib.rs`) takes 8 with `classes-kind`, whose readers then decode it; until then it stays as it is.

A variant holds one record or nothing (skein's `docs/design/codec.md`, its enumerations), hence `ReachKindOf` and `ReachSet`.
