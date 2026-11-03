# reference-m6: contracts

The shapes that cross this plan's tasks, binding as written: names, fields, rules. Each task
builds its own half to this note; inside them the choices are the implementer's. The design
they build is `temper/docs/design/domain/forge.md`, 8.11; `temper/docs/design/domain/worker.md`,
4.1 and section 5; `temper/docs/design/protocol/channel.md`, sections 4 and 5;
`jig/docs/design/domain/connectors.md`, 6.1 and 6.3; `jig/docs/design/domain/tasks.md`,
section 7.

## C1. jig: a procedure that starts kept (task `kept-start`)

- In `jig-domain-tasks` (`src/boundary.rs`), exported from its `lib.rs`:

  ```rust
  /// A connector's procedure whose tasks, made with a pending dependency, are active and
  /// kept rather than waiting (jig/docs/design/domain/connectors.md, 6.1).
  #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
  pub struct KeptStart {
      pub connector: u16,
      pub code: u32,
  }
  ```

- `jig_domain_tasks::Domain::set_kept_starts(&mut self, starts: Box<[KeptStart]>)`: the
  tasks child keeps them; `Domain::new` is unchanged and starts with none.
- `jig_domain_core::Config` gains `pub kept_starts: Box<[jig_domain_tasks::KeptStart]>`,
  doc "Connectors' procedures that start kept (jig/docs/design/domain/connectors.md, 6.1).",
  after `jobs`. The core hands them to its tasks child where it makes it
  (`jig-domain-core/src/lib.rs`, `tasks::Domain::new(..)`). `jig_protocol_config::core`
  gives `Box::new([])`, as it gives `jobs`.
- **The rule:** a task whose executor is `Executor::Procedure { connector, code }` naming a
  kept start leaves `Phase::Waiting` as soon as its holds are taken, whatever its pending
  dependencies, and becomes active as any task does (its procedure started); while any
  dependency is pending it is kept (tasks.md, 5.7), with every refusal 5.7 lists, and its
  last pending dependency done or removed is the `Met` notice as for a kept task today.
  Every other task waits as today.

## C2. jig: a dependency started from (task `kept-started-from`)

- `jig_domain_core::ProcedureStep` (`src/runs/delegation.rs`) gains, after `proposals`:

  ```rust
  /// Dependencies of the procedure's task its work started from; an amendment may never
  /// remove them (jig/docs/design/domain/connectors.md, 6.1).
  pub started_from: Box<[u64]>,
  ```

  `ProcedureStep::new` gives `Box::new([])`.
- The tasks child keeps a task's marks with its dependencies: its task record gains
  `started_from: Box<[u64]>` (in the order first marked, each once), stored by the tasks
  family's record beside `dependencies` (`TaskRecord.started_from: list 64 Number`), read
  back on restore.
- A step's mark naming a task that is not among its task's dependencies refuses the step
  whole, its connector told why, as other step checks are; a mark already kept is no change.
  A mark of a dependency that is done is kept all the same.
- An amendment whose dependencies leave out a marked one is refused as
  `Refusal::Dependencies`, naming exactly the marked ones it leaves out in the problem's
  `blocked_by`, whether they are pending or done (tasks.md, section 7).

## C3. temper: the worker's start with heads merged in, and its conflict (task `checkout-heads`)

- Checkout (`temper-worker-domain-checkout/src/boundary.rs`):
  - `Start::Merged { branch: Box<[u8]>, heads: Box<[Commit]> }`, doc "The tip of a base
    branch with each head merged into it in order, each merge committed (domain/worker.md,
    section 5)." The base is fetched as `Base`'s is, but a missing one is missing (it is
    never created).
  - `Failure::Conflicted { repository: u32, head: Commit }`: the head that did not merge.
- Worker domain (`temper-worker-domain/src/wire.rs`): `Start::Merged { branch, heads }` and
  `Preparation::Conflicted { tag: u64, head: <its commit type> }`, translated both ways as
  their siblings are.
- Wire (`temper-protocol-worker-wire`, `schema/worker-wire-v1.schema`): `enum Start` gains
  `merged: Merged`, `record Merged { branch: <as Merge's branch> heads: list 64 fixed 32 }`;
  `enum Preparation` gains `conflicted: Conflicted`, `record Conflicted { tag: u64 head:
  fixed 32 }` (`channel.md`, section 5). Ids are full object ids in 32 bytes as everywhere on
  this wire.
- Entrance: `heads` empty or longer than 64 is an invalid start, refused as its siblings'
  invalid starts are.
