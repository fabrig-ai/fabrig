# Restarts, stops and older layouts as injected faults

Provisional, 2026-11-21. The restart drafts' faults as faults of temper's
worlds: each recorded in its run's fault ledger and drawn from the seed
at any iteration (skein's `testing/sweeps.md`), so that a sweep can mix
them with the rest of the failure worlds' faults. Written from
`../draft/failure-worlds.md`, 4.8 and section 9, and the restart drafts
(`../draft/reconciling-start.md`; `../draft/worker-restarts.md`). It
fixes nothing a restart breaks: that is the restart drafts' work (the
failure-worlds draft, 9 and 10).

## 1. In one page

- **One vocabulary** (section 2): every kind of this family is a
  constant of `temper-faults`, a kit of temper's `testing/`, so that
  every world and a sweep mixing worlds name the same kinds.
- **Drawn at any iteration** (section 3): each world has one restart
  injector, drawing from its own part of the seed's fault stream, which
  picks each fault and the iteration it falls at before the run, and
  injects it there whatever is in flight. Nothing drains first.
- **The engine's root** (section 4), in the engine's domain world: a
  crash at any iteration; a settled stop at any moment; a stop in each
  class of a start's phases; a second signal during a stop's settle.
- **The forge at a restart** (section 5), in the engine's forge world: a
  crash at any iteration; the forge down through a start, until a moment
  or for good; tables full at a restart, as two synthetic store fixtures
  and two scripts.
- **A worker mid-turn** (section 6), in the worker's domain world:
  drained over its link, signalled by its service manager or crashed, in
  an LLM call, a tool call, a delivery check or a save, and back as a
  new process on the same worker or another.
- **The engine restarted while a worker is mid-turn** (section 7), in the
  system world, announced or not.
- **The simulated tier** (section 8), in the one-process world: every
  process of a unit signalled mid-turn, and a start over the previous
  release's image.
- **One sweep per world** (section 9) shows that each kind it configures
  fell, under the world's checks as they stand.

## 2. The vocabulary

