# Session 07: the fake forge's host in `testing/`

The one-process world's `ForgeHost` (`tests/one-process/src/forge.rs`)
becomes a crate of the fake forge's family, `testing/temper-fake-forge-host`,
that names no engine or worker type, so that the forge kit can move it
with the fake forge (domain/testing.md, 2.4 and 7). It takes everything from its
world: the shared in-memory forge, the protocol's tables and limits, the
seed and the address to listen on. With port 0 it listens where the
kernel says, writes that address into its documents' base, and tells it
on the shared forge, where the real loop's other hosts read it. It writes
the fake forge's signed deliveries for hooked repositories
(`temper_fake_forge_protocol::Service::delivery`) on connections to the
listener its world names. The one-process world moves onto the host
without changing its behaviour, and names its forge's address once.
Every increment is a forge-kit hotspot: it changes the fake forge.

## Read first

- temper's `docs/design/domain/testing.md`: 2.4 (the bullets "What runs", whose
  rule on listeners is the real loop's, "The forge and git", "The forge
  host" and "CI and webhooks"), 2.5, 4.2 (the protocol layer: "For each
  of the domain's webhooks ... a signed delivery, which the world
  writes"), 7 (the row `testing/temper-fake-forge-host`), 9 (the real-loop
  world, the forge host among it) and 10 (the real-git bridge is later
  work).
