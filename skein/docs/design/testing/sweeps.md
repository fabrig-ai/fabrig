# Sweeps: the seed's streams, the fault ledger and exclusions

Provisional, 2026-11-26. What a world's sweep is made of: the independent streams it draws from its seed, the ledger of the faults it injected, the faults it configures and the assertion that each fell, the cases it takes and the modes they come in, and its named exclusions. Written from `temper/docs/design/draft/failure-worlds.md`, 3.3, 3.5, section 4's opening, 5 and 6, and from `temper/docs/design/draft/fuzzy-campaigns.md`, 4. Its modules are `skein_world::streams`, `skein_world::faults`, `skein_world::sweep` and `skein_world::test_env`, in the world harness (service-tiers.md, 1), so that every tree's worlds share one form. A sweep stays a test of the fuzzy suite looping over its cases, as the strategy has it (../../foundation/testing-strategy.md, 3 and 8): these are what the loop holds, not a runner.

## 1. In one page

- **The seed's streams** (section 2): the workload, the faults, the parties' actions and the scheduling each draw from their own stream of the seed, and each part of a stream (an injector, a party) from its own part, so that a change to one leaves the others' draws as they were (the draft, 3.5 and 5).
- **The fault ledger** (section 3): every fault a world or a fake injects, with its kind, its interval and the identities it strikes, and beside each the counters of the draft's 3.3, so that a check can judge blame and tries against the cause.
- **A sweep** (section 4) names the faults it configures, folds each case's ledger, counting the cases each kind fell in, and fails when a fault it configures never fell in any of the gate's seeds.
- **Its cases** (4.2 to 4.5) come through one seam, `cases`, which takes the test's identity, its seeds and a check: at the gate it yields the seeds as given, in a campaign fresh ones derived from the campaign's seed, the test and the case's number, and under a replay one seed. A failing case names its test and seed, whether its check returns a finding or panics. A check over the gate's seeds holds only at the gate (`gate()`).
- **Exclusions** (section 5): an exclusion names its ignored scenario and the work that fixes it, and leaves out one fault in one combination of state the world names; it acts before a fault is injected, never on a finding, and nothing is matched by category. One that names no ignored scenario is refused.

## 2. The seed's streams

```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Stream { Workload, Faults, Parties, Scheduling }

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Streams { /* seed */ }

impl Streams {
    pub const fn new(seed: u64) -> Streams;
    /// The seed itself, which a finding names.
    pub const fn seed(&self) -> u64;
    /// The seed of one part of a stream, for a component or fake that takes a `u64`.
    pub fn seed_of(&self, stream: Stream, part: &str) -> u64;
    /// `Rng::new(self.seed_of(stream, part))`.
    pub fn rng(&self, stream: Stream, part: &str) -> skein_lib::Rng;
}
```

- **The derivation,** fixed, so that a pinned seed replays the same draws in every release:
  - `mix(x)` is `Rng::new(x).next_u64()`, SplitMix64's step and finalizer;
  - a stream's tag is its name's eight ASCII bytes, read little-endian: `workload`, `faults\0\0`, `parties\0`, `schedule`;
  - a part's hash is FNV-1a 64 over its UTF-8 bytes (basis `0xcbf29ce484222325`, prime `0x100000001b3`); the empty part is the stream's own;
  - `seed_of(stream, part)` is `mix(mix(seed ^ tag(stream)) ^ fnv(part))`.

  Adding the golden ratio to a seed only shifts a SplitMix64 sequence by one draw, so a stream is never seeded by an offset of another's seed: every derived seed passes through `mix`.
- **Independent by construction:** a stream's draws depend on the seed, the stream and the part alone, never on another stream's draws, so a run with restarts and one without draw the same goals, faults and parties' actions (the draft, 3.5).
- **Parts:** each injector its own part of `Faults` (`forge`, `llm`, `restarts`), each party its own of `Parties`, so that a fault family added to a sweep, or a restart, leaves the others' draws as they were. A part's name is the world's.
- **Scheduling** is what the simulator and a world's own delivery order draw: a world seeds its simulated world, its latencies and its delivery order from `Scheduling`.
- **Adoption is each world's,** as its goal moves it: today's worlds keep their seeding, and a world that moves to streams moves its pinned seeds with it.

## 3. The fault ledger

