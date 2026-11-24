# Workloads drawn from the seed, and the sweeps that mix every failure

Provisional, 2026-11-24. The failure worlds' last part: a random workload
in each root's world, the composed workload in the system world, people's
actions and runs that park, resume and move at any step, a restart that
changes no outcome it shouldn't, and the sweeps: one per family in the
cheapest world, and one in the composed workload mixing every family.
Written from `../draft/failure-worlds.md`, 3.5, 4.4, 4.6, 4.7 and
sections 5 and 6, over skein's streams, fault ledger and sweeps
(`skein/docs/design/testing/sweeps.md`) and the checks the milestone's
other goals built: the engine's (`engine-checks.md`), the worker's
(`worker.md`), jig's (`jig/docs/design/domain/testing-fakes.md`, 1),
smith's (`smith/docs/design/faults.md`), the restarts
(`restarts.md`), blame (`blame.md`) and the store check
(`store-check.md`). jig's workloads are jig's
(`jig/docs/design/domain/testing-workloads.md`). It fixes nothing a sweep
finds: a finding outside the draft's catalog is held as section 8.3 says
and repaired as a red.

## 1. In one page

- **One vocabulary** (section 2): every fault and act a sweep configures
  is a `Kind` of one of the draft's section 4 families, named once, in the
  kit of the tree whose mechanism it is. The forge's kinds move from the
  forge world's tests into `temper-faults`, and the engine's open
  problems' kinds take the names of the families this goal builds.
- **The workload kit** (section 3): `temper-workload` draws a workload
  of goals, a milestone, a landing train and an after-landing tier from
  the seed's `Workload` stream, the parties' actions as logical events
  from `Parties`, and gives a run's outcome in logical names, with its
  comparison.
- **A random workload in each root's world** (section 4): the engine's
  domain world runs the kit's workload at the shipped stock limits; the forge
  world's landing train takes parties' acts; jig's core and
  engine-clients worlds draw their own; the worker's domain world draws
  several runs at once; smith's sweeps draw their runs.
- **The composed workload** (section 5): the system world runs the kit's
  workload with two parties on two accounts, one forge, a milestone, a
  train and a tier, each goal's scripts and CI cued by its names and its
  tree.
- **People's actions and parks at any step** (section 6): each act waits
  for its logical moment, then a drawn number of steps, and each goal's
  expectation follows from the acts that strike it.
- **A restart changes no outcome it shouldn't** (section 7): each seed
  runs twice, without and with its restarts, and the two outcomes are
  equal, goal by goal, but for the goals left out by name.
- **The sweeps** (section 8): a table of every sweep, the bounds reached
  as faults that fall, the open problems' exclusions, and room within
  the user's figures.

## 2. The vocabulary

### 2.1 Families

A kind's family is the draft's section 4 family it falls in, and its name
the fault or act in that section's words. Names are fixed: a sweep, an
exclusion and a check compare kinds by value, so a name changes only with
every reader of it (`restarts.md`, 2). Each kind has one constant, in the
kit of the tree whose mechanism makes it; a world of another tree names
that constant.

| The draft | Family | Its kinds |
|---|---|---|
| 4.1 | `busy`, `lagging`, `bounds`, `refusing` | `temper_faults::forge` (2.2) |
| 4.1, CI | `ci` | `temper_blame` (`blame.md`) |
| 4.2 | `provider` | `skein_llm_kit::faults`; `smith_faults` |
| 4.3 | `worker` | `temper_worker_inspect::faults`; `smith_faults` |
| 4.4 | `people and parties` | `temper_faults::people` (2.3); `jig_core_kit::people` and `::open`; `jig_engine_clients_kit::open` |
| 4.5 | `stale` | `temper_faults::stale` (2.2) |
| 4.6 | `runs that park, resume and move` | `temper_faults::parks` (2.3); `jig_core_kit::parks` and `::open` |
| 4.7 | `the engine's own bounds` | `temper_faults::bounds` (2.4); `jig_core_kit::bounds` and `::open`; `jig_engine_clients_kit::bounds` |
| 4.8 | `restarts` | `temper_faults::restarts` (`restarts.md`, 2) |

smith's `WINDOW_ELSEWHERE`, a run resumed on another endpoint with its
saved window, is the draft's 4.6 in words and stays in the `provider`
family, where smith's sweep configures it (`smith/docs/design/faults.md`,
5). The family `bounds` is 4.1's *past its bounds*, the forge's answers
past their limits; the engine's tables at their capacity are 4.7's, `the
engine's own bounds`.

### 2.2 The forge's kinds move into `temper-faults`

The forge world's sweeps keep their kinds as constants of their test
modules (`temper/tests/engine/forge/tests/suite/fuzzy/`), where no other
world can name them. They move, each value unchanged, into two modules
of `temper-faults`:

