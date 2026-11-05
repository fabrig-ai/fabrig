# brief-failure: the shapes that cross tasks

The run's binding calls. Each is built by the task named, to these names; a later task builds on them.

## 1. The brief child (`brief-words`)

In `jig/crates/jig-domain-brief/src/orchestrated.rs`, exported from `lib.rs` as `GatherFailure` and `GatherFate`, as its neighbours are:

```rust
/// How a brief's failure counts (domain/engine.md, 9.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Fate {
    /// Nothing is wrong but room: the task pauses, no try spent.
    Capacity,
    /// Counted in the tasks child's transient class.
    Transient,
    /// Counted in the tasks child's permanent class.
    Permanent,
}

/// Why a section or a brief could not be prepared, in fixed words (domain/engine.md, 9.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Failure {
    pub fate: Fate,
    /// The section's name, or `b"brief"`.
    pub section: &'static [u8],
    pub words: &'static [u8],
}

impl Failure {
    /// `SECTION: WORDS`; whoever keeps it holds it to its own bound.
    #[must_use]
    pub fn reason(&self) -> Box<[u8]>;
}
```

```rust
pub enum Event {
    // Plan, Ready, Cut, Abandon as they are.
    Missing { brief: Token, section: Token, why: Failure },
}

pub enum Request {
    // Gather, CutTo, Drop, Complete as they are.
    Failed { brief: Token, why: Missing, failure: Failure },
    Refused { brief: Token, failure: Failure },
}
```

`Planned::Connector` gains `name: &'static [u8]`, the section's name for a failure's words. `planned.rs` gains `pub fn core_name(kind: Core) -> &'static [u8]`.

## 2. The tasks child (`retry-reason`)

In `jig/crates/jig-domain-tasks`:

```rust
/// At most this many bytes of a preparation's reason are kept (domain/engine.md, 9.1).
pub const REASON_BYTES: u32 = 128;

// `Copy` goes; `Clone, PartialEq, Eq, Hash, Debug` stay.
pub enum RetryNotice {
    Failed { class: Class, count: u32, allowance: u32, reason: Option<Box<[u8]>> },
    Refused { count: u32, reason: Option<Box<[u8]>> },
}

pub enum HoldSource {
    // Resource, Effect, Circle, Account as they are.
    /// Why preparing failed past its allowance, only on `Hold::Failures`.
    Preparation { words: Box<[u8]> },
}

pub enum Event {
    /// Pause a preparing task with no try spent, saying why.
    PreparationPaused { task: u64, reason: Box<[u8]> },
    /// Count a failure of a preparing task, its hold past the allowance naming the reason.
    PreparationCounted { task: u64, class: Class, reason: Box<[u8]> },
}
```

`PreparationFailed { task }` stays through `retry-reason`, with no reason, and goes in `brief-counts`. The tasks child cuts each reason to `REASON_BYTES` at a character's edge as it takes the event.

`tasks-v9`:

```
record FailedRetry { class: Class count: u32 allowance: u32 reason: option bytes 128 }
record RefusedRetry { count: u32 reason: option bytes 128 }
record HoldPreparation { words: bytes 128 }
enum HoldSource { resource: Name, effect: HoldEffect, circle: Circle, account: HoldAccount, preparation: HoldPreparation }
```

## 3. The core (`brief-counts`)

```rust
// jig-domain-core vocabulary.rs
Event::BriefAssembled { task: u64, failure: Option<brief::GatherFailure> },
/// A brief failed, for the application's log (domain/engine.md, 9.1).
Now::BriefFailed { task: u64, failure: brief::GatherFailure },
```

`Fate::Capacity` routes to `PreparationPaused`, `Transient` and `Permanent` to `PreparationCounted` of that class. Every failure of the table in 9.1 passes through one function, so each says `Now::BriefFailed`.

## 4. The client's wire (`reason-shown`)

`client-v19`:

```
record RetryNoticeOwnerFailed { class: HoldFailure count: u32 allowance: u32 reason: option bytes 128 }
record RetryNoticeFailed { class: FailureClass count: u32 allowance: u32 reason: option bytes 128 }
record RetryNoticeRefused { count: u32 reason: option bytes 128 }
```

`jig_client::contract::RetryNotice` gains `reason: Option<DocumentText>` on each arm, losing `Copy` as the tasks child's does.

## 5. The forge connector (`forge-sections`)

`temper-engine-domain-forge` names no jig child, so it has its own words, and the root translates (`BriefFate` to `GatherFate`, one to one):

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BriefFate { Capacity, Transient, Permanent }

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BriefUnread { pub fate: BriefFate, pub words: &'static [u8] }

pub enum BriefRead { Got { bytes: Box<[u8]>, left: u64 }, Failed(BriefUnread) }

// Requests:
BriefSized { section: Token, size: Result<u32, BriefUnread> },
/// A section's read failed, for the log, whether or not the brief goes on.
BriefReadRefused { section: Token, stage: &'static str, error: Option<skein_forge::api::Error> },

// Events:
PlanBrief { section: Token, source: BriefSource, partner: Option<Token> },
```

`partner` names the earlier section of a repair's pair, on the later's plan; the connector knows the pair both ways. The root's section names are `b"pull"`, `b"ci"`, `b"reviews"`, `b"home"`, `b"document"`.
