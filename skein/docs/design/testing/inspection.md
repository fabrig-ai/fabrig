# Inspection: a harness invariant over a component's frozen state

Provisional, 2026-11-20, revised 2026-11-27. How a world's harness invariant reads what a component holds between iterations: the component's inspection adapter, the records it gives, where a world calls it, the two checks every component's waits and bounded tables share, and the check of relations across components. Written from `temper/docs/design/draft/failure-worlds.md`, section 3's opening and 3.1 and 3.2, and the relations from `temper/docs/design/draft/misbehaving-agents.md`, 7. The invariants are the strategy's (../../foundation/testing-strategy.md, 6): a world's own checks of its frozen state once a driver is quiet, never the referee's, which watches from outside (section 7 there). The module is `skein_world::inspect`, in the world harness (service-tiers.md, 1).

## 1. In one page

- **A check is one of three kinds** (the draft, 3): a referee expectation, which observes only what crosses the boundary; a harness invariant, which reads a component's frozen state through its adapter; and a settings test, outside every world. This document is the second kind's form.
- **An adapter** (section 2) is test code in a kit of its component's tree, one function from a borrow of the component to the records of section 3. It reads, never changes: the borrow is shared, and it runs between iterations, when no step runs.
- **Its records** (section 3) are skein's, so that every tree's checks read one form: each wait a live waiter is in, with its documented kind and what would wake it; and each bounded table, with each entry's releasing rule and that rule's state.
- **Two checks every component shares** (section 4): every wait has a live wake, and every entry is released by its own rule. A component's worlds call them over its adapter's records; what is particular to a component stays in its adapter.
- **Relations across components** (section 4.3): what no one component sees, as each side of a relation holds it, keyed in words every inspection of the world agrees on. A record a relation's rule names is matched under its key on another side, or by a skew the relation names; an orphan, a record matched by none, is a finding naming both sides.
- **Where a world calls them** (section 5): at its driver's own quiet point, outside every process's heap meter.
- **A finding** (section 6) is a value naming the seed, the moment, the component, the object and what broke.

## 2. The adapter

- **Its place:** a kit under its component's tree's `testing/`, beside the component, and built against it: it changes in the change that changes what it reads. A world of the component, and a world of a component above it, depend on the kit; no shipped binary does, as no shipped binary links a testing crate (service-tiers.md, 1).
- **Its form:** one function, `pub fn inspect(component: &C, into: &mut skein_world::inspect::Inspection)`, `C` the component's root or child type, concrete. A component whose worlds hold several children has a function per child, each adding to the one inspection. It allocates, as test code may, and panics on nothing.
- **What it reads:** the component's own `pub` reads. Where the component keeps a table or a wait private, it gains a `pub` read that borrows it, documented as its adapter's (`/// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).`), returning a borrow, a count or an entry by key, under the strict subset as any step code. The workspace has no feature and no `cfg` for test-only code (tools/crates/design.md, 3.2), and none is added for this: a read is a borrow, and a shipped binary that never calls it pays nothing.
- **A gap is the adapter's.** A check that can't read what it needs is a gap in its adapter, built in its adapter, never a finding of the product's (the draft, 3).
- **Names agree across adapters:** a waiter is named the same in every inspection of one world (`task 12`, `landing 3`), so that a wait in one component on a waiter in another resolves (section 4.1).

## 3. The records