- **`temper_faults::forge`:** `NO_ROOM`, `UNAVAILABLE`, `TIMED_OUT`,
  `LANDING`, `LATE`, `LIMITED` (`busy`); `HEAD_BEHIND`, `BASE_BEHIND`,
  `MERGE_LATE` (`lagging`); `PULL_FILES`, `COMPARISON`, `JOB_LOG`
  (`bounds`); `CLOSE_REFUSED`, `CLOSED_BY_ANOTHER`, `MERGED_BY_ANOTHER`
  (`refusing`); and each sweep's group: `BUSY: [Kind; 6]`, `LAGGING:
  [Kind; 3]`, `PAST_BOUNDS: [Kind; 3]`, `REFUSING: [Kind; 3]`.
- **`temper_faults::stale`:** `REOPENED`, `SQUASHED`, `MOVED_BASE`
  (`stale`), and `STALE: [Kind; 7]`, the stale sweep's group: `HEAD_BEHIND`,
  `MERGE_LATE`, `REOPENED`, `SQUASHED`, `MOVED_BASE`,
  `CLOSED_BY_ANOTHER`, `MERGED_BY_ANOTHER`. The stale sweep's copies of
  `lagging`'s and `refusing`'s kinds are those constants: one constant
  for each value.

The forge world's sweeps then name these, their draws and assertions as
they are.

### 2.3 Acts and parks: temper's new kinds

`temper_faults::people`, `FAMILY = "people and parties"`, the acts whose
mechanism is temper's:

| Constant | Name |
|---|---|
| `CANCELS_A_GOAL` | `cancelling a goal` |
| `CANCELS_A_CHANGE` | `cancelling a change` |
| `CANCELS_A_LANDING` | `cancelling a landing` |
| `HOLDS_A_MEMBER` | `holding a member of a live landing` |
| `RELEASES_A_MEMBER` | `releasing a held member of a live landing` |
| `ANSWERS_LATE` | `answering a change's question late` |
| `NEVER_ANSWERS` | `never answering a change's question` |
| `ANSWERS_TWICE` | `answering a change's question twice` |
| `ANSWERS_AN_ENDED_RUN` | `answering a question whose run has ended` |
| `WORDS_TO_AN_ENDED_PRODUCER` | `words to a change whose producer has ended` |

Amending a deadline after it has passed is an amendment of a task's
authority, jig's mechanism: temper's worlds name
`jig_core_kit::open::AMENDS_A_PASSED_DEADLINE`, whose exclusion (S15)
is jig's. A sign-in sending a request again while the first is pending
is jig's `jig_engine_clients_kit::open::RESENDS_A_PENDING_REQUEST`.

`temper_faults::parks`, `FAMILY = "runs that park, resume and move"`:

| Constant | Name |
|---|---|
| `A_CHANGE_WAITS_ON_ANOTHER_GOAL` | `a change waits on another goal` |
| `DELIVERED_THEN_PARKED` | `a run that delivered parks and finishes in its next run` |
| `RESUMED_ON_ANOTHER_WORKER` | `a run resumes on another worker` |

A run parking with a question open, its answer arriving while it is
parked, after it ended or to its successor, and a coordinator ending
while a child waits for its answer are jig's, in `jig_core_kit::parks`.

### 2.4 The bounds: a table at its capacity

The draft's 4.7 asks the workload to reach each bound of 3.2's table. A
bound reached is recorded as a fault that falls, so that a sweep's
`check_fell` holds that each bound it configures was reached in some seed:

- **Its kind** is `Kind { family: "the engine's own bounds", name }`,
  `name` the table's name as its adapter reports it
  (`skein/docs/design/testing/inspection.md`, 3): `forge write entries`.
- **In force** from the first quiet point at which the table holds as
  many entries as its capacity, or refuses a newcomer, until the first
  quiet point at which it does neither, recorded by
  `FaultLedger::saturation` (`skein/docs/design/testing/sweeps.md`, 3),
  `Struck::default()`. A table whose capacity is `u64::MAX` (git's locks,
  trash) has no bound to reach.
- **`temper_faults::bounds`**, `FAMILY`, and `pub const fn table(name:
  &'static str) -> Kind`, with a constant for each table a temper
  workload reaches: `FORGE_WRITE_ENTRIES`, `FORGE_READ_ADMISSIONS`,
  `BRANCH_HOLDS`, `LANDING_ROWS` (the engine's adapter,
  `engine-checks.md`, 2.3); `WORKSPACES`, `PREPARE_PERMITS`,
  `PER_RUN_SCRATCH`, `RUN_SLOTS` and `BUILD_PLACES` (`workspaces`,
  `prepare permits`, `per-run scratch`, `run slots`, `build places`: the
  worker's adapter and the domain world's builder, `worker.md`, 2.3).
  jig's tables are jig's (`jig_core_kit::bounds`,
  `jig_engine_clients_kit::bounds`).
- **Their exclusions** act at the moment the world would add the load that
  fills the table (starting a goal, a change or a run), reading their
  conditions through the adapters there; a left-out load is recorded with
  `leave_out`.

### 2.5 The engine's open problems' kinds, renamed

`temper_engine_forge_world::open` (`engine-checks.md`, 5.2) named the
faults its exclusions leave out until their families were built; this
goal builds four of those families, and renames the kinds it names
otherwise, the scenarios, works and conditions unchanged:

| Was | Is |
|---|---|
| `bounds` / `forge write entries full` | `temper_faults::bounds::FORGE_WRITE_ENTRIES` |
| `parties` / `a change waits on another goal` | `temper_faults::parks::A_CHANGE_WAITS_ON_ANOTHER_GOAL` |
| `restarts` / `the engine restarted cold` | `temper_faults::restarts::ENGINE_CRASH`: the same fault, F8's kind |
| `forge` / `a starved gate` | `temper_blame::STARVED`: the same fault, the CI family's kind |
| `state` / `a path deleted on the base` | `stale` / `a path deleted on the base`, a constant of `open` |

`open`'s `runs` kinds and `forge` / `a write refused` stay as they are:
their families are their works'.

## 3. The workload kit

`temper/testing/temper-workload`, crate `temper-workload`: data and draws
alone, over `skein_world::streams` and `skein_lib::Rng`, naming no world,
no kind and no component. Every temper world that runs goals draws its
workload here, so that the engine's domain world and the composed world
run the same goals for the same seed, and their outcomes compare in the
same names. Its modules: `draw` (3.1), `names` (3.2), `parties` (3.3)
and `outcome` (3.4).

### 3.1 The draw