```rust
/// A fault's kind: its family, as the draft's section 4 names them, and its name within it.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Kind { pub family: &'static str, pub name: &'static str }

/// The identities a fault strikes; a field it doesn't strike is `None`.
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Struck {
    pub account: Option<String>,
    pub runner: Option<String>,
    pub gate_run: Option<String>,
    pub candidate: Option<String>,
    pub base: Option<String>,
    pub request: Option<String>,
}

/// The counters of the draft's 3.3, each its own.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Counter { ProviderRetries, SessionRetries, TaskTries, GateRetries, Repairs, Asks }

/// A fault's entry in its ledger.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Injection(/* index */);

pub struct Injected {
    pub kind: Kind,
    pub from: Time,
    /// When it stopped; `None` while it is in force, or for one that never stops in the run.
    pub until: Option<Time>,
    pub struck: Struck,
    /* a count for each counter */
}

impl Injected {
    pub fn count(&self, counter: Counter) -> u64;
    /// `from <= at`, and `at <= until` where it stopped.
    pub fn in_force(&self, at: Time) -> bool;
}

pub struct FaultLedger { /* entries */ }

impl FaultLedger {
    pub fn new() -> FaultLedger;
    /// Records a fault as it starts.
    pub fn inject(&mut self, kind: Kind, from: Time, struck: Struck) -> Injection;
    /// Records when it stops; an instant's fault stops at its start.
    pub fn stop(&mut self, entry: Injection, at: Time) -> Result<(), String>;
    /// Counts one transition of a counter the world attributes to the fault.
    pub fn count(&mut self, entry: Injection, counter: Counter);
    pub fn get(&self, entry: Injection) -> &Injected;
    pub fn entries(&self) -> &[Injected];
    /// The faults in force at `at` that strike an identity `struck` names, in the order they fell.
    pub fn striking(&self, at: Time, struck: &Struck) -> Vec<Injection>;
    /// The faults that fell at or before `at` that strike an identity `struck` names,
    /// in force or stopped, in the order they fell.
    pub fn fallen(&self, at: Time, struck: &Struck) -> Vec<Injection>;
    /// The kinds that fell, each once.
    pub fn fell(&self) -> BTreeSet<Kind>;
    /// Each kind of `bounds` whose name is a table's name in `inspections`:
    /// in force from the first call that sees that table at its bound, as
    /// many entries as its capacity or a newcomer refused, until the first
    /// call that sees it at neither. A refusal of `stop` is returned.
    pub fn saturation(&mut self, bounds: &[Kind], inspections: &[&crate::inspect::Inspection], at: Time) -> Result<(), String>;
}
```

