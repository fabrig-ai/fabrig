# The store check: every commit leaves a store a start would open

Provisional, 2026-11-21. How temper's worlds hold
`../draft/failure-worlds.md`, 3.7: after a commit, a store restored from
that committed snapshot, apart from the live world, passes everything a
start checks, jig's checks and temper's connector checks above them,
and a record a start would refuse fails the world at the commit that
made it, naming the record and the commit. jig gives the operation, its
choice of commits and its finding (`jig/docs/design/domain/testing-fakes.md`,
1, the store check); this document is temper's: its root's rehearsal as
the start that checks, and where each world calls it.

## 1. In one page

- **The check is a rehearsal** (section 2): temper's root restores as at
  a start and ends before the switch (`../domain/root-restart.md`, 1.1), over the
  world's store copied at a commit, so that the forge's checks run on
  what jig's restores, and nothing the live world holds is touched.
- **Every commit** in the conformance world's commit-cut cases, where
  each child's restart is the check of its commit; **sampled** from the
  seed in the engine's domain, forge and system worlds and the
  one-process world; each run reports how many of its commits it checked
  (section 3).
- **A finding** is jig's: the commit, the phase, the refusal naming its
  record (section 4).

## 2. The rehearsal

- **temper's start** is its root's (`../domain/root-restart.md`, 1.1): jig's core
  records, the forge's records, their links, the sweep; a refused start
  stops with `temper_engine_domain::StartStop`, and a rehearsing root
  (`Domain::rehearse`, before the start begins) ends before the switch,
  `Domain::rehearsed()` then true. No step before the switch leaves the
  process (`jig/docs/design/domain/engine/restart.md`), so a rehearsal needs no
  peer: its forge fake, scripted workers and parties are never reached.
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
  commits landing in the copy alone.
- **One function for the engine's worlds:**
  `temper_engine_domain_world::check::store(store: &Store, config: &Config, limits: &Limits, wall: Wall, commit: u64) -> Result<(), String>`,
  `Config` and `Limits` what `Driver::from_store_at` takes, `wall` the
  world's wall time at the commit: a rehearsing driver over the fork,
  driven until `rehearsed()` or a `start_stop()`, its `Err` jig's finding
  (section 4). The system world, whose store is the domain world's, calls
  it too.
- **In the conformance world** temper's `Application` gives jig's harness
  the root's rehearsal: `rehearse` is `Domain::rehearse`, `rehearsed` is
  `Domain::rehearsed`, `refused` is `start_stop()` in its `Debug` form,
  and `start_phase` is the root's `start_phase()`.
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
  those. The root maps it to
  `StartStop::ForgeRow`, `ForgeFit` and `ForgeLink` as it does now, in
  the order it checks them now.
- **The one-process world** rehearses as `temper-box upgrade --check`
  does (`../protocol/binaries.md`; `../domain/root-restart.md`, 1.1): the real
  service, in the upgrade tool's rehearsal, over a copy of the fake
  machine's disk taken at the sampled commit, as its upgrade stories
  already run it over the floor's disk; a report that refuses is the
  finding.

## 3. Which commits, in which world

jig's `StoreChecks` choose them (`jig/docs/design/domain/testing-fakes.md`,
1): every commit, or each commit with one chance in `one_in` drawn from
the run's seed, each run's count reported.

| World | Commits | Where it calls the check |
|---|---|---|
| `temper-conformance-world` | every commit of its commit-cut cases, by each child's restart; sampled elsewhere | jig's harness: `Harness::check_store`, and a cut's restart through `refused` |
| `temper-engine-domain-world` | sampled | once its driver applies a commit (`Driver::advance`) |
| `temper-system-world` | sampled | once its composed world's store applies a commit |
| `temper-engine-forge-world` | sampled | once the world has applied one step's writes: a `take_top` output's saves and erases together, its projections' included, as the root commits them in one commit (a change's cold and hot parts are two saves of one commit, never two commits) |
| `temper-one-process-world` | sampled | once its referee observes a commit |

- **The sample:** each sampled world names its `one_in`, chosen to keep
  its checks within the gate's instruction-growth bound
  (`tools/tiers/budgets.md`, 8). Most runs check no commit; every run
  reports its checked and total counts. The core kit, conformance
  harness, engine domain, forge, system and one-process worlds each use
  one in 1024. The conformance application's ordinary stories use the
  harness's sample; their commit-cut cases still check every commit.
  These samples cover ordinary runs of a few commits or a few dozen,
  rather than promising one check per run. Every commit everywhere
  remains a separate, priced choice (section 6).
- **Its call is one line at a world's commit.** The engine's worlds'
  shared kit is the engine checks' goal's
  (`../draft/failure-worlds.execution.md`, 2): where that kit holds a
  world's driver, the call moves into it, and nothing else of the kit is
  this check's.
- **A story about a refused start** whose own input is a record a start
  refuses reads the check's `Err` as its outcome, or sets
  `StoreChecks::none` with a reason naming the refusal it tests.

## 4. The finding and the report

- **The finding** is jig's form (`jig_conformance::store_check::finding`):
  `store check: commit <n>: start refused at <phase>: <refusal>`, the
  refusal naming its record's key wherever it names one. The world
  returns it as its run's finding, as its referee's, and fails at that
  commit.
- **A sampled world's finding** names the commit it checked, at or after
  the one that made the record; the same seed under
  `StoreChecks::every()` names the commit that made it.
- **The report:** each run writes `StoreChecks::report` to stderr as it
  ends, the commits checked of the run's commits, so that no sampled run
  claims more than it checked; an exhaustive claim is only a commit-cut
  sweep's (`jig/docs/design/domain/testing.md`, 5).

## 5. Testing

Focused, in each world the check runs in; each is a scenario where the
check finds what it should, and one where a correct store passes it (the
draft, 11):

- **The engine's domain world:** a story that commits, as a writer's
  defect would, a forge record a start refuses (a change with one of its
  two parts), under `StoreChecks::every()`, fails at that commit, naming
  the record's key and the commit; the same story without it passes,
  with every commit counted checked; a sampled run reports its count,
  and its seed replays the same sample.
- **The conformance world:** a commit-cut case whose child's restart a
  record refuses fails naming the commit and the refusal; the stories'
  commit-cut cases pass with every commit checked.
- **The forge world:** a stored record whose task the world doesn't hold
  fails the sampled check under `every()`, naming the record; its
  stories pass. A 128-commit binding story reports and replays its
  default sample without changing the live world; seeds 4 and 26 check
  different commits.
- **The system and one-process worlds:** their stories pass with the
  sampled check, each reporting its count. The one-process world's
  one-commit startup story checks every commit and compares the complete
  live disk, handles and counters against an unchecked control. Its
  22-commit chat at seed 45 checks and reports one sampled commit; a
  corrupt-commit story still fails at commit 2, naming the tool's refusal.

## 6. Later

- Every commit in the composed and simulated worlds, priced on its own
  once the budget plan's moves are decided
  (`../../development/failure-worlds-budget.md`).
