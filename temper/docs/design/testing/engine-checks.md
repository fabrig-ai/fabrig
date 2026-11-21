# The engine's world checks

Provisional, 2026-11-21. What every engine world holds of the engine,
beyond its own stories: every wait of a landing or a change has a live
wake, every entry of the engine's bounded tables is released by its own
rule, finite goals end or are held with words within explicit bounds, and
the engine opens within a bound after a restart. And the engine's open
problems, kept as ignored scenarios with their sweeps' exclusions.
Written from `../draft/failure-worlds.md`, 3.1, 3.2, 3.4, 6 and section
7's engine rows, on skein's harness: the inspection adapter's form and
its two shared checks (skein's `testing/inspection.md`), and the fault
ledger, sweeps and exclusions (skein's `testing/sweeps.md`). `checks.md`,
6.1 gives every component's adapter in one place; this document is the
engine's in full.

## 1. In one page

- **The engine's inspection adapter** (section 2), `temper/testing/temper-engine-inspect`:
  each live landing's named wait and each change's step, with what wakes
  it, and the forge's bounded tables (write entries, read admissions,
  branch holds, landing rows with their asks), each entry with its
  releasing rule and that rule's state. It reads by shared borrow,
  between iterations.
- **The engine's world kit** (section 3), in `temper-engine-domain-world`:
  one `Checks` every engine driver calls at its own quiet point, which
  runs the adapter and skein's two checks and arms a referee deadline for
  each bounded wait, so that a wait starved while other work goes on is
  found at its bound. The forge world, which drives the forge component
  alone, calls the same checks over the forge's part of the adapter.
- **Goals end** (section 4): a referee expectation over durable records,
  `Ends`: each goal a scenario names ends, is held with words naming its
  fault, or keeps going untouched, within a bound in simulated minutes and
  in steps; and after every restart the engine opens within a bound.
- **Open problems** (section 5): thirteen ignored scenarios, named as the
  draft's section 7 names them, each ignored with the work that fixes it,
  and an exclusion for each that every engine sweep drawing its fault
  takes.
- **Out:** jig's, the worker's and smith's adapters and checks; the fault
  families (the draft, section 4); the settings tests and the store check
  (the draft, 3.6 and 3.7), whose call the settings goal adds to the kit's
  `Checks` and nothing else.

## 2. The adapter

### 2.1 Its form

`temper/testing/temper-engine-inspect`, package `temper-engine-inspect`,
of skein's form (skein's `testing/inspection.md`, 2), component `engine`,
two functions, one per child a world holds:

```rust
/// The root: its forge component's waits and tables, with what only the root knows.
pub fn inspect(root: &temper_engine_domain::Domain, into: &mut skein_world::inspect::Inspection);
/// The forge component alone, as the forge world drives it.
pub fn forge(forge: &temper_engine_domain_forge::Domain, into: &mut skein_world::inspect::Inspection);
/// The waits section 2.2 documents, each in its design's words.
pub const KINDS: &[&str];
```

- **`inspect` gives what `forge` gives,** with what only the root knows in
  place of the forge's own reading: a landing's prepare job by its task
  number, an owner's end as jig's tasks record it, and the root's line of
  tasks held for a forge table's room (`forge_limit_holds`).
- **Its reads** are the components' `pub` reads, each private table or
  map gaining one documented as its adapter's (skein's
  `testing/inspection.md`, 2): a borrow, a count, a list of keys or an
  entry by key, in the strict subset. The root gains
  `pub const fn forge(&self) -> &temper_engine_domain_forge::Domain`; the
  rest are the adapter's choice.
- **Names:** every waiter and owner is `task N`, its task's number, as in
  every adapter of a world (`checks.md`, 6.1); an outbox entry is
  `entry N`; a branch hold is its resource's name.

### 2.2 Waits