```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Party { First, Second }

/// The least and the most of a count, inclusive.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Span { pub least: u8, pub most: u8 }

/// What a world draws: each world names its own.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Shape {
    pub goals: Span,
    /// Changes of each goal that isn't the train.
    pub changes: Span,
    /// Goals in the milestone's chain; 0 and 1 mean none.
    pub milestone: Span,
    /// Members of the train goal's landing; 0 and 1 mean no train.
    pub train: Span,
    /// An after-landing tier following main.
    pub tier: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Workload { pub goals: Vec<Goal>, pub tier: bool }

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Goal {
    /// `g0`, `g1`, ...
    pub name: String,
    pub party: Party,
    /// The milestone's earlier goal it depends on, if any.
    pub after: Option<String>,
    /// Its changes land as one train of several members.
    pub train: bool,
    pub changes: Vec<Change>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Change {
    /// `g0.c0`, `g0.c1`, ...
    pub name: String,
    /// Changes it builds on: earlier ones of its goal, or the last change
    /// of the milestone goal its goal depends on.
    pub after: Vec<String>,
    pub files: Vec<File>,
    /// Its producer asks one question before it delivers.
    pub asks: bool,
    /// Its first gate is red for its own tree, and one repair makes it green.
    pub red_once: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct File { pub path: String, pub text: String }

impl Workload {
    /// Draws from `Stream::Workload`, part `goals`, alone.
    pub fn draw(streams: &skein_world::streams::Streams, shape: &Shape) -> Workload;
    pub fn goal(&self, name: &str) -> Option<&Goal>;
    /// A change by its name, with its goal.
    pub fn change(&self, name: &str) -> Option<(&Goal, &Change)>;
}

/// The engine's domain world's shape (4.1).
pub const ENGINE: Shape;
/// The composed world's shape (section 5).
pub const COMPOSED: Shape;
```

- **Goals:** a count drawn in `goals`, named `g0` up; each goal's party
  drawn, both parties holding at least one goal once there are two.
- **The milestone:** a count `m` drawn in `milestone`; where `m` is 2 or
  more, the first `m` goals form a chain, each after the one before. Where
  a goal depends on another, its first change names the earlier goal's
  last change in its `after` with chance one half, P113's shape. A goal
  after another starts only once that one has ended (its batch's
  dependency, `temper/docs/design/domain/milestones.md`, 4), so the earlier goal's change has ended by the time this
  `after` is made, and the change never waits on it: no world running
  this draw makes a change wait on another goal (8.1).
- **The train:** where `train` allows one, the last goal outside the
  milestone, or the milestone's last where every goal is in it, is the
  train goal, its changes a count drawn in `train`, at least 2; every
  other goal's changes are a count drawn in `changes`. The train goal
  has no branch of its own and no closer: its changes land into main,
  the one train that takes the project's batch, where a train into any
  other branch takes one unit (`temper/docs/design/domain/landing-turn.md`,
  2; `Forge::train_batch`), so on a goal's branch they would land one by
  one. Every other goal keeps its branch and closer. The train goal's
  changes name none of one another in their `after`: they share one
  repository, and a change after another in its own repository waits
  until that one lands (`forge::joined_changes` joins only dependencies
  across repositories, `temper/crates/temper-engine-domain-forge/src/landing.rs`),
  so an `after` among them would land them one by one, never as one
  train. Their coin is still drawn and its result set aside, so no other
  draw of the seed moves. The train goal's first change keeps a
  milestone's `after` on the earlier goal, which has landed before it
  starts.
- **Changes:** named `<goal>.c0` up; each after the change before it in
  its goal with chance one half, but in the train goal; one or two files at
  `<goal>/c<j>/f<k>.txt`, each text one line naming the change, the file
  and a drawn `u64` in hex, so no two changes write one path and the
  workload makes no conflict of its own: conflicts are the forge's and
  the stale family's to make. `asks` with chance one third; `red_once`
  with chance one quarter, but in the train goal, where its coin is still
  drawn and its result set aside: no CI runs on a pushed head, so a
  train member's first gate is its train's, at the top
  (`temper/docs/design/domain/landing.md`, 3), and a member red there
  leaves the train and lands alone after its repair. A red within a
  train is the faults' to make, never the workload's.
- **The tier:** `tier` as the shape says; its runs are green.
- **The shapes:** `ENGINE` is 2 to 4 goals, 1 to 3 changes, a milestone
  of 0 to 2, a train of 2 to 3, a tier. `COMPOSED` is 3 goals, 1 to 2
  changes, a milestone of 2, a train of 2 to 3, a tier: every element in
  every seed, at the fewest goals that hold them all.

### 3.2 Names

```rust
/// The tasks a world made for the workload's logical names.
pub struct Names { /* both ways */ }

impl Names {
    pub fn new() -> Names;
    /// The task a world made for a goal's or a change's name. A name already
    /// given another task, or a task already given another name, is refused.
    pub fn name(&mut self, logical: &str, task: u64) -> Result<(), String>;
    pub fn task(&self, logical: &str) -> Option<u64>;
    pub fn logical(&self, task: u64) -> Option<&str>;
}
```

A world records each goal's and change's task as the engine answers it.
Numbers a restart doesn't keep stable (attempts, pulls, landings) are the
world's to map through these.

### 3.3 The parties' actions

```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Act {
    CancelGoal, CancelChange, CancelLanding, HoldMember, ReleaseMember,
    AmendPassedDeadline, AnswerLate, NeverAnswer, AnswerTwice, AnswerEndedRun,
    WordsToEndedProducer,
}

/// The logical moment an act waits for, as a world reads it from the durable records.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum When {
    /// The target goal's task exists.
    Started,
    /// The target change's task exists.
    Opened,
    /// The target change's producer has a question open.
    Asked,
    /// The target change is a member of its goal's live landing.
    Landing,
    /// The target change is held as a member of its goal's live landing.
    Held,
    /// The target goal is held for its deadline.
    DeadlinePassed,
    /// The target change's producer has ended.
    ProducerEnded,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Action {
    pub party: Party,
    pub act: Act,
    /// A goal's or a change's logical name.
    pub target: String,
    pub when: When,
    /// The world's driver steps after `when` first holds.
    pub after: u32,
}

/// Draws from `Stream::Parties`, each party from its own part, `first` or
/// `second`, among `acts` alone, over `workload`'s goals and changes.
pub fn draw(streams: &skein_world::streams::Streams, workload: &Workload, acts: &[Act]) -> Vec<Action>;

/// The most steps an act waits after its moment.
pub const AFTER_MOST: u32;
```