```rust
/// When an invariant ran: the run's seed, the world's time, and its iteration.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Moment { pub seed: u64, pub at: Time, pub iteration: u64 }

/// What one component holds at a quiet point, as its adapter read it.
pub struct Inspection { /* component, kinds, waits, tables */ }

impl Inspection {
    /// `kinds`: the waits its design documents, each in the design's words.
    pub fn new(component: &'static str, kinds: &'static [&'static str]) -> Inspection;
    pub fn component(&self) -> &'static str;
    pub fn wait(&mut self, wait: Wait);
    pub fn table(&mut self, table: Table);
    pub fn related(&mut self, related: Related);
    pub fn waits(&self) -> &[Wait];
    pub fn tables(&self) -> &[Table];
    pub fn relateds(&self) -> &[Related];
}

/// One wait a live waiter is in.
pub struct Wait {
    /// The waiter, by the name every inspection of the world gives it: `task 12`.
    pub waiter: String,
    /// Its kind, one of the inspection's `kinds`, or what the component holds where it is none.
    pub kind: String,
    /// What would wake it.
    pub wakes: Vec<Wake>,
    /// The deadline its design gives it, where it has one.
    pub bound: Option<Time>,
}

pub enum Wake {
    /// An armed deadline of the component's own.
    Timer(Time),
    /// Another waiter of the world, by its name, which wakes it when it moves.
    Waiter(String),
    /// A party outside the component: a person, a peer, a fake, by name.
    Party(String),
    /// A source the component still names that has ended.
    Ended(String),
}

/// One bounded table or slot.
pub struct Table {
    /// The table, in its design's words: `forge write entries`.
    pub name: &'static str,
    pub capacity: u64,
    pub entries: Vec<Entry>,
    /// Newcomers refused for room that are still waiting to be admitted.
    pub refused: Vec<Refused>,
}

pub struct Entry {
    pub key: String,
    /// The rule that releases it, in its design's words: `the write's settlement`.
    pub rule: &'static str,
    pub release: Release,
}

pub enum Release {
    /// Its rule's moment hasn't come.
    Pending,
    /// Its rule's moment has passed, and the entry is still held.
    Due,
    /// An ended record kept on purpose: an ended task's row, a kept answer within its retention.
    Kept,
}

pub struct Refused {
    pub key: String,
    /// When it was first refused.
    pub since: Time,
    /// The bound its design gives its admission, where it has one.
    pub bound: Option<Time>,
}

/// One record of a relation across components, as its component holds it (section 4.3).
pub struct Related {
    /// The relation, by its name: `a run's host calls`.
    pub relation: &'static str,
    /// Its key, in the words every inspection of the world gives it:
    /// `task 12 attempt 3 completion 2 position 1`.
    pub key: String,
    /// What the component holds under the key, in the relation's words: `an open call`.
    pub holds: &'static str,
}

/// A relation across components, stated by the kit of the tree that may name every side.
pub struct Relation {
    pub name: &'static str,
    /// Every side a record of the relation may be.
    pub sides: &'static [Side],
    /// The records that must be matched.
    pub rules: &'static [Rule],
}

/// A side: the component, as its inspection names itself, and what it holds.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Side { pub component: &'static str, pub holds: &'static str }

