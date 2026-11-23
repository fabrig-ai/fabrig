# The store check: every commit leaves a store a start would open

Provisional, 2026-11-23. How temper's worlds hold
`../draft/failure-worlds.md`, 3.7: after a commit, a store restored from
that committed snapshot, apart from the live world, passes everything a
start checks, jig's checks and temper's connector checks above them,
and a record a start would refuse fails the world, naming the record
and the commit. jig gives the operation, its choice of commits and its
finding (`jig/docs/design/domain/testing.md`, 7, the store check); this
document is temper's: its root's rehearsal as the start that checks,
what one costs, and where each world calls it.

## 1. In one page

- **The check is a rehearsal** (section 2): temper's root restores as at
  a start, takes the switch inside its copy and ends at the opening
  (`../domain/root-restart.md`, 1.1), over the world's store copied at a
  commit, so that the forge's checks run on what jig's restores, and
  nothing the live world holds is touched. It pays for the store it
  reads and the start it checks, never again for what the run's
  configuration fixes.
- **Every commit** in the conformance world's commit-cut cases, where
  each child's restart is the check of its commit; **one per run**
  everywhere else: each ordinary run of the engine's domain, forge and
  system worlds, the one-process world and jig's kits keeps one of its
  commits, drawn from its seed, and rehearses it as the run ends; each
  run reports how many of its commits it checked, and which (section 3).
- **A finding** is jig's: the commit, the phase, the refusal naming its
  record (section 4).

## 2. The rehearsal

- **temper's start** is its root's (`../domain/root-restart.md`, 1.1): jig's core
  records, the forge's records, their links, the sweep, then the switch
  inside the copy, the start's wires and the runs adopted; a refused
  start stops with `temper_engine_domain::StartStop`, and a rehearsing
  root (`Domain::rehearse`, before the start begins) ends at the
  opening, `Domain::rehearsed()` then true. A rehearsing root runs with
  no network to the opening, and no step before the switch leaves the
  process (`jig/docs/design/domain/engine/restart.md`), so a rehearsal
  needs no peer: its forge fake, scripted workers and parties are never
  reached.
- **What it checks:**
  - jig's: every record decodes within its bounds, the live sets fit the
    limits, the settings pass their check whole, and every link is made
    (`StartStop::Core`, carrying jig's `start::Failure`);
  - the root's: a page that reads (`Load`), a charter leaving the brief
    room (`Room`);
  - the forge's, temper's connector checks: the deployment's binding
    (`Binding`), its stored settings (`Setting`), each record within its
    room (`ForgeFit`), a change with both its parts (`ForgeRow`), and
    each record's forge, repository, task and entry held (`ForgeLink`).
- **Over a copy:** the world's store forked at the commit (the domain
  world's `Store::fork`: the durable image, cold-opened in memory maps of
  its own), a root built from the world's configuration, the start's
  commits landing in the copy alone. A kept commit's copy (section 3) is
  the same image, taken when the commit is kept and opened once, at its
  check.
- **One function for the engine's worlds:**
  `temper_engine_domain_world::check::store(store: &Store, config: &Config, limits: &Limits, wall: Wall, commit: u64) -> Result<(), String>`,
  `Config` and `Limits` what `Driver::from_store_at` takes, `wall` the
  world's wall time at the commit: a rehearsing root over the fork,
  driven until `rehearsed()` or a `start_stop()`, its `Err` jig's finding
  (section 4). The system world, whose store is the domain world's, calls
  it too, and a world's kept check rehearses its kept copy the same way,
  without forking it again.
- **What it costs:** a rehearsal pays for the store it reads and the
  start it checks, not for building again what the world's
  configuration and limits fix:
  - the rehearsing side builds no more than the start needs: the
    configuration copied once; none of a live world's bookkeeping (a
    driver's transaction, load, switch and host-request logs, its
    delivered outputs, a chat-creation table, a second restart
    configuration); no typed row mirror in the copy, which only a live
    world's readback reads; the copy's routes and codec bounds taken from
    its source's where the limits are the same; and a loop that drives
    the root and the copy alone, as jig's core kit's does;
  - fixed work that every start repeats over the same configuration, in
    the root's construction and its restore (the same inputs validated
    or derived more than once, positions probed that no entry holds,
    bounds derived again for limits already checked), is made cheaper in
    the product for every start, the start's behaviour unchanged; each
    check over each set it checks stays;
  - nothing a start checks is skipped to save cost: every phase and
    check above runs, to the same endpoint, over the same records.