- temper's `docs/design/protocol/forge.md`: 5 (the webhooks' listener) and
  9 (the fake forge's protocol layer).
- Not designs, read only to keep the host easy to move: temper's
  `docs/design/draft/forge-kit.md`, 2, and skein's
  `docs/design/draft/forge.md`, 7 ("skein-fake-peers hosts it on
  loopback").
- The code: `tests/one-process/src/{forge.rs, world.rs, fixtures.rs,
  stories.rs, referee.rs, referee/forge.rs}`;
  `testing/temper-fake-git/src/transport.rs` (`Forge`, `Transport::call`);
  `testing/temper-fake-forge-protocol/src/{lib.rs, service.rs,
  delivery.rs}` (`Config`, `Service::target`, `Service::delivery`,
  `sign`, `Endpoint`, `Post`); `testing/temper-fake-forge-domain/src/hooks.rs`;
  `crates/temper-engine-protocol-forge/src/hooks.rs` (`Hooks`, `Webhooks`).
- skein's `programming-model.md`: 2.1, 4.3, 6.3, 10.2 (by number only);
  skein's `testing-strategy.md`: 2.7, 2.8, 4.1, 6, 10 (by number only).

## What the design asks

- **2.4, the forge host:** it serves the fake forge's protocol layer on a
  loopback listener and writes signed deliveries to the listeners of
  hooked repositories; a crate of the fake forge's family under
  `testing/`, naming no engine or worker type; the one-process world and
  the real loop both host it. 07.1 moves it, 07.3 writes the deliveries.
- **2.4, listeners:** on loopback, on a port the kernel assigns, passed
  to their clients once bound; no fixed port in the real loop. 07.2.
- **2.4, the forge and git:** the forge host and the fake git program
  share one in-memory forge. 07.1 moves that forge with the host.
- **2.4, CI and webhooks; 4.2:** the host delivers a hooked repository's
  webhooks to the engine's webhooks' listener; polling stays the
  backstop. 07.3 writes them; temper 08 joins them to the engine.
- **7:** the crate's place and one line of what it is.

## Choices this plan makes

- **The shared forge moves with the host.** `temper_fake_git::Forge`
  (`testing/temper-fake-git/src/transport.rs`) becomes
  `temper_fake_forge_host::Forge`, unchanged but for a method that hands
  `Transport::call` its serial: temper-fake-git names the worker's crates
  in its manifest, so the host cannot depend on it, and the two share one
  forge (2.4). temper-fake-git depends on the host's crate for it, a
  dependency that still points upstream once the forge kit moves the
  host to skein and temper-fake-git stays (draft/forge-kit.md, 2).
- **The world gives the host its settings:** the address to listen on,
  the protocol's tables, its limits and the seed. The io limits stay the
  host's own, today's values, as do its worst case and operations.
- **The bound address is told on the shared forge** (`Forge::bound`),
  and the tables' `base` is written from it once the listener is bound,
  so that the documents' `html_url` names the address the kernel gave.
  Both worlds run their hosts in one loop to the end (skein-world's
  `World::run` and `real::World::run`), so a client in that loop reads
  what the loop shares: the shared forge, which the fake git program and
  any scripted peer already hold.
- **The one-process world keeps a named address,** `fixtures::FORGE`,
  127.0.0.1:3000 as today, and no longer spells it anywhere else. Port 0
  would change its behaviour: its service is spawned before any host
  runs, by a `fn(&Inherited)` factory, with the forge's address in its
  configuration document, `fixtures::forges()`, its grants and its
  referee's base; and the simulator draws a port 0 from the seed. The
  listeners rule of 2.4 is the real loop's: 2.5 names none, and
  `fixtures::CLIENTS` is fixed already. The real loop (temper 08) gives
  port 0 and reads `Forge::bound`.
- **Deliveries go where the world says,** by `Forge::deliver_to`, set
  once the engine's webhooks' listener is bound; one connection for
  each delivery, the post written and the connection closed. A hook with
  no listener told, past the host's bound on deliveries in flight, or
  whose connection fails is dropped, as the fake forge's domain loses a
  webhook (`hooks.rs`: "best effort"); the engine polls (2.4). The
  `Host` header stays the tables' `Endpoint.host`: the engine's webhook
  connections read none (`crates/temper-engine-protocol-forge/src/hooks/`).
- **The one-process world keeps webhooks off** (`"webhooks":null`, its
  repository not hooked), so no delivery is written there; temper 08's
  real loop, with webhooks on, is the first world that joins the host's
  deliveries to the engine's listener.

## Rules that bind it

- **Extend the existing fake forge, build no new one** (README, seams):
  the host serves `temper-fake-forge-protocol` over
  `temper-fake-forge-domain` as one-process's does today, and this
  session changes neither crate; a change either needs is a question.
- **The host names no engine or worker type:** no `temper_engine_*`,
  `temper_worker_*`, `temper_service` or `temper_fake_git` item in its
  sources, tests included, and no such crate in its manifest (Done when).
- **The one-process world is unchanged in behaviour:** its focused and
  fuzzy tests keep their names and expected values, and pass with no
  change but the address's spelling and the host's crate.

## Increments

### 07.1 The forge host's crate, and the shared forge with it

Scope: `testing/temper-fake-forge-host`, a crate of ordinary Rust
(programming-model.md, 10.2) holding one-process's `ForgeHost` and
fake-git's `Forge`, moved: the host built from `Settings` and the shared
forge in place of one-process's fixtures and thread-local; one-process,
temper-fake-git and their users onto it, with no re-export left behind
(domain/testing.md, 2.4, 7).
Needs: nothing beyond the session's start.
Files (forge-kit hotspot, the fake forge; each named exactly):
`testing/temper-fake-forge-host/{Cargo.toml, clippy.toml, src/lib.rs,
src/forge.rs, src/host.rs}` (new; the manifest's lints and `clippy.toml`
as temper-fake-git's); `testing/temper-fake-git/{Cargo.toml,
src/transport.rs, src/lib.rs, src/fixtures.rs}`; `tests/one-process/{Cargo.toml,
src/forge.rs, src/world.rs, src/stories.rs, src/referee/forge.rs}`; the
workspace `Cargo.toml` (a member and a `[workspace.dependencies]` row,
beside `temper-fake-forge-protocol`'s) and `Cargo.lock`;
`testing/temper-fake-forge-host/tests/{host.rs, support/mod.rs}` (new).
Shape:
- `forge.rs`: `pub struct Forge`, fake-git's fields (`domain`, `config`,
  `now`, `pending` public; `serial` private) and `Forge::new`;
  `pub fn next_serial(&mut self) -> u64`, which `Transport::call` uses
  for its `(7 << 60) + serial` tokens.
- `host.rs`: `pub struct Settings { listen, tables, limits, seed }`
  (Interface sketches) and `ForgeHost::new(settings, forge:
  Rc<RefCell<Forge>>) -> Self`, building the `Service` from
  `settings.tables` and listening at `settings.listen`; the rest as in
  `tests/one-process/src/forge.rs` today (`event`, `progress`,
  `shutdown`, `impl skein_shell::Host`, its io limits, worst case and
  operations).
- one-process's `forge.rs` keeps `set` and `transport` and gains
  `pub(crate) fn host() -> ForgeHost`, the settings one-process gives
  today: 127.0.0.1:3000, `temper_fake_git::fixtures::table(b"acme/app",
  None)`, `temper_engine_protocol_forge_world::fixtures::fake_limits()`,
  `fixtures::SEED`. `world.rs`'s `Proc::Forge` holds
  `SignalHost<temper_fake_forge_host::ForgeHost>`, and `fake_forge` and
  the two spawns call `forge::host()`.
Test shape: the host's world in `tests/support/mod.rs`, on skein-world's
`World` over skein-sim (`skein-sim` by git, as one-process names it): a
`Proc` of `Forge(ForgeHost)` and scripted peers (`Peer::get(addr,
path)`, a caller that writes one GET with `support::TOKEN` and keeps the
answer); `support::forge()` builds the domain as one-process's
`stories::forge` does (`acme/app`, id 2, the worker granted write) over
`support::CONFIG`, a domain `Config` with the values of
`tests/worker/checkout/src/forge.rs`'s `CONFIG`, which the host's tests
may not name; `support::settings(listen, hooks)`, its tables as
`temper_fake_git::fixtures::table`'s and its limits the values of
`tests/engine/protocol-forge/src/fixtures.rs`'s `fake_limits` (the limits
one-process gives); the referee shuts the host down (`Referee::act`)
once each peer is done. The reference test:
```rust
#[test]
fn a_call_on_the_listener_is_answered_from_the_shared_forge() {
    let forge = support::forge();
    let host = ForgeHost::new(support::settings(support::LISTEN, None), Rc::clone(&forge));
    let seen = support::run(host, &forge, vec![support::Peer::get(support::LISTEN, b"/api/v1/repos/acme/app")]);
    assert_eq!(support::status(&seen.answers[0]), 200, "{seen:?}");
}
```
Tests, each under 0.5 s (world):
`a_call_on_the_listener_is_answered_from_the_shared_forge`,
`a_shut_down_host_closes_its_listener_and_settles`; and one-process's
tests, unchanged.
Proof: joint, the one-process world, once: `cargo nextest run -p
temper-one-process-world -E
'test(an_adopted_repository_gets_its_change_pushed_through_git)'`
(about 3 s): the engine adopts, reads and pushes through the moved host
and the shared forge.
Don't: change what the host does (that is 07.2's and 07.3's); keep
`temper_fake_git::Forge` as an alias or re-export; name a temper engine
or worker item in the new crate, its tests included.
Size: at most 60 new production lines, beside about 290 moved unchanged
(one-process's `forge.rs`, fake-git's `Forge`), each moved file named in
the commit body.
Removes: `ForgeHost` from `tests/one-process`; `Forge` from temper-fake-git.

### 07.2 The listener on the address given, told once bound

Scope: the host listens at its settings' address, port 0 included; once
`Listening`, it builds its `Service` with the tables' `base` written as
`http://` and the bound address, and tells the address on the shared
forge, cleared when its listener closes; one-process names its address
once, `fixtures::FORGE` (domain/testing.md, 2.4).
Needs: 07.1.
Files (forge-kit hotspot, the fake forge): `testing/temper-fake-forge-host/src/{forge.rs,
host.rs}`, `tests/{host.rs, support/mod.rs}`;
`tests/one-process/src/{fixtures.rs, forge.rs, stories.rs, referee.rs}`.
Shape:
- `Forge { .., bound: Option<SocketAddr> }`, `pub fn bound(&self) ->
  Option<SocketAddr>`, written by the host alone.
- `ForgeHost` keeps `settings.tables` until `io::Event::Listening { addr:
  io::Addr::Ip(bound), .. }`, then builds the `Service` from them, its
  `base` replaced; tables the protocol refuses fail the world there, as
  `expect("protocol tables")` does today.
- one-process: `pub(crate) const FORGE: SocketAddr` (127.0.0.1:3000) in
  `fixtures.rs`, beside `CLIENTS`; `document()`'s forge `api.socket` and
  `git`, `forges()`'s `address`, `stories.rs`'s two grant segments and
  its push URL, `referee.rs`'s `base`, and `forge::host()`'s `listen`
  derive from it.
Test shape: the host's world; `support::Peer::get_bound(path)` waits for
`Forge::bound` and calls there; `support::html_url(answer)`. The
reference test:
```rust
#[test]
fn on_port_zero_the_documents_name_the_address_the_kernel_bound() {
    let forge = support::forge();
    let host = ForgeHost::new(support::settings(support::ANY_PORT, None), Rc::clone(&forge));
    let seen = support::run(host, &forge, vec![support::Peer::get_bound(b"/api/v1/repos/acme/app")]);
    let bound = seen.bound.expect("told once listening");
    assert_ne!(bound.port(), 0);
    assert_eq!(support::html_url(&seen.answers[0]), format!("http://{bound}/acme/app").as_bytes());
}
```
(`seen.bound` is `Forge::bound` as the peer read it; `ANY_PORT` is
127.0.0.1:0.) Tests, each under 0.5 s (world):
`on_port_zero_the_documents_name_the_address_the_kernel_bound`,
`the_bound_address_is_told_once_listening_and_cleared_at_shutdown`,
`a_restarted_host_tells_the_address_it_bound_again`; and one-process's
tests, unchanged.
Proof: joint, the one-process world, once, with 07.1's command (about
3 s): the configuration, the grants and the referee name
`fixtures::FORGE`, and the host listens there.
Don't: give one-process port 0 (Choices); change fake-git's
`fixtures::table`, whose `base` the host replaces.
Size: at most 80 lines.
Removes: every `127.0.0.1:3000` of `tests/one-process`.

### 07.3 Signed deliveries to the listener told

Scope: each `domain::Request::Hook` the host takes from the shared
forge's `pending` (`Service::target` hands it back) is rendered by
`Service::delivery` and written on a connection of its own to the
listener `Forge::deliver_to` names, then closed; a hook with nowhere to
go is dropped (domain/testing.md, 2.4, 4.2; protocol/forge.md, 5, 9).
Needs: 07.1; after 07.2 (the same files).
Files (forge-kit hotspot, the fake forge): `testing/temper-fake-forge-host/src/{forge.rs,
host.rs}`, `tests/{host.rs, support/mod.rs}`.
Shape:
- `Forge { .., deliveries: Option<SocketAddr> }`, `pub fn
  deliver_to(&mut self, listener: SocketAddr)`.
- `host.rs`: `deliveries: BTreeMap<Token, Delivery>`, at most
  `MAX_DELIVERIES` (4) in flight, owned by tokens from `DELIVERIES` (`1
  << 62`) up, apart from accepted connections' and the listener's;
  `struct Delivery { socket: Option<Token>, bytes: Box<[u8]>, sent: usize }`,
  the post's head then body. `io::Request::Connect { owner, addr }`;
  `Connecting` keeps the socket; `Connected` and each `Up::Room` give one
  `Down::Send` of at most the io limits' `output`, after a
  `Down::Demand { read: Read::Nothing, room }`; all sent, `Close`;
  `Failed` and `Closed` end it. `event` takes these four events for
  delivery owners alone; any other still fails the world.
- `work_pending` also while `pending` holds a hook, which fake git or a
  peer may leave there; `is_empty` also waits for `deliveries`.
Test shape: the host's world gains `Peer::hooked(hook)`, a scripted
listener on port 0 that, once listening, calls `deliver_to` and leaves
`hook` on the shared forge's `pending`, then keeps what each connection
it accepts sends; `support::endpoint()` (forge 1, secret
`support::SECRET`); `support::push_hook(branch, commit)`, after the
protocol's `tests/deliveries.rs` `hook`; `support::rendered(&hook)`,
the `Post` a `Service` on the same tables renders. The reference test:
```rust
#[test]
fn a_hooked_repositorys_hook_is_written_signed_to_the_listener_told() {
    let forge = support::forge();
    let host = ForgeHost::new(support::settings(support::ANY_PORT, Some(support::endpoint())), Rc::clone(&forge));
    let hook = support::push_hook(b"main", 7);
    let seen = support::run(host, &forge, vec![support::Peer::hooked(hook.clone())]);
    let post = support::rendered(&hook);
    assert_eq!(seen.deliveries, [[&*post.head, &*post.body].concat()]);
    assert!(support::signed_with(&seen.deliveries[0], &delivery::sign(&support::secret(), &post.body)));
}
```
Tests, each under 0.5 s (world):
`a_hooked_repositorys_hook_is_written_signed_to_the_listener_told`,
`with_no_hooks_configured_nothing_is_written`,
`a_hook_before_any_listener_is_told_is_dropped`,
`a_refused_delivery_is_dropped_and_calls_are_still_answered`.
Proof: world, once: `cargo nextest run -p temper-fake-forge-host
-E 'test(a_hooked_repositorys_hook_is_written_signed_to_the_listener_told)'`
(under 0.5 s): a delivery reaches a listener on the loopback, signed.
No higher: no world joins the engine's webhooks' listener to the host
until temper 08's real loop, with webhooks on, connects it.
Don't: retry or queue a dropped hook; write a delivery in one-process
(webhooks stay off there); route deliveries to the org hook reconcile
made (forge-setup's `later.md`, its row on the fake forge's deliveries).
Size: at most 150 lines.
Removes: the host's silent drop of every hook in `progress`.

## Interface sketches

The host owns its listener, its connections and its deliveries; the
world owns the shared forge, and each of its hosts holds a clone of the
`Rc`. The host writes `bound`; the world, or a host of it, writes
`deliveries`.

```rust
// testing/temper-fake-forge-host/src/forge.rs
/// The one in-memory forge the host and the fake git program share (domain/testing.md, 2.4).
pub struct Forge {
    pub domain: temper_fake_forge_domain::Domain,
    pub config: temper_fake_forge_domain::Config,
    /// The world's time, as the last host to run saw it.
    pub now: skein_lib::Time,
    /// Replies and hooks another host left for this host to route.
    pub pending: Vec<temper_fake_forge_domain::Request>,
    serial: u64,
    bound: Option<core::net::SocketAddr>,      // 07.2
    deliveries: Option<core::net::SocketAddr>, // 07.3
}
impl Forge {
    pub fn new(domain: temper_fake_forge_domain::Domain, config: temper_fake_forge_domain::Config) -> Self;
    /// The next serial of the git transport's calls.
    pub fn next_serial(&mut self) -> u64;
    /// Where the host listens, from `Listening` until its listener closes (07.2).
    pub fn bound(&self) -> Option<core::net::SocketAddr>;
    /// Where hooked repositories' deliveries go from now on (07.3).
    pub fn deliver_to(&mut self, listener: core::net::SocketAddr);
}

// testing/temper-fake-forge-host/src/host.rs
/// What a world gives its forge host.
pub struct Settings {
    /// Loopback; port 0 for the kernel's choice (07.2).
    pub listen: core::net::SocketAddr,
    /// The protocol's tables; their `base` is written from the bound address (07.2).
    pub tables: temper_fake_forge_protocol::Config,
    pub limits: temper_fake_forge_protocol::Limits,
    pub seed: u64,
}
/// The fake forge's protocol layer on a loopback listener, and its deliveries.
pub struct ForgeHost { /* .. */ }
impl ForgeHost {
    pub fn new(settings: Settings, forge: std::rc::Rc<std::cell::RefCell<Forge>>) -> Self;
    /// Close the listener and the connections, then settle.
    pub fn shutdown(&mut self);
}
impl skein_shell::Host for ForgeHost { /* .. */ }
```

## Questions to raise when reached

- If one of one-process's focused or fuzzy tests changes its outcome
  under 07.1 or 07.2, raise it with the seed and the trace's first
  difference, rather than changing the story.
- If the host's world tests take more than 0.5 s each, raise it with the
  measured times and what the world's setup costs.

## Done when

- 07.1 to 07.3 have landed on temper's `origin/main`.
- `git show origin/main:Cargo.toml | grep -n '"testing/temper-fake-forge-host"'`
  prints the member's line.
- `git grep -n -e 'temper[-_]engine' -e 'temper[-_]worker' -e
  'temper[-_]service' -e 'temper[-_]fake[-_]git' origin/main --
  testing/temper-fake-forge-host` prints nothing.
- `git grep -n -e 'struct ForgeHost' origin/main -- tests/one-process`
  and `git grep -n -e 'pub struct Forge\b' origin/main --
  testing/temper-fake-git` print nothing.
- `git grep -n -F '127.0.0.1:3000' origin/main -- tests/one-process`
  prints nothing.
- `git grep -n -e 'pub fn bound' -e 'pub fn deliver_to' -e '\.delivery('
  origin/main -- testing/temper-fake-forge-host/src` prints a line for
  each.
