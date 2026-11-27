# Contracts: parks against wakes, and relations across components

The shapes this goal's changes share while it runs (`temper/docs/design/draft/design-economy.md`, 4.3): the signatures one change builds and another calls. The lasting design is `smith/docs/design/parks.md`, `jig/docs/design/domain/testing-cells.md` and `skein/docs/design/testing/inspection.md`, 3 and 4.3, which say what each is and its rules; this document only fixes the names and signatures. The goal's last change deletes it.

## 1. smith's park states

```rust
// smith/testing/smith-faults, module parks (smith/docs/design/parks.md, 2)
pub const FAMILY: &str = "a session's state at a park";
pub const WITHHELD_WAIT: Kind;        // "a withheld wait"
pub const JOB_RUNNING: Kind;          // "a job running"
pub const CUT_TURN: Kind;             // "a cut turn with kept parts"
pub const HOST_CALL_ANSWERED: Kind;   // "a host call answered"
pub const HOST_CALL_UNANSWERED: Kind; // "a host call unanswered"
pub const SUB_AGENT_KEPT: Kind;       // "a sub-agent's conversation kept"
pub const KINDS: [Kind; 6];           // in this order
```

`Kind` is `skein_world::faults::Kind`.

## 2. The cells and their pairing

```rust
// jig/testing/jig-core-kit, module cells (jig/docs/design/domain/testing-cells.md)
pub const FAMILY: &str = "parks against wakes";

/// One constant per cell, smith's half first: WITHHELD_WAIT_ANSWERED_WHILE_PARKED,
/// ..., JOB_RUNNING_PARKS_ON_NOTHING, ..., HOST_CALL_ANSWERED_STUCK_IN_A_CIRCLE:
/// the three pairing smith kinds by the seven jig kinds of testing-cells.md, 2.
pub const JOB_RUNNING_PARKS_ON_NOTHING: Kind; // "a job running, a run parks with nothing to await"

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Column { Resumes, Ends }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cell { pub smith: Kind, pub jig: Kind, pub kind: Kind, pub column: Column }
pub const CELLS: [Cell; 21];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Never { pub smith: Kind, pub jig: Kind, pub why: &'static str }
pub const NEVER: [Never; 27];

/// The cell of a smith kind and a jig park kind; `Err` the pair's reason, or that
/// one of the two is no kind of its half.
pub fn cell(smith: Kind, jig: Kind) -> Result<&'static Cell, &'static str>;

/// A park's key (testing-cells.md, 4).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Park { pub task: u64, pub attempt: u64, pub number: u32 }

/// Each park's fallen halves, and whether its cell was injected.
pub struct Pairing { /* private */ }

impl Pairing {
    pub fn new() -> Pairing;
    /// One half fell at `park`: `kind` of `smith_faults::parks::FAMILY` or of
    /// `jig_core_kit::parks`' family. Once the park holds one of each, injects the
    /// cell's kind at `at`, once, its request `task N`, and returns that cell.
    /// `Err` is a finding: an impossible pair, a second kind of one half, or a kind of neither.
    pub fn fell(
        &mut self,
        park: Park,
        kind: Kind,
        at: skein_lib::Time,
        ledger: &mut skein_world::faults::FaultLedger,
    ) -> Result<Option<&'static Cell>, String>;
}
```

`jig-core-kit` takes `smith-faults` as a dependency for this module.

## 3. smith's inspection adapter

```rust
// smith/testing/smith-inspect, package smith-inspect (smith/docs/design/testing.md, 6)
use skein_world::inspect::{Inspection, Moment};

/// The waits smith's design documents, in its words; component "smith".
pub const KINDS: &[&str];
/// The root and every child's waits and tables.
pub fn inspect(domain: &smith_domain::Domain, into: &mut Inspection);
/// The session child's alone, for a world that holds it alone; `inspect` calls it.
pub fn session(domain: &smith_domain_session::Domain, into: &mut Inspection);
/// smith's own check of the session child's room, cells and shares.
pub fn room(domain: &smith_domain_session::Domain, moment: Moment) -> Result<(), String>;
/// Each host call a run holds open, as `jig_inspect::HOST_CALLS`' smith side;
/// `task` the task the domain's run serves.
pub fn host_calls(domain: &smith_domain::Domain, task: u64, into: &mut Inspection);
```

## 4. jig's inspection adapter, its host and its relations

```rust
// jig/testing/jig-inspect (jig/docs/design/domain/testing-fakes.md, 1.1)
use skein_world::inspect::{Inspection, Relation};

/// Unchanged in form; it now adds the tables `placements` and `flights`
/// and jig's records of both relations.
pub fn core(core: &jig_domain_core::Core, into: &mut Inspection);
/// A host's hub, component "jig host": its relation records.
pub fn host(hub: &jig_domain_host::Hub, into: &mut Inspection);
pub const HOST_CALLS: Relation;  // "a run's host calls"
pub const PLACEMENTS: Relation;  // "a run's placement"
```

The words, as testing-fakes.md, 1.1's table gives them:

| Relation | Key, by `format!` | Components and what they hold |
|---|---|---|
| `a run's host calls` | `"task {task} attempt {attempt} completion {completion} position {position}"` | `smith`: `an open call`; `jig`: `a call record`, `an answer retained for replay`, `a cancelled run's call`, `a relay in flight`; `jig host`: `a relay in flight`; `links`: `a relay in flight` |
| `a run's placement` | `"run {run} attempt {attempt}"`, the fleet's numbers | `jig`: `a placement`; `jig host`: `a hosted slot`, `a terminal awaiting its acknowledgement`; `links`: `a relay in flight` |

A composed world's own inspection is `Inspection::new("links", &[])`, holding only `Related` records.