`temper/testing/temper-faults`, crate `temper-faults`, module `restarts`,
over `skein_world::faults::Kind` (skein's `testing/sweeps.md`, 3). It
holds constants and two small enums, and no world's code: each world
injects its own kinds.

```rust
pub const FAMILY: &str = "restarts";
/// The part of `Stream::Faults` every restart injector draws from.
pub const PART: &str = "restarts";

pub const ENGINE_CRASH: Kind = Kind { family: FAMILY, name: "engine crash" };
// ... one constant for each row of the table below

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Way { Drained, Signalled, Crashed }

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Point { LlmCall, ToolCall, DeliveryCheck, Save }

/// The kind of a worker stopped or crashed at a point of a turn: one of twelve.
pub const fn worker(way: Way, point: Point) -> Kind;

/// Each world's kinds, as its sweep configures them (section 9).
pub const ENGINE: [Kind; 5];     // section 4, without STOP_BEFORE_OPENING
pub const FORGE: [Kind; 7];      // section 5, ENGINE_CRASH first
pub const WORKER: [Kind; 12];    // section 6, by `Way` then `Point`
pub const MID_TURN: [Kind; 2];   // section 7
pub const SIMULATED: [Kind; 2];  // section 8
```

| Constant | Name | The draft's 4.8 | Marked | Where it falls |
|---|---|---|---|---|
| `ENGINE_CRASH` | `engine crash` | the engine restarted cold | composition | 4, 5 |
| `SETTLED_STOP` | `settled stop` | a settled stop at any moment | behaviour | 4 |
| `STOP_BEFORE_MARK` | `stop before the mark` | a stop before the start's mark | behaviour | 4 |
| `STOP_BEFORE_OPENING` | `stop before the opening` | after the mark, before the opening | behaviour | no world (4); the core's step test |
| `STOP_AFTER_OPENING` | `stop after the opening` | after the opening, while the forge catches up | behaviour | 4 |
| `SECOND_SIGNAL` | `second signal` | a second signal during a stop's settle | behaviour | 4 |
| `FORGE_DOWN_UNTIL` | `forge down until a moment` | the forge down through a start, until a named moment | script | 5 |
| `FORGE_DOWN_FOR_GOOD` | `forge down for good` | the same, for good | script | 5 |
| `ENTRIES_HELD_BY_ENDED` | `write entries held by ended owners` | write entries all held by ended owners | fixture | 5 |
| `BRANCH_THE_FORGE_LACKS` | `held branch the forge lacks` | a held goal branch the forge lacks, with no creation entry | fixture | 5 |
| `FIND_KEEPS_FAILING` | `uncertain write's find failing` | an uncertain write whose find keeps failing | script | 5 |
| `READ_ALWAYS_LIMITED` | `read limited at every attempt` | a read rate-limited at every attempt | script | 5 |
| `worker(way, point)` | `worker drained in an LLM call`, ... `worker crashed in a save` | a worker stopped or crashed mid-turn | behaviour, composition | 6 |
| `ENGINE_RESTART_MID_TURN` | `engine restarted mid-turn` | the engine restarted while workers are mid-turn, announced | composition | 7 |
| `ENGINE_CRASH_MID_TURN` | `engine crashed mid-turn` | the same, not announced | composition | 7 |
| `UNIT_SIGNALLED` | `unit signalled mid-turn` | every process of a unit signalled | behaviour | 8 |
| `OLDER_LAYOUT` | `older layout` | an older layout | fixture | 8 |

- **The worker's twelve names** are `worker <way> in <point>`, the way
  `drained`, `signalled` or `crashed` and the point `an LLM call`, `a
  tool call`, `a delivery check` or `a save`: `worker signalled in a
  delivery check`. `worker` matches both enums whole and returns a
  constant.
- **Names are fixed:** a sweep, an exclusion and a check compare kinds
  by value, so a name changes only with every reader of it.
- **Other families** join this crate as their goals move them, each in a
  module of its own; this design names only `restarts`.

## 3. Drawing them

- **One injector per world,** drawing from
  `Streams::new(seed).rng(Stream::Faults, restarts::PART)` alone (skein's
  `testing/sweeps.md`, 2; `checks.md`, 6.2). It takes no draw from the
  world's own generator and gives it none, so a run with the injector
  off draws the same workload, parties and scheduling. A world whose
  other draws keep their own seeding keeps it.
- **At any iteration:** before the run, the injector draws each fault it
  injects, its kind among those its sweep configures, and the iteration
  it falls at, an index of the world's own driver loop (its `tick`,
  `iterate`, `event` or `advance`, as the world names it) within the
  run's horizon. At that iteration the world injects it whatever is in
  flight: store answers outstanding, forge calls sent and not answered,
  a turn not yet said. A fault drawn past the run's last iteration falls
  at that last iteration. A fault that needs a restart (a stop in a
  start, the forge down through a start, a table full at a restart) falls
  at a restart the injector drew, and the injector draws such a restart
  where its sweep configures one.
- **Recorded as it falls:** `FaultLedger::inject(kind, from,
  Struck::default())`, `from` the world's clock at the injection. A
  restart fault strikes the engine or a worker whole, not an identity of
  `Struck`'s; a check finds it by kind and interval. Its `until` is
  each section's; a fault that never stops in the run keeps `None`.
- **Synthetic fixtures** (the failure-worlds draft, 4) are built by
  functions named `synthetic_...`, each documenting the state it stands
  for and that no legitimate sequence of events makes it. Each is laid
  into the store before the start that reads it and is recorded as an
  instant at that restart. A world's quiet-point checks leave out the
  fixture's own laid rows for the rule they break, by exact identity,
  and nothing else.
- **Exclusions** are skein's `testing/sweeps.md`, 5: an exclusion leaves
  out a fault only where its condition holds as the world reads it, at
  the moment the fault would fall, and is recorded with `leave_out`.

## 4. The engine's root

`temper/tests/engine/domain/`, its `forge_routes::World`: the engine's
root over the fake forge's domain, the world of the follows workload
(`tests/suite/fuzzy/follows.rs`).

- **A crash at any iteration** (`ENGINE_CRASH`): the world drops the root
  at the drawn iteration with whatever it has in flight. A commit
  submitted and not answered is kept whole or lost, as the injector
  draws, and its answer reaches no one. What is kept is a prefix in
  commit order: the store applies commits whole and in number order, so
  once one is lost every later one is lost with it. A forge call sent and not
  answered is made by the fake forge or not, as the injector draws, and
  its answer reaches no one. A new root then starts from the store, as
  `restart_same` starts one, with no `drain_cut` before it. Its interval
  runs from the crash to the next start's opening.