| Act | Target | When | Drawn |
|---|---|---|---|
| `CancelGoal` | a goal | `Started` | any goal |
| `CancelChange` | a change | `Opened` | any change |
| `CancelLanding` | a train member | `Landing` | the train goal's |
| `HoldMember` | a train member | `Landing` | the train goal's |
| `ReleaseMember` | the held member | `Held` | only with a `HoldMember` of the same member, which it follows |
| `AmendPassedDeadline` | a goal | `DeadlinePassed` | any goal, which the world then gives a deadline that passes before its work can end; its party drawn from both, so that the amending party isn't always the goal's starter |
| `AnswerLate`, `NeverAnswer`, `AnswerTwice` | a change that `asks` | `Asked` | only where a change asks |
| `AnswerEndedRun` | a change that `asks` | `ProducerEnded` | only where a change asks; the answer to its question, after its run ended |
| `WordsToEndedProducer` | a change | `ProducerEnded` | any change |

- **Each party draws its own:** for each act of `acts`, with chance one
  half, one target among those its row allows of the goals the party
  holds (`AmendPassedDeadline` among every goal); a target takes one act
  at most, but a held member's release. An act no target allows this seed
  isn't drawn. `after` is drawn up to `AFTER_MOST`, so that an act falls at
  any step of its target's life from its moment on. Neither party's draws
  read the other's, nor the workload's stream.
- **Logical events:** an act waits for its moment as the world reads it,
  never for an instant, so a run with restarts and one without take the
  same acts (the draft, 3.5).

### 3.4 The outcome

```rust
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum End { Landed, Done, Failed, Cancelled, Held(String), Waiting }

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GoalOutcome {
    pub end: End,
    /// Its changes whose files main holds, by name, sorted.
    pub landed: Vec<String>,
    /// The kinds of the holds its tasks took, in jig's words, sorted.
    pub holds: Vec<String>,
    /// Its tasks' tries, normalized (section 7).
    pub tries: u64,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Outcome {
    pub goals: BTreeMap<String, GoalOutcome>,
    /// Main's files under the workload's paths at the run's end: path to text.
    pub main: BTreeMap<String, String>,
}

/// A goal left out of a comparison, and why, in the design's words.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Left { pub goal: String, pub why: &'static str }

impl Outcome {
    /// Equal goal by goal, and main equal on each compared goal's paths,
    /// but for the goals `left` names.
    pub fn compare(&self, restarted: &Outcome, left: &[Left]) -> Result<(), String>;
}
```

- **A finding** names the goal, the field and both values: `compare: g1:
  end Landed without restarts, Held("deadline") with them`; a goal one
  outcome has and the other hasn't; a path under a compared goal's
  directory whose text differs or that one main lacks. A `Left` naming no
  goal of either outcome is refused, as a finding.
- **Normalized:** an outcome holds logical names and no number or time the
  engine gives, so two runs compare whatever ids and clocks they took.

## 4. A random workload in each root's world

### 4.1 The engine's domain world

