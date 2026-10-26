# Session 13: the forge in the real loop, and a landing

skein's forge peer joins `tests/real-loop` beside the fake LLM peer and
shares its forge with the fake git program (decisions.md, 1). The engine's
webhooks reach it at the address the world learns from io (decisions.md,
2). On that world: `adopt` through `temper ctl`, and a chat that lands its
change, CI by the content cue (testing.md, 2.4; tools.md, 11.3). It
extends temper 08's world and fixtures, never redefining them; stories
without the forge keep 08's unserved forge (decisions.md, 8). No temper
forge host: draft 07 is dropped. Each story is smoke, one path per
feature; its corner cases are tested in the lower world that owns them
(testing.md, 2.4; ruling 130).

Needs at start: temper 08; forge-kit's skein-05.5 (the forge peer's
delivery; landed, inside the start pin) and forge-kit's temper-05.2 (fake git's `Transport` on the
peer's shared `Forge`; the one-process world's port-0 hooks address).
13.4 needs temper 05's acts, 13.5 its waits and `task --tree` read.
Names said to be forge-kit's temper-05.2's are created there: check each
as landed. Steps: `temper-13.1` to `temper-13.5`, then `temper-13.s2`
(`13.s2-git-test-shapes.md`, ruling 142); `temper-13.s3`
(`13.s3-wake-across-fresh-chain.md`, ruling 145), then `temper-13.s4`
(`13.s4-overtaken-chain-sends-nothing.md`, ruling 148), before 13.5 goes
on.

## Read first

- temper's `testing.md` 2.4, 4.3, 6.1, 10; `tools.md` 5.3, 6.3, 11.3;
  `client/README.md` 4.4, 4.7; `protocol/forge.md` 5; `domain/forge.md`
  8.2. skein's `fake-forge.md` 2, 3, 5; `testing-strategy.md` 2.8, 6.
