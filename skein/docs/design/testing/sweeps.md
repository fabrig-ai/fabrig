# Sweeps: the seed's streams, the fault ledger and exclusions

Provisional, 2026-11-20. What a world's sweep is made of: the independent streams it draws from its seed, the ledger of the faults it injected, the faults it configures and the assertion that each fell, and its named exclusions. Written from `temper/docs/design/draft/failure-worlds.md`, 3.3, 3.5, section 4's opening, 5 and 6. Its modules are `skein_world::streams`, `skein_world::faults` and `skein_world::sweep`, in the world harness (service-tiers.md, 1), so that every tree's worlds share one form. A sweep stays a test of the fuzzy suite looping over its seeds, as the strategy has it (../../foundation/testing-strategy.md, 3 and 8): these are what the loop holds, not a runner.

## 1. In one page

- **The seed's streams** (section 2): the workload, the faults, the parties' actions and the scheduling each draw from their own stream of the seed, and each part of a stream (an injector, a party) from its own part, so that a change to one leaves the others' draws as they were (the draft, 3.5 and 5).
- **The fault ledger** (section 3): every fault a world or a fake injects, with its kind, its interval and the identities it strikes, and beside each the counters of the draft's 3.3, so that a check can judge blame and tries against the cause.
- **A sweep** (section 4) names the faults it configures, folds each seed's ledger, and fails when a fault it configures never fell in any of its seeds.
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
}
```

- **Its name:** a fault ledger only grows, where `domain::Ledger` keeps requests in flight until each ends; the two stay apart, as do `skein_sim::Faults`, the simulator's table of chances.
- **Every fault is recorded as it falls** (the draft, 3.3): one the world injects (a party's action, a restart, a cut) by the world, and one a fake injects by the world from the fake's own report of what fell. A fake in `crates/` is step code and names no testing crate: it keeps what it injected in its own vocabulary, each with its moment and identities, and the world copies each into its ledger as the fake reports it.
- **Interval:** closed, `from` to `until`. `stop` returns a finding for an entry already stopped, or a time before its `from`. A fault still in force when the run ends keeps `until` as `None`.
- **Identities:** each is the identity's own name in the fake that holds it: an account's name, a runner's, a gate run's number in words, a commit's hex. A wrong tree is recorded with the candidate it is wrong in, and a base made red as the base's (the draft, 3.3). `striking` matches a field where both name the same identity.
- **Counters:** the world's observers count a transition (a retry, a try, a repair, an ask) against each fault they attribute it to, by `striking`, or by `fallen` where the transition answers a fault that has stopped, as a gate's retry answers a job lost at an instant; the expected transitions of each failure class are the 3.3 check's table, held by the tree that holds the check. The six counters are skein's because every tree's worlds count them and skein is the lowest vocabulary all of them name (../../foundation/programming-model.md, 4.6).
- **A ledger is one run's:** a world makes one per seed and returns it with its outcome.

## 4. A sweep

```rust
pub struct Sweep { /* name, configured faults, exclusions, fell, seeds, left out */ }

impl Sweep {
    pub fn new(name: &'static str) -> Sweep;
    /// A fault the sweep configures: one that must fall in at least one of its seeds.
    pub fn configure(&mut self, fault: Kind);
    /// Section 5's refusals, as a finding.
    pub fn exclude(&mut self, exclusion: Exclusion) -> Result<(), String>;
    /// The exclusion that leaves out `fault` while the conditions in `holding` hold, if any.
    pub fn leaves_out(&self, fault: Kind, holding: &[&str]) -> Option<&Exclusion>;
    /// Folds one seed's ledger: what fell, and what its exclusions left out.
    pub fn ran(&mut self, ledger: &FaultLedger);
    /// Each configured fault fell in at least one seed folded.
    pub fn check_fell(&self) -> Result<(), String>;
    /// How many draws an exclusion left out, over the seeds folded.
    pub fn left_out(&self, scenario: &str) -> u64;
}
```

- **The loop is the test's:** it makes the sweep, configures its faults and exclusions, and for each seed writes the seed to stderr, runs its world with `Streams::new(seed)` and the sweep borrowed, folds the run's ledger with `ran`, and returns the run's findings at once as any run does. After the last seed it calls `check_fell`.
- **Each configured fault fell** (the draft, 4): `check_fell` returns `sweep <name>: <family>/<name> never fell in <n> seeds`, one line for each that never fell, in `Kind`'s order. A sweep that configures a fault and never makes it fall fails, as any red. A fault that fell and isn't configured is no finding.
- **Left out:** where an exclusion leaves a draw out, the world doesn't inject it and records it in its ledger, `FaultLedger::leave_out(&mut self, scenario: &'static str, kind: Kind, at: Time)`; `ran` sums them by scenario.

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
- **The ledger:** an entry keeps its kind, interval and identities; `stop` refuses a second stop and a time before the start; `striking` finds a fault in force on a shared identity and not one stopped, or one on another identity; `fallen` finds one stopped too, and not one that falls after its moment; each counter counts on its own; `fell` gives each kind once.
- **A sweep:** a sweep whose configured fault never falls in any seed fails `check_fell`, naming the fault and the seeds; one where it falls in one seed of several passes.
- **Exclusions:** an exclusion that names no ignored scenario (empty, or not a function's name) is refused; so are one with no work, one with no conditions, one on a fault not configured, and a duplicate; `leaves_out` answers only while every condition holds; left-out draws are counted by scenario.
- **The boundary rule:** its step tests over source texts: an exclusion naming an ignored function of its tree passes; one naming a function without `#[ignore`, or none, fails naming file, line and scenario; the harness's own packages are outside it.

## 7. Later

- A sweep's report of its coverage, each check's finding kinds over its seeds (the draft, 12).
- Moving today's worlds to streams, each with its own goal's sweeps.
