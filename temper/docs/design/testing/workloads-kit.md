# Drawing and comparing workloads

The kit draws seeded workloads and parties' acts, and compares outcomes in
logical names. The worlds that run the workloads are in `workloads.md`.

## 1. The workload kit

`temper/testing/temper-workload`, crate `temper-workload`: data and draws
alone, over `skein_world::streams` and `skein_lib::Rng`, naming no world,
no kind and no component. Every temper world that runs goals draws its
workload here, so that the engine's domain world and the composed world
run the same goals for the same seed, and their outcomes compare in the
same names. Its modules: `draw` (1.1), `names` (1.2), `parties` (1.3)
and `outcome` (1.4).

### 1.1 The draw

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

/// The engine's domain world's shape (workloads.md, 4.1).
pub const ENGINE: Shape;
/// The composed world's shape (workloads.md, section 5).
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
  this draw makes a change wait on another goal (`workloads-sweeps.md`, 1.1).
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

### 1.2 Names

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

### 1.3 The parties' actions

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
  same acts (`../draft/failure-worlds.md`, 3.5).

### 1.4 The outcome

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
    /// Its tasks' tries, normalized (workloads.md, section 7).
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

## 2. Open questions

None.