- This plan's `decisions.md` (design question 1; 1, 2, 7, 8),
  `temper/08-real-loop.md`, `temper/10-process.md` (10.2's stale text);
  forge-kit's `temper/05-fakes.md` (05.2, sketches A and B).
- Code: `skein_fake_peers::forge::{Peer, Forge, Observation, Delivery}`;
  the fake forge domain's `repository`, `grant`, `advance`, `api::{Setup,
  Checks, Cue, Protection}`; skein-world's `World::new_controlled` and
  `Heap::release`. temper's `testing/temper-fake-git/tests/transcripts.rs`
  (`Disk`, the `hosted_*` cases) and `src/fixtures.rs`;
  `tests/one-process/src/world.rs` (`git`), `referee/forge.rs` (`green`);
  `crates/temper/tests/support/forgejo.rs` (`landing_scripts`,
  `configuration`, `admit`); skein's `skein-forge-client/src/keep/poll.rs`
  (`hint`, the hint's alarm); `temper-service/src/limits/names.rs`.

## What the design asks

- **testing.md 2.4 and tools.md 11.3:** stories that clone, push or land
  run on the fake git program over the peer's forge; CI follows the
  content cue; the peer delivers a hooked repository's webhooks to the
  engine's listener, polling the backstop "at paces the stories'
  configuration sets short (`poll`, `poll_max` and `slow`)"; listeners on
  ports the kernel assigns. 13.2, 13.5.
- **testing.md 4.3:** fake git runs on the files it is given; here, the
  real filesystem's, measured first. 13.1.
- **tools.md 6.3 and client/README 4.7:** adopted exits 0, in 4.7's
  words. 13.4. Not found, exit 2, is the tools' and `tests/client`'s
  (13.4; ruling 130).
- **domain/forge.md 8.2:** a merge lands the change and closes its task.
  13.5. **testing.md 6.1:** a real-loop story aims at 1 s.

## Choices this plan makes

- **One forge, built in the peer's spawn closure** (`Domain::new`,
  `repository`, `grant`, `Forge::new`); git's transport goes in a
  thread-local, as one-process's does. The referee borrows the handle
  between passes (fake-forge.md, 5).
- **The shared forge runs unchecked** (decisions.md, design question
  4): hosted git grows the shared domain in its own turn, so a checked
  world fails git's exact release at exit. The forge stories' binaries
  (13.1 on) use `World::new_controlled` and their commit bodies say so;
  08's and 09's stay checked.
- **The hooks address is the world's:** `ServiceHost` keeps each port-0
  `Done::Bound`; hooks is the one `Service::links()` does not print. A
  function adapted from forge-kit's `hooks_address`, named in the commit body; the
  engine gets no accessor.
- **The peer's limits are copied** from forge-kit's temper-05.2
  `fixtures::{peer_limits, forge_limits}` (08 depends on nothing of
  `tests/one-process`; skein ships no builder for them).
- **The landing's scripts and configuration move to the world** (the
  least moved): their bodies leave `support/forgejo.rs`, which keeps its
  signatures and calls `landing`.
- **One landing:** the process tier lands by deliveries already
  (`a_change_lands_by_deliveries_with_every_poll_raised`), so draft 13's
  webhooks-only story goes; late or lost hooks are later.md's. The
  not-found adopt joins: one call on 13.4's world.

## Rules that bind it

- **No edit** to skein, `tests/one-process` or `testing/temper-fake-git`
  (hotspots), but temper-13.s1's (ruling 112): a need there is a
  question. **No sleeps:** a story ends on
  a cue, an `Until` or a wait, within 08's `PATIENCE`.
- **Paces from the builder:** `application.limits.overrides`, applied by
  `temper_service::limits::{profile, apply}` and admitted as `admit`
  does, each beside its shipped value.

## Increments

### 13.1 Fake git on the real filesystem, measured

Scope: the four hosted transcripts on the real kernel and a scratch,
before a story relies on them (testing.md, 4.3).
Needs: the session's start.
Files: `tests/real-loop/src/{git.rs (new), calls.rs (the caller's helper), lib.rs}`,
`tests/real-loop/tests/git.rs` (new), `tests/real-loop/Cargo.toml`
(`temper-fake-git`, `temper-worker-protocol-git`), `Cargo.lock` (hotspot).
Shape:
- `git::Disk::new(make)` lays `make().checkout.files()` under a
  `Scratch`; `write`, `read` and `exists` resolve against `scenario.at`;
  `forge()` borrows `scenario.transport.forge`.
- `Disk::run` is one `World::new_controlled` run hosting `/usr/bin/git`
  as `git::program()` (instances 1, as one-process's `world.rs::git`),
  rooted `worker` at the scratch as 08's agent roots, made by
  `temper_fake_git::Git::new(spawn, inherited, git::transport())`. A root
  spawns `spawn_of(command)` with `dir = scenario.at`, writes the input,
  drains 1 and 2 and waits: 08's caller's spawn, drain and wait, factored
  into one helper both use.
Test shape: `tests/git.rs`, with 08's counting allocator:
`hosted_clone_creates_only_the_destination`,
`hosted_git_commits_the_staged_real_files`,
`hosted_clean_after_checkout_leaves_only_the_commits_files` (reference)
and `hosted_commit_tree_keeps_exact_parents`, each with the transcripts'
assertions (exits, output, files, `disk.forge()`'s objects) on real
files. Each is 1 to 4 runs of a few ms: under 0.5 s.
```rust
#[test]
fn hosted_clean_after_checkout_leaves_only_the_commits_files() {
    use temper_real_loop_world::git::Disk;
    use temper_worker_protocol_git::invocation::{self, Command};
    let mut disk = Disk::new(temper_fake_git::fixtures::scenario);
    disk.write(b"untracked/deep/file", b"remove\n");
    disk.write(b".git/keep", b"metadata\n");
    let topic = invocation::object(&temper_fake_git::id::id(2)).expect("counted id");
    assert_eq!(disk.run(&Command::CheckOut { commit: topic }, b"").exit, 0);
    assert_eq!(disk.run(&Command::Clean, b"").exit, 0);
    assert_eq!([disk.read(b"conflict.txt"), disk.read(b".git/keep")], [b"topic\n".to_vec(), b"metadata\n".to_vec()]);
    assert!(!disk.exists(b"untracked"), "a nested untracked directory is removed");
}
```
Proof: world (git alone on the real kernel and files), the lowest layer
that shows a file defect of the fake; 13.5 reaches it at joint. The
commit body gives each test's time.
Don't: edit `testing/temper-fake-git`; import the transcripts' `Disk`.
Size: at most ~150 lines of `tests/real-loop/src`, by part 1's command
over that directory alone; 0 by part 1's count, which leaves out `tests/`.
Removes: nothing.

### 13.2 The forge peer in the loop: its delivery meets the engine

Scope: skein's forge peer and fake git hosted in 08's world; the hooks
address handed to the forge; one push hook answered by the engine's
listener (testing.md, 2.4; protocol/forge.md, 5).
Needs: the session's start.
Files: `tests/real-loop/src/{forge.rs (new), world.rs, service.rs,
calls.rs, lib.rs}`, `tests/real-loop/tests/forge.rs` (new), its
`Cargo.toml` (`skein-fake-forge-domain`,
`skein-fake-forge-protocol`; `skein-fake-peers` is 08.1's), `Cargo.lock`.
Shape:
- `forge::fixture()`: `acme/app`, id `forge::ID` (2, as `table` gives),
  a `conflict.txt` and `README.md` tree, context `check`, `hooked: true`,
  and the worker (user 1) granted `Permission::Write`. `forge::OTHER` is
  2, the table's stranger.
- `run_forge(fixture, scripts, steps)` is 08's `run_on` on
  `World::new_controlled`; `Ran` gains that outcome. The order is 08's
  `Proc::Llm`, `Proc::Service`, `Proc::Caller`, then `Proc::Forge`, then
  the hosted children, git as 13.1's. `Proc::Forge` is `Peer::new(
  127.0.0.1:0, forge::peer_limits(), table(name, protected),
  forge::protocol_limits(), handle, SEED)`, both from the setup.
  `Proc::Llm` builds `llm::scripted(scripts(address))` once handed the
  forge's address.
- The referee hands the forge's and the LLM's addresses to `ServiceHost`
  in place of 08's held listener. Once `ServiceHost.hooks` is known it
  sets `Forge.hooks = Some(delivery::Endpoint { forge: 1, host, secret:
  Secret::new(HOOK_SECRET.into()) })`, as forge-kit's `hand_hooks`. Each
  pass it drains `Domain::pop_observation` into `Seen`; it shuts the
  forge peer with the LLM peer.
- `forge::write(scratch, forge, &fixture)`, after 08's
  `fixtures::write`, adds `application.webhooks.listener.socket`
  `127.0.0.1:0` and the forge's `webhook_secret`
  `secrets/forge-1.webhook-secret`, sets the identity's login to the
  table's user 1, applies `fixture.configure`, and writes the secrets
  (0600): both tokens `temper_fake_git::fixtures::TOKEN`, user `1`,
  `HOOK_SECRET`.
- `calls::advance()` (`Step::Advance`): once hooks are handed, the
  referee runs the domain's `advance` on `main` as `OTHER`, and releases
  the caller once the peer observes the `Delivery` and `ServiceHost` has
  the engine's status line on that hooks connection (`Seen::responses`).
Test shape: `tests/forge.rs`, the counting allocator. Aim 1 s: 08.2's
start and drain plus one delivery at `CONFIG`'s zero latencies.
```rust
#[test]
fn a_forge_delivery_is_accepted_by_the_engines_listener() {
    use skein_fake_peers::forge::Delivery;
    use temper_real_loop_world::{calls::advance, forge, run_forge, scripts};
    let ran = run_forge(forge::fixture(), |_| scripts::chat_scripts(1), || vec![advance()]);
    let seen = ran.forge();
    assert_eq!((seen.deliveries(), seen.responses()), (&[Delivery::Written][..], &[204_u16][..]), "{:?}", ran.served());
}
```
Proof: joint (the real service, both peers, the kernel), no higher than
its test: the process tier runs no forge peer (decisions.md, 7).
Don't: name a port; build a forge host; hand the endpoint before the
hooks listener's address is known; edit the peer.
Size: at most ~280 lines of `tests/real-loop/src`, counted as 13.1.
Removes: 08's held forge listener, from forge stories only.

### 13.3 The landing's scripts and configuration, the world's

Scope: one landing script set and configuration for the process tier
and the real loop (README, "a helper two stories need").
Needs: the session's start.
Files: `tests/real-loop/src/{landing.rs (new), lib.rs}`,
`tests/real-loop/tests/landing.rs` (new), `tests/real-loop/Cargo.toml`
(`jig-protocol-config`, `jig-protocol-store`, `temper-protocol-store`,
`temper-engine-domain`, for `landing::admit`); `crates/temper/tests/support/
{forgejo.rs, mod.rs}`, `crates/temper/Cargo.toml` (dev-dependency
`temper-real-loop-world`), `Cargo.lock`.
Shape:
- `landing::scripts(forge, repository, readme)` is `landing_scripts`'s
  body less the `Hello` script: grants on `["forge", forge.to_string()]`,
  the edit appending `landing::EDIT` to `readme`.
- `landing::configure(config, agent, forge)` is `configuration`'s
  engine part (authority, grants, policy, engine overrides, worker,
  charter, agent profile), ending in `admit`, moved with it.
  `forgejo.rs` keeps the application part and `root.brief.gather_ms`.
- `landing_scripts` and `configuration` keep their signatures and
  callers (`process.rs`, `support/live.rs`); `landing_scripts` appends
  its chat script; `support::EDIT` re-exports `landing::EDIT`.
- The order on `forgejo.rs`: temper 10.2 changes `Throwaway::pid`,
  gives `Throwaway` a doc and rewrites the module doc; 13.3 the bodies
  above. Disjoint: the second to land rebases, neither waits; the stale
  text is 10.2's.
Test shape: `tests/landing.rs`, unit, under 20 ms, no world;
`call_arguments` is a test-local decoder of a turn's `Line::Call`.
```rust
#[test]
fn the_landing_grants_name_the_forges_bound_authority() {
    use temper_real_loop_world::landing;
    let forge = std::net::SocketAddr::from(([127, 0, 0, 1], 41_234));
    let scripts = landing::scripts(forge, 2, landing::README);
    let delegate = call_arguments(&scripts[0].turns[0]);
    let grants = delegate["batch"][0]["authority"]["grants"].as_array().expect("grants");
    let named = serde_json::json!(["forge", forge.to_string()]);
    assert!(!grants.is_empty() && grants.iter().all(|grant| grant["segments"] == named), "{grants:?}");
}
```
Proof: unit for the world's part, no higher than its test (13.5 runs
the scripts at joint); process for the landing stories the adapter
feeds, `scripts/proof-one-process.sh` once, `TEMPER_FORGEJO` and
`TEMPER_FORGEJO_RUNNER` exported; the commit body gives the tier's time.
Don't: copy the scripts; rename `Throwaway`; change a script's turn.
Size: at most ~150 lines of `tests/real-loop/src`, moved, counted as
13.1; 0 by part 1's count (`*/tests/*`).
Removes: those lines from `support/forgejo.rs`.

### 13.4 `adopt` through `temper ctl`, against the peer

Scope: the tools' `adopt` as a call on 13.2's world, adopted (tools.md,
5.3, 6.3; client/README.md, 4.7). Not found is the tools'
`each_forge_refusal_exits_2_in_its_words`, `tests/client`'s
`adopting_a_repository_the_forge_lacks_says_not_found` and the engine
domain's `an_adoption_of_a_repository_the_forge_lacks_is_refused_as_not_found`
(ruling 130).
Needs: temper 05's acts and 13.2; the tools' `adopt` has landed
(`crates/temper-client-tools/src/adopt.rs`); temper-13.s1, the
one-process world's cost (ruling 112).
Files: `tests/real-loop/tests/forge.rs`.
Shape: a test-local `adopt(name) -> Called` signs in on `D1` and calls
`adopt --project 1 --owner acme --name NAME --landing main --key
adopt-app` on `forge::fixture()`, with tools.md 5.3's defaults (forge 1,
role `owned`).
Test shape: `tests/forge.rs`; each aims at 1 s, near 08.3's story.
```rust
#[test]
fn adopt_through_ctl_adopts_the_forges_repository() {
    use temper_real_loop_world::Exit;
    let adopted = adopt("app");
    assert_eq!(adopted.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&adopted.stderr));
    assert!(adopted.stdout.starts_with(b"Adopted acme/app on forge 1 as owned.\n"), "{}", String::from_utf8_lossy(&adopted.stdout));
}
```
Proof: joint, as 13.2's.
Don't: read the forge from the test; add the adoption form (later.md).
Size: 0 lines of `tests/real-loop/src`.
Removes: nothing.

### 13.5 A chat lands its change

Scope: adopt, chat, delegate, then clone, edit, push, CI, review and
land through the hosted agents, fake git and the peer (tools.md, 11.3;
domain/forge.md, 8.2).
Needs: 13.1 to 13.4; temper 05's waits and `task --tree` read;
temper-13.s3 (ruling 145); temper-13.s4 (ruling 148).
Files: `tests/real-loop/src/{landing.rs, calls.rs, forge.rs, lib.rs}`,
`tests/real-loop/src/agent.rs` (`agent` and `roots`: each agent adopts,
in Start order, the directory the service names for it, ruling 143),
`tests/real-loop/tests/forge.rs`, `scripts/check.sh` (`delegate` in its
real-loop call positions' pattern).
Shape:
- `landing::fixture()` builds on `forge::fixture()`: README
  `landing::README`; checks at latency 0 with `cue: Some(Cue { path:
  README.md, green: EDIT })`; `main` protected on `check`, approvals 0.
  Its configure is `landing::configure` plus testing.md 2.4's paces,
  through the overrides and `admit`: `forge.client.poll_ms` 10 (5 s
  shipped), `poll_max_ms` 50 (30 s), `slow_ms` 100 (40 s), and
  `hinted_ms` 10 (1 s; decisions.md, 11).
- `calls::Called` derives `Debug` for the story's call diagnostics.
- `scripts/check.sh`'s pattern of literal call positions names
  `delegate` beside `task`, `watching`, `join` and `number`.
- `calls::delegate(i)` (`Arg::Delegate`): the one child task call i's
  `task N --tree --json` names, in temper 05's member names as landed.
- `Seen::merged_green()`: at `Observation::Merged` the referee reads
  `api::Read::Checks` at the head, as one-process's `green`;
  `Seen::main_file(path)` is copied from `main` before the peer shuts.
Test shape: `tests/forge.rs`.
```rust
#[test]
fn a_chat_lands_its_change_on_the_forge() {
    use temper_real_loop_world::{argv, calls::{call, delegate, sign_in, task, D1}, forge, landing, run_forge, Exit};
    const CHAT: usize = 2;
    const TREE: usize = 4;
    let ran = run_forge(landing::fixture(), |at| landing::scripts(at, u64::from(forge::ID), landing::README), || vec![
        sign_in(D1),
        call(D1, argv!["adopt", "--project", "1", "--owner", "acme", "--name", "app", "--landing", "main", "--key", "adopt-app"]),
        call(D1, argv!["chat", "--project", "1", "--key", "land", "@forge_chat Land a small change"]),
        call(D1, argv!["wait", "task", task(CHAT), "--until", "replied"]),
        call(D1, argv!["task", task(CHAT), "--tree", "--json"]),
        call(D1, argv!["wait", "task", delegate(TREE), "--until", "ended"]),
    ]);
    assert!(ran.calls().iter().all(|called| called.exit == Some(Exit::Code(0))), "{:?}", ran.calls());
    assert!(ran.forge().merged_green() && ran.forge().main_file(b"README.md").ends_with(landing::EDIT));
}
```
What it waits on: three agent spawns (chat, coding, review), six
scripted turns at latency zero, about ten hosted git runs, the client's
reads at the paces above, CI at latency 0. Each delivery re-arms a read
at `forge.client.hinted_ms`, set short with the others (decisions.md,
11). Expected: near 1 s; measured three times. Over 1 s with the four
paces short, it is profiled, then made cheaper or moved to
`tests/fuzzy_landing.rs`, naming what it waits on.
Proof: joint, as 13.2's: real git on a forge is the process tier's
landing (decisions.md, 7).
Don't: set a pace past the builder; poll the forge from the test; end a
wait by stopping the engine.
Size: at most ~120 lines of `tests/real-loop/src`, counted as 13.1.
Removes: nothing.

## Interface sketches

```rust
// tests/real-loop/src/git.rs: Scenario, Transport and Answer are temper_fake_git's.
pub struct Disk { /* scratch, scenario */ }
impl Disk {
    pub fn new(make: fn() -> Scenario) -> Self;
    pub fn run(&mut self, command: &invocation::Command, input: &[u8]) -> Answer;
    pub fn write(&mut self, name: &[u8], bytes: &[u8]);
    pub fn read(&self, name: &[u8]) -> Vec<u8>;
    pub fn exists(&self, name: &[u8]) -> bool;
    pub fn forge(&self) -> std::cell::Ref<'_, skein_fake_peers::forge::Forge>;
}
pub fn share(transport: Option<Transport>);
pub fn transport() -> Transport;
pub(crate) fn program() -> skein_world::HostedProgram<Proc>;

// tests/real-loop/src/forge.rs: api is skein_fake_forge_domain's.
#[derive(Clone, Copy)]
pub struct Fixture { pub setup: fn() -> api::Setup, pub configure: fn(&mut Value, &mut Value, SocketAddr) }
pub const ID: u32 = 2;
pub const OTHER: u64 = 2;
pub const HOOK_SECRET: &[u8] = b"real-loop-hook-secret";
pub fn fixture() -> Fixture;
pub fn peer_limits() -> skein_fake_peers::Limits;
pub fn protocol_limits() -> skein_fake_forge_protocol::Limits;
pub fn write(scratch: &Path, forge: SocketAddr, fixture: &Fixture) -> std::io::Result<()>;
pub struct Seen { /* deliveries, responses, merged checks, main's files */ }
impl Seen {
    pub fn deliveries(&self) -> &[skein_fake_peers::forge::Delivery];
    pub fn responses(&self) -> &[u16];
    pub fn merged_green(&self) -> bool;
    pub fn main_file(&self, path: &[u8]) -> &[u8];
}

// lib.rs, service.rs (the port-0 bound address the link does not name), calls.rs.
pub fn run_forge(fixture: forge::Fixture, scripts: fn(SocketAddr) -> Box<[skein_fake_llm_domain::api::Script]>,
    steps: impl FnOnce() -> Vec<Step>) -> Ran;
impl Ran { pub fn forge(&self) -> &forge::Seen; }
fn hooks_address(bound: &[SocketAddr], link: &[u8]) -> Option<SocketAddr>;
pub fn advance() -> Step;          // Step::Advance
pub fn delegate(call: usize) -> Arg; // Arg::Delegate(usize)

// tests/real-loop/src/landing.rs
pub const EDIT: &[u8] = b"Changed by temper.\n";
pub const README: &[u8] = b"# app\n";
pub fn scripts(forge: SocketAddr, repository: u64, readme: &[u8]) -> Box<[skein_fake_llm_domain::api::Script]>;
pub fn configure(config: &mut Value, agent: &mut Value, forge: SocketAddr);
pub fn fixture() -> forge::Fixture;
```

## Questions to raise when reached

- **13.3, the dev-dependency** `crates/temper` → `temper-real-loop-world`
  refused by cargo or a lint: raise it, never copy.
- **A story over 1 s:** profiled, then raised with what it waits on.
  The real loop's stories together past testing.md 6.1's 6 s hold no
  story within its 1 s (README, Rules, "The worlds' sums").
- **The landing stalls,** or the delegate does not end at the merge
  (domain/forge.md, 8.2): keep the trace, the peer's observations and
  the last stdout, and raise it before changing a fixture.
- **The engine follows the table's `base`** (`http://127.0.0.1:3000`):
  raise it; the table is temper-fake-git's.
- **forge-kit's temper-05.2 or temper 05 differ** from these names:
  follow the landed code; raise a missing capability.

## Done when

Each prints what it says, on `origin/main`, after `git fetch`:

- `git grep -c -E 'fn (hosted_clone_creates_only_the_destination|hosted_git_commits_the_staged_real_files|hosted_clean_after_checkout_leaves_only_the_commits_files|hosted_commit_tree_keeps_exact_parents|a_forge_delivery_is_accepted_by_the_engines_listener|the_landing_grants_name_the_forges_bound_authority|adopt_through_ctl_adopts_the_forges_repository|a_chat_lands_its_change_on_the_forge)\(' origin/main -- tests/real-loop/tests`
  prints counts that sum to 8 (ruling 130).
- `git grep -n -F -e 'Git::new(' -e 'skein_fake_peers::forge::Peer' -e 'delivery::Endpoint' -e 'cue: Some(' -e 'hooked: true' -e '@coding Small fix' -e 'poll_ms' origin/main -- tests/real-loop/src`
  prints a line for each of the seven, and with `-e 'Command::Clean' -e 'Command::CommitTree' -e '"adopt"' -e 'merged_green()'` over `tests/real-loop/tests`, each of the four.
- `git grep -n -F '@coding Small fix' origin/main -- crates/temper/tests/support/forgejo.rs`
  and `git grep -n -E '127\.0\.0\.1:[1-9]' origin/main -- tests/real-loop` print nothing.
