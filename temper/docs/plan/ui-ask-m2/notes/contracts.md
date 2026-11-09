# ui-ask-m2: contracts

The shapes that cross tasks, binding. A task builds its half to this note
and needs no other task's code but what its `needs` name.

## 1. jig: a run's read as its person

`jig/docs/design/domain/engine.md`, 7.3, *A run's read as its person*.

In `jig/crates/jig-domain-core/src/vocabulary.rs`:

```rust
pub enum NamedAction {
    // ... as today, and:
    /// A run reads as its task's requester, a person, of its task's project alone
    /// (engine.md, 7.3, *A run's read as its person*). Decides nothing, keeps no call record.
    PersonRead(crate::reads::Read),
}

pub enum Now {
    // ... as today, and:
    /// A run's read as its person answered: the party read's own answer, `None(Why::Unknown)`
    /// for what is not admitted, `None(Why::Busy)` where the loads are full (engine.md, 7.3).
    PersonAnswer { to: ReplyTo, key: CallKey, answer: Box<crate::reads::Answer> },
}
```

- **Routed** in `routing/run/calls.rs`'s `named_action` before replay and
  call admission, as `NamedAction::Settings` is, to
  `crate::reads::person::run(core, env, to, key, read) -> Queue<Request>`;
  `named_tool_kind` maps it to `ToolKind::Ask`.
- **Refused for want of the asking family** as the settings run read is,
  `Now::EffectAnswer { to, key, part }` with the part `authorize_tool`
  gives; every other refusal is `Now::PersonAnswer` with
  `Answer::None(Why::Unknown)`.
- **An application's read** (`Read::Application { scope:
  ApplicationScope::Project(p), connector, read, window }`, `p` the run's
  project) is answered by the connector: the core emits `Request::Ask {
  connector, ask: Ask::Read { reply: to, scope, read, party, window } }`,
  `party` the requester, exactly as for a signed-in party's read, and
  emits no `PersonAnswer` for it. The root recognises `reply` as its own
  call's (section 2).
- **Admitted reads** (engine.md, 7.3): `Project { project: p }`,
  `Spend { project: p }` (p never 0), `Tasks` with `query.project ==
  Some(p)`, `Notes { project: p }`, `ScopedNotes` whose scope is
  `notes::Scope::Project { project: p }` or `notes::Scope::Goal { project: p, .. }`,
  `Inbox` with `filter.project == Some(p)`, `Application` in
  `ApplicationScope::Project(p)`; and `Task`, `TaskText`, `Activity`,
  `ActivityBeside`, `Turn`, `EndedDelegates`, `History`, `Result`,
  `Reply`, `Question`, `Escalation`, `PersonTask` and `Proposal {
  proposer: Proposer::Task(t), .. }` whose task is of `p`, live or
  retained. Every other read answers unknown.

## 2. temper: `see`'s M2 addresses and answers

`temper/docs/design/domain/agent.md`, 8.4.1: the rows `me` to `inbox`, *Read
as the person*, *How an answer reads* and the person's *A refusal*.

In `temper/crates/temper-engine-domain/src/route/see.rs` (crate-private
`Address`, public `SeeAnswer` and its records, exported beside it as today):

```rust
pub(crate) enum Address {
    // ... as today, and, each parsed from the line 8.4.1's table gives:
    Me,                                              // `me`
    Project,                                         // `project`
    Goals { ended: bool },                           // `goals`, `goals ended`
    Tasks,                                           // `tasks`
    Task { number: u64 },                            // `task N`
    Tree { number: u64 },                            // `task N tree`
    Activity { number: u64 },                        // `task N activity`
    Turn { number: u64, attempt: u64, turn: u32 },   // `task N turn A.T`
    Change { number: u64 },                          // `change N`, N a task number
    Changes,                                         // `changes`
    Inbox,                                           // `inbox`
    // `NotYet` leaves with `see-route`, once nothing parses to it.
}

pub enum SeeAnswer {
    // ... as today, and:
    /// An address read as the chat's person (domain/agent.md, 8.4.1).
    Person(Box<SeePerson>),
    /// An address read as the person whose first read was refused.
    NotRead(SeeNotRead),
    // `NotYet` leaves with `see-page`, once nothing delivers it.
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SeeNotRead {
    /// Answered unknown: "nothing you may read in this project is there."
    Unknown,
    /// Answered busy: "temper is busy; read it again."
    Busy,
    /// The chat's requester is a task or the deployment: "this chat has no person to read as."
    NoPerson,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct SeePerson {
    pub read: SeeRead,
    pub names: SeeNames,
}

/// Each read's answer as the core gave it, `jig_domain_core::reads::Answer`
/// (`Answer` below), in the order 8.4.1's *Over* names them. A read after the
/// first may be `Answer::None(_)`: its part says it was not read.
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum SeeRead {
    /// `Read::Project`, then `Read::Inbox { group: Needs, filter: { project, goal: None }, from: None }`.
    Me { person: u64, project: Answer, needs: Answer },
    /// `Read::Project`, then `Read::Tasks` of the project's held tasks
    /// (`phase: Some(Held)`, `order: Number`), live group; repositories as the inventory names them.
    Project { project: Answer, held: Answer, repositories: Box<[RepositoryIdentity]> },
    /// `Read::Tasks` of the project's tracked tasks (`tracked: Some(true)`), live by
    /// `Priority`, or ended (`phase: Some(Ended)`, `order: Ended`); then `Read::Notes { project }`.
    Goals { ended: bool, goals: Answer, reports: Answer },
    /// `Read::Tasks` of the project, `order: Activity`, live group.
    Tasks { tasks: Answer },
    /// `Read::Task { task }`; then, for a tracked task, `Read::ScopedNotes` of its goal scope.
    Task { number: u64, task: Answer, report: Option<Answer> },
    /// `Read::Tasks` with `parent: Some(number)`, `order: Number`: its live group, then its ended group.
    Tree { number: u64, live: Answer, ended: Answer },
    /// `Read::Activity { task, before: None }`.
    Activity { number: u64, activity: Answer },
    /// `Read::Turn { task, attempt, turn }`.
    Turn { number: u64, attempt: u64, turn: u32, read: Answer },
    /// The forge's panel of change `number`, `Document::Change`.
    Change { number: u64, panel: Box<temper_engine_domain_forge::Document> },
    /// The forge's `changes_subject(project)` and `landings_subject(project, false)` documents;
    /// `None` where the forge answered unknown.
    Changes {
        changes: Option<Box<temper_engine_domain_forge::Document>>,
        landings: Option<Box<temper_engine_domain_forge::Document>>,
    },
    /// `Read::Inbox { group: Needs, filter: { project, goal: None }, from: None }`.
    Inbox { needs: Answer },
}

/// Names the answer's numbers need (8.4.1, *How an answer reads*).
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct SeeNames {
    /// The chat's project, by number and name.
    pub project: SeeName,
    /// The project's people, each by number and name, their login where no name is kept.
    pub people: Box<[SeeName]>,
    /// The charters at the newest revision, each by number and name.
    pub charters: Box<[SeeName]>,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct SeeName {
    pub number: u64,
    pub name: Box<[u8]>,
}
```

- **The renderer** (`temper-engine-smith`'s `see_answer`) gets these with
  the catalogue it gets today, which names a model by its index.
- **A forge read's reply:** the root keeps a `see` call's flight under its
  reply token (`call_renders`), as today; `Ask::Read { reply, .. }` whose
  reply is such a flight is answered by `domain.forge.document(read.raw(),
  scope, party, now, limits)` to that flight; any other stays unknown, as
  today.