Each live landing is in the wait its row keeps (`landing-waits.md`,
2), kind `landing: WAIT`, WAIT the table's first column there, or
`landing: room` while it holds a place in the outbox's line. Each change
not ended is in the wait of its step, kind `change: STEP`. Every wait
also names, as a `Timer`, the deadline the component has armed for it,
where one is armed: a landing's poll (`landing-waits.md`, 3) and a
change's due step.

| Kind | Wakes, besides its armed deadline |
|---|---|
| `landing: release` | `Party("person")`: a person's release |
| `landing: ready`, `landing: judgement` | `Waiter(task C)`, the member's change |
| `landing: turn` | `Waiter(task L)`, the landing holding the turn; none where the turn is free |
| `landing: prepare` | `Waiter(task J)`, the round's live prepare job, from `inspect`; `Party("prepare job")` from `forge` alone; `Ended("prepare job")` where the root has no live prepare child |
| `landing: open`, `close`, `delete`, `post`, `merge`, `record` | `Party("forge: entry N")`, the landing's write in the outbox for it; `Ended("<wait> write")` where none is outstanding |
| `landing: prefixes`, `run`, `receipt`, `ancestry`, `read` | its deadline alone |
| `landing: room`, `change: room` | for each entry holding the outbox's room: `Waiter(task T)`, its owner, while its owner lives; `Ended("entry N of task T")` once its owner has ended |
| `change: held` | `Party("person")`: a person's release |
| `change: ready`, `change: preparing` | `Waiter(task L)`, its landing |
| `change: ready` with no landing | while the forge's limits refuse its train a landing, `Waiter(task L)` for each live landing; otherwise its landing's making, as the root's turn below |
| every other `change:` step | its live delegate (producer, repair, resolution, gate or merge job), `Waiter(task D)`; its write in flight, `Party("forge: entry N")`; its open question's answerer, `Waiter(task R)` or `Party("person")`; else the root's turn below |

