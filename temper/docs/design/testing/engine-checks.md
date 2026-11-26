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
- **Open problems** (section 5): a scenario for each of twelve rows,
  named as the draft's section 7 names them, each ignored with the work
  that fixes it until that work lands, and an exclusion for each that
  every engine sweep drawing its fault takes; a scenario whose handling
  holds already, or whose work has landed, runs active with none (P90,
  S7, P120; P63's residual and P124's bound). P124's first scenario is its fix's own
  active test, not one of them.
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
  `entry N`; an admitted read is `call N`, N its admission's number in
  the read admissions' table; a branch hold is its resource's name.

### 2.2 Waits

Each live landing is in the wait its row keeps (`landing-waits.md`,
2), kind `landing: WAIT`, WAIT the table's first column there, or
`landing: room` while it holds a place in the outbox's line. Each change
not ended is in the wait of its step, kind `change: STEP`. Each goal
branch's making that waits in the outbox's line is in the wait
`making: room` of its task (`../domain/forge/branches.md`, 1). Every wait
also names, as a `Timer`, the deadline the component has armed for it,
where one is armed: a landing's poll (`landing-waits.md`, 3) and a
change's due step.

| Kind | Wakes, besides its armed deadline |
|---|---|
| `landing: release` | `Party("person")`: a person's release |
| `landing: ready`, `landing: judgement` | `Waiter(task C)`, the member's change |
| `landing: turn` | `Waiter(task L)`, the landing holding the turn; none where the turn is free |
| `landing: prepare` | `Waiter(task J)`, the round's live prepare job, from `inspect`; the current round's outstanding write, `Party("forge: entry N")`, while the answered prepare opens its pull or retires its stale prefix; `Party("prepare job")` from `forge` alone while no terminal job result is recorded; `Ended("prepare job")` where neither a live prepare child nor its round's write remains |
| `landing: open`, `close`, `delete`, `post`, `merge`, `record` | `Party("forge: entry N")`, the landing's write in the outbox for it; `Ended("<wait> write")` where none is outstanding |
| `landing: prefixes`, `run`, `receipt`, `ancestry`, `read` | its deadline; and its read in flight, `Party("forge: call N")` |
| `landing: room`, `change: room`, `making: room` | for each entry holding the outbox's room: `Waiter(task T)`, its owner, while its owner lives; `Ended("entry N of task T")` once its owner has ended; for a making, `Party("forge: room")`, its ask while it stands unanswered |
| `change: held` | `Party("person")`: a person's release |
| `change: ready`, `change: preparing` | `Waiter(task L)`, its landing |
| `change: ready` with no landing | its live pushing children from the root, `Waiter(task D)`, while their final heads keep its train from forming; while the forge's `landings` refuse its train (`../domain/landing-forming.md`, 3), for each landing holding them, `Waiter(task L)` while its task lives, else its entry still settling, `Party("forge: entry N")`; while a live landing holds its train, `Waiter(task L)`, that landing, even with room for other trains; while the operator's landing hold stands (`../domain/landing-settings.md`, 1.3), `Party("operator restart")`, the start without that hold; while jig's refusal of its train awaits its stored retry (`../domain/landing-forming.md`, 3), `Timer(retry)`; while changes into its train settle in a cohort (`../domain/landing-turn.md`, 2), `Waiter(task M)`, each settling change; otherwise its landing's making, as the root's turn below |
| `change: first` whose merge the operator's landing hold withholds | `Party("operator restart")`, the start without the hold (`../domain/landing-settings.md`, 1.3); the engine keeps the last step's withheld-merge observation for inspection alone, never stored or read by policy |
| `change: preparing` with its own write | `Party("forge: entry N")`, its outstanding rebase write, including before it belongs to a landing |
| a change or landing parked for restart freshness | each still-owed resource read in its awaited repositories, `Party("forge: restart read F/R/RESOURCE")` (below) |
| every other `change:` step | its live delegate (producer, repair, resolution, gate or merge job), `Waiter(task D)`; its write in flight, `Party("forge: entry N")`; its write still settling, `Party("forge: entry N")` (below); its read in flight, `Party("forge: call N")`; its open question's answerer, `Waiter(task R)` or `Party("person")`; each dependency's end, `Waiter(task D)`; its follow owner's word or act, `Waiter(task T)` or `Party("person")`, including an unstarted change held behind an ended entry awaiting the owner's start or skip (below); else the root's turn below |

