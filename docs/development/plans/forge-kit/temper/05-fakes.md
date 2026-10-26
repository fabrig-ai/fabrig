# Session 05: temper on skein's fakes and forge peer

Put temper's users of the fake forge on skein's crates and delete temper's
three copies; then run the one-process world on skein's forge peer, so that
the engine's webhooks are delivered in a world that checks them. Two
increments: the repin, the users and the deletions (05.1); the peer in the
one-process world, with temper-fake-git's forge handle swapped (05.2).
Needs at start: **skein 05.2** on skein's `origin/main`
(`skein-fake-forge-domain`, 100 tests; `skein-fake-forge-protocol`, 56).
Increment 05.2 also needs **skein 05.5** (`skein_fake_peers::forge`, with its
delivery), after skein 05.4, which needs temper 05.1 (decision 26). These are
the README table's; the code shows no other. Design built: skein `fake-forge.md` 2, 4 and 5; temper
`docs/design/domain/testing.md` 2.4 and 4.2.

## Read first

- skein `docs/design/fake-forge.md` 2, 4 and 5; temper `docs/design/domain/testing.md`
  2.4, 2.5 and 4.2.
- This plan's README (sessions; Rules: the repin and the tools plan's
  hotspots; Done when 4 and 6), `code-organization.md` (the fakes' rows),
  `decisions.md` 15 and 26, and Design questions 32 and 33.
- `skein/05-fakes.md` (05.1 and 05.2: copies "field for field"; 05.4, 05.5: the peer,
  "temper's side", sketch A); findings `base-temper-05.md`, `survey-05-fakes.md`,
  `trace-fakes.md`, `base-runs.md`. development.md 5.7.

## What the design asks

- fake-forge.md 5: the peer hosts the protocol layer over a forge its owner
  hands it; `address()` is known after io's listening event; it delivers a
  hooked repository's hooks to the endpoint's `host`, which an owner whose
  listener binds port 0 sets once that listener listens, "not at its bind";
  delivery is best effort, and "A world asserts a delivery written only for
  a hook due inside its listener's life" (skein `1b33f85`). Its frozen
  records are two per connection, two per call and one per delivery, never
  drained or dropped: "a world's caps are a stated room for a whole story,
  each named for what it holds, with its reason, and a story that outgrows
  it fails by name rather than losing its history" (skein `3df1162`).
- domain/testing.md 4.2: the world
  hands the peer the webhooks' listener's address once it listens; the
  setup's hooks are lost; the adopted repository's story is the hooked
  one, its setup's push lost and the agent's push written, since "the story
  ends only once the peer holds an outcome for each of the story's own two
  hooks, one per push it makes".
- domain/testing.md 2.4 and 4.2: the one-process world and the real loop both host
  the one peer; the real loop's listeners are on kernel-assigned ports passed
  to their clients once bound, and a story never uses a fixed port.
- Decision 15: the one-process world hands the peer's `address()` to the engine.

## Choices this plan makes

- 05.1 hands no value. Skein's copies keep the production API field for field
  (only their tests were renamed), so every user changes a path.
- The sim cannot hand a port-0 `address()` to the engine: the engine's
  configuration is built when `World::admit` calls `make` at spawn, before
  the peer has run; a restarted peer must rebind the address the engine
  holds; and `table().base` renders `http://127.0.0.1:3000`. So the world
  gives peer and engine one constant, `fixtures::FORGE` (127.0.0.1:3000, the
  base's value, now written once), and the referee asserts that `address()`
  is `Some(FORGE)` once the peer listens. The kernel-assigned hand-over is the
  real loop's (domain/testing.md 4.2), as decision 15 says.
- Port-0 binds work in the sim, so the engine's webhooks listener binds
  `127.0.0.1:0`. `ServiceHost` records the port-0 bind's address and names
  it once that socket's `Listen` completes; the referee's `act` (between
  passes) sets `Forge.hooks` when it first appears and when a restart
  changes it. Until then a hook gets `NoAddress`, dropped and observed: the
  Adopt story's setup push, released at `Time(0)` before any host runs, is
  one. The Adopt story ends once its push is seen and the live peer holds a
  `Delivery` for each of the story's own two hooks, so the agent's hook,
  scheduled at its push and released after it, is written before
  `Terminate`. The count is the story's, never the domain's tally, which
  counts a hook only at its release (skein's `fake-forge.md`, 5). No
  barrier holds a hook, the setup or the engine's listener back, and no
  skein API is added.
- The peer's observation room is the world's, a stated room (05.2,
  "Observation room"); skein's peer fixture's 16 sizes skein's own tests
  only.
- Only the Adopt story's repository is `hooked`. The document carries the
  webhooks section for every story (one document); an unhooked story fires none.
- `forge.rs` is deleted whole (temper 07's Needs say so). Its thread-local
  handle, which `spawn_restartable`'s fn pointer forces, moves to `world.rs`
  with the `peer()` factory. The one-process world owns a copy of the 20
  `fake_limits()` values (3 top-level, 17 http and documents), so it names `temper-engine-protocol-forge-world` no more.

## Rules that bind it

- The repin is each increment's first commit: `cargo update -p skein-lib`,
  then `cargo check --workspace --all-targets`, repairing what skein's
  steps since the last repin broke. A `Cargo.lock` conflict takes
  `origin/main`'s and runs the update again. At the base the check fails on
  `Op::Lock` in `temper-worker-protocol-git` and on jig's pin (findings); the
  tools plan's first repin repairs both before temper 01.1, hence before this
  session.
- No re-export, no alias: every user names `skein_fake_forge_domain`,
  `skein_fake_forge_protocol` or `skein_fake_peers::forge`. No test moves in
  this session; the one-process tests keep their names, assertions and times.
- Both increments are hotspots (README Rules). 05.1 ends the fake crates'
  wait. 05.2 ends the forge host's, and a change to `ForgeHost` or to the
  fake-git handle waits from skein 05.4's start to its landing.
- A Size counts `git diff --numstat origin/main --. ':!Cargo.lock'`,
  first column summed. Paths and counts are at the base (temper dd12b8e, skein
  064c55f); copies come from `origin/main` at the step's start.
- The process tier runs once, in 05.2.

## Increments

### 05.1 The repin, the users, the three deletions
Scope: fake-forge.md 2 and 4; the end of the fake crates' hotspot.
Needs: skein 05.2.
Files, at the base:
  - Root `Cargo.toml`: adds `skein-fake-forge-domain` and `-protocol` as git
    entries beside `skein-lib`; removes the members
    `testing/temper-fake-forge-domain`, `testing/temper-fake-forge-protocol`,
    `tests/fake-forge` and the two path entries. `Cargo.lock`: the repin.
  - 13 dependency lines in 10 manifests: conformance 1, engine/domain 1,
    engine/forge 1, engine/forge/client 1, engine/protocol-forge 2,
    forge-setup/reconcile 1, one-process 2, system 1, worker/checkout 1,
    `testing/temper-fake-git` 2.
  - 35 Rust files, 122 lines name the crates outside them; files/lines by
    package: temper-fake-git 2/7, conformance 2/3, engine/domain 3/3,
    engine/forge and client 9/30, engine/protocol-forge 7/22,
    forge-setup/reconcile 5/8, one-process 3/23, system 2/3, worker/checkout 2/23.
    `crates/`, `tools/`, jig, smith and hearth have none; a user the tools plan
    has added by then (a tool) is renamed too (Done when 3).
  - Deleted: `testing/temper-fake-forge-domain`,
    `testing/temper-fake-forge-protocol`, `tests/fake-forge`.
Shape: a rename, in three commits: the repin; the users (`temper_fake_forge_domain`
  to `skein_fake_forge_domain`, `_protocol` likewise, in each `use` and path,
  and the 13 manifest lines); the deletions. `cargo fmt` moves a renamed `use`
  among its neighbours. `tests/one-process/src/forge.rs` imports both
  crates `as domain` and `as protocol` (the only `as` among the users): it keeps
  those local names, and 05.2 deletes it. No value is handed, no type
  aliased, no fake's source touched.
Test shape: no new test, none moved. The 154 tests of the deleted crates (96
  domain, 55 protocol, 3 memory; none fuzzy) leave temper; skein runs them (100
  and 56 there, skein/05-fakes.md). Times at the base: the three crates 0.118 s.
Proof: joint; `cargo nextest run -p temper-fake-git`: its count at the
  step's start (14 at the base: check 6, transcripts 8; 0.021 s).
Proof: joint; `cargo nextest run -p temper-conformance-world
  -p temper-engine-domain-world -p temper-engine-forge-world
  -p temper-engine-forge-client-world -p temper-engine-protocol-forge-world
  -p temper-forge-setup-reconcile-world -p temper-system-world
  -p temper-worker-checkout-world -p temper-one-process-world
  -p temper-fake-git`: the set's count at the step's start, unchanged (452 at
  the base, 3.603 s), no package slower than at the step's start.
Proof: joint; the same packages with `--profile fuzzy`: their count at the step's
  start, unchanged (39 at the base, 27.1 s while the queue was loaded; 15.5 s with the three fake crates
  among 13 packages), under the 60 s fuzzy ceiling.
Proof: joint; `scripts/check.sh`: origin/main's focused count at the
  step's start less 154 (1725 from the base's 1879) and its fuzzy count unchanged
  (50 at the base), clean clippy, focused <15 s (6.8 s at the
  base), fuzzy <60 s (23.9 s).
Don't: hand or rename a value; alias or re-export a crate; edit a test's name
  or body; touch `crates/`, the engine or the worker; move the host or the handle.
Size: about 0 lines moved, 140 new (estimated: 122 Rust lines, 13 manifest
  lines, 2 root entries, lines `cargo fmt` reorders).
Removes: the three crates, 14,086 Rust lines (8,320 + 5,309 + 457; manifests aside)
  and 154 tests; 3 workspace members; 2 path entries.

### 05.2 The one-process world on the forge peer
Scope: fake-forge.md 5; domain/testing.md 2.4, 2.5 and 4.2; decisions 15 and 26, answer 32.
Needs: 05.1; skein 05.5. For the shared compatibility block, jig01.s2 and temper02.1 must land before this repin resumes; rebase their settled adapters, preserving the independent peer checkpoint and every proof.
Files, at the base:
  - Repin first (commit 1). Follow README's compatible smith source rule if
    the new skein interface breaks pinned smith; retain all existing checks.
  - `testing/temper-fake-git/{Cargo.toml, src/lib.rs, src/transport.rs,
    src/fixtures.rs}`; `tests/one-process/{Cargo.toml, src/lib.rs, src/world.rs,
    src/fixtures.rs, src/stories.rs, src/referee.rs, src/referee/forge.rs,
    tests/one_process.rs}`.
  - Deleted: `tests/one-process/src/forge.rs` (268 lines at the base:
    `ForgeHost`, `Connection`, the handle).
Shape: temper-fake-git's `Forge` (struct and `new`, 18 lines) is deleted;
  `lib.rs` exports `Transport` only; `Transport.forge` and `Transport::new`
  take `Rc<RefCell<skein_fake_peers::forge::Forge>>`; `call` takes its token
  from `forge.next_reply()` in place of `serial` and `(7 << 60) + serial`, and
  reserves room in the pending Queue before stepping as skein 05.4's tiny git
  caller does (copy its form). `testing/temper-fake-git/src/fixtures.rs::named`
  contains one `Forge::new`; it and `tests/one-process/src/stories.rs::forge`
  plus `referee/forge.rs` import the new type. `stories.rs`'
  `prepare_merge_branch` steps the domain itself and pushes into `forge.pending`
  at two sites, so it takes the same reservation and `next_reply()` for its
  hand-made `Token::new(1 << 60)`; `Cargo.toml` adds
  `skein-fake-peers.workspace = true`. Commit 2 does this and adapts the old
  host's pending drain; commit 3 replaces the host.
  One-process: rename the existing LLM `world.rs::peer` factory to `llm_peer`
  and update `run_phase`'s `spawn_restartable` call before adding sketch B's forge
  `peer()`. `world.rs` takes the handle. `Proc::Forge` holds
  `Box<SignalHost<skein_fake_peers::forge::Peer>>`; `fake_forge` and the two
  `world.spawn` closures call `peer()`; the `forge::set` calls become `share`.
  `fixtures.rs`: `FORGE` replaces the three literals of `document()` and
  `forges()` (as `FORGE.to_string()` and `format!("http://{FORGE}")`, as the
  document already does for `CLIENTS`; the four data strings of `stories.rs`
  and `referee.rs` stay); the document gains `application.webhooks.listener.socket`
  `127.0.0.1:0` and the forge's `webhook_secret` `secrets/forge-1.webhook-secret`,
  whose file `files()` adds (the list's capacity 9 becomes 10, which the
  scratch run found); `HOOK_SECRET`, `peer_limits()` and `forge_limits()` are
  new. `stories.rs`: `hooked: matches!(story, Story::Adopt)`.
  `ServiceHost.hooks: Option<SocketAddr>`, set in `iterate` by `hooks_address`
  (sketch A): a port-0 `Op::Bind`'s `Done::Bound` address, kept with its fd
  and named when that fd's `Op::Listen` completes, never at the bind; it is
  read by the referee's `act`, which calls `referee::forge::Forge::hand_hooks`
  when it changes. The referee asserts each peer's `address()` is `None` or
  `Some(FORGE)`, and gathers the live peer's `Delivery` outcomes into
  `Outcome.deliveries`. `referee.rs`'s completion of `Story::Adopt` becomes
  `forge.complete()` and the live peer's `Delivery` records numbering
  `stories.rs`' `ADOPT_HOOKS` (2: one per push the story makes to its hooked
  repository, the setup's topic push and the agent's), read between passes:
  `referee/forge.rs::hooks_complete` takes that count, never
  `forge.domain.tally().hooks`. At the end the referee asserts
  `forge.domain.tally().hooks` equals `ADOPT_HOOKS`. No other story's
  completion changes.
  Observation room: `peer_limits()`' `observations` and `observation_bytes`
  are `fixtures.rs`' `STORY_OBSERVATIONS` (1,024) and
  `STORY_OBSERVATION_BYTES` (2,097,152), "a whole story's frozen peer
  records", their comment giving the reason: a room for the largest story,
  about 16 times the most a story has recorded (Adopt's 65), at skein's peer
  fixture's 2 KiB a record. They are a room, not a derived bound: a story
  that outgrows them fails by the peer's own assertion, `frozen record
  count fits` or `frozen payload admitted before copying`, and returns as
  a finding naming the story. Its derivation is later.md's. `Peer::new`
  refusing the sum also returns.
Test shape: three new units in `world.rs`' tests, no world, each <20 ms:
  `a_port_zero_listener_names_the_hooks_address_once_it_listens` passes
  `hooks_address` a `Complete` for a bind of `127.0.0.1:0` done as
  `Bound(127.0.0.1:41000)`, expecting `None`, then that fd's `Listen` done,
  expecting `Some` of that address; `a_fixed_port_bind_is_not_a_hooks_address`
  uses fixed port 7102, bound and listening, and expects `None`;
  `a_failed_bind_is_not_a_hooks_address` uses a port-0
  bind failed `AddressInUse` and expects `None`. Follow testing-strategy.md 10,
  One behaviour per test. One changed test,
  `an_adopted_repository_gets_its_change_pushed_through_git`
  (`tests/one_process.rs`, `world::run(fixtures::calm(SEED), Story::Adopt)`),
  adds `assert_eq!(outcome.deliveries, [Delivery::NoAddress, Delivery::Written])`:
  the 2 hooks the Adopt story's domain fires (counted in a scratch with
  `hooked: true`, findings): the setup's topic push, released at `Time(0)`
  before any host runs, which has no address, and the agent's push, which
  ends the story, written whole to the listening engine; none resent. The
  adoption's own assertions pass with the first lost. A second outcome of
  `Refused` or `Reset` after this change is a finding: return it with its
  io failure, never an edit. Other stories are unhooked: no delivery. Fault and crash phases
  may see `Refused` or `Reset` and are not asserted. The other 8 tests keep
  names and assertions; the referee's address rule covers them all. The
  engine's reaction (a hint refreshes, a read that fires no hook) is unmeasured
  until the peer exists: a count other than `deliveries` that moves, or a
  delivery count (or `tally().hooks`) other than 2, or an Adopt that meets
  its existing deadline still short of 2 records, goes to the supervisor,
  not into an edit.
Proof: unit; `cargo nextest run -p temper-one-process-world -E
  'test(a_port_zero_listener_names_the_hooks_address_once_it_listens) | test(a_fixed_port_bind_is_not_a_hooks_address) | test(a_failed_bind_is_not_a_hooks_address)'`: 3 passed, each <20 ms.
Proof: joint; `cargo nextest run -p temper-one-process-world`: its count at
  the step's start plus 3 (12 from the base's 9, 0.953 s; the scratch with webhooks configured ran them in
  1.96 s serially, the Adopt story in 1.08 s). The Adopt story alone took
  0.953 s at the base, over the 0.5 s focused ceiling: its time and cause are
  reported, never a raised budget or a new world; the added check is an assertion.
Proof: joint; `cargo nextest run -p temper-one-process-world --profile
  fuzzy`: its count at the step's start, unchanged (4 at the base, 2.10 to 2.14 s, 2.18 to 2.28 s in the scratch over
  two rounds), under about 5 s.
Proof: joint; `cargo nextest run -p temper-fake-git`: its count at the step's start (14, 0.021 s, at the base).
Proof: joint; `scripts/check.sh`: origin/main's focused count at the
  step's start plus 3 and its fuzzy count unchanged, clean clippy, focused <15 s, fuzzy <60 s.
Proof: process; `TEMPER_FORGEJO=$WS/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64
  TEMPER_FORGEJO_RUNNER=$WS/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64
  scripts/proof-one-process.sh`, once: its count at the step's start passed (19, 145.440 s
  at the base). That script runs the real-binary tier; the one-process stories are the
  joint lines above.
Don't: add a story or a world; edit a story's name or an assertion beyond the
  two above; resend a delivery; hold a hook back, move the setup's push, or
  wait in the engine or the peer for the listener; name the hooks' address
  at a bind; drop, drain or clear the peer's history, or catch its
  overflow; raise the room past an overflow rather than return it; change
  skein's peer or its fixture; keep `Forge`
  or `ForgeHost` anywhere; touch the engine's webhooks listener (temper 01.3,
  03.1); name `temper` in skein's peer.
Size: about 12 lines moved (the handle), 245 new (estimated: `fixtures.rs` 95,
  with the observation room about 6; `world.rs` 80 plus the three unit
  fixtures, the learner's listen about 5; `referee/forge.rs` and `referee.rs` 35,
  Adopt's end about 5; `transport.rs` 15, `stories.rs` and `one_process.rs` 14,
  manifests 6).
Removes: `forge.rs` (about 256 lines of the host); the fake-git `Forge` (18 lines); the one-process
  dependency on `temper-engine-protocol-forge-world`.

## Interface sketches

A: `world.rs`, the delivery learner (compiled in the scratch, clippy clean).
```rust
// The port-0 bind's address, named once that socket listens (amended by
// the startup-hooks answer, not compiled; `Fd` is the kernel's fd type).
fn hooks_address(bound: &mut Option<(Fd, SocketAddr)>, complete: &Complete) -> Option<SocketAddr> {
    use skein_io::kernel::{Done, Op};
    match (&complete.kind, &complete.result) {
        (Op::Bind { fd, addr: asked, .. }, Ok(Done::Bound(addr))) if asked.port() == 0 => {
            *bound = Some((*fd, *addr));
            None
        }
        (Op::Listen { fd, .. }, Ok(_)) if bound.is_some_and(|(at, _)| at == *fd) => bound.take().map(|(_, addr)| addr),
        _ => None,
    }
}
// referee::forge::Forge, between passes
pub(crate) fn hand_hooks(&mut self, bound: SocketAddr) {
    self.transport.forge.borrow_mut().hooks = Some(Endpoint { forge: 1,
        host: bound.to_string().into_bytes().into(),
        secret: Secret::new(fixtures::HOOK_SECRET.into()) });
}
```
B: `world.rs`, the factory (not compiled at the base: skein's base has no peer).
```rust
thread_local! { static SHARED: RefCell<Option<Transport>> = const { RefCell::new(None) }; }
fn share(transport: Option<Transport>);   // was forge::set
fn transport() -> Transport;              // was forge::transport
fn peer() -> Peer {
    Peer::new(fixtures::FORGE, fixtures::peer_limits(),
        temper_fake_git::fixtures::table(b"acme/app", None),
        fixtures::forge_limits(), transport().forge, fixtures::SEED)
        .expect("forge peer")
}
```
`peer_limits()` gives `io` every field of the host's `Listen` in `forge.rs` (sockets
16, refusals 8, intake 65,536, receive 1024, output 65,536, sends 8, accepts 4,
backlog 8, names 0, close_timeout 1 s, retry 1 ms), `connections`
4, `queue` 64, `observations` and `observation_bytes` the world's own
(05.2, "Observation room"), the rest from skein's `tests/forge/fixtures.rs::limits` at its
landed revision; `Peer::new` checks the sum. `forge_limits()` copies
`fake_limits()`: calls 4, connections 4, raw_bytes 4096, the http and documents
values of `tests/engine/protocol-forge/src/fixtures.rs`.

## Done when

- `cargo check --workspace --all-targets` exits 0 after each increment.
- `ls -d testing/temper-fake-forge-domain testing/temper-fake-forge-protocol
  tests/fake-forge` prints "No such file" three times (all exist at the base).
- `rg -n 'temper[-_]fake[-_]forge' Cargo.toml crates testing tests tools
  -g '*.rs' -g Cargo.toml` prints nothing (174 matches in 63 files at the base,
  counted with `--count-matches`; the three crates are among them).
- `rg -n 'temper_fake_git::Forge|ForgeHost|pub use transport::\{Forge'
  tests testing crates -g '*.rs'` prints nothing (11 lines at the base).
- `rg -n '3000' tests/one-process/src/fixtures.rs` prints one line, the
  definition of `FORGE` (3 lines at the base: the document's two strings and
  `forges()`' address).
- `rg -n 'skein_fake_peers::forge' testing tests -g '*.rs'` prints the
  users, among them temper-fake-git's `transport.rs` and `fixtures.rs`, and
  one-process's `world.rs`, `referee/forge.rs`, `stories.rs`, `tests/one_process.rs`.
- The Proof lines of both increments print their counts; the process tier ran
  once, in 05.2.