The module `workload` of `temper-engine-domain-world`, over the
forge-route world (`temper/tests/engine/domain/src/forge_routes.rs`): the
engine's root over the fake forge's domain, its agents scripted by tool
calls, at the shipped stock limits, `forge_routes::box_limits()`, as
the forge-route stock stories run. No `small` profile ships (the
service's startup refuses `profile: small`), and the raw `small`
fixtures (`jig_protocol_config_fixtures::small()`,
`temper_limits_fixtures::small()`) do not hold the standing landing
requirements of a branch, a train, a closer and a milestone, so the
domain refuses them at its start. A sweep that needs a table at its
capacity sooner changes that one figure by name on `box_limits()`'s,
with its reason beside it (`scenarios.md`, 1.1); no other figure is set
by hand.

- **It runs `ENGINE`'s draw** as its stories run each element: a party
  starting each goal. The chat's `goal` call
  (`tests/suite/forge_routes/goal_tool.rs`) makes a goal beneath the chat
  with no branch, grants or dependencies of its own, so it starts only a
  goal that needs none; a goal with its own branch and closer is
  delegated by its party through the people's route,
  `people::Ask::Delegate` beneath the party's chat task, with the
  branch's symbol and the closer, as the branch and closer stories make
  it (`forge_routes/goal_closer.rs`, `goal_closes.rs`): an agent's
  `Tool::Delegate` carries no branch symbol, its `symbolic_grants` being
  ordinary grants. A milestone's goals are delegated by the same route
  beneath a gathering milestone, each after the one before
  (`forge_routes/milestones.rs`). The train goal, branchless (3.1), is
  started by the chat's `goal` call where it is independent, and in a
  milestone by the same delegate beneath the gathering milestone with
  its dependency but no branch's symbol or closer. The product's `goal`
  and delegation routes are left as they are. Each goal's changes by its
  coordinator's `change` calls with their `after`; the train goal's
  changes landing together into main, gathered by a latch as section 5 gathers
  them (`forge_routes/landing_trains.rs`, and `forge_routes/landing.rs`'
  `a_train_forms_after_its_due_readiness_steps_settle`); each
  producer writing its files and delivering, asking its question first
  where `asks`, its first gate red where `red_once` and repaired; each
  other goal's closer landing its branch; and the tier following main
  (`temper/tests/engine/domain/src/tier_runs.rs`,
  `forge_routes/goal_tier.rs`). Each goal's party answers its questions
  at once unless an act says otherwise.
- **Its streams:** the workload from `Workload`, the acts from `Parties`,
  its own latencies and the fake forge's delivery order from
  `Scheduling`, part `engine`, and each injector from its `Faults` part.
- **Its checks:** the world kit's `Checks` at every quiet point and `Ends`
  over every goal, each goal's expectation as 6.2 gives it
  (`engine-checks.md`, 3 and 4), with the store check as the world runs
  it (`store-check.md`).
- **`Names`** holds each goal's and change's task.
- **The acts** (`temper_engine_domain_world::acts`), read and fired as
  section 6 says:

```rust
/// The kind an act is, for a sweep's configuration and its ledger.
pub fn kind(act: temper_workload::parties::Act) -> skein_world::faults::Kind;

pub struct Watch { /* each action, when its moment first held, whether it fell */ }

impl Watch {
    pub fn new(actions: Vec<temper_workload::parties::Action>) -> Watch;
    /// The actions due at `step`: each whose moment holds in the store's
    /// committed rows, its target resolved through `names`, `after` steps
    /// since it first held. Each is due once.
    pub fn due(&mut self, store: &crate::store::Store, names: &temper_workload::names::Names, step: u64) -> Vec<temper_workload::parties::Action>;
}
```

  The composed world reads the same store, and takes `Watch` as it is
  (section 5).

### 4.2 The engine's forge world

The forge world's landing-train workload
(`temper/tests/engine/forge/tests/suite/fuzzy/landing_batches.rs`) draws
from `Workload`, part `landing_batches`, in every run, as its sweeps
already derive it. Parties' acts on the train fall in it at any step
(section 6): cancelling a landing, holding a member, releasing it, and
cancelling a member's change, each drawn from `Parties`, part `first`,
and each fired at its moment as the forge world reads it from the forge
domain's state: a landing live with the member in it; the member held.
The world has no jig core, so it keeps its own small watch over the same
`Action`s.

### 4.3 jig's worlds

The core world's workload of tasks, questions, words, funding and
authority, with people's acts, parks and its tables' bounds, and the
engine-clients world's sign-ins, are jig's
(`jig/docs/design/domain/testing-workloads.md`).

### 4.4 The worker's domain world

The module `whole::workload` of `temper-worker-domain-world`: one to
three runs at once, each one of the domain world's stories
(`temper/tests/worker/domain/src/whole/faults.rs`' draw), started at
drawn steps over one domain at the world's limits, so that its tables
reach their capacity; drawn from `Workload`, part `worker`, as that draw
is. Five of those stories strike the whole worker (a drain, a drain that
expires, a link lost that drains, one whose drain expires, and a contact
whose grace passes, which cancels every run), so runs at once share
them: the cohort's first run draws among all eight; when it draws one of
the five, every run of the cohort takes that story, each started at its
own drawn step, and the one worker-wide event falls once, on all of
them; otherwise each other run draws among the three that strike one run
(delivered, a delivery cancelled, a contact regained within its grace).
Every run then ends as its story says. Its parks:

- **`DELIVERED_THEN_PARKED`:** a run delivers, parks, and finishes in its
  next run with the commit it delivered, as
  `temper/tests/worker/domain/src/whole.rs`' `delivered_across_park`
  scripts it.
- **`RESUMED_ON_ANOTHER_WORKER`:** a parked run's next run is placed on
  another worker's domain, with a disk of its own, and goes on from the
  work its first run saved on the branch.

Each is drawn from `Faults`, part `worker`, at a drawn step of its run,
and recorded as it falls. A run a park strikes ends as its park says, in
place of its drawn story; the workload's draw is the same whether a park
is drawn or not, and a cohort's worker-wide event falls after its parks,
on every run still live. The worker's adapter and the builder's table
are checked at every quiet point as they are (`worker.md`, 4).

### 4.5 smith's worlds

Each of smith's sweeps (`smith/docs/design/faults.md`, 5) draws the runs
it strikes from `Workload`, part `runs`: how many, their calls, turns and
tool calls, where a world has them. The protocol and agent worlds draw
so already; the agent-process and run worlds' sweeps move to it, their
faults' draws keeping their parts.

## 5. The composed workload

The module `composed::workload` of `temper-system-world`: the composed
system world (`tiers.md`, 1.1) running `COMPOSED`'s draw.

- **Two parties on two accounts:** each party a person with its own key
  account on the one project, as `World::two_accounts` makes them
  (`checks.md`, 6.3); each goal on its party's account.
- **One forge:** the fake forge serves the engine's API and the worker's
  git, with the project's repository and its main.
- **Its goals at once:** every goal started as the workload says, by its
  party as 4.1 starts it (a milestone's goals by the party's
  authenticated delegate beneath a gathering milestone, each with its
  dependency; a goal needing no branch of its own by the chat's `goal`
  call), the
  milestone's later goal after the earlier one's landing, the train goal's
  changes landing as one train into main (3.1), every other goal's
  closer landing its branch, and the tier following main. The train is the world's to gather, never
  the engine's, by the readiness the product ships: a change kept by a
  dependency is not ready (`temper/docs/design/domain/landing.md`, 3),
  and a train forms once its due readiness steps settle. The train
  goal's coordinator delegates one report task, the latch, to its
  party's person before its `change` call, and each member's
  dependencies name it (by the member's `after` where the call admits
  it, else by amending the member as
  `forge_routes/landing.rs`' `a_train_forms_after_its_due_readiness_steps_settle`
  does); the party reports on the latch once every member is gating with
  no procedure step due, so every member becomes ready in one settling
  and the train forms with them all. Holding producers' finishes is not
  enough: members opened at different times are ready at different
  times, and a train forms as soon as it frees. The engine's landings,
  holds and batching run as they ship; a train goal's member is never
  `red_once` (3.1). smith's activations take turns on the one
  agent world, as the system world runs them; the goals are live at once
  in the engine.
- **Cued by names and trees:** each activation's script is chosen by the
  logical name its prompt holds (the goal's or the change's), never by the
  order activations come in, so that a goal's agents do the same work at
  every seed and under any interleaving; each change's CI reads its
  verdict from its tree, red once where `red_once`, so that its gates are
  the same at every seed.
- **A producer's question to its party** is a person task, as jig's
  workload asks one (`jig/docs/design/domain/testing-workloads.md`, 2):
  where `asks`, the producer delegates one task to its goal's party's
  person, and the party's report on that task is the answer, at once
  unless an act says otherwise. A producer holds no reference to its
  party's chat (the engine's references admit its requester, its
  delegates and those it was introduced to), and the world wires no
  introduction for it.
- **Its streams:** the workload from `Workload`; the acts from `Parties`;
  the world's scheduling from `Scheduling`, part `system`; each injector
  from its `Faults` part.
- **Its checks:** the engine's world kit and `Ends` at its ticks and quiet
  points, as `tick_at_now` calls them; the worker's and jig's adapters
  beside the engine's at the same quiet point (`checks.md`, 6.1); the
  store check; and the account runs' judge where the accounts' faults
  fall (`checks.md`, 6.3).
- **`acts::Watch`** of 4.1 over the composed world's store, each act fired
  through the composed world's routes for people.

## 6. People's actions and runs that park, resume and move

### 6.1 At any step

Each act waits for its moment (3.3) and then its `after` steps of the
world's driver, so that over the seeds it falls at every step of its
target's life. The world records it in its ledger as it fires it,
`FaultLedger::inject(kind, now, Struck::default())`, an instant, its
`request` naming the target's logical name. A parks kind falls as the
world makes it, and is recorded the same way.

### 6.2 What each act expects

`Ends` (`engine-checks.md`, 4.1), and jig's expectation in its worlds:

| Act on a goal | The goal expects |
|---|---|
| none | `Untouched` where another act strikes another goal of the run; `Ends` otherwise |
| cancelling the goal | `Ends`: ended cancelled, its changes ended |
| cancelling a change, or a landing | `Ends`: the goal goes on as its procedure says, landed or ended |
| holding a member, never released | `Waits` |
| holding a member and releasing it | `Ends` |
| amending its passed deadline | `Ends` |
| answering late, twice, or after the run ended | `Ends`: an answer twice reaches its question once; one after the run ended is dropped, with its one notice |
| never answering | `Waits` |
| words to an ended producer | `Ends`: the words take the route's fallback |
| a change waiting on another goal | `Ends` |

A goal the faults strike takes the faults' expectation as each family's
design gives it, the later bound of the two where both apply.

### 6.3 Where each falls

| Act or park | Worlds |
|---|---|
| cancelling a goal or a change; answering late, never, twice or after the end; words to an ended producer; amending a passed deadline | the engine's domain world; the composed world |
| cancelling a landing; holding and releasing a member | the engine's forge world, its cheapest; the engine's domain world; the composed world |
| a change waiting on another goal | the engine's domain world; the composed world |
| a run that delivered parks and finishes next; a run resumed on another worker | the worker's domain world; the composed world |
| a task's acts, questions through parks, a coordinator ending while its child waits, a sign-in's resend | jig's worlds (`jig/docs/design/domain/testing-workloads.md`) |
| a run resumed on another endpoint with its saved window | smith's agent world (`smith/docs/design/faults.md`, 5) |

## 7. A restart changes no outcome it shouldn't

The draft's 3.5, the restart drafts' oracle.

- **Where:** the engine's domain world over its workload, with the
  restart family's `ENGINE` kinds drawn by its injector
  (`temper/tests/engine/domain/src/restarts.rs`; `restarts.md`, 4); and
  the composed world over its workload, with `MID_TURN`'s
  (`temper/tests/system/src/composed/restarts.rs`; `restarts.md`, 7).
  Those are the worlds whose restarts strike goals' durable outcomes; a
  worker world's restarts strike runs, which 3.4 and its own checks hold.
- **Comparable runs:** each seed runs twice from `Streams::new(seed)`:
  once with the restart injector off, once with it on. The workload,
  every other fault and the acts draw the same in both, from their own
  streams; the acts are replayed by their moments (3.3).
- **Each run's outcome,** `temper_engine_domain_world::outcome::read`:

```rust
/// The run's outcome from the store's committed rows and main's files.
pub fn read(
    store: &crate::store::Store,
    main: &BTreeMap<String, String>,
    names: &temper_workload::names::Names,
    workload: &temper_workload::draw::Workload,
    ledger: &skein_world::faults::FaultLedger,
) -> temper_workload::outcome::Outcome;
```

  A goal's end from its task's row: landed where its closer landed its
  branch, otherwise done, failed or cancelled, held with its hold's kind,
  or waiting; its changes landed from main's files; its holds' kinds from
  its tasks' holds; main from the forge.
- **Tries, normalized:** each goal's tasks' tries, less each attempt that
  ended lost while a crash of the ledger (`ENGINE_CRASH`,
  `ENGINE_CRASH_MID_TURN`) was in force: the draft's 3.3 table spends one
  try for a run a crash loses, today's design, and none for a settled
  stop or a restart announced to the workers, so those compare equal.
- **What holds:** `Outcome::compare` of the run without restarts against
  the run with them.
- **Left out by name:** a goal any of whose deadlines, or a bound its
  procedure arms over it, fell within a restart's interval in the run
  with restarts, as `Left { goal, why: "a deadline racing a restart" }`;
  its recovery is `Ends`', which holds on both runs as it does on every
  run. Nothing else is left out.
- **Both runs** keep every check of the world as it stands: 3.4's
  openings and goals, 3.7's store check.

## 8. The sweeps

### 8.1 One per family, in the cheapest world

A family whose faults fall in different worlds takes a sweep in each
world that is cheapest for some of them, as smith's providers take two
(`smith/docs/design/faults.md`, 5). The sweeps the milestone's other
goals built stay as they are, each its family's in its world: the
forge's and stale state's in the forge world (`blame.md`;
`fakes.md`, 2), the providers' and the worker's in smith's worlds and
the worker's (`smith/docs/design/faults.md`, 5; `worker.md`, 4), the
accounts' in the system world (`checks.md`, 6.3), and the restarts' in
each world (`restarts.md`, 9). This goal adds:

| Sweep | World | Family | Configures | Takes |
|---|---|---|---|---|
| `people_act_on_goals_at_any_step` | the engine's domain world | people and parties | `CANCELS_A_GOAL`, `CANCELS_A_CHANGE`, `CANCELS_A_LANDING`, `HOLDS_A_MEMBER`, `RELEASES_A_MEMBER`, `ANSWERS_LATE`, `NEVER_ANSWERS`, `ANSWERS_TWICE`, `ANSWERS_AN_ENDED_RUN`, `WORDS_TO_AN_ENDED_PRODUCER`, jig's `AMENDS_A_PASSED_DEADLINE` | `open::exclude`; jig's S15 exclusion |
| `people_act_on_the_landing_train` | the engine's forge world | people and parties | `CANCELS_A_LANDING`, `HOLDS_A_MEMBER`, `RELEASES_A_MEMBER`, `CANCELS_A_CHANGE` | `open::exclude` |
| `the_engines_tables_reach_their_bounds` | the engine's domain world | the engine's own bounds | `FORGE_WRITE_ENTRIES`, `FORGE_READ_ADMISSIONS`, `BRANCH_HOLDS`, `LANDING_ROWS` | `open::exclude` |
| `a_restart_changes_no_outcome` | the engine's domain world | restarts, compared (section 7) | `temper_faults::restarts::ENGINE` | `open::exclude` |
| `worker_runs_park_resume_and_move` | the worker's domain world | runs that park, resume and move | `DELIVERED_THEN_PARKED`, `RESUMED_ON_ANOTHER_WORKER` | none |
| `the_workers_tables_reach_their_bounds` | the worker's domain world | the engine's own bounds | `WORKSPACES`, `PREPARE_PERMITS`, `PER_RUN_SCRATCH`, `RUN_SLOTS`, `BUILD_PLACES` | none |
| jig's four | jig's core and engine-clients worlds | people and parties; runs that park, resume and move; the engine's own bounds | `jig/docs/design/domain/testing-workloads.md`, 7 | jig's exclusions |
| `every_family_falls_in_the_composed_workload` | the composed world | every family (8.2) | 8.2 | 8.3 |
| `a_restart_changes_no_composed_outcome` | the composed world | restarts, compared (section 7) | `temper_faults::restarts::MID_TURN` | none |

- **The loop** is skein's (`skein/docs/design/testing/sweeps.md`, 4): for
  each seed it writes the seed to stderr, runs the world with
  `Streams::new(seed)`, folds the run's ledger and returns its findings
  at once; after the last seed, `check_fell`. A comparison's seed folds
  the ledger of its run with restarts.
- **A kind no draw of the world can reach** at its limits is not
  configured: it is named, with why, in a comment beside its sweep's
  `configure` calls, as smith's sweeps name theirs
  (`smith/docs/design/faults.md`, 5), and the design's status says so.
- **`A_CHANGE_WAITS_ON_ANOTHER_GOAL` takes no sweep:** the workload's
  milestone starts a goal only once the goal it depends on has ended, so
  the `after` its first change names on the earlier goal's change never
  waits (3.1), and a draw that started the later goal while the earlier
  lives would change what a milestone is. The kind stays in
  `temper_faults::parks`, named by `open`'s P113 exclusion, whose ignored
  scenario `a_change_waits_on_another_goal` makes the wait by hand; the
  family's other kinds fall in the worker's sweep and jig's.
- **A sweep's checks** are its world's as they stand, with `Ends` over
  every goal where the world runs goals, and its acts' expectations.

### 8.2 The composed sweep mixing every family

`every_family_falls_in_the_composed_workload` configures one kind of each
family, the one production met most, so that every seed can mix them
all and the fewest seeds make each fall:

| Family | Kind | Drawn from |
|---|---|---|
| busy | `temper_faults::forge::TIMED_OUT` | `Faults`, `forge` |
| lagging | `temper_faults::forge::HEAD_BEHIND` | `Faults`, `forge` |
| bounds | `temper_faults::forge::JOB_LOG` | `Faults`, `forge` |
| refusing | `temper_faults::forge::CLOSED_BY_ANOTHER` | `Faults`, `forge` |
| ci | `temper_blame::LOST` | `Faults`, `forge` |
| stale | `temper_faults::stale::MOVED_BASE` | `Faults`, `forge` |
| provider | `skein_llm_kit::faults::RATE_WINDOW` | `Faults`, `llm` |
| worker | `temper_worker_inspect::faults::DISK_FULL` | `Faults`, `worker` |
| people and parties | `HOLDS_A_MEMBER` and `RELEASES_A_MEMBER`, on the train | `Parties` |
| runs that park, resume and move | `DELIVERED_THEN_PARKED` | `Faults`, `worker` |
| the engine's own bounds | `FORGE_READ_ADMISSIONS` | observed (2.4) |
| restarts | `ENGINE_CRASH_MID_TURN` | `Faults`, `restarts` |

- **Each seed** draws every family's kind, its moment a drawn step of the
  goal it strikes, from its own part, so that dropping a family leaves the
  others' draws as they were. Each fault stops by a moment the run names,
  so every goal expects `Ends` past the faults' last moment, but a held
  member's goal until its release (6.2).
- **Where the composed world can't make a kind** (its worker keeps no fake
  disk, say), the sweep takes the family's next kind the world reaches,
  named in its comment with why: `PLACEMENT_BUSY` for the worker; another
  table the workload fills for the bounds. Every family keeps one kind.
  Three of the table's kinds are such replacements: `JOB_LOG` stands for
  `PULL_FILES`, since the composed workload's repairs read their gate's
  evidence and never a pull's files; `MOVED_BASE` stands for `SQUASHED`,
  since a dependent goal starts once its predecessor has landed, so no
  checkout taken early is left with a squashed parent;
  `FORGE_READ_ADMISSIONS` stands for `FORGE_WRITE_ENTRIES`, since the
  composed workload makes too few concurrent writes to fill the stock
  write table, and its faulted world bounds the read admissions to two,
  recorded at its quiet points (2.4).
