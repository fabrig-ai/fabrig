# model-check: the shapes that cross tasks

The run's binding calls, from `temper/docs/design/domain/trial.md` and jig's `domain/authority.md`, 6.1. Each side
builds to these, and neither needs the other's task.

## jig (`person-task`)

In `jig-domain-core`:

```rust
/// A person's task in no project (jig/docs/design/domain/authority.md, 6.1).
#[derive(Debug)]
pub struct PersonSeed { /* pub(crate) fields */ }
impl PersonSeed {
    #[must_use]
    pub const fn number(&self) -> u64;
}
impl Core {
    /// None: the person is unknown, no task number is left, or the pool has no room for a run.
    pub fn person_seed(&mut self, person: u64) -> Option<PersonSeed>;
    /// Whether the task's current run is past "Starting" (the Task document's `chip.work`).
    #[must_use]
    pub fn run_past_starting(&self, task: u64) -> bool;
}
// in `Event`:
PersonTask { seed: Box<PersonSeed>, executor: tasks::Executor, spec: tasks::Spec, contract: tasks::Contract },
```

`jig_domain_people::DEPLOYMENT` is the task's project. A person's watch of the Task document is admitted where
the task's funder is `Funder::Pool { project: 0, person, .. }`.

## The child (`trial-machine`)

Crate `temper-engine-domain-trials`, step profile, `skein-lib` its only dependency:

```rust
pub struct Limits { pub rows: u32 }
pub struct Trials { /* its rows */ }
impl Trials {
    pub fn new(limits: &Limits) -> Self;
    pub fn rows(&self, person: u64) -> impl Iterator<Item = &Row>;
    /// The tasks of running rows, for the root's `Observed`.
    pub fn running(&self) -> impl Iterator<Item = u64> + '_;
    pub fn holds(&self, task: u64) -> bool;
    #[must_use]
    pub fn next_due(&self) -> Option<Wall>;
}
pub struct Row {
    pub account: u32,
    pub person: u64,
    pub task: Option<u64>,
    pub began: Wall,
    pub took: Option<u32>,
    pub step: Step,
    pub state: State,
}
pub enum Step { Starts, Answers }
pub enum State { Running, Answered { seconds: u32 }, Failed { why: Why } }
pub enum Why { Ended { class: Class }, Refused, Waited }
pub enum Class { Transient, Permanent, Run, Agent, Lost, Invalid, Refused }
pub enum Event {
    Try { account: u32, person: u64, again: bool },
    Made { account: u32, task: u64 },
    Refused { account: u32 },
    Observed { task: u64, past_starting: bool, took: Option<u32> },
    Turn { task: u64 },
    Finished { task: u64 },
    RunFailed { task: u64, class: Class },
    Due,
    AccountGone { account: u32 },
}
pub enum Request {
    Make { account: u32, person: u64 },
    Cancel { task: u64 },
    Changed { person: u64 },
}
/// `temper/docs/design/domain/trial.md`, 3.2.
pub const WAIT: Duration = Duration::from_secs(60);
pub fn step(trials: &mut Trials, env: &Env<Limits>, event: Event, out: &mut Queue<Request>);
```

The clock is `env.wall`. `Class::Refused` is `EndKind::Refused`.

## The wire (`trial-wire`)

In `temper-client-protocol-wire/schema/client-wire-v1.schema`, in place, the forge family's:

```
record TryAccount { account: u32 again: bool }
enum TrialStep { starts, answers }
enum TrialClass { transient, permanent, run, agent, lost, invalid, refused }
record TrialEnded { class: TrialClass }
enum TrialWhy { ended: TrialEnded, refused, waited }
record TrialAnswered { seconds: u32 }
record TrialFailed { why: TrialWhy }
enum TrialState { running, answered: TrialAnswered, failed: TrialFailed }
record TrialRow { account: u32 task: option u64 took: option u32 step: TrialStep state: TrialState }
versioned record Trials { rows: list 64 TrialRow }
```

`Request` gains `try_account: TryAccount`; `Watch` gains `trials`; `Outcome` gains `tried` and `not_theirs`. The
watch's scope is `c::Scope::Own`; the ask's scope too.

In `temper-engine-domain`'s `boundary.rs`, `Application` gains `Try { account: u32, again: bool }` and
`ApplicationAnswer` gains `Try(TryAnswer)`, with `pub enum TryAnswer { Tried, NotTheirs }`.

The application document's subject is `pub const TRIALS_SUBJECT: u64` in `temper-engine-domain-trials`,
`(1 << 62) | (1 << 36)`, beside `temper-engine-domain-forge`'s subjects, whose highest bit is `1 << 35`.