- Prepare: fetch the base branch and each head (by commit), check out the base's tip clean,
  then for each head in order merge it; a clean merge is committed at once with two parents
  (the checkout's head so far, then the head), the repository's identity as its author and
  the message `Merge <the head's full hex id>`; a head already contained is no merge. A
  conflict ends the prepare as `Failure::Conflicted { repository, head }`, nothing of it kept
  for the next hold. The last commit is the checkout's start for saves and pushes; the push
  creates the push branch, as a `Base` start's first push does.

## C4. temper: the engine's side of C3 (task `checkout-heads-engine`)

- `temper_engine_domain_forge::items::Start::Merged { branch: Box<[u8]>, heads:
  Box<[api::Commit]> }` (`items.rs`): the landing branch and the heads, in order.
- `temper_engine_domain::ForgeStart::Merged { branch: Box<[u8]>, heads:
  Box<[skein_forge::api::Commit]> }`; `run_workspace` (`route/forge_route.rs`) maps one to
  the other, its `save` as for `Base`.
- `temper_engine_domain::Preparation::Conflicted { tag: u64, head: skein_forge::api::Commit
  }`; `temper-service/src/translate.rs` maps `Start::Merged` down and `conflicted` up into
  `Parts.preparation`; `link.rs` reports it as a fact of the attempt as its siblings are.

## C5. temper: the change procedure's early start, as pure step (task `early-step`)

In `temper-engine-domain-forge-change/src/boundary.rs`, exported:

```rust
/// A pending dependency of the change as the top reads it each step
/// (temper/docs/design/domain/forge.md, 8.11).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Pending {
    pub task: u64,
    pub kind: PendingKind,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PendingKind {
    /// A change into the same repository and landing branch: its pushed head, if any, and
    /// whether it is itself kept by a pending dependency.
    Change { head: Option<Commit>, kept: bool },
    /// A change into another repository: it orders the landings alone.
    Elsewhere,
    /// Anything else: no head to start from, so the producer waits for it to be done.
    Other,
}

/// One dependency's pushed head a producer started from.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct StartedFrom {
    pub task: u64,
    pub head: Commit,
}

/// The change's early start (temper/docs/design/domain/forge.md, 8.11).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Early {
    /// No producer has started early.
    None,
    /// Its producer started from the landing branch's tip with these heads merged in, in order.
    Started(Box<[StartedFrom]>),
    /// Its early start is spent: refused for a conflict, or its early producer ended failed.
    Spent(Box<[StartedFrom]>),
}
```

- `Facts` gains `pub pending: Box<[Pending]>`, the change's pending dependencies in the
  order of its dependencies, empty when none is pending. `kept` keeps its meaning.
- `Change` gains `pub early: Early`, `Early::None` for a change made today.
- `Heard` gains `pub start_conflict: Option<Commit>`: the head its live producing delegate's
  preparation failed on as a conflict, until that delegate ends.
- `Decision` gains `CancelProducer`, doc "Cancel the live producing delegate: its early
  start conflicted (domain/forge.md, 8.11)."
- **The step's rules (8.11):**
  - At `Producing { requested: false }` with no head of its own and `Early::None`: if any
    pending dependency is `Other`, or a `Change` with no head or `kept: true`, it decides
    `Wait { until: None }` with no stall armed; otherwise, if any is a `Change`, it decides
    `Delegate(Produce)` with `early` `Started(heads)`, the `Change` dependencies' heads in
    `pending`'s order (`Elsewhere` ones left out); with no pending dependency, it produces as
    today.
  - `Early::Started` with `start_conflict` set: `early` becomes `Spent` (same heads) and it
    decides `CancelProducer`; the cancelled delegate's end then holds nothing.
  - `Early::Started` and its producing delegate ended failed: `early` becomes `Spent`, the
    state `Producing { requested: false }`, no hold, and it waits; with `Early::Spent` it
    makes the next producer only once `pending` is empty, and that producer's failure for
    good holds as today (8.4).
  - The next producer of a change whose `early` is `Started` or `Spent` starts from a merge in
    progress where it would start from its branch or saved work (C6).
- The top reads `early` for the start; nothing else of the decision changes shape.

## C6. temper: the top's half of C5 (task `early-route`)

- The root declares the change procedure kept-starting: where it makes jig's core
  (`temper-engine-domain/src/domain.rs`, `Core::new(config.core, ..)`), it sets
  `kept_starts` to the forge connector's change procedure, `KeptStart { connector: <the
  forge connector's number>, code: CHANGE_CODE }`.
- `Facts.pending` from jig's task (`waiting_on`, live ones, in its dependencies' order) and
  the forge's `changes` map: a change row with the same repository and base is `Change`, its
  head the forge's head of its branch (`branch_head`), `kept` whether that change's task has
  a live pending dependency; another repository is `Elsewhere`; else `Other`.
- A change whose step moves `early` from `None` to `Started` sends `ProcedureStep.started_from`
  with those heads' dependencies (C2), none when it has no heads, in the same decision that
  makes its producer.
- `workspace_item` (`items.rs`) for `Delegate::Produce`: `Early::Started` with heads and no
  head pushed gives `Start::Merged { branch: <landing branch>, heads }` (with no heads, only
  `Elsewhere` dependencies pending, `Base` as today); `Started` or `Spent` where it
  would give `Branch(row.branch)` or `Saved(..)` gives `Start::Merge { branch: <that branch>,
  base: <the landing branch's tip as the forge last read it> }`.
- `Preparation::Conflicted` for a run whose task is a change's live producing delegate steps
  that change with `Heard.start_conflict` set; `Decision::CancelProducer` cancels that
  delegate with the reason "its early start conflicts with <head>".
- A change's branch head first read or moved, and a change's task going on from kept, step
  every live change whose `pending` names it and that waits at producing with no producer.