- **In the conformance world** temper's `Application` gives jig's harness
  the root's rehearsal: `rehearse` is `Domain::rehearse`, `rehearsed` is
  `Domain::rehearsed`, `refused` is `start_stop()` in its `Debug` form,
  and `start_phase` is the root's `start_phase()`. What its peers need
  from the configuration alone (the first start's receipt evidence) is
  built once per harness, not once per check.
- **The forge world** has no root and no core, and keeps its records in
  a map of its own. Its check is the forge's half: its stored records
  restored into a forge domain of its own, then the forge's checks at a
  start's link, one function of the forge's that the root's link calls
  too, so that the two never differ:
  `temper_engine_domain_forge::Domain::start_links`, taking what the
  root's link hands the forge today (its limits, and the tasks
  `missing_link` reads) and answering `Result<(), StartLink>`,
  `StartLink` one of `Row(Key)`, `Fit(RestoreFit)` and
  `Link(MissingLink)`, the vocabulary's own records. The forge world
  holds no tasks of its own: it keeps a scripted tasks domain, built
  from `jig-tasks-kit`'s fixtures, holding the tasks its scripted
  inputs hand it (never inferred from its stored rows), and passes
  those; a kept check passes the tasks its scripted inputs had handed
  it by the kept commit. The root maps it to
  `StartStop::ForgeRow`, `ForgeFit` and `ForgeLink` as it does now, in
  the order it checks them now.
- **The one-process world** rehearses as `temper-box upgrade --check`
  does (`../protocol/binaries.md`; `../domain/root-restart.md`, 1.1): the real
  service, in the upgrade tool's rehearsal, over a copy of the fake
  machine's disk taken once the kept commit is observed, as its upgrade
  stories already run it over the floor's disk; a report that refuses is
  the finding, naming the newest commit the copy holds.

## 3. Which commits, in which world

jig's `StoreChecks` choose them (`jig/docs/design/domain/testing.md`,
7): every commit, one per run drawn from the run's seed, or each commit
with one chance in `one_in`, each run's count reported.

| World | Commits | Where it counts a commit | Where its run ends |
|---|---|---|---|
| `temper-conformance-world` | every commit of its commit-cut cases, by each child's restart; one per run elsewhere | jig's harness: `Harness::committed`, and a cut's restart through `refused` | the harness's end of a story, called by its story and scenario runners |
| `temper-engine-domain-world` | one per run | once its driver applies a commit (`Driver::advance`, `Driver::apply_commit`) | the driver's close (`Driver::close_erasing`, from its `Drop`) |
| `temper-system-world` | one per run | once its composed or forge world's store applies a commit | the world's `Drop` |
| `temper-engine-forge-world` | one per run | once the world has applied one step's writes: a `take_top` output's saves and erases together, its projections' included, as the root commits them in one commit (a change's cold and hot parts are two saves of one commit, never two commits) | the world's `Drop` |
| `temper-one-process-world` | one per run | once its referee observes a commit | its phase's end (`store_check::scope`) |

The client world's engine is the engine domain world's driver, and
checks as it does.

- **The sample is one per run** (`StoreChecks::one`): each ordinary run
  that commits checks exactly one of its commits, every commit it counts
  with the same chance, drawn from the run's seed, and reports which.
  The world keeps the store of the commit the sample holds, replacing
  it as the sample moves on, and rehearses it once, as the run ends.
  Why: a sample of one in `n` from a fixed seed checks only the runs
  long enough to draw a commit. At one in 1024 almost no ordinary story
  checks one; with `n` near the stories' median commit count, about one
  per run checks one, none in many short runs and several in long ones.
  One per run gives every story its check, the same seed the same
  commit, and a cost of one rehearsal a run, which section 2's cost keeps
  a small part of a run, within the gate's growth bound
  (`tools/tiers/budgets.md`, 8). The draw is over the run's counted
  commits, not their numbers, so its first commit is as likely as its
  last; an `n`-commit run replaces its kept copy about `ln n` times, each
  a copy of a small image.