- **The root's turn:** a change the forge has decided an act for that
  the root carries out (a delegate to make, a landing task to make,
  a decision's effect to run), with no other wake, waits on the root.
  `forge` gives `Party("root")` for it while that act stands unanswered
  in the forge's state, read from the state itself (the step's
  procedure state awaiting the root's answer), never as a fallback for
  a wait it can't name: the forge world's scripted root answers in its
  own time. `inspect` gives none: the root carries out each act in the
  step that decides it, so a change still awaiting one at the root's
  quiet point has no wake, and is a finding.

- **The change's steps,** in the change procedure's words
  (`../domain/forge/change/README.md`, 2): `producing`, `opening`,
  `checking`, `gating`, `queued`, `first`, `landing`, and beside them
  `ready`, `preparing` and `judging` in a landing, `updating`,
  `resolving`, `repairing`, `cancelling gates`, `held` and `room`. The
  adapter maps each state of the change's procedure to one of these; an
  ended change waits on nothing.
- **Bounds:** a change's wait has the bound its stall gives
  (`../domain/forge/change/stalls.md`, 1): the stall's start plus the
  deployment's stall, where the change's row keeps the moment the stall
  counts from and no exemption stands; none otherwise. A landing's
  deadlines are polls, not bounds (`landing-waits.md`, 3), and a wait for
  room has no bound in its design (`../domain/forge/room.md`, 4), so
  neither has one. A wait past its bound at a quiet point is a finding
  (skein's `testing/inspection.md`, 4.1): the stall should have held it.
- **What it finds:** a landing or a change whose every wake has ended or
  waits on one another in a cycle, with no deadline armed; a room wait
  behind an outbox whose every entry outlived its owner (the draft's
  P124).

### 2.3 Tables

| Table | Capacity | Entries | Released by | Due once |
|---|---|---|---|---|
| `forge write entries` | `forge.entries` | each outbox entry, `entry N (task T)`, and each reservation of its room, `reserved for task T` | the write's settlement, or its owner's end with the entry withdrawn | its owner has ended and no write or withdrawal of it is outstanding with the forge client |
| `forge read admissions` | `forge.client.pending` | each call the forge client admitted, `call N`, with its task where its reader names one | the read's answer | its answer has come and its place isn't reclaimed at the quiet point |
| `branch holds` | `forge.holds` | each hold, `NAME (task T)` | the closing settlement of the landing or change that holds it | its holder's task has ended and no release of it is under way |
| `landing rows` | `forge.landings` | each landing's row with its asks, `landing L` | the closing settlement of the landing | the landing has ended, landed or cancelled, and its row still names write entries or holds an ask not handed over |

- **Kept:** an ended landing's row naming nothing, kept until a new
  landing needs its place, as the forge retires ended rows. A hold its
  holder's failed ending hands to the root is the root's, and pending
  while the root lives.
- **Refused newcomers,** each `task N`, with no bound, since none of
  these waits has one in its design:
  - `forge write entries`: each waiter in the outbox's line
    (`../domain/forge/room.md`, 3), a change, a landing, a making or a
    release, and each task the root holds for the table's room;
  - `forge read admissions`: each reader the forge keeps waiting for read
    room with no deadline of its own (a landing's gather, a lost,
    written or goal branch read waiting for room, a brief fetch waiting
    for the client's reclaim); a reader that retries at its own deadline
    is a wait, not a refusal;
  - `branch holds`: each task the root holds for the table's room;
  - `landing rows`: none, since a change ready while no place is free
    stays ready and waits for room, woken by a landing's end
    (`../domain/landing.md`, 12).
- **Reservations count as entries,** so that a waiter refused while the
  line's room is reserved for another is no finding (skein's
  `testing/inspection.md`, 4.2).
- **Build places** are the worker's, not the engine's
  (`../domain/worker/README.md`; `checks.md`, 6.1, the worker's adapter):
  the engine keeps none.

## 3. The world kit

### 3.1 `Checks`

The module `checks` of `temper-engine-domain-world`, which every world
of the engine's root depends on (`README.md`, 5):

```rust
pub struct Checks { /* seed, the bounded waits seen at the last quiet point, the first finding */ }

impl Checks {
    pub fn new(seed: u64) -> Checks;
    /// At the driver's quiet point: inspects the root, runs skein's
    /// `check_waits` and `check_tables`, and arms a deadline for each bounded wait.
    pub fn quiet(&mut self, root: &temper_engine_domain::Domain, at: skein_lib::Time, iteration: u64) -> Result<(), String>;
    /// At every step: a wait seen at the last quiet point whose bound has
    /// passed, with no quiet point since, is a finding.
    pub fn step(&mut self, at: skein_lib::Time, iteration: u64) -> Result<(), String>;
}
```

- **A referee deadline for each bounded wait** (the draft, 3.1): `quiet`
  keeps each wait with a bound; `step` finds one whose bound has passed
  while no quiet point has come since, so a wait starved by other work is
  found at its bound. A quiet point forgets the waits it no longer sees.
- **A finding** has skein's form (skein's `testing/inspection.md`, 6),
  component `engine`; a step's names the waiter, its kind and its bound.
- **The settings goal adds its store check's call** here, and nothing
  else (`../draft/failure-worlds.execution.md`, 2).

### 3.2 Where each driver calls it

A driver's quiet point is where it has drained its immediate work and
before it moves time (skein's `testing/inspection.md`, 5), as each
defines it; a driver that moves time every iteration has one where its
queues, its outputs and its pending commits are empty.

- **The direct driver,** `direct::Driver`: at its fence, as `drain`
  leaves it quiet, and as `settle` ends.
- **The walking, escalation and roles worlds,** and **the forge-route
  world** (`forge_routes::World`): at their quiet points between
  iterations, and `step` at each iteration.
- **The forge world** (`temper-engine-forge-world`), below the kit: its
  `World::check(&self) -> Result<(), String>` runs `forge` and the two
  checks, at the quiet point where `iterate` moves time.
- **The composed system world** (`README.md`, 2.1): at the end of each
  tick whose immediate work is drained.
- **Each run's finding:** a driver keeps the first finding and its loop
  panics with it, as a run's finding (`README.md`, 4.2); a returning form
  (`World::check`, `Checks::quiet`) is what a check's own test reads.

## 4. Goals end

### 4.1 `Ends`

The module `ends` of `temper-engine-domain-world`: a referee expectation
(the draft, 3.4), which observes the store's durable rows, the driver's
steps and clock, and the driver's reading of the root's readiness as the
shell reads it to announce its start, never the root's state.

```rust
/// Both nonzero: simulated minutes, and the driver's steps, the step bound
/// catching a loop that works without moving time.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bound { pub minutes: u64, pub steps: u64 }

pub enum Expect {
    /// Ends, landed or ended as its procedure says, within its bound past
    /// the faults' last moment.
    Ends,
    /// Struck by a fault that never stops: held within its bound, its
    /// hold's words holding these words.
    Held(&'static str),
    /// Untouched by the run's faults: ends within its bound from when it
    /// is named, as if they weren't there.
    Untouched,
    /// A party holds it, or is scripted never to answer: it may stay held
    /// or waiting, which the scenario names.
    Waits,
}

pub struct Ends { /* expectations, the faults' last moment, restarts and openings, steps */ }

impl Ends {
    /// `opening`: the bound within which the engine opens after a restart.
    pub fn new(seed: u64, opening: Bound) -> Ends;
    /// A task the scenario names, with its outcome and bound; a zero bound is refused.
    pub fn expect(&mut self, task: u64, expect: Expect, bound: Bound) -> Result<(), String>;
    /// The faults' last moment, which the scenario names.
    pub fn faults_stop(&mut self, at: skein_lib::Time);
    pub fn restarted(&mut self, at: skein_lib::Time);
    /// The driver saw the root ready, as the shell does before its first period.
    pub fn opened(&mut self, at: skein_lib::Time);
    /// Each driver step, over the store's committed rows.
    pub fn step(&mut self, at: skein_lib::Time, store: &crate::store::Store) -> Result<(), String>;
    /// At the run's end: an expectation neither met nor past its bound is a finding.
    pub fn close(&mut self, at: skein_lib::Time, store: &crate::store::Store) -> Result<(), String>;
}

/// The opening bound every engine driver applies after a restart.
pub const OPENING: Bound;
```

- **Ended** is the task's ended row, done, failed or cancelled; **held
  with words** is its live row held, its hold's words, as jig keeps them
  with the hold, holding the expected words.
- **Bounds count** from the later of the faults' last moment and the
  last opening, and for `Untouched` from when it is named: a restart is a
  fault that stops at the opening that follows it (the draft, 3.4).
- **Opening:** after `restarted`, `opened` must come within the opening
  bound, in minutes and in steps. `OPENING` is explicit and nonzero, its
  reason beside it: the minutes the start's own report bound gives where
  the profile gives one, and the steps within which every engine driver
  settles a start today, with headroom.
- **Findings** have skein's form (skein's `testing/inspection.md`, 6),
  component `engine`, object `task N` or `opening`: a task that neither
  ended nor was held past its bound; one held without the expected words;
  one held or ended otherwise than its expectation; an opening past its
  bound; a task still unmet when the run closes.

### 4.2 Where it runs

- **After every restart, in every engine driver** that restarts the root:
  the direct driver's restarts and its oracle's, the walking, escalation,
  roles and forge-route worlds', and the composed system world's
  `reopen_engine`, each with `OPENING`.
- **Goals' expectations** where a scenario names them: the forge-route
  world, which runs goals with changes, landings and a closer, and the
  composed system world. A scenario that names none is held only to its
  openings: a story that stops mid-way names no goal.

## 5. Open problems

### 5.1 The ignored scenarios

Each reproduces its row of the draft's section 7, named as that row
names it, in its world's focused suite, in a module `open` with a file
per problem, and is marked `#[ignore = "<work>: <problem>"]`, `<work>`
the row's *Fixed by* in its words. Each asserts the behaviour its row
wants, so it fails today; a symptom's shows the handling the design
wants of that symptom.

| Problem | Scenario | Package | Fixed by |
|---|---|---|---|
| P124 | `cancelled_landing_frees_its_write_entries` | `temper-engine-forge-world` | landing train |
| P124 | `a_change_waiting_for_room_has_a_bound` | `temper-engine-forge-world` | bounded waits |
| P88 | `a_refused_release_is_accounted` | `temper-engine-forge-world` | landing train |
| P90 | `a_refused_close_of_an_open_pull_holds_with_words` | `temper-engine-forge-world` | landing train |
| P67 residual | `a_no_change_landing_repair_regates_once` | `temper-engine-forge-world` | a change's reds |
| P37 | `a_starved_gate_is_the_runners` | `temper-engine-forge-world` | a change's reds |
| S7 | `a_landing_without_its_prepare_task_moves` | `temper-engine-domain-world` | landing train |
| P53 | `a_resolution_keeping_a_deleted_path_is_refused` | `temper-engine-domain-world` | a change's reds |
| P58 residual | `a_repair_whose_tests_pass_regates_by_itself` | `temper-engine-domain-world` | a change's reds |
| P120 | `the_newest_ruling_reaches_the_resolution` | `temper-engine-domain-world` | bounded waits |
| P85 | `a_held_answer_tells_its_requester` | `temper-engine-domain-world` | bounded waits |
| P113 | `a_change_waits_on_another_goal` | `temper-engine-domain-world` | bounded waits |
| P63 residual | `a_stored_charter_is_checked_at_start` | `temper-engine-domain-world` | bounded waits |

- **P124's bound:** no design gives a wait for room a bound
  (`../domain/forge/room.md`, 4). Its second scenario holds the change to
  its stall (`../domain/forge/change/stalls.md`, 1), which bounds every
  other wait on the forge; the bounded-waits work settles the figure and
  the scenario with it.
- **P90's refusal:** the fake forge refuses no close by branch policy
  (the draft's 4.1, *Refusing*, is the forge faults' work), so its
  scenario refuses the close of a change's pull request by permission,
  the pull request left open, which is the refusal the row's handling is
  about.
- **P37's report:** a gate red whose report names no failing test; where
  the fake forge and the gate's report reader can say tests went
  unobserved, it says so too.
- **A scenario that passes today** shows its row's handling already
  holds: it runs, not ignored, with no exclusion, and its row is
  reported for the catalog (the draft, 6, *A problem fixed before this
  draft lands*).

### 5.2 Their exclusions

The module `open` of `temper-engine-forge-world`, below every engine
world that sweeps, gives each scenario's exclusion (skein's
`testing/sweeps.md`, 5):

```rust
/// Adds each engine exclusion whose fault `configured` holds.
pub fn exclude(sweep: &mut skein_world::sweep::Sweep, configured: &[skein_world::faults::Kind]) -> Result<(), String>;
```

Every engine sweep calls it once it has configured its faults. A fault's
`Kind` has as its family the part of the seed's `Faults` or `Parties`
stream that draws it, in the draft's section 4's terms (`forge`, `llm`,
`worker`, `parties`, `state`, `runs`, `bounds`, `restarts`; `checks.md`,
6.2), and as its name the fault in that section's words. These are the
engine's names for the faults its exclusions leave out until their
families are built; the goal that builds a family renames any of them it
names otherwise, here, in its own change.

| Scenario | Fault | When |
|---|---|---|
| `cancelled_landing_frees_its_write_entries` | `parties` / `cancel a landing` | `a gate's dispatch in flight` |
| `a_change_waiting_for_room_has_a_bound` | `bounds` / `forge write entries full` | `a change waiting for room past its stall` |
| `a_refused_release_is_accounted` | `forge` / `a write refused` | `a task's release under way` |
| `a_refused_close_of_an_open_pull_holds_with_words` | `forge` / `a close refused` | `the pull request still open` |
| `a_no_change_landing_repair_regates_once` | `runs` / `a repair ends with no change` | `a landing's gate red` |
| `a_starved_gate_is_the_runners` | `forge` / `a starved gate` | `a landing's gate` |
| `a_landing_without_its_prepare_task_moves` | `parties` / `hold a member of a live landing` | `its landing's prepare job gone` |
| `a_resolution_keeping_a_deleted_path_is_refused` | `state` / `a path deleted on the base` | `a resolution under way` |
| `a_repair_whose_tests_pass_regates_by_itself` | `runs` / `a repair finds its tests passing` | `a landing's gate red` |
| `the_newest_ruling_reaches_the_resolution` | `parties` / `a ruling on a held change` | `an earlier ruling stands` |
| `a_held_answer_tells_its_requester` | `parties` / `answer a question` | `a dependency holds the answer` |
| `a_change_waits_on_another_goal` | `parties` / `a change waits on another goal` | `its requester parked` |
| `a_stored_charter_is_checked_at_start` | `restarts` / `the engine restarted cold` | `a stored charter past its room` |

- **No engine sweep draws these combinations yet** (2026-11-21): the
  sweeps that will are the workloads' (the draft, 5), and each takes them
  through `exclude`. A sweep of today's that draws one, and whose checks
  find it, becomes a `Sweep` configuring the fault it draws, takes
  `exclude`, and asks `leaves_out` before the draw.
- **An existing story whose check finds one of these problems** moves off
  its combination where what it tests holds without it; otherwise it is
  ignored as that problem's scenario is, its reason naming the same work
  and scenario.

## 6. Testing

Focused and fuzzy suites only, each check's scenarios in the world that
reaches them most cheaply (the draft, 11):

- **3.1 and 3.2 over the forge** (`temper-engine-forge-world`): an outbox
  whose entries outlived their owners, from a restored store, with a
  change in the outbox's line, is a finding of both checks, naming the
  entry and the room wait; the same with live owners passes; a landing
  through its round and its landing leaves every entry released and
  passes at every quiet point.
- **3.1 and 3.2 at the root** (`temper-engine-domain-world`): a landing
  waiting on its prepare job whose child has ended, with no deadline, is
  a finding naming the landing and the job; a live prepare job passes; a
  bounded wait whose bound passes with no quiet point is `step`'s
  finding, and one a quiet point sees end passes.
- **3.4** (`temper-engine-domain-world`, the forge-route world): a goal
  that ends within its bound passes, and the same goal held to a bound
  it can't meet is a finding naming it; a goal struck by a fault that
  never stops is held with words naming it and passes, and the same
  expecting other words is a finding; a goal in another repository than
  the fault ends, untouched, within its bound; a restart opens within
  `OPENING`, and `Ends`' own step tests find an opening past its bound,
  a zero bound refused, and a task still unmet at `close`.
- **The kit in every driver:** every engine world's existing stories and
  sweeps pass with the checks on, but for the moves and ignores of 5.2.
- **The open problems:** each scenario compiles, and is ignored with its
  reason; a step test configures every fault of 5.2 on one sweep and
  takes `exclude` with no refusal, and `leaves_out` answers each
  scenario only while its conditions hold. The boundary rule holds each
  exclusion to its ignored scenario (skein's `testing/sweeps.md`, 5).

## 7. Later

- **jig's inspection beside the engine's,** in every engine world, once
  jig's adapter is built (`jig/docs/design/domain/testing.md`, 7): the
  kit's `quiet` adds `jig_inspect::tasks` over `Domain::tasks()`, so a
  wait on a task resolves against that task's own wait.
- **The conformance and one-process worlds,** whose drivers are jig's
  harness and skein's `World`: a quiet point of their own (skein's
  `testing/inspection.md`, 8).
- **The forge's other bounded tables** (its change rows, releases,
  subscriptions, makings) and the root's own (loads, brief sections), and
  the root's line for the forge tables the adapter doesn't list.