- **Its name:** a fault ledger only grows, where `domain::Ledger` keeps requests in flight until each ends; the two stay apart, as do `skein_sim::Faults`, the simulator's table of chances.
- **Every fault is recorded as it falls** (the draft, 3.3): one the world injects (a party's action, a restart, a cut) by the world, and one a fake injects by the world from the fake's own report of what fell. A fake in `crates/` is step code and names no testing crate: it keeps what it injected in its own vocabulary, each with its moment and identities, and the world copies each into its ledger as the fake reports it.
- **Interval:** closed, `from` to `until`. `stop` returns a finding for an entry already stopped, or a time before its `from`. A fault still in force when the run ends keeps `until` as `None`.
- **Identities:** each is the identity's own name in the fake that holds it: an account's name, a runner's, a gate run's number in words, a commit's hex. A wrong tree is recorded with the candidate it is wrong in, and a base made red as the base's (the draft, 3.3). `striking` matches a field where both name the same identity.
- **Counters:** the world's observers count a transition (a retry, a try, a repair, an ask) against each fault they attribute it to, by `striking`, or by `fallen` where the transition answers a fault that has stopped, as a gate's retry answers a job lost at an instant; the expected transitions of each failure class are the 3.3 check's table, held by the tree that holds the check. The six counters are skein's because every tree's worlds count them and skein is the lowest vocabulary all of them name (../../foundation/programming-model.md, 4.6).
- **A ledger is one run's:** a world makes one per seed and returns it with its outcome.
- **A bound reached is a fault that falls** (the draft, 4.7: the workload reaches each bound of 3.2's table): `saturation`, called at a world's quiet point after its adapters inspect (inspection.md, 5), records a table at its bound under the kind its caller names for it, its name the table's (inspection.md, 3), with `Struck::default()`, and stops it once the table is below its bound and refuses no newcomer, so that a sweep configuring the kind holds, by `check_fell`, that the bound was reached in some seed. A table matched in several components is at its bound where any is; a table of capacity `u64::MAX` never is. Each tree's kit names its tables' kinds (`temper/docs/design/testing/workloads.md`, 2.4).

## 4. A sweep

### 4.1 Its faults

```rust
pub struct Sweep { /* name, configured faults, exclusions, the cases each kind fell in, seeds, left out */ }

impl Sweep {
    pub fn new(name: &'static str) -> Sweep;
    /// A fault the sweep configures: one that must fall in at least one of its seeds.
    pub fn configure(&mut self, fault: Kind);
    /// Section 5's refusals, as a finding.
    pub fn exclude(&mut self, exclusion: Exclusion) -> Result<(), String>;
    /// The exclusion that leaves out `fault` while the conditions in `holding` hold, if any.
    pub fn leaves_out(&self, fault: Kind, holding: &[&str]) -> Option<&Exclusion>;
    /// Folds one case's ledger: one more case for each kind that fell, and what its exclusions left out.
    pub fn ran(&mut self, ledger: &FaultLedger);
    /// At the gate, each configured fault fell in at least one seed folded;
    /// outside it, passes and writes the sweep's reach (4.5).
    pub fn check_fell(&self) -> Result<(), String>;
    /// How many draws an exclusion left out, over the seeds folded.
    pub fn left_out(&self, scenario: &str) -> u64;
}
```

- **The loop is the test's:** it makes the sweep, configures its faults and exclusions, and for each case runs its world with `Streams::new(seed)` and the sweep borrowed, folds the run's ledger with `ran`, and returns the run's findings at once as any run does. After the last case it calls `check_fell`. A sweep that takes part in campaigns takes its cases from `cases` (4.2), which names a failing case itself (4.4); a loop of its own over fixed seeds writes each seed to stderr.
- **Counted by case:** `ran` adds one to the count of each kind that fell in the case, since `FaultLedger::fell` names each kind once, so a sweep knows how many of its cases each kind fell in, not only that it fell.
- **Each configured fault fell** (the draft, 4), at the gate: `check_fell` returns `sweep <name>: <family>/<name> never fell in <n> seeds`, one line for each configured kind whose count is nought, in `Kind`'s order. A sweep that configures a fault and never makes it fall fails, as any red. A fault that fell and isn't configured is no finding. Outside the gate's mode `check_fell` passes and writes its reach (4.5).
- **Left out:** where an exclusion leaves a draw out, the world doesn't inject it and records it in its ledger, `FaultLedger::leave_out(&mut self, scenario: &'static str, kind: Kind, at: Time)`; `ran` sums them by scenario.

### 4.2 Its cases

```rust
/// The test a sweep belongs to: its package, its module path and its name,
/// and a part where one test, or one helper serving several, runs several sweeps.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Test { /* package, module, name, part */ }

impl Test {
    pub const fn new(package: &'static str, module: &'static str, name: &'static str, part: Option<&'static str>) -> Test;
}

/// The identity: `package::module::name`, then `/part` where it has one.
impl core::fmt::Display for Test;

/// `this_test!("name")` or `this_test!("name", "part")`: the calling test,
/// its package (`env!("CARGO_PKG_NAME")`) and module (`module_path!()`) taken where it is written.
#[macro_export]
macro_rules! this_test { /* ... */ }

/// A failing case: its test, its seed, and the finding its check returned.
#[derive(Debug)]
pub struct Case<E> { pub test: Test, pub seed: u64, pub finding: E }

/// 4.4's line: `fuzzy case: TEST seed SEED: FINDING`.
impl<E: core::fmt::Display> core::fmt::Display for Case<E>;

/// Judge each of a sweep's cases, in the process's mode (4.3). The first case whose
/// check returns a finding ends the sweep, and comes back with its test and seed.
pub fn cases<S, E, F>(test: Test, seeds: S, check: F) -> Result<(), Case<E>>
where
    S: IntoIterator<Item = u64>,
    E: core::fmt::Display,
    F: FnMut(u64) -> Result<(), E>;

/// Whether the cases are the gate's seeds: for a check over all of them,
/// and for a sweep's configuration.
pub fn gate() -> bool;
```

They are `skein_world::sweep`'s, and `this_test!` is at the crate's root.

- **`Test`** is the case's identity, and the name its lines give it. Its module is `module_path!()`'s, so its first segment is the test's crate (`suite::fuzzy::walking` in a `suite` binary). A helper serving several tests takes its caller's identity, passed in, never its own; a test, or a helper, that runs several sweeps gives each a part.
- **`seeds`** is a range or a hand-picked list: the gate's seeds, unchanged. Its length is the case count, at the gate and in a campaign.
- **One case a seed of its test:** a test whose loop runs several parts at each seed (a scenario, a provider, a file kind, an edit) runs them all inside that seed's case, so that a replay of the test runs one case (`tools/tiers/campaign.md`, 4). The gate runs the same parts at the same seeds, in another order. Parts with seeds of their own are sweeps of their own: tests of their own where no check over the loop spans them, else parts of the test's identity, whose cases a campaign keeps unreplayed (section 7).
- **A stream across a loop:** a loop whose rounds draw from one `Rng` made before it, not one a round, is one case at that stream's seed, running every round inside it, so that the gate's draws stay. A campaign gives it a fresh stream.
- **A check returns its finding,** as jig's core sweeps return theirs, or panics (4.4). A test returns `cases`'s result, or words it in its own failure.
- **A case's seed is the world's seed.** A world on streams takes it as `Streams::new(seed)`, and one that isn't takes it as it took its loop's seed. A draw made some other way, as a fixture picked by `seed % n`, takes it from the case's seed too, so that a campaign moves every draw.
- **Counted as `each_seed` counts:** each case `cases` runs is a `seed` check (`skein_checks::count`, checks.md), in every mode, so a sweep that moves from `each_seed` keeps its counts. A loop that counted its seeds by hand stops, since `cases` counts them, and one that counted none gains one a case: no count falls.
- **`each_seed` stays** as it is, for the loops that keep fixed seeds, focused tests' among them: it runs its range in every mode, as given, and counts it.

### 4.3 Its modes

- **The gate's,** with neither variable set: the seeds as given, in order. It is the same work as a loop over them.
- **A campaign's,** `SKEIN_TEST_FUZZY_CAMPAIGN=SEED`: as many fresh seeds as the seeds given. Case k's, k counting from 0, is `mix(mix(SEED ^ fnv(TEST)) ^ k)`, with `mix` and `fnv` as section 2 fixes them and TEST the identity's text (4.2). A case depends on the campaign's seed, its test and its number alone: never on another test, on the order tests run, or on the seeds given beyond their count.
- **A replay's,** `SKEIN_TEST_FUZZY_CASE=SEED`: that one seed, once, in each sweep of the test its run selects, whatever the seeds given. A replay is of a campaign's case, so it runs the campaign's configuration: `gate()` is false under it.
- **SEED** is sixteen hex digits, either case, with no prefix: the form a campaign prints and every line of 4.4 writes, in lower case.
- **The variables** are skein's test set (../../foundation/cli.md, 4 and 5), declared in skein's test table, `skein_world::test_env::TESTS`, whose document is skein's test-env document (the same, 8). They are read once in a process, through that table, the first time `cases` or `gate()` asks, so every sweep of a process runs in one mode. Empty is unset. A value that isn't sixteen hex digits, or both set at once, is a setup error: `cases` and `gate()` panic before any case, naming the variable, or both, and the value.
- **Out of the gate's way:** `tools/tiers/one-process` removes both from the environment of the processes it runs (`tools/tiers/design.md`, 11.5), so the gate, the delivery check and anyone's run of the focused and fuzzy suites take the gate's mode, whatever the caller's shell holds. Only a campaign sets them, for its pair's runs (`temper/docs/design/draft/fuzzy-campaigns.md`, 5).

### 4.4 A failing case names itself

- **A finding a check returns** ends the sweep: `cases` writes `fuzzy case: TEST seed SEED: FINDING` to stderr, the finding as its `Display` writes it, and returns the `Case`, whose `Display` is the same line.
- **A check that panics:** the seam installs a panic hook once in a process, as its first case starts, keeping the hook it replaced. While a case's check runs, the thread running it holds the case's test and seed, for that case alone, and lets go of them as the check returns or unwinds. The hook writes `fuzzy case: TEST seed SEED` for the case its thread holds, then calls the hook it replaced. Nothing catches the panic (../../foundation/programming-model.md, 10.4). A panic outside a case, an assertion after the loop or a setup error, writes no case line, so a campaign reads it as a failure outside a case.
- **One line a failing case:** the seam writes exactly one `fuzzy case:` line for it, whichever way it failed, and a reader names a case by its TEST and SEED.
- **Outside the gate's mode,** `cases` writes `fuzzy cases: TEST N` to stderr as it returns, N the cases it ran, a failing one included, so that a campaign tells a sweep that ran from a test that took no case. A case that panics leaves the hook's line in its place.
- **At the gate too,** a failing case names its seed, as no loop of its own does unless its test prints it.
- **These lines are the contract** a campaign reads (`temper/docs/design/draft/fuzzy-campaigns.md`, 5.4); the replay command is the campaign's to write, since the campaign knows the binary and the full test name.

### 4.5 What a case checks

- **A case's own checks** run in every mode: contracts as it goes, invariants once it settles, its referee, its goals' expectations (the strategy's section 6).
- **A check over the gate's seeds** holds only for those seeds: `check_fell`, an exact count of what was left out, a coverage array filled across the loop. Outside the gate's mode, `check_fell` returns `Ok` and writes `fuzzy reach: SWEEP FAMILY/KIND N` to stderr for each configured kind, in `Kind`'s order, SWEEP the sweep's name and N the cases it fell in, nought included. A test's own checks over its loop run only where `gate()` holds.
- **Every seed a valid case.** A world takes its cases from `cases` only once any seed draws a case it can run. A draw its world can't run, a configured kind with no eligible target or a hold drawn with no release, is the world's to make valid: drawn again from the case's own stream, or skipped and counted as ineligible in the sweep's reach. It is never failed as the product's.
- **A full-width seed:** a case's seed is any `u64`. A world that does arithmetic on its seed wraps it or derives from it, and keeps the draws of the gate's seeds. A seed reduced to an index or a duration is reduced in `u64` before it narrows. A loop that stopped once its coverage was complete takes the fixed seeds its gate ran, its coverage checked after them. A change that makes a draw total pins it: a fixed test draws the case at `u64::MAX` through the code it changed, without running the case where the draw alone shows it valid.
- **A fixed scenario keeps its seed.** A test that asserts what its seed was chosen to show (a pinned seed's named outcome, a golden, a digest) takes no case from the seam, and a campaign runs it as the gate does.
- **The gate's counts stay:** a sweep that adopts `cases` keeps its binary's checks counts as they were, and its instructions within the growth allowance (`tools/tiers/budgets.md`, 8).

## 5. Exclusions

```rust
pub struct Exclusion {
    /// The ignored scenario it names: its test function's name.
    pub scenario: &'static str,
    /// The work that fixes it, as the draft's section 7 names it.
    pub fixed_by: &'static str,
    /// The fault it leaves out.
    pub fault: Kind,
    /// The combination: each condition of workload or state that must hold, in the world's words.
    pub when: &'static [&'static str],
}