- **The root's turn:** a change the forge has decided an act for that
  the root carries out (a delegate to make, a landing task to make,
  a decision's effect to run), with no other wake, waits on the root.
  `forge` gives `Party("root")` for it while that act stands unanswered
  in the forge's state, read from the state itself (the step's
  procedure state awaiting the root's answer), never as a fallback for
  a wait it can't name: the forge world's scripted root answers in its
  own time. `inspect` gives none: the root carries out each act in the
  step that decides it, so a change still awaiting one at the root's
  quiet point has no wake, and is a finding. A parked producing procedure
  with no delegate, dependency or other named wake is a finding too:
  registering its forge row alone does not wake the root's procedure.
- **What only the scripted root knows:** a decision or a wake the forge
  has emitted and forgotten (a change's or a landing's `Wait` decision
  whose timer only the root arms, a fired deadline handed over as
  `WakeProcedure`) leaves nothing in the forge's state, so `forge`
  can't read it. The forge world, which is that root, keeps the tasks
  it owes an act: each task of a `ChangeDecision`, `LandingDecision`
  or `WakeProcedure` it has taken from the forge's output, until it
  passes the forge that task's `StepChange` or `StepLanding`. A row the world registers (a change's, a landing's,
  or one restored) owes its first step the same way: the root steps a
  row in the turn that makes or restores it, and the script steps it
  in its own time, so the task is owed from its registration until
  the world first passes its `StepChange` or `StepLanding`. At its
  quiet point it adds `Party("root")` to the wait of each such task,
  in the inspection `forge` gave, and to nothing else: a task the
  world owes nothing has no fallback wake. The adapter's surface stays
  2.1's; the forge's state gains nothing for it.
- **A landing's read-admission retry:** while its gather waits in the read line, its retained retry is `Timer(at)`; at the root this is read only while the landing's procedure step is due.
- **A read in flight:** a read the forge's client has admitted and
  not yet had answered, its holder's owner the token the forge maps to
  the waiting change or landing, wakes it when its answer comes:
  `Party("forge: call N")`, read from the client's admissions, like a
  write's `Party("forge: entry N")` from the outbox. A call answered or
  closed names nothing, and no read stands in for a wait it can't
  name. A read still waiting for admission is not in flight. This also
  applies while a room waiter's place remains on its row: once asked,
  it reads afresh before its decision keeps an entry or leaves the line
  (`../domain/forge/room.md`, 3).
- **An offered room reservation:** another task's reservation names that
  task as a waiter. The reservation's own task, once its procedure is due
  in the root, names `Party("root")`: the line has already offered that
  step, even before its fresh read is in flight. A reservation without a
  due step or a live read gives its own task no fallback wake.
- **A task waiting for restart freshness:** the forge's parked step or explicit
  freshness wait names the repositories it awaits (`../domain/forge/catching-up.md`,
  2 and 4). Each resource the client still owes in those repositories names
  `Party("forge: restart read F/R/RESOURCE")`, including its admission and retry
  before it is in flight. These are the client's resource reads, not a change's
  read token. A task not parked for freshness borrows none of them; a resource
  already read afresh names nothing.
- **A write still settling:** a change's procedure can move on from a
  write it no longer needs before the forge's client has answered it:
  containment takes a landed merge while the client still withdraws its
  uncertain attempt. The change's release still waits for that entry's
  answer before its effects settle, so each outbox entry of the change,
  not held, while its release's effects are unsettled, wakes it:
  `Party("forge: entry N")`, read from the forge's releases and outbox,
  once beside its step's own write. A release whose effects have settled
  names none, and no entry stands in for a wait it can't name.