- **A settled stop at any moment** (`SETTLED_STOP`): at the drawn
  iteration, opened or still starting, the world gives the root its stop
  (`engine::Event::Stop`), runs it until it has stopped as
  `../domain/root-restart.md`, 1.2 has it, then starts a new root from the
  store. Its interval runs from the stop to the next opening.
- **A stop in each class of a start:** during a start that follows a
  crash or a settled stop, the injector may draw a stop in one of two
  classes, each its own kind, read from the root's
  `Domain::start_phase()` (jig's `start::Phase`):
  - `STOP_BEFORE_MARK`: `LoadCore` to `Wires`, the wires not yet
    committed;
  - `STOP_AFTER_OPENING`: once the root has opened, while its forge
    still catches up (`../draft/reconciling-start.md`, 3), at the first
    iteration after the opening.

  The world holds the start in the drawn class long enough for the stop
  to fall there (by holding the store's answers, `store::Fault::Hold`, or
  stepping the start one event at a time), gives the root its stop, runs
  it until the start has ended as `../domain/root-restart.md`, 1.2 has it, then
  starts a new root from the store. Its interval runs from the stop to
  the next opening.
- **No stop between the mark and the opening:** since the engine opens
  before its forge has caught up, the start adopts its runs
  (`AdoptRuns`) in the same root input that takes the mark's answer and
  opens, so nothing a deployment does stops a start there.
  `STOP_BEFORE_OPENING` keeps its name in the kit and is in no world's
  sweep; the core's step test holds the start at `AdoptRuns` by its own
  fixture and covers that stop
  (`temper/crates/temper-engine-domain/src/tests/start_stop.rs`). A
  world adds no hook to product code to reach it. A stop with the mark's
  commit stored and not yet answered has no pinned behaviour; it is the
  restart drafts' to design.
- **A second signal during a stop's settle** (`SECOND_SIGNAL`): after a
  settled stop's signal, and before the root has stopped, at an
  iteration the injector draws, the world aborts the root as the
  service's second signal does
  (`temper/crates/temper-engine-service-component/src/stop.rs`): a crash,
  as above, from the settle's state. It is an instant; the stop it cut
  keeps its own entry, stopped at the next opening.
- **A claim committed and not delivered** at a crash or a stop is not
  in the worker's hello, which lists only what reached it. A hello from
  the same process proves only what it lists, so the new root holds
  that claim until the stored restart wait ends, as jig's fleet does
  (`jig/tests/fleet/tests/suite/restart.rs`), and places it again
  after that. The restart sweep's workload moves the world's clock to
  the root's next alarm when the world is quiet and not yet ready. The
  follows workload's other tests and their bounds are unchanged.
- **The service's own handling** of a signal in each phase, of a second
  signal and of a signal before the store opens stays its step tests'
  (`temper/crates/temper-engine-service-component/src/tests/stop/start.rs`;
  `temper/crates/temper-engine-domain/src/tests/start_stop.rs`). The
  world models what they leave the root and the store with.

## 5. The forge at a restart

`temper/tests/engine/forge/`, its `World`: the forge connector under a
durable scripted root, over the fake forge. A restart there is the
connector's: `World::restart` builds it anew from the stored rows and
drops every answer of an older generation.

- **A crash at any iteration** (`ENGINE_CRASH`): `World::restart` at the
  drawn iteration, with calls in flight; a row submitted and not yet
  stored is kept or lost, as the injector draws. Its interval runs from
  the crash until the connector has settled its outbox again, `None`
  where it never does in the run.
- **The forge down through a start:** at a drawn restart, every
  connector call is refused (`World::unavailable`) from the restart
  until a moment the injector draws within the run (`FORGE_DOWN_UNTIL`,
  its `until` that moment), or to the run's end (`FORGE_DOWN_FOR_GOOD`,
  `until` `None`). Changes made on the forge by others still reach the
  fake.
- **Tables full at a restart,** each laid before the restart's start
  reads the store:
  - `ENTRIES_HELD_BY_ENDED` [fixture]: every place of the forge's write
    entries held by owners that have ended, landing rows cancelled
    holding their gate dispatches' entries, as
    `tests/suite/goal_branch_restart.rs` builds one (`cancelled_landing`,
    `held_dispatch`);
  - `BRANCH_THE_FORGE_LACKS` [fixture]: a held goal branch the fake forge
    doesn't have, with no creation entry (`../domain/forge/branches.md`,
    1);
  - `FIND_KEEPS_FAILING` [script]: a write the fake forge makes whose
    answer is lost before the restart, so that the start restores it
    uncertain, and every find of it refused from the restart on;
  - `READ_ALWAYS_LIMITED` [script]: a read the start asks, answered
    rate-limited at every attempt from the restart on.

  The scripts are the world's asks of its fake, made per call, and never
  stop in the run (`until` `None`).