- **A run** is a world's story from its construction, or its last
  `store_checks` setter, to its end (the table's last column). A setter
  starts the count afresh and drops what was kept. A world whose store
  moves on to another (the engine domain driver's `take_store` and
  `into_store`, the first start behind `Driver::configured`) ends no
  run: it keeps nothing and checks nothing; the world that takes the
  store begins its own. A rehearsing world checks nothing of its own
  (`StoreChecks::none`).
- **The kept check runs as the run ends,** after its last counted
  commit, the closing oracle's included where the world has one, and
  before its report. Its finding fails the run there: through the
  world's fallible end where it has one (the core kit's `finish`, the
  driver's `close_erasing`, the harness's end of a story, the one-process
  phase's result), and from its `Drop` otherwise, unless the thread is
  already panicking. Under `every()` and `sampled` a commit is still
  checked as it is counted.
- **A commit-cut child** keeps nothing: its cold restart is its cut's
  check, counted `restarted` by its parent, and its own later commits
  keep none. A parent keeps one of the commits it counts with `due`, the
  cuts not among them.
- **`sampled`** stays for a story that names it; no world uses it by
  default. Every commit everywhere remains a separate, priced choice
  (section 6).
- **Its calls are one line at a world's commit and one at its end.** The
  engine's worlds' shared kit is the engine checks' goal's
  (`../draft/failure-worlds.execution.md`, 2): where that kit holds a
  world's driver, the calls move into it, and nothing else of the kit is
  this check's.
- **A story about a refused start** whose own input is a record a start
  refuses reads the check's `Err` as its outcome, or sets
  `StoreChecks::none` with a reason naming the refusal it tests.

## 4. The finding and the report

- **The finding** is jig's form (`jig_conformance::store_check::finding`):
  `store check: commit <n>: start refused at <phase>: <refusal>`, the
  refusal naming its record's key wherever it names one. The world
  returns it as its run's finding, as its referee's: at that commit
  under `every()`, as the run ends for a kept commit.
- **A kept commit's finding** names the commit it kept, at or after the
  one that made the record, and fails the run as it ends; the same seed
  under `StoreChecks::every()` names the commit that made it, failing at
  that commit.
- **The report:** each run writes `StoreChecks::report` to stderr as it
  ends, after its kept check, the commits checked of the run's commits,
  and under the default the commit it checked:
  `store check: 1 of 22 commits checked (one per run, seed 45, commit 17)`,
  so that no run claims more than it checked; an exhaustive claim is
  only a commit-cut sweep's (`jig/docs/design/domain/testing.md`, 5).

## 5. Testing

Focused, in each world the check runs in; each is a scenario where the
check finds what it should, and one where a correct store passes it (the
draft, 11):

- **The engine's domain world:** a story that commits, as a writer's
  defect would, a forge record a start refuses (a change with one of its
  two parts), under `StoreChecks::every()`, fails at that commit, naming
  the record's key and the commit; the same story without it passes,
  with every commit counted checked; a run under the default checks
  exactly one of its commits as it ends and reports it, its seed replays
  the same commit, and the defect's story at a seed whose kept commit is
  at or after the defect's fails as the run ends, naming the kept
  commit; a sampled run reports its count, and its seed replays the
  same sample.
- **The conformance world:** a commit-cut case whose child's restart a
  record refuses fails naming the commit and the refusal; the stories'
  commit-cut cases pass with every commit checked; an ordinary story
  checks exactly one of its live commits.
- **The forge world:** a stored record whose task the world doesn't hold
  fails the check under `every()`, naming the record; its stories pass.
  A 128-commit binding story checks and reports exactly one commit under
  its default and replays it without changing the live world; seeds 4
  and 26 check different commits.
- **The system and one-process worlds:** their stories pass with the
  default check, each checking one commit and reporting it. The
  one-process world's one-commit startup story checks every commit and
  compares the complete live disk, handles and counters against an
  unchecked control. Its 22-commit chat at seed 45 checks and reports one
  commit; a corrupt-commit story still fails at commit 2, naming the
  tool's refusal.

## 6. Later

- Every commit in the composed and simulated worlds, priced on its own
  once the budget plan's moves are decided
  (`../../development/failure-worlds-budget.md`).