- **A task a person holds:** jig can hold a change's or a landing's
  task while the forge's step stands as it was: a person's stop, a
  placed hold, or a hold on a failure, a stall, a budget or a deadline
  that a person releases. Its wait then also names `Party("person")`,
  once, from `inspect` alone, which reads the root's tasks. A hold no
  person releases names none: a dependency's, a wait for holds, a
  resource's (a forge limit's among them), a resource's access, a lost
  pool's and a circle's each need their own wake, and a person is not
  their fallback. A wait for room gains a person's wake only where its
  task is held so: the line passes over a held waiter, and the release
  steps it again (`../domain/forge/room.md`, 3).
- **Admission before a delegate:** `inspect` reads the root's retained forge
  limit account only while its exact task hold still stands, naming
  `Party("operator limits: MEMBER")` for a raise and `Party("operator restart")`
  for the startup retry (`../protocol/configuration/limit-settings.md`, 6).
  A change with jig's live-task `room_wait` names `Waiter(task T)` for each
  other live task counted by that room (the deployment, or its project),
  whose ending frees a slot; no recorded room wait or current account means
  no such wake.
- **A change that waits its turn:** each dependency a change's task
  waits on names `Waiter(task D)`, from `inspect`'s root tasks and from
  `forge`'s pending step dependencies and follow entry's `WaitsFor`.
  A follow entry in `WaitsWord` or `HeldByOwner` names its owner's word
  or act: `Party("person")` for a person, `Waiter(task T)` for a standing
  role, from the forge's follow entry and setting in both adapters.
  An unstarted follow change held behind an ended entry names the same
  owner, whose start or skip of that entry remakes the held tail: `inspect`
  reads the root's dependency-hold chain and the forge's still-queued ended
  entry and owner. An entry already started, skipped or gone names no such
  wake. A task known to have ended names `Ended` instead. A dependency's
  end or an owner's word or act is their only wake, never a fallback for
  another wait.