impl Exclusion {
    pub const fn new(scenario: &'static str, fixed_by: &'static str, fault: Kind, when: &'static [&'static str]) -> Exclusion;
}
```

- **Exactly its scenario's combination** (the draft, 6): an exclusion leaves out its fault only while every condition of `when` holds, as the world reads them, through its adapters (inspection.md) or its own state, at the moment it would inject the fault. Cancelling a landing stays in a sweep; cancelling one with a gate's dispatch in flight is left out: `Exclusion::new("cancelled_landing_frees_its_write_entries", "landing train", CANCEL_LANDING, &["a gate's dispatch in flight"])`.
- **Never on a finding:** an exclusion acts before a fault is injected. Every finding fails its run, and nothing compares a finding with an exclusion or a category.
- **`Sweep::exclude` refuses,** each with a finding naming the sweep and the exclusion:
  - a `scenario` that is empty, or isn't a test function's name (lowercase letters, digits and `_`, starting with a letter): it names no ignored scenario;
  - an empty `fixed_by`: it names no work that fixes it;
  - an empty `when`: it would leave out the whole fault, not a combination;
  - a `fault` the sweep doesn't configure;
  - a second exclusion with the same scenario, fault and conditions.
- **Its scenario is ignored:** the scenario an exclusion names is a test function marked `#[ignore = "<reason>"]` in the same tree, its reason naming the work that fixes it (the draft, 6). `tests/boundary` holds it from the sources: a rule of `skein_worlds.rs`'s one walk of the tracked Rust sources (tools/crates/design.md, 3) takes each `Exclusion::new(` whose first argument is a string literal, and fails naming the file, the line and the scenario where no tracked Rust source of the same tree (`skein/`, `smith/`, `jig/`, `temper/`, `hearth/`) has a function of that name under an `#[ignore` attribute: a sweep runs in the cheapest world that reaches a fault, and its scenario in its lowest harness, which may be another package of the tree. The harness's own packages, `skein-world` and `skein-world-tests`, whose tests exercise the refusals, are outside the rule.
- **The fixing work** removes the exclusion and the `#[ignore]` in the change that fixes it (the draft, 6); the rule fails a change that removes the `#[ignore]` alone.

