# Contracts: wait records

Provisional, 2026-11-24. The shapes the bounded-waits record goal's
changes share and no other area reads (`temper/docs/design/draft/design-economy.md`, 4.3), binding as
`waits.md` binds. The goal's last change deletes this document once the
code holds them.

## 1. The record, in the core's vocabulary

`jig-domain-core-vocabulary`, a module `waits`, its types re-exported at
the crate's root. Every type derives `Clone`, `PartialEq`, `Eq`, `Hash`
and `Debug`, and `Copy` where its members allow.

```rust
pub struct Wait {
    pub task: u64,
    pub kind: WaitKind,
    pub object: WaitObject,
    pub since: skein_lib::Wall,
    pub bound: Bound,
    pub hears: Option<jig_domain_tasks::EscalationHolder>,
}
pub enum WaitKind { Party, Question, Dependency, Resource, Account, Host, Delegate, News, Step, Room, Turn, SignOff }
pub enum WaitObject {
    /// `party`: the entry and the holder it waits at.
    Entry { kind: jig_domain_people::EntryKind, at: jig_domain_tasks::EscalationHolder },
    /// `question`: its number and its answerer.
    Question { number: u64, answerer: u64 },
    /// `dependency`, `delegate`, `turn`.
    Task(u64),
    /// `resource`: the tasks child's resource name, and its holder.
    Resource { name: jig_domain_tasks::Name, holder: Option<u64> },
    /// `account`, `host`: the next run's wait, as the task's document gives it.
    Run(crate::documents::TaskRunWait),
    /// `news`: subscription numbers, at most four, oldest first, and the armed timer.
    News { subscriptions: skein_lib::List<u64>, timer: Option<skein_lib::Wall> },
    /// `step`, `room`, `sign-off`: the connector's step code and at most four things.
    Procedure { step: u32, things: skein_lib::List<Thing> },
}
pub struct Thing { pub what: u32, pub number: u64, pub owner: Option<u64> }
pub enum Bound { At { at: skein_lib::Wall, then: AtBound }, None(Unbounded) }
pub enum AtBound { Hold, PassUp, Overdue, GiveUp }
pub enum Unbounded { NoDeadline, Standing, Behind(u64), OperatorHold }
pub struct Nearest { pub at: skein_lib::Wall, pub then: AtBound, pub kind: WaitKind }
```

The re-export paths may differ where the crate's own names require it
(`TaskRunWait`'s, `Name`'s); the members and variants may not.

## 2. The figures

```rust
/// The `waits` table (`waits.md`, 5), one figure per kind with no clock of its own.
pub struct Figures {
    pub question: skein_lib::Duration,
    pub party: skein_lib::Duration,
    pub news: skein_lib::Duration,
    pub host: skein_lib::Duration,
    pub account: skein_lib::Duration,
    pub step: skein_lib::Duration,
    pub room: skein_lib::Duration,
    pub sign_off: skein_lib::Duration,
}
```

In `jig-domain-core-vocabulary`'s `waits` module; the core's `Limits`
(`vocabulary.rs`) gains `pub waits: Figures`, named `waits.question_ms`
and so on in `jig-protocol-config`'s names (`waits.md`, 5).

## 3. The core's read

```rust
impl Core {
    /// A live task's waits (`waits.md`, 4); an unknown or ended task's are empty.
    pub fn waits(&self, task: u64) -> Waits;
    /// A kind's floor (`waits.md`, 7), none until it is set.
    pub fn wait_floor(&self, kind: WaitKind) -> Option<skein_lib::Wall>;
}
pub struct Waits {
    /// The line's first wait, which may be a task's in the subtree.
    pub first: Option<Wait>,
    pub nearest: Option<Nearest>,
    /// The task's own, in order, within the document's lines.
    pub own: skein_lib::List<Wait>,
    /// How many the task has, those past the lines included.
    pub count: u32,
}
```

`Waits` is in the vocabulary's `waits` module; `wait_floor` comes with
the floors' change and answers none before it.

## 4. Lines and documents

- The core's `TaskLine` (`documents/task_lines.rs`) gains
  `pub wait: Option<Wait>` and `pub nearest: Option<Nearest>`; its `Task`
  (`documents/task_tree.rs`) gains `pub waits: skein_lib::List<Wait>` and
  `pub wait_count: u32`. Both keep `waits_on`.
- The client contract (`jig-client-domain`, `contract`) has the same
  types under the same names, its own: `Wait`, `WaitKind`, `WaitObject`,
  `Thing`, `Bound`, `AtBound`, `Unbounded`, `Nearest`, with its
  `EscalationHolder` for the holder, its `WaitedFor` with a `number: u64`
  beside it for the entry (`Entry { kind, number, at }`, the number the
  question's, the proposal's, the escalation's revision or the person
  task's own), its `TaskRunWait` for the run, and the resource's name as
  the contract carries other names' text. Its `TaskLine` and `Task` gain
  the same four members.