- **A conflict waiting for its producer:** `inspect` names every live pushing
  child of the change as `Waiter(task D)`, even after a gate displaces the
  current delegate; its `WaitingProducer` row has only the gather's armed
  read-session bound, absent while exempt, not a stall from the old conflict.
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
  counts from and no exemption stands; none otherwise. A follow change at
  gating whose check workflow has ended counts its stall from that end,
  as a delegate's end does. A turn change whose gates have settled and whose
  task still waits on a dependency has no gate-stall bound; its readiness poll
  does not make that dependency a forge wait. A queue's first whose last step
  decided a merge the operator's landing hold withheld has no stall bound:
  it waits for restart (`../domain/landing-settings.md`, 1.3). The engine keeps
  that volatile observation for inspection alone, clearing it when the next
  step decides otherwise or the row ends. Every other queue's first retains
  its stall bound, including under the hold while forge facts stay unknown.
  A landing's
  deadlines are polls, not bounds; its wait on the forge has the bound
  the stall gives from its last advance (`../domain/landing-waits.md`,
  3), and a wait for room behind holders whose owners have all ended the
  room figure from the last end (`../domain/forge/waits.md`, 7); a room
  wait behind a live writer has none. A wait past its bound at a quiet
  point is a finding (skein's `testing/inspection.md`, 4.1): the bound
  should have held it.
- **What it finds:** a landing or a change whose every wake has ended or
  waits on one another in a cycle, with no deadline armed; a room wait
  behind an outbox whose every entry outlived its owner, past its bound
  and not held (the draft's P124).
- **A landing that can't move** (`../domain/landing-liveness.md`, 2.1),
  at each quiet point, is a finding too: a live landing, not held, whose
  task is not live, a member of which, neither left nor landed, has a
  task closing, ended, gone or held, or is another landing's by the
  index, or whose train another live landing holds. `inspect` reads
  jig's tasks for the tasks and holds; `forge` reads the forge's change
  rows, with the script's own landing tasks as live. A landing due to
  step, its wake given and its step not yet taken or its step's reads in
  flight, is passed over until that step.

### 2.3 Tables

| Table | Capacity | Entries | Released by | Due once |
|---|---|---|---|---|
| `forge write entries` | `forge.entries` | each outbox entry, `entry N (task T)`, and each reservation of its room, `reserved for task T` | the write's settlement, or its owner's end with the entry withdrawn | its owner has ended, the entry has been handed to the forge client, and that client no longer holds it (no write or withdrawal outstanding) |
| `forge read admissions` | `forge.client.pending` | each call the forge client admitted, `call N`, with its task where its reader names one | the read's answer | its answer has come and its place isn't reclaimed at the quiet point |
| `branch holds` | `forge.holds` | each hold, `NAME (task T)` | the closing settlement of the landing or change that holds it; on an ended forge, the forge's return or a person's act | its forge is live, its holder's task has ended and no release of it is under way |
| `landing rows` | `forge.landings` | each landing's row with its asks, `landing L` | the closing settlement of the landing | the landing has ended, landed or cancelled, and its row still names a write entry no longer in the outbox or holds an ask not handed over |

- **An owner's end, as each function reads it:** `inspect` reads it
  from jig's tasks. `forge` reads what the forge keeps: for write
  entries and landing rows, a landing's end is its phase, landed or
  cancelled; a task outlives its landing's phase, closing its holds
  after it finishes, so for branch holds `forge` knows a holder's end
  only where the forge's own release state shows that holder's ending
  settled while the hold still names it. Elsewhere a hold is pending
  in `forge`: the forge world can't tell a living holder from one
  whose root has not yet asked its ending, and the kit's worlds,
  through `inspect`, find a leaked hold.
- **An ended forge's hold** remains pending, including a taskless hold:
  no closing settlement can come while its forge is ended. Its release
  rule is the forge's return or a person's act, not the absent holder's
  end (`../domain/landing-forming.md`, 4).
- **Before handoff, a write has not settled:** an entry waiting to commit,
  restored before catch-up, or committed and awaiting its place in the client's
  handoff order is pending even after its owner ends. Absence from the client's
  outbox alone is not settlement (`../domain/forge/room.md`, 2).
- **An ended landing's outstanding writes** keep its row pending until their
  outcomes settle (`../domain/forge/room.md`, 2). Ending does not retire a
  write still in flight; a row's reference left after its outbox entry
  retires is a finding.
- **Kept:** an ended landing's row naming nothing, kept until a new
  landing needs its place, as the forge retires ended rows. A hold its
  holder's failed ending hands to the root is the root's, and pending
  while the root lives.
- **Refused newcomers,** each `task N`, with no bound, since none of
  these waits has one in its design:
  - `forge write entries`: each waiter in the outbox's line
    (`../domain/forge/room.md`, 3), a change, a landing, a making or a
    release, and each task the root holds for the table's room;
    a waiter whose task is held, other than for the table's room, is
    passed over, not refused (`../domain/forge/room.md`, 3), and waits
    for its release (2.2);
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
    /// The earliest bound of the waits kept at the last quiet point.
    pub fn next_bound(&self) -> Option<skein_lib::Time>;
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
- **jig's record check** (`jig/docs/design/domain/testing-fakes.md`,
  1.1): `quiet` runs `jig_inspect::records` over the root's core after
  skein's two checks, with the same moment, so every live task's waits,
  a change's and a landing's tasks among them, are jig's records at
  each quiet point. Their bounds arm no deadline here (that section's
  third rule). A change's, a landing's and a closer's are the records
  the forge reports (`../domain/forge/waits.md`, 2); the landing and
  change waits of 2.2 stay named for the worlds as well, since they judge
  wakes and bounds the records only carry. A wait report the core refuses
  is a finding, naming the task and the refusal. `forge`'s world holds no
  core, and runs it not.

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
- **The composed system world** (`tiers.md`, 1.1): at the end of each
  tick whose immediate work is drained.
- **A bound is a moment every driver visits:** the referee's deadline is
  one of the driver's timers, as strict as the bound. A driver moves time
  past no kept bound (`next_bound`): a tick or a jump that would cross one
  stops at it, delivers the timers due there, and drains its immediate
  work at that moment to a quiet point before time moves on, so the
  stall's own timer, due at the bound, is answered there and a wait gone
  by then is no finding. A drain that doesn't come quiet within the
  driver's own step limit is the finding. A story moves the clock only
  through its driver, never by setting it. No tolerance is added: one
  would hide a stall timer armed late.
- **Each run's finding:** a driver keeps the first finding and its loop
  panics with it, as a run's finding (`scenarios.md`, 1.2); a returning form
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
- **Waiting,** for `Waits`, is the task's live row waiting on its
  dependencies, or held; or the task live, neither held nor ended, with a
  task beneath it that is held or holds a question its party has not
  answered, read from the store's rows. A goal whose producer's question
  is never answered waits so while its coordinator parks: the scenario
  makes it wait on nothing it would not otherwise wait on. A live task
  with no such descendant is not waiting.
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
  A store-check rehearsal whose start is refused and a fixture declared
  broken carry no opening expectation.
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
| P90 | `a_refused_close_of_an_open_pull_holds_with_words` | `temper-engine-forge-world` | holds already: active |
| P67 residual | `a_no_change_landing_repair_regates_once` | `temper-engine-forge-world` | a change's reds |
| P37 | `a_starved_gate_is_the_runners` | `temper-engine-forge-world` | a change's reds |
| S7 | `a_landing_without_its_prepare_task_moves` | `temper-engine-domain-world` | holds already: active |
| P53 | `a_resolution_keeping_a_deleted_path_is_refused` | `temper-engine-domain-world` | a change's reds |
| P58 residual | `a_repair_whose_tests_pass_regates_by_itself` | `temper-engine-domain-world` | a change's reds |
| P120 | `the_newest_ruling_reaches_the_resolution` | `temper-engine-domain-world` | holds already: active |

- **P124's first scenario,** `cancelled_landing_frees_its_write_entries`
  (a landing cancelled while its gate's dispatch is in flight withdraws
  its write entries), is the active test of its fix, the goal *A
  cancelled landing gives back its forge write slots*: this design writes
  no ignored copy of it and no exclusion for it (the milestone,
  2026-11-21).
- **P124's bound** (`../domain/forge/waits.md`, 7): its second
  scenario, `a_change_waiting_for_room_has_a_bound`, ends the writer of
  the outbox's sole entry while that entry's write stays uncertain, and
  holds the change waiting for room at the room figure from that end,
  with words naming the room, the entry and its ended writer. While the
  writer lives, the waiter is behind it and the writer is the one held,
  at its stall, keeping its write in flight; the scenario checks that
  too. It runs active, with no exclusion (*A scenario its work has
  fixed*, below).
- **P90's refusal:** the fake forge refuses no close by branch policy
  (the draft's 4.1, *Refusing*, is the forge faults' work), so its
  scenario refuses the close of a change's pull request by permission,
  the pull request left open, which is the refusal the row's handling is
  about. Its handling holds already: the cleanup holds with an account
  naming the close, the pull request and the permission, so the scenario
  runs, not ignored, and takes no exclusion (the last bullet; the goal's
  coordinator, 2026-11-22).
- **S7's and P120's handling holds already:** a landing whose prepare
  child is cancelled, with a member held by a person, ends or holds with
  words, with no person's release of the landing: the member's hold
  wakes it, and the member leaves (`../domain/landing-liveness.md`, 4);
  and a held change amended twice, its first
  resolver failed, is resolved from its newest spec, the resolution's
  brief built from the live task's current spec and its rulings. Both
  scenarios run, not ignored, and take no exclusion (the last bullet; the
  goal's coordinator, 2026-11-22).
- **P37's report:** a gate red whose report names no failing test; where
  the fake forge and the gate's report reader can say tests went
  unobserved, it says so too.
- **A scenario that passes today** shows its row's handling already
  holds: it runs, not ignored, with no exclusion, and its row is
  reported for the catalog (the draft, 6, *A problem fixed before this
  draft lands*).
- **A scenario its work has fixed** runs the same way, active with no
  exclusion, in the file its row named: P63's residual,
  `a_stored_charter_is_checked_at_start`, by the bounded waits' start
  check (`../domain/agent/room.md`, 4.1), whose words it pins; P124's
  bound, `a_change_waiting_for_room_has_a_bound`, by a room wait's bound
  (`../domain/forge/waits.md`, 7); and P88's,
  `a_refused_release_is_accounted` in `temper-engine-forge-world`, by a
  round cleanup's hold (`../domain/landing-prepare.md`, 1.1), whose
  account it pins: the deletion, the branch and the forge's refusal.

### 5.2 Their exclusions

The module `open` of `temper-engine-forge-world`, below every engine
world that sweeps, gives each scenario's exclusion (skein's
`testing/sweeps.md`, 5):

```rust
/// Adds each engine exclusion whose fault `configured` holds.
pub fn exclude(sweep: &mut skein_world::sweep::Sweep, configured: &[skein_world::faults::Kind]) -> Result<(), String>;
```

Every engine sweep calls it once it has configured its faults. A fault's
`Kind` has as its family the draft's section 4 family it falls in, and
as its name the fault in that section's words; where a family's goal has
built it, the kind is that family's constant
(`workloads.md`, 2.5), and the rest are `open`'s own until their
families are built, the goal that builds one renaming them here in its
own change.

| Scenario | Fault | When |
|---|---|---|
| `a_prepare_is_admitted_when_branch_holds_are_released` | `temper_faults::bounds::BRANCH_HOLDS` | `the holds and pending prepares, including the new change or train, exceed capacity` |
| `a_no_change_landing_repair_regates_once` | `runs` / `a repair ends with no change` | `a landing's gate red` |
| `a_starved_gate_is_the_runners` | `temper_blame::STARVED` | `a landing's gate` |
| `a_resolution_keeping_a_deleted_path_is_refused` | `stale` / `a path deleted on the base` | `a resolution under way` |
| `a_repair_whose_tests_pass_regates_by_itself` | `runs` / `a repair finds its tests passing` | `a landing's gate red` |
| `a_goal_cancelled_while_its_branch_is_being_made_ends_cancelled` | `temper_faults::people::CANCELS_A_GOAL` | `a goal branch making in flight` |

- **The sweeps that draw these combinations** are the workloads'
  (`workloads.md`, 8), and each takes them through `exclude`. A sweep
  that draws one, and whose checks find it, configures the fault it
  draws, takes `exclude`, and asks `leaves_out` before the draw.
- **The branch-holds exclusion** is read before the workload starts a
  change or its closer. It adds the current holds, each live change's
  still-unclaimed branch and prepare prefix, a started closer's pending
  prefix, and the new load's demand. The first member admits a train as
  one load: all its members' branches and prefixes are reserved until
  each starts, so a partial train cannot consume its own remaining room.
  A total equal to capacity is admitted. A left-out load records one
  `leave_out`, and may start once the demand fits.
- **P124's first combination** (`temper_faults::people::CANCELS_A_LANDING`,
  while a gate's dispatch is in flight) has no exclusion: its fix landed
  before the workloads' sweeps, and its scenario is active (5.1).
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
  jig's adapter is built (`jig/docs/design/domain/testing-fakes.md`, 1): the
  kit's `quiet` adds `jig_inspect::tasks` over `Domain::tasks()`, so a
  wait on a task resolves against that task's own wait.
- **The conformance and one-process worlds,** whose drivers are jig's
  harness and skein's `World`: a quiet point of their own (skein's
  `testing/inspection.md`, 8).
- **The forge's other bounded tables** (its change rows, releases,
  subscriptions, makings) and the root's own (loads, brief sections), and
  the root's line for the forge tables the adapter doesn't list.