## 6. Testing

Step tests in the crate's own `src/tests/`, with no world:

- **Streams:** one seed's four streams draw four different sequences; a stream's first draws are the same whether or not another stream, or another part of its own, was drawn first; two parts of one stream differ; one seed replays; `seed_of` for seed 0 and each stream's own part is pinned to its value, so the derivation never changes silently.
- **The ledger:** an entry keeps its kind, interval and identities; `stop` refuses a second stop and a time before the start; `striking` finds a fault in force on a shared identity and not one stopped, or one on another identity; `fallen` finds one stopped too, and not one that falls after its moment; each counter counts on its own; `fell` gives each kind once; `saturation` starts a kind at the first call seeing its table full, keeps one entry while it stays full, starts it for a refused newcomer below capacity too, stops it at the first call seeing it below and refusing none, starts it afresh when it fills again, and leaves a kind whose table no inspection holds, or one of capacity `u64::MAX`, unrecorded.
- **A sweep:** a sweep whose configured fault never falls in any seed fails `check_fell` at the gate, naming the fault and the seeds; one where it falls in one seed of several passes; `ran` counts each kind's cases, once a case however often it fell in it; outside the gate's mode, `check_fell` passes and writes one reach line for each configured kind, in `Kind`'s order, nought included.
- **Cases,** each mode given to the seam directly, so that no test sets or depends on the process's environment: at the gate, the seeds as given, in order; in a campaign, as many fresh seeds as given, their values pinned for a campaign seed and a test with and without a part, and the same whatever other tests derived before; under a replay, the one seed once, whatever the seeds given; the first finding ends the sweep with its test and seed; `gate()` in each mode; the lines' format; `this_test!`'s identity.
- **The variables:** read from captured pairs, empty is unset; a value of other than sixteen hex digits (fifteen, seventeen, a `0x` prefix, a byte that isn't a hex digit, bytes that aren't UTF-8) and both set at once are setup errors naming the variable; either case reads.
- **The hook,** with no panic, since no test catches or expects one (`tools/lints/panics.py`): it writes its line for the case its thread holds and then calls the hook before it, writes none when its thread holds no case, and a case is let go of once its check returns.
- **The test table:** its names, each `Set::Test`, in order, and its document, byte for byte, rewritten under the rewrite switch.
- **Exclusions:** an exclusion that names no ignored scenario (empty, or not a function's name) is refused; so are one with no work, one with no conditions, one on a fault not configured, and a duplicate; `leaves_out` answers only while every condition holds; left-out draws are counted by scenario.
- **The boundary rule:** its step tests over source texts: an exclusion naming an ignored function of its tree passes; one naming a function without `#[ignore`, or none, fails naming file, line and scenario; the harness's own packages are outside it.

## 7. Later

- A sweep's report of its coverage, each check's finding kinds over its seeds (the draft, 12).
- Moving today's worlds to streams, each with its own goal's sweeps.
- Moving sweeps to `cases`, world by world (`temper/docs/design/draft/fuzzy-campaigns.md`, 9).
- A sweep asking a campaign for more cases than its gate seeds (the same draft, 6), with the first sweep that needs it.
- A replay naming one part of a test that runs several sweeps: today it runs its seed once in each, so a campaign replays such a test's case as one test and one case only where it has one part.