- **The reconciling start's first milestone's scenarios** stay their own
  focused tests (`tests/suite/goal_branch_restart.rs`): its two fixtures
  here are drawn from the seed and mixed with the rest.

## 6. A worker mid-turn

`temper/tests/worker/domain/`, its `whole::World`: the worker's root,
which the world drives as the engine's link and as each agent at its
process boundary.

- **The points of a turn,** as the worker sees them, each a `Point`:
  - `LlmCall`: a run admitted or past a turn, its next turn not yet said:
    its agent waits on its model;
  - `ToolCall`: a call of the agent's that its host serves, asked and not
    yet answered;
  - `DeliveryCheck`: the run's delivery check running, its push not yet
    begun;
  - `Save`: a save's push sent and held.

  The world holds the run at the drawn point, playing the agent as it
  would be there, and injects the fault while it is held.
- **The ways,** each a `Way`:
  - `Drained`: the engine asks the worker to drain over its link, as the
    `worker drain` verb does (`../client/ctl/workers.md`;
    `../domain/worker/README.md`, 2);
  - `Signalled`: the worker alone signalled by its service manager, as
    the shipped unit's `KillMode=mixed` does: its first termination
    drains it (`../domain/worker/README.md`, 2);
  - `Crashed`: the worker gone with no settle, as a kill leaves it: the
    world drops the worker's root and every agent and command it ran.
    A new worker process comes back on the same worker (the same fake
    disk, a new root) or on another (a new disk), as the injector draws,
    and the engine the world plays sends the run's next attempt to it, as
    a suspended attempt is placed (jig's `domain/engine/executors.md`,
    6).
- **Its interval** runs from the ask, the signal or the crash to the
  worker's next hello, `None` where it isn't back in the run.
- **Every process of a unit signalled** is the simulated tier's
  (section 8), where processes are the world's.
- **What a drain or a crash keeps** of a run's work is the restart
  drafts' (`../draft/worker-restarts.md`, sections 3, 4 and 7), asserted
  by their milestones' focused tests, not by this world's sweep.

## 7. The engine restarted while a worker is mid-turn

`temper/tests/system/`, its composition of the engine's root, the
worker's root and smith's agent over the fake LLM
(`temper/tests/system/src/composed/absence.rs`).

- **The moment:** a run held mid-turn, its agent holding its Nth model
  query, N drawn (`absence_chat`), or a host call where the run's script
  makes one.
- **The engine then:**
  - `ENGINE_RESTART_MID_TURN`: stops settled, telling its workers its
    restart wait (`settle_engine`), then starts again (`reopen_engine`);
  - `ENGINE_CRASH_MID_TURN`: crashes, its store losing what wasn't
    durable (`stop_engine`), then starts again.
- **Its interval** runs from the stop or the crash to the next opening.

## 8. The simulated tier

`temper/tests/one-process/`: the engine's and one worker's services in
one process on skein's simulator, its agents and their programs hosted
as the world's programs (`tiers.md`, 1.5).

- **A unit signalled mid-turn** (`UNIT_SIGNALLED`): at a drawn moment of
  a run's turn, the world delivers the termination signal to the service
  and to every program the machine runs for it at once, as a
  control-group kill does (`../draft/worker-restarts.md`, 2). Each agent
  gets its own copy, which its live host answers
  (`smith/docs/design/domain/host.md`), and the run's check and shell
  programs end. The service drains and stops, and the world starts it
  again on the same disk. Its interval runs from the signal to the next
  opening. The real service manager stays outside the proof.
- **An older layout** (`OLDER_LAYOUT`): a run that starts from the
  previous release's image, `golden/dev/` (`../protocol/floor.md`, 5):
  the store and configuration dev.fabrig.ai's build wrote in this world,
  never a deployment's own data, loaded and configured as the floor's
  is. Its first start is the upgrade, and the sweep's other faults fall
  at drawn iterations of that start and after it. The injector draws,
  for each seed, whether its run starts from the image or from the
  world's own first start. Its interval runs from the run's start to
  that start's opening.
