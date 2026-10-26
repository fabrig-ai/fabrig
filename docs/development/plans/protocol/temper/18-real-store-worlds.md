# Session 18: the engine worlds on jig's real store; the system world on the service's translations

temper's engine worlds stop answering the root's commits and loads with
jig's fake store and run jig's store component instead, over skein-kv's
in-memory mode, with temper's families encoded by
`temper-protocol-store` and routed by `temper-service`'s own store
routing, so every story runs the real keys, codecs, interval and paging
(store/README.md, section 6). The driver keeps the face the worlds use
today (a typed mirror of the durable rows, the commits held between the
root and the store, paging), so most stories change nothing. The
engine's domain world and the composed system world move first, since
they share the driver; the conformance world follows, and jig's fake
store leaves the workspace. Last, the composed system world calls the
service's translations of temper's parts and `temper_engine_smith`'s
charter in place of its own (channel.md, section 7), so the domain tier
and the protocol layer translate alike. All three increments are world
code.

## Read first

- `docs/design/store/README.md`, section 6 ("The real store in the domain
  worlds") and section 5 (the restart, the cursor).
- `docs/design/domain/testing.md`, 4.5 (temper's own components; the composed
  system world "jig's store component over skein-kv's in-memory mode, in
  place of jig's fake store").
- `docs/design/channel.md`, section 7 (the translations' table, and "The
  composed system world ... calls these functions").
- jig at the pin: `docs/design/protocol/store.md`, section 14;
  `testing/jig-test-store-files/src/{store.rs, files.rs, durable.rs}` (the
  driver to follow: `Store::new`, `submit`, `tick`, `crash`, `fork`,
  `seed`, `ask`, `drive`, `component_limits` at store.rs:62-386);
  `testing/jig-conformance/src/harness.rs:10-25` (`Durable`).
- `findings/jig-components.md`, section 1 and Discrepancies 3, 4, 5;
  `findings/temper-engine.md`, section 6.
- `decisions.md`, 14.
- The sessions this one builds on: `temper/04-store-codecs.md` (04.5:
  `temper_protocol_store::{STORE_VERSION, application}`),
  `temper/08-service-engine.md` (08.2: the store routing; 08.3 and 08.4:
  the engine's translations and `temper_engine_smith::charter`),
  `temper/09-service-worker.md` (09.1: the worker's translations).
- The code: `tests/engine/domain/src/store.rs` (whole, 153 lines);
  `tests/engine/domain/src/{direct.rs:79-145, walking.rs:515-530, 640-660,
  escalation.rs:595-605, 650-660, roles.rs:185-195, 440-450, 500-508}`;
  `tests/engine/domain/tests/{root_routes.rs:970-1012, 1290-1320,
  2390-2500, 4145-4160, loads.rs:40-70}`;
  `tests/system/src/composed.rs:380-390, 470-495`,
  `tests/system/tests/composed.rs:125-140`;
  `tests/conformance/src/{lib.rs:140-190, 280-300, 335-375, 420-470,
  observations.rs:140-160}`; `tests/system/src/composed/link.rs` (whole,
  as 01.3 leaves it).

## What the design asks

- **The real store in the domain worlds** (store/README.md, 6): "temper's
  engine worlds run jig's component over skein-kv's in-memory mode, with
  temper's families encoded by `temper-protocol-store`, in place of
  jig's fake store ... So every story runs the real keys, codecs and
  interval. Faults come from the world: it holds commits between the
  root and the store, delays them, fails one to stop the engine, and a
  crash keeps what reached the store."
- **The system world** (domain/testing.md, 4.5): "jig's store component over
  skein-kv's in-memory mode, in place of jig's fake store once temper's
  families have their codecs".
- **One translation** (channel.md, 7): "The composed system world
  (`tests/system`, `composed/link.rs`) calls these functions, each record
  passed from the encoding side to the decoding side without its bytes,
  in place of its own translation of assignments and answers. So the
  domain tier and the protocol layer translate alike, and drift is a
  build error on either."

## Choices this plan makes

- **temper writes its own driver** (decision 14): jig's
  `jig_test_store_files::Store` is bound to jig's test root. The driver
  replaces `tests/engine/domain/src/store.rs`'s `Store` in place, after
  jig's (store.rs:62-386 at the pin), and reuses
  `jig_test_store_files::Files` for the transcripts' file requests the
  component makes in memory mode (`findings/jig-components.md`,
  Discrepancy 5).
- **The routing is the service's, called, never copied:** 08.2's store
  routing (`temper_service`'s; the names 08.2 lands) turns the root's
  `StoreRequest`s into `jig_protocol_store::Request`s and the
  component's events back into the root's `StoreInput`s, and 08.2's
  derivation of the component's limits (its `sizes::check` call) sizes
  it. Either one private to `temper-service` is a question (below).
- **The face stays,** so the worlds' code changes little: `pub applied`,
  `pub rows` (a typed mirror the driver updates on each `Committed`, as
  jig's does at store.rs:269-285), `pub pending`, `new()`, `apply()`,
  `page()` (now `&mut self`: it drives a load through the component and
  returns its page synchronously, the in-memory stores answering at
  once), `header()`. New: `with_limits`, `submit`, `tick`,
  `release_held`, `fault`, `crash`, `fork`, `seed`, `idle`.
- **`Store::new()` sizes the component for the widest limits a world
  uses** (`store::WIDEST`: `walking::limits()` with the journal, task and
  brief bounds `forge_routes.rs:113-129` raises), so the 30
  `Store::new()` calls of root_routes.rs and the others stay; a world
  whose limits pass `WIDEST` calls `Store::with_limits`.
- **A test that edits the mirror edits the store:** a copy of a store is
  `fork()` (the durable commits replayed with their numbers, jig's
  store.rs:171-180); a changed or removed row is `seed(saves, erases)`,
  one numbered transaction after the last applied with the header's
  `commits` set to its number (jig's store.rs:181-197). Writing `rows`
  directly reaches nothing, and no test does it after 18.1.
- **Core reads stay the stand's:** the engine world's protocol stand
  answers `StoreRequest::Read` with `Loaded::Gone`
  (`tests/engine/domain/src/protocol.rs:199-205`), as today; the
  driver takes commits and loads only.
- **The skeleton's 18.1 and 18.2 split by the driver's users:** the
  composed system world uses the engine world's `Store`, so it moves in
  18.1 with it; 18.2 is the conformance world, which has its own
  `jig_fake_store::Store` alias, and the workspace's dependency.

## Rules that bind it

- "**Large files.** ... An increment adds at most 40 lines to each"
  (README, "Rules"). Here: `tests/engine/domain/tests/root_routes.rs`,
  `forge_routes.rs`, `tests/engine/domain/src/{escalation.rs, walking.rs}`.
- "**Keep the pins** (code-organization.md, section 3): an increment that
  adds a crate of jig, smith or skein runs the three `cargo update
  --precise` commands, in that order, and the check prints exactly the
  three pinned revisions before it lands" (README, "Rules"): 18.1 adds
  `jig-test-store-files`.
- "a fixture that gains what a real component requires (18's real
  store), its assertions exact, built from the fixture's constants"
  (README, "Edits outside an increment's Files"), and "**A new test
  world** costs at most about 1 second focused and 5 seconds fuzzy"
  (development.md, 5.3): the worlds' suites keep their budgets (15 s
  focused, 60 s fuzzy, README "Pulse").

## Increments

### 18.1 The engine's domain world and the system world on jig's store

Scope: store/README.md, 6; domain/testing.md, 4.5. `tests/engine/domain/src/store.rs`'s
`Store` becomes the driver (the sketch below): jig's
`jig_protocol_store::Store::over(&limits, temper_protocol_store::application(&forge_limits),
Stores { store: skein_kv::Store::memory(..), secrets: skein_kv::Store::memory(..) })`,
opened on `Files::root()`; the root's commits and loads routed by 08.2's
store routing; `Files` answering the transcripts' file requests; the
typed mirror kept on `Committed`; faults `Hold { commits }`, `Fail {
commit }`, `SlowPages { by }` as jig's (store.rs:11-17). Its users
follow: the world drivers' `page` calls (direct.rs:96, walking.rs:654,
escalation.rs:657, roles.rs:504) compile as they are with `&mut self`;
the mirror's writers become `seed` or `fork` (root_routes.rs:979, 1010,
1297, 1310, 2398-2470, 2495, 4155; loads.rs:54; roles.rs:191); the
system world submits the root's writes to the driver as they are
(composed.rs:480-492, no `jig_fake_store::Write`), reads
`self.store.pending` (composed.rs:385-388), and holds a commit with
`store::Fault::Hold { commits: 1 }` (tests/composed.rs:129).
`jig-fake-store` leaves `tests/engine/domain/Cargo.toml` and
`tests/system/Cargo.toml`; `jig-test-store-files` joins the workspace's
`[workspace.dependencies]` (`{ git = "https://github.com/forgetic/jig.git" }`)
and `tests/engine/domain`'s, with `jig-protocol-store`, `skein-kv`,
`temper-protocol-store` and `temper-service`; then, in this order, the
three commands of code-organization.md, section 3 and its check.
Needs: 04.5's `temper_protocol_store::application`; 08.2's store routing
and the component's limits it derives.
Files: `tests/engine/domain/{Cargo.toml, src/store.rs, src/direct.rs,
src/roles.rs, tests/root_routes.rs (at most 40 lines added),
tests/loads.rs}`; `tests/system/{Cargo.toml, src/composed.rs,
tests/composed.rs}`; the workspace `Cargo.toml` and `Cargo.lock`; and,
compile-only, `tests/engine/domain/src/{walking.rs, escalation.rs}` (no
line added) and `tests/engine/domain/tests/forge_routes.rs` (its
`store.page` and `store.apply` calls, :400-410, 500-520). Shared
hotspots: `root_routes.rs`, the workspace `Cargo.toml`, `Cargo.lock`.
Shape: the sketch below (ordinary Rust, not compiled).
Test shape: no new test; the engine's domain world's and the system
world's tests pass unchanged in name and expected value, but those
README's "fixture that gains what a real component requires" allows
(a test that copied or edited the mirror seeds or forks instead). One
new step-level check in `tests/engine/domain/tests/loads.rs`:
`a_forge_row_survives_the_real_store_byte_for_byte`, written out in
full:

```rust
#[test]
fn a_forge_row_survives_the_real_store_byte_for_byte() {
    let (driver, _) = Driver::chat();
    let mut store = driver.store.fork();
    assert_eq!(store.page(&Range::Forge, None, 8).0.len(), 0, "a chat's store holds no forge row");
    let row = Record::Forge { row: Box::new(temper_engine_domain_forge::Stored::Landed { commit: [7; 32], task: 3 }) };
    store.seed(Box::new([row.clone()]), Box::new([]));
    let (rows, next) = store.page(&Range::Forge, None, 8);
    assert_eq!((rows.as_ref(), next), ([row].as_slice(), None));
}
```

Proof: world, `cargo nextest run -p temper-engine-domain-world -p
temper-system-world`, run once: every story of both worlds passes with
jig's store component holding the root's commits, temper's forge rows
encoded by `temper-protocol-store` and read back through its interval;
under 20 s. The suites' budgets are the gate's (the landing's
`Gate-Result`, focused at most 15 s).
Don't: copy 08.2's routing or 04's codecs into the world; answer a load
from the mirror; add a lint exception to a world; move a pin (the check
prints the three pinned revisions).
Size: world code; production 0.
Removes: `jig-fake-store` from `tests/engine/domain` and `tests/system`.

### 18.2 The conformance world on the driver; jig's fake store leaves

Scope: store/README.md, 6. `tests/conformance` runs on 18.1's driver:
`jig_conformance::Durable` implemented for it in
`tests/engine/domain/src/store.rs` (as jig's `durable.rs`, 29 lines;
`type Write = engine::Write`), the conformance world's `type Store` the
driver and `type Write` the root's (`lib.rs:154-155`), `Output::Commit`
carrying the root's writes as they are (`lib.rs:344-370`), a load paged
by `store.page` (`lib.rs:440-458`, where it pages the fake's rows by
hand), `observations::Store` the driver and `snapshot` reading its
mirror (`observations.rs:140-160`). `jig-fake-store` leaves
`tests/conformance/Cargo.toml` and the workspace's
`[workspace.dependencies]` (Cargo.toml:60).
Needs: 18.1's driver.
Files: `tests/conformance/{Cargo.toml, src/lib.rs, src/observations.rs,
src/systems.rs (its `observations::key(store, ..)` reads, :113-135, if
the alias's type changes them)}`; `tests/engine/domain/{Cargo.toml
(jig-conformance), src/store.rs}`; the workspace `Cargo.toml`.
Shape: `impl jig_conformance::Durable for Store` (ordinary Rust, after
jig's `testing/jig-test-store-files/src/durable.rs`).
Test shape: no new test; the conformance world's tests pass unchanged.
Proof: world, `cargo nextest run -p temper-conformance-world`, run
once: jig's conformance scenarios pass against temper's root with jig's
store component under jig's harness, crashes included; under 20 s.
Don't: keep `observations::Record`'s wrapper if nothing but the fake's
rows used it; page by hand.
Size: world code; production 0.
Removes: `jig-fake-store` from the workspace (`git grep -n jig-fake-store
-- '*.toml'` prints nothing).

### 18.3 The composed system world on the service's translations

Scope: channel.md, section 7; llm.md, section 5 (the charter).
`tests/system/src/composed/link.rs` keeps no translation of temper's
parts or the charter of its own: `assignment` builds the hub's
assignment as it does, but its workspace from 08's engine `items`
(the root's `Assignment` and the world's forge's `git` base to the
`Items` record) handed to 09.1's worker `workspace` (the record to
`wire::Workspace` and `save`), the record passed without its bytes;
`parts` is 09.1's worker `work` and `preparation` (the worker's
`wire::Work` and `wire::Preparation` to `Left`, `Saved` and
`Preparation` records) handed to 08's engine `parts` (the records to
`engine::Parts`); `charter` is `temper_engine_smith::charter` (08's, the
resume rule in it) encoded as today. The world's git base is a constant
of its own (`b"http://forge.test"`); the worker's checkout face maps a
remote `http://forge.test/org/repo.git` (channel.md, 4's form) to the
fake forge's `org/repo` (`tests/system/src/forge.rs`'s git side, one
function). `link.rs`'s own `repository()` (:229 today), `parts`'s and
`charter`'s bodies go, and its unit tests (01.3's) go with them: the
translations' own step tests are 08's and 09.1's. `decode`, `render` and
`end` stay (the routing's decode and render are
`temper_engine_smith`'s, which they already call).
Needs: 08's engine translations (`items`, `parts`) and
`temper_engine_smith::charter`; 09.1's worker translations (`workspace`,
`work`, `preparation`); 03.1's records.
Files: `tests/system/{Cargo.toml (temper-service,
temper-protocol-worker-wire), src/composed/link.rs, src/composed.rs (the
two call sites, :582-584 and :679 as 01.3 leaves them), src/forge.rs
(the remote's mapping)}`.
Shape: `link::assignment(&engine::Assignment) -> (host::Assignment,
wire::Workspace, Option<Box<[u8]>>)` (the worker's `save` beside, from
`workspace`); `link::parts(wire::Work, Option<wire::Preparation>) ->
engine::Parts`, each a call of the service's functions, as 08 and 09.1
land their names and signatures.
Test shape: no new test; the system world's stories pass unchanged
(`tests/system/tests/*.rs`), a run's pushed heads and saved work reaching
the engine through the service's translations. The removed unit tests
are named in the commit body.
Proof: world, `cargo nextest run -p temper-system-world`, run once:
the composed stories (a chat answered, a change pushed and landed, a run
resumed from its transcript) pass with the engine and the worker joined
by the service's translations and charter; under 20 s.
Don't: keep a translation, a tag table or a resume rule in the world;
encode a record to bytes between the two calls.
Size: world code; production 0.
Removes: the bodies of `link.rs`'s own `assignment`'s workspace, `parts`
and `charter` (each function stays as calls of the service's or
`temper_engine_smith`'s), its `repository` (:229 today) and `saved`
(:93-102), no caller left, and its unit tests.

## Interface sketches

None compiled (`scratch/plan-sketches/src/s18.rs` holds a doc line only):
this session writes world code. The driver, ordinary Rust, not compiled,
in `tests/engine/domain/src/store.rs`:

```rust
/// Faults between the root and the component (jig's store.rs:11-17).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Hold the next `commits` durable acknowledgements until released.
    Hold { commits: u32 },
    /// Fail commit `commit`: the root stops, as a failed commit does.
    Fail { commit: u64 },
    /// Hold each page `by` ticks.
    SlowPages { by: u32 },
}

/// jig's store component over skein-kv's memory mode, temper's families
/// routed by the service's store routing, with a typed mirror.
pub struct Store {
    /// The last commit the component made durable.
    pub applied: u64,
    /// The durable rows as the root wrote them, updated on `Committed`.
    pub rows: BTreeMap<Key, Record>,
    /// Commits the root made that the world has not handed on yet.
    pub pending: VecDeque<(u64, Box<[Write]>)>,
    // the component, `Files`, the routing, the limits, the faults, the
    // durable commits for `fork`, the held acknowledgements.
}

impl Store {
    /// Sized for `WIDEST`, every world's limits within it.
    pub fn new() -> Store;
    pub fn with_limits(limits: &engine::Limits) -> Store;
    /// The front pending commit through the component; its number.
    pub fn apply(&mut self) -> u64;
    pub fn submit(&mut self, number: u64, writes: Box<[Write]>, delay: u32);
    /// # Errors
    /// The number of a commit the fault failed.
    pub fn tick(&mut self, hold: bool) -> Result<Option<u64>, u64>;
    pub fn release_held(&mut self) -> Option<u64>;
    pub fn fault(&mut self, fault: Fault);
    /// The stores and files as a crash leaves them; the component opened again.
    pub fn crash(&mut self, seed: u64);
    /// A fresh store with the durable commits replayed in their numbers.
    pub fn fork(&self) -> Store;
    /// One numbered transaction after the last applied: `saves`, `erases`,
    /// and the header with that number.
    pub fn seed(&mut self, saves: Box<[Record]>, erases: Box<[Key]>);
    /// One load of `range` after `after`, at most `most` rows, through the
    /// component: its rows and the next page's cursor.
    pub fn page(&mut self, range: &Range, after: Option<&Key>, most: u32) -> (Box<[Record]>, Option<Key>);
    pub fn header(&self) -> Deployment;
    /// No commit pending and no load in flight.
    pub fn idle(&self) -> bool;
}
```

## Questions to raise when reached

- **A world past `WIDEST`'s sizes** (a record its limits allow that the
  component sized for `WIDEST` refuses): default, that world calls
  `Store::with_limits(&its limits)`; a test whose rows no store can hold
  is a question with its file and line.
- **A test that reads the header's `commits` after a seed** (the seed's
  number differs from the copied header's): default, it forks instead;
  if it must seed, a question.
- **08.2's routing or limits not public** to a world crate: default,
  `temper-service` exports them (`pub use`), named in the commit body.
- **A world's suite past its time** with the real store (a focused test
  past 0.5 s): profiled with `--profile measure` and made cheaper (fewer
  pages, a smaller `WIDEST`); past twice, a question.

## Done when

- The worlds pass on the real store: `cargo nextest run -p
  temper-engine-domain-world -p temper-system-world -p
  temper-conformance-world` passes.
- jig's fake store is gone: `git grep -n jig-fake-store -- '*.toml'`
  prints nothing (README, "Done when" 7).
- The system world keeps no translation of its own: `grep -nE 'fn
  (repository|saved)\b' tests/system/src/composed/link.rs` prints nothing.
- The pins held: `grep -oE 'forgetic/(jig|smith|skein)\.git#[0-9a-f]+'
  Cargo.lock | sort -u` prints exactly the three pinned revisions.
- The gate passed on this session's last landing within the budgets:
  `git -C ~/src/rust/temper log --format='%(trailers:key=Gate-Result,valueonly)'
  --grep='^Step: temper-18\.' origin/main | head -1` names
  `scripts/check.sh passed`, a focused time at most 15 s and a fuzzy time
  at most 60 s.
