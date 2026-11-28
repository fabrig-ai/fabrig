# Contracts: a run's brief planned by the engine

The shapes the changes of the goal that moves a run's brief into the engine share, and no other
area reads, while that goal runs (`temper/docs/design/draft/design-economy.md`, 4.3). They bind
as a delivered design section does. The lasting design is `briefs.md`, 1.4; the goal's last
change deletes this document and its status.

## 1. The brief child

`jig-domain-brief` depends on `jig-domain-contract` and names its values.

```rust
pub enum Planned {
    Core { kind: Core, text: Box<[u8]>, limit: u32, priority: u16, required: bool },   // as today
    Connector { connector: u16, section: parts::Section },
}

pub enum Event {                     // re-exported as GatherEvent
    Plan { brief: Token, budget: u32, deadline: Time, sections: Box<[Planned]> },   // as today
    Sized { brief: Token, connector: u16, section: Token, size: u32 },  // a gather or a cut answered
    Missing { brief: Token, connector: u16, section: Token, why: Failure },
    Abandon { brief: Token },                                           // as today
}

pub enum Request {                   // re-exported as GatherRequest
    Gather { connector: u16, brief: Token, section: Token, budget: u32, deadline: Time },
    CutTo { connector: u16, brief: Token, section: Token, size: u32 },
    Drop { connector: u16, brief: Token, section: Token },
    Complete { .. }, Failed { .. }, Refused { .. },                     // as today
}

pub struct Limits { pub briefs: u32, pub sections: u32, pub read_bytes: u32, pub brief_bytes: u32,
                    pub gather: Duration }   // the time from placing a brief to its deadline

pub struct ConnectorPlan { pub connector: u16, pub sections: Box<[parts::Section]> }
pub enum Placing {
    Placed { sections: Box<[Planned]>, dropped: Box<[ConnectorAction]> },
    Failed { failure: Failure, dropped: Box<[ConnectorAction]> },
}
pub fn place(core: &[Planned], documents: &[u16], plans: &[ConnectorPlan], sections: u32) -> Placing;
```

- **`Sized`** replaces `Ready` and `Cut`: the section's own state says which it answers, a
  gather while it gathers and a cut while it is cut. A section is found by its connector and its
  token together, in `Sized`, `Missing` and the child's check that no section is planned twice.
- **`place`** is `briefs.md`, 1.4's placing, a pure function: `core` the core's sections, each a
  `Planned::Core`, in the brief's order; `documents` each document's connector, in the
  preparation context's order; `plans` every answer, one per connector asked, in any order;
  `sections` the brief's bound, `brief.sections`. It returns the brief's sections in the placed
  order, or `briefs.md`, 1.1's `brief: no room for its required sections`, permanent; with
  `dropped`, a `ConnectorAction::Drop` for each connector section left out, a plan out of bounds
  dropped whole. It never fails for a plan out of bounds.
- **`Failure` and `Fate`** are the contract's `parts::Failure` and `parts::Fate`, re-exported here
  under their old names, `GatherFailure` and `GatherFate` among them.
- **`gather`** is positive, a limit `valid` checks. Each root's is the time it gave its plan
  before: `ops`'s `policy.brief_timeout`, the test root's 30 seconds, temper's `brief.gather` of
  its own limits (`root.brief.gather_ms` in its configuration); a fixture takes 30 seconds.

## 2. The core's vocabulary and the hub link

- **`Now::RunPrepared`** gains `brief: Box<[brief::GatherPlaced]>`: the placed sections, in
  order, the core's with their text and each connector's by its connector, kind, token and size.
  A root puts it in `Assignment`'s `brief`.
- **Gone:** `Now::BriefCorePlanned`, `Now::CompleteBrief`, `Event::BriefAssembled`,
  `Event::Brief`, `Ask::Gather`, `Ask::CutTo`, `Ask::Drop`, `PlannedDocument` and `DOCUMENT_KIND`;
  `jig-domain-hub-link`'s `brief_plan`, `keep_brief` and `take_brief`, its `Prepared` keeping the
  assignment's brief as the root gives it.
- **`DOCUMENT_PRIORITY`** is the contract's, still named along its old path.