- **What checks an image run:** jig's conformance referee starts from
  an empty store at commit zero, so it watches only the runs that start
  fresh. A run from the image is checked by the golden reader's checks
  (census, history, preserved sources, forge) and the secrets' checks.
  These are the world's checks as they stand (section 9). Its waits are
  checked too, at the upgrade's first quiet point: every live task's
  waits, as jig's core computes them, read *since* that first start, its
  kinds' floors, and none is past its bound
  (`jig/docs/design/domain/engine/waits.md`, 7;
  `../draft/bounded-waits.md`, 12).
- **Why the older layout falls at a run's start:** a world runs one
  build, so the only store a previous release writes is one that release
  wrote before, kept as its image, and the upgrade is the first start
  over it. A restart within the run still falls at any iteration.

## 9. The sweeps

One per world, each a test of its world's fuzzy suite:

| World | Over | Configures |
|---|---|---|
| the engine's domain world | the follows workload | `ENGINE` |
| the engine's forge world | a workload drawn from its existing steps | `FORGE` |
| the worker's domain world | its delivery and contact stories | `WORKER` |
| the system world | its absence stories' runs | `MID_TURN` |
| the one-process world | its stories through a run mid-turn, and dev's image | `SIMULATED` |

- **The loop** is skein's `testing/sweeps.md`, 4: for each seed it
  writes the seed to stderr, runs the world with `Streams::new(seed)`,
  folds the run's ledger (`Sweep::ran`) and returns the run's findings
  at once; after the last seed it calls `check_fell`. Its seeds are the
  fewest at which every kind it configures falls, pinned in the test,
  within its suite's budget, which this design raises nowhere.
- **What it asserts:** each kind it configures fell; the world's
  referee, contracts and checks as they stand, with the shared
  harness invariants wherever the world calls them; and that the run
  ends. A run with a fault that never stops in it (the forge down for
  good, a find that keeps failing, a read limited at every attempt)
  expects no goal of its to end: the engine opening within a bound
  after any restart, and every goal going on, are the draft's 3.4 check,
  held where its goal builds it.
- **The existing sweeps** keep their own draws: a world's restart
  sweep is a new test beside them.
- **A finding** fails the sweep, as any red. One that a restart draft's
  milestone fixes (a start that never opens while the forge is away is
  the reconciling start's S3; a turn's work lost to a drain or a crash is
  worker restarts' W3 or W4) gets its scenario in its lowest harness,
  `#[ignore]` with a reason naming that milestone, and the sweep's
  exclusion naming the scenario, with the narrowest condition its world
  reads (the failure-worlds draft, 6). Any other finding is a defect of
  the injection, repaired, or a problem the goal's coordinator places.

## 10. Testing

Focused and fuzzy suites only.

- **The vocabulary,** in `temper-faults`' own tests: every kind's family
  is `restarts`; every name is unique; `worker` gives twelve distinct
  kinds over its pairs, each named as section 2 says; each group holds
  its section's kinds and no other, `ENGINE` without
  `STOP_BEFORE_OPENING`.
- **Each fault's own step test,** in its world's focused suite: the world
  injects that fault alone, at a moment the test names, and the test
  reads what the fault's section says it does (the root dropped with its
  answers in flight lost and a new root started from the store; the
  start stopped in the drawn class; every forge call refused until the
  moment; the synthetic store laid; the run held at its point, and the
  worker drained, signalled, or crashed and back), and the ledger's
  entry: its kind, its `from` and `until`, and `Struck::default()`. A
  table of cases over a world's kinds is one test.
- **Each sweep** of section 9, in its world's fuzzy suite.

## 11. Later

- The restart drafts' checks over these sweeps: a restart or a settled
  stop spending no try (3.3), the opening and every goal going on within
  a bound (3.4), and every commit leaving a store a start would open
  (3.7), as their goals build them. A restart changing no outcome it
  shouldn't (3.5), and the composed workload's sweep mixing the restarts
  with every other family, are `workloads.md`'s, sections 7 and 8.
- A stop in a start class the reconciling start adds, should it add one,
  or with the mark's commit stored and not yet answered, once the
  restart drafts pin what it does: a kind of its own in `ENGINE`, beside
  the two classes of section 4.
- jig's conformance referee started from a store's baseline, a compacted
  prefix such as dev's image, so that it also watches a run from the
  image (section 8).