- **The kinds are a list the caller gives,** and each family's draw keeps
  a fixed place in its part, so that leaving one kind out moves no other
  kind's draw.
- **3.3 for runs** holds where the provider's fault falls, by the account
  runs' judge (`checks.md`, 6.3); 3.3 for gates where CI's falls, by
  `temper_blame` (`blame.md`).

### 8.3 Exclusions

- **Each sweep takes the exclusions** of the open problems its kinds
  strike, as their owners give them: the engine's
  `temper_engine_forge_world::open::exclude` (`engine-checks.md`, 5.2),
  jig's `jig_core_kit::open::EXCLUSIONS` and
  `jig_engine_clients_kit::open::EXCLUSIONS`
  (`jig/docs/design/domain/testing-fakes.md`, 1.3), each acting before its
  fault falls, its conditions read through the adapters or the world's
  state at that moment, and nothing broader (the draft, 6).
- **P124's first combination,** cancelling a landing while its gate's
  dispatch is in flight, takes no exclusion: its fix landed, and
  `cancelled_landing_frees_its_write_entries` is active in the forge
  world's suite (`engine-checks.md`, 5.1).
- **A finding** fails its sweep, as any red. One that is the sweep's own
  defect is repaired in its change. One that is the product's and outside
  the draft's catalog is asked up before anything is written for it; once
  confirmed, it takes its scenario in its lowest harness, `#[ignore =
  "failure worlds: <what failed>, repaired as a red"]`, and the narrowest
  exclusion its world reads, so that the sweep holds everything else and
  the problem is repaired as a red (the request, *Out*).