/// Each record of `each` is matched by a record of one of `by` under the same key.
pub struct Rule { pub each: Side, pub by: &'static [Side] }
```

- **`Time`** is `skein_lib::Time`, the world's clock.
- **A wait's kind** is a `String` so that an adapter can name what the component holds even where the design documents no such kind; that it is one of `kinds` is what section 4.1 checks.
- **The records are plain values:** fields public, no behaviour but the constructors and reads above. An adapter fills them; the checks and a component's own checks read them.
- **A relation's words are agreed, not named:** an adapter writes a `Related` with the relation's name, its key and what it holds as text, so that a tree that may not name another's kit (smith names no jig) still takes part in a relation that kit states. A key's words follow section 2's rule for names: one form in every inspection of a world.
- **A skew is a side no rule's `each` names:** a state a valid quiet point may hold on one side alone, such as a relay in flight or an answer kept for replay. It needs no match, and may stand as one in a rule's `by`. The relation lists it among its `sides`, so it is allowed by name.

## 4. The checks every component shares

Each takes the inspections a world made at one moment and returns its first finding (section 6); a component's worlds call both, and a component's own checks beside them.

### 4.1 Every wait has a live wake

`pub fn check_waits(inspections: &[&Inspection], moment: Moment) -> Result<(), String>`

- **A wake is live:** a `Timer`, which time reaches; a `Party`, which the scenario answers or names as never answering (the draft, 3.4); a `Waiter` that is live; never an `Ended`.
- **A waiter is live** when no inspection given has a wait of it, or one of its waits has a live wake. Liveness is the least fixed point: waiters whose waits name only one another, in a cycle, are none of them live (a parked coordinator whose only watched task waits on its answer, the draft, 3.1).
- **A finding,** the first in the inspections' order, then the waits' order:
  - a wait whose kind isn't one of its inspection's `kinds`;
  - a wait with no live wake, naming the waiter, the wait's kind and each wake, and for a `Waiter` wake, the waits that waiter is in;
  - a wait whose `bound` is before the moment's `at`.

### 4.2 Every entry is released by its own rule

`pub fn check_tables(inspections: &[&Inspection], moment: Moment) -> Result<(), String>`

- **A finding:**
  - an entry whose release is `Due`, naming the table, the key and the rule;
  - a refused newcomer whose `bound` is before the moment's `at`;
  - a refused newcomer, with or without a bound, while the table holds fewer entries than its capacity: room has freed and it isn't admitted.
- **No finding:** a `Kept` entry, and a full table refusing a newcomer within its bound.

### 4.3 Every related record is matched, or is a skew its relation names

`pub fn check_relations(inspections: &[&Inspection], relations: &[&Relation], moment: Moment) -> Result<(), String>`

- **A match** is a record, in any inspection given, of the same relation, under the same key compared as text, whose component and `holds` are one of the rule's `by`.
- **A finding,** the first in the relations' order, then the inspections', then their records':
  - a record whose relation isn't among those given, or whose component and `holds` are none of its relation's `sides`;
  - an orphan: a record of a rule's `each` with no match, naming the relation, the key, the side that holds it, and each side of the rule's `by` that holds nothing under that key.
- **No finding:** a skew's record alone; a record matched by several; a relation none of whose records was given.
- **The relations are their kits':** the kit of the lowest tree that may name every side states each one, and the composed world that runs those components calls this check over all its inspections at the moment, beside section 4's.

## 5. Where a world calls them

- **At its driver's quiet point,** as that driver defines it, never only once the whole world is quiet, since another task's steady work can keep a world busy for good (the draft, 3.1): a domain world's driver once it has drained its immediate work, before time moves; a world on `World` once nothing is due before time moves.
- **Between iterations:** no step runs while an adapter reads, so what it reads is frozen.
- **Outside every heap meter:** an adapter and the checks allocate as test code, and their bytes are no process's.
- **Each call is a run's finding:** the world returns the `Err` as its run's finding, as a referee's (service-tiers.md, 1); only the loop or the test at its top panics on it.

## 6. A finding

A finding is the text of the `Err`, one line: `seed <seed>: at <ns> ns, iteration <n>: <component> <object>: <what>`, the object its waiter, or its table and key. `Inspection::finding(&self, moment: Moment, object: &str, what: &str) -> String` writes it, for the shared checks and a component's own alike. An example: `seed 7: at 120000000 ns, iteration 48: engine task 12: waits on its watch with no live wake: task 14 waits on task 12's answer`. A relation's finding is written by the inspection of the side that holds the orphan, its object the relation and the key, and its what the side and the sides that would match it: `seed 7: at 120000000 ns, iteration 48: smith a run's host calls task 12 attempt 3 completion 2 position 1: holds an open call, matched by none of jig's a call record, jig's an answer retained for replay, links' a relay in flight`.

## 7. Testing

Step tests in the crate's own `src/tests/`, over inspections a test builds by hand, no world:

- a cycle of two waiters, each waiting only on the other, is a finding naming both; the same two with a `Timer` on one pass;
- a wait whose only wake is `Ended`, and one of a kind its inspection doesn't declare, are findings;
- a wait in one inspection on a live waiter of another passes, so names resolve across components;
- a `Due` entry is a finding naming its table, key and rule; a `Kept` one passes;
- a refused newcomer while its table has room is a finding; one past its bound is too; one within its bound at a full table passes;
- a related record matched on another side, in another inspection, passes; so does one matched by a skew, and a skew's record alone;
- an orphan is a finding naming its relation, key and side and each side that would match it; a record of a side its relation doesn't list, or of a relation not given, is a finding; keys that differ in any word don't match;
- a finding's text has section 6's form.

## 8. Later

- A quiet-point hook in `World`'s own loop, when a world on it needs one: the worlds that call these checks first (the draft, 5) drive their own loops.
- Coverage of each check's finding kinds over a sweep's seeds (the draft, 12).