### 8.4 Room

- **The user's figures:** each sweep runs in its world's fuzzy suite, in
  its binary's layer, within `tools/tiers/budgets.toml` as it stands,
  with the moves the user made from F7's room on 2026-11-22
  (`../../development/failure-worlds-room.md`, *The moves for the user*).
  This design raises nothing.
- **Seeds:** the fewest at which every kind it configures falls, at
  least one, pinned in the test.
- **Measured:** each change gives its new sweeps' CPU and instructions
  from the suites' own reports. A layer its sweeps take over its budget,
  or past its growth allowance, is red (`tools/tiers/budgets.md`, 8), and
  is repaired as `skein/docs/foundation/testing-strategy.md`, 8.1 says:
  the same checks for less where they can be made faster; otherwise the
  measurements go to the user as a question. No sweep takes fewer seeds
  or checks to fit.

## 9. Testing

Focused and fuzzy suites only.

- **The kit** (`temper-workload`'s own step tests): one seed draws one
  workload, and the same one whether or not `Parties` was drawn first;
  each shape's spans hold, with `COMPOSED` holding every element at every
  seed; names are unique and no two changes write one path; a change's
  `after` names only earlier changes of its goal, or of the goal its goal
  depends on. The parties' draw takes only the acts it is given, each on a
  target its row allows, a release only after its hold, and one party's
  draws are the same whatever the other's acts. `Names` refuses a second
  task for a name and a second name for a task. `compare` passes equal
  outcomes, finds each field's difference naming the goal, skips a left
  goal and its paths, and refuses a `Left` naming no goal.
- **The vocabulary** (`temper-faults`' own tests): every kind's family is
  its module's; names are unique within each family; each group holds its
  sweep's kinds; the moved forge kinds keep their values; `bounds::table`
  of each adapter's table name gives its constant.
- **`saturation`** (skein-world's step tests,
  `skein/docs/design/testing/sweeps.md`, 6).
- **`Watch` and `read`** (the engine's domain world's focused suite): an
  act due only once its moment holds and its steps have passed, and once;
  an outcome read from a run of one goal that lands, and one cancelled.
- **Each sweep** of section 8, in its world's fuzzy suite: each kind it
  configures fell, its world's checks and its goals' expectations held.
  A comparison shows, in its world's focused suite, one control: a run
  whose restarted side is given another outcome is found, naming the
  goal.

## 10. Open questions

- **Room in instructions:** temper's fuzzy domain and system layers, and
  jig's fuzzy domain, are judged in instructions
  (`tools/tiers/budgets.md`, 1), and the user's moves of 2026-11-22
  carried CPU seconds alone. Whether this goal's sweeps fit their
  layers' instruction budgets is measured as each lands (8.4); a sweep
  that doesn't fit goes to the user with its figures.

## 11. Later

- The comparison of 3.5 in jig's worlds and the worker's, whose restarts
  strike tasks and runs rather than goals' outcomes.
- A live run's twin replaying a stopped deployment's agents in the
  composed workload (the draft, 12).
- The composed workload at more goals than its shape's fewest, as the
  system layer's room allows.
