# Session 08: the real-loop world

`tests/real-loop` starts domain/testing.md 2.4's general layer: one thread and ring, scratch files,
the standalone service and worker, hosted smith agents, native clients and fake peers. The
first stories sign in, ask a chat and wait for its reply, and meet a wait with another
client's act. Each drains. The forge host meets the engine's webhook listener here; temper
09 extends this world's fixtures, caller and observations.

Needs at start: `temper ctl`; the fake forge's host in `testing/`. Steps: `temper-08.1`
through `temper-08.6`.

## Read first

- temper's `docs/design/domain/testing.md`: 2.4, 4.1 to 4.3, 5.2, 6.1, 7, 10;
  `docs/design/client/tools.md`: 8.1 to 8.3, 11.3, 11.4; `docs/design/client/README.md`: 7;
  `docs/design/protocol/forge.md`: 5, 8, 9; `docs/design/protocol/configuration.md`: 3 and
  7.
- This plan's `decisions.md` (real-loop first, the hosted cli, seams), `later.md`,
  `findings/temper-area-1.md` 3 and 6; `temper/07-forge-host.md` 07.2 and 07.3;
  `temper/09-real-loop-stories.md` (Read first, Needs, increments and interfaces);
  `temper/05-ctl.md` (Host, Invocation, Startup and parent-root ownership).
- Code: `crates/temper-service/src/{system.rs,server.rs,clients.rs,
  clients/printed.rs,forge.rs,startup.rs,limits/names.rs}`;
  `crates/temper-engine-protocol-forge/src/{hooks.rs,component.rs}`;
  `tests/one-process/src/{world.rs,fixtures.rs,stories.rs,referee/forge.rs}`.
- skein: `testing/skein-world/src/{real.rs,program.rs,heap.rs}`; `tests/world/tests/real.rs`
  (hosted pipes, Kill and Wait); `testing/skein-fake-peers/src/llm.rs` (`Peer::address`,
  `observations`). Templates: smith's `tests/real/src/{lib.rs,settled.rs}`; jig's
  `testing/jig-test/tests/support/mod.rs` (scratch and printed link).
- skein's `programming-model.md`: 2.1, 4.3, 6.2, 6.3, 10.2; `testing-strategy.md`: 2.8, 4.1,
  6, 7, 8, 9, 10 (by number only).

## What the design asks

- **domain/testing.md 2.4:** the assembled stack over one ring, kernel-assigned ports,
  ownership/operation/descriptor checks and retained failure traces.
- **client/tools.md 11.3:** real files, locks, hosted call exits, the three first stories and
  drained shutdown; **11.4:** executable paths elsewhere.
- **domain/testing.md 2.4, 4.2:** signed webhooks, polling as backstop.
- **6.1:** each focused story within 1 s, the lower worlds within 4 s, the focused suite
  within 15 s; no time-guide exception.

## Choices this plan makes

- **Ports are values obtained after binding.** The forge gives `Forge::bound`; the LLM gives
  `Peer::address`; the service's client listener already keeps `Listening.addr` in
  `Server::listening`, uses it for the Host guard and prints it through `Service::links`.
  The engine's client listener and the service's HTTP listener are this same listener. The
  worker uses standalone's `in_process: workers`, so it introduces no further TCP listener.
  Only webhooks need an accessor (08.1). No bind-close-reserve probe, fixed port or
  rewritten `address` file: ctl sign-in writes that file from the printed link.
- **Construction waits for its inputs inside the loop.** Register all programs before `run`;
  a root host may wait before constructing its service. In a forge story the order is forge
  bound, scripted LLM built and bound, service constructed, hooks bound, `deliver_to` set,
  then calls admitted. Scripts that contain a forge authority are made by a function taking
  its bound address, as temper 09 requires.
- **Fixtures belong to the layer.** `fixtures.rs` owns the profile documents, paths and
  constants; `Scratch` outlives a run. Reuse jig's channel fixture inputs and smith's
  profile builders, and one-process's composition and outside expectations where applicable.
  Do not import its fixed-port constructors or the executable tests' private support.
  `run_on` exists here; temper 09 adds the restart assertions.
- **Memory has explicit owners.** Skein's checked outcome assumes each program frees
  precisely its own allocations. The shared mutable forge and fake git cross that boundary.
  Use `real::World`'s kernel ledger with a test-only host wrapper using `skein_heap::Span`:
  one owner per service, caller, cli invocation, agent and LLM; one owner for the forge
  family, including its shared domain and later hosted git. Construction, iteration and
  destruction are charged to that owner. No service heap is excluded, and the fake family's
  bound is a sum, not an unlimited allowance. Snapshots for `Ran` are copied outside these
  spans.
- **Later pieces retain the names in `later.md`:** The real-git bridge; CI by a subset of
  sh; Faults in the real loop; Several workers and their agents; temper's terminal; The
  adoption form; The setup; Webhooks late, out of order, twice or never; A second connector,
  test environments; The QA loop's own stories; The browser; A real `temper agent` process;
  The forge over TLS; A GitHub forge. The tools of areas 2, 3 and 4 and the MCP endpoint
  likewise remain there.

## Rules that bind it

- Extend the existing fakes. Forge-component work (08.1) and fake-LLM wiring (08.2) have
  their own increments and exact hotspot paths. No provider change, no design or STATUS
  edit, no behaviour from drafts.
- Use shipped profile builders, with each capacity or pace override named beside its source
  and reason. Keep lock retry, link deadline, idle and teardown times as shipped. Short
  forge polling is the explicit exception designed in domain/testing.md 2.4, not a way to end a
  client.
- A peer stays available until its clients settle. A story past 1 s is profiled and made
  cheaper or moved to `fuzzy_*`; no optimized builds, sleeps for readiness, additional ring
  or process-tier substitution.

## Increments

### 08.1 Tell the webhook listener's bound address
Scope: domain/testing.md 2.4's listener rule, behind the existing component.
Needs: nothing beyond the session's start.
Files (forge-component hotspot; each named exactly):
`crates/temper-engine-protocol-forge/src/hooks.rs`,
`crates/temper-engine-protocol-forge/src/component.rs`,
`crates/temper-engine-protocol-forge/src/tests/hooks.rs`,
`crates/temper-service/src/system.rs`, `crates/temper-service/src/tests/clients.rs`;
`tests/engine/protocol-forge/src/hooks.rs`, `tests/engine/protocol-forge/tests/hooks.rs`.
Shape: `Hooks` keeps `bound: Option<SocketAddr>` from `Listening.addr`, cleared on listener
failure or close; `Component::hooks_address()` reads it; `Service::hooks_address()` forwards
through its existing forge layer. No new event, queue, engine fact or listener owner. Keep
the client listener's existing bound-address path, and test that a given `Listening.addr`
reaches its printed link and Host guard.
Test shape: extend the existing hook fixtures and `hooks::Listener` with `address()`. Unit
cases: absent before bind, actual address after bind, absent after failure/close, disabled
hooks absent (each under 20 ms). The existing signed-push world also asserts its listener's
address; its simulated Listening completion supplies an address different from the requested
port 0. Keep the signed push's 204 and hint assertions.
Proof: world, `cargo nextest run -p temper-engine-protocol-forge-world --test hooks -E
'test(a_signed_push_is_answered_204_before_its_hint_goes_up)'`, under 0.5 s. The forge
host's first joint connection is 08.5.
Don't: expose tokens or secrets; change hook admission or delivery semantics.
Size: at most 50 production lines.
Removes: the loss of `Listening.addr` in `Hooks::up`.

### 08.2 The scripted LLM peer on its bound address
Scope: domain/testing.md 2.4 and 4.1, fake-LLM wiring in its own increment.
Needs: nothing beyond the session's start.
Files (fake LLM hotspot; each named exactly): `tests/real-loop/src/llm.rs`,
`tests/real-loop/src/scripts.rs`, `tests/real-loop/src/lib.rs`,
`tests/real-loop/tests/peers.rs`, `tests/real-loop/Cargo.toml` (new); workspace
`Cargo.toml`, `Cargo.lock`.
Shape: add `temper-real-loop-world`, ordinary test Rust under `tests/`'s lints. `llm.rs`
owns skein's existing `llm::Peer`, its observations and the deferred scripts factory.
Construct `Peer::new` at 127.0.0.1:0; use smith's fake-peer limits and skein's LLM fake
builders, as smith's `fake::configured_bounds` does, with scripted faults and latency zero.
Use plaintext for these loopback stories. `scripts::chat_scripts(n)` gives n chat scripts,
each using jig's `provider_turns` and the report shape of one-process's `provider`; export
`WORDS` and `REPLY`. The host waits for `Forge::bound` before invoking a forge scripts
factory; ordinary scripts need no forge. The endpoint configuration is built from
`Peer::address`, never smith's fixed-port `fake::configured` helper.
Test shape: `tests/peers.rs`, with the counting allocator, a small
`real::World::new_checked` and a referee that closes the unused peer only after observing
its address. Clone script inputs within the factory.
```rust
#[test]
fn the_scripted_peer_tells_its_bound_port_and_drains() {
    let seen = llm::bound_and_drained(chat_scripts(1));
    assert!(seen.address.ip().is_loopback());
    assert_ne!(seen.address.port(), 0);
    assert!(seen.settled);
}
```
Proof: world, `cargo nextest run -p temper-real-loop-world --test peers`, under 0.5 s.
No higher until 08.4 runs the chat against the agent.
Don't: change skein's fake LLM, smith's helpers, providers or account setup.
Size: at most 220 lines of harness; no production change.
Removes: nothing.

### 08.3 Scratch, the service and hosted agents, checked as they run
Scope: domain/testing.md 2.4's general harness, startup and drained shutdown.
Needs: the scripted LLM peer (08.2).
Files: `tests/real-loop/src/{lib.rs,fixtures.rs,scratch.rs,world.rs,
service.rs,agent.rs,check.rs}`; `tests/real-loop/tests/lifecycle.rs`;
`tests/real-loop/Cargo.toml`.
Shape: `Scratch::new` creates unique state, worker and client directories; private
configuration and credential inputs are 0600. Materialize the files that startup reads.
Build the checked deployment from the profile builders used by
`tests/one-process/src/fixtures.rs`; resolve the LLM address after bind and set application
clients to port 0. With no forge story there is no configured forge or webhook listener.
`ServiceHost` wraps `temper_service::startup::startup`, `adopt_store`, `iterate` and
`links`; it observes completions before forwarding them, ready for 09's `cue::Watch`.
`agent.rs` registers smith's `Agent` and `AgentErrors::pipe` as one-process's `world::agent`
does: inherited stdin/stdout/stderr and signal, roots chosen from the spawn's directory.
Register factories and instance/operation bounds before `World::run`. `check.rs` meters
constructors, iterations and drops with `Span`; its ledger is updated outside spans. Inputs
are cloned within their owner's construction. Bound each owner by its hosts' `worst_case`
and fixture storage; check peak after every turn and zero retained bytes after drop. The
ring checks operation limits and descriptor ownership/release. The referee drains the
service before the peers and implements last word and `next_policy_deadline` forwarding.
Failures keep scratch and a bounded operation/peer trace; successful runs remove scratch
when its guard ends.
Test shape: `tests/lifecycle.rs`, counting allocator; fixtures from the shipped small
deployment and smith standard profile, every changed value listed in `fixtures.rs`; startup
must admit the combined memory/operation envelope, which the proof reports once and later
additions recompute.
```rust
#[test]
fn a_bound_service_prints_its_link_and_drains() {
    let scratch = Scratch::new();
    let seen = world::until_link(&scratch);
    assert!(seen.client_address.ip().is_loopback());
    assert_ne!(seen.client_address.port(), 0);
    assert_eq!(link_address(&seen.link), seen.client_address);
    assert!(seen.drained);
}
```
Proof: world, `cargo nextest run -p temper-real-loop-world --test lifecycle`, under
0.5 s; the real store and worker drain, all owners and descriptors release. Calls and the
first agent turn meet them in 08.4.
Don't: replay the kernel schedule; copy the simulator; use real git or sh to establish this
fixture; weaken ownership checks to fit shared state.
Size: at most 300 lines of harness; no production change.
Removes: nothing.

### 08.4 One hosted program per call, from sign-in to a chat reply
Scope: client/tools.md 8 and 11.3, including the first agent turn.
Needs: the checked service and agent hosts (08.3).
Files: `tests/real-loop/src/{calls.rs,lib.rs,world.rs,fixtures.rs}`;
`tests/real-loop/tests/tools.rs`; `tests/real-loop/Cargo.toml`.
Shape: admit the service before its caller, and hosted children after the caller (09's cue).
The caller spawns, writes stdin, drains both output pipes, then reaps with `Op::Wait`;
`Called.exit` is that wait's result. Register the cli factory by a distinct
program name: parse to `Invocation` and call `Host::new` with `Startup { parent, stdin,
stdout, stderr, signals, user, seed }`. Open a fresh parent startup root per spawn;
DIRECTORY is its leaf name, opened and locked independently by the host. Close unused pipe
ends; the cli owns its signal source. A new call is a new host. No executable, `main` or
`skein_shell::drive`. Implement `run`, `run_on`, `sign_in`, `same_party`, `call`, `argv!`
and `task(i)` (interfaces below). Sign-in passes `Service::links` bytes on stdin;
`same_party` copies a settled sign-in's private token and address bytes to an empty
DIRECTORY, leaving its store empty. It invents neither credential nor address. `ran.calls`
excludes sign-ins and includes every Call and Start in step order, never completion order.
Test shape: `tests/tools.rs`, counting allocator; same fixtures, `chat_scripts`, profile
admission and every drain check. After sign-in assert DIRECTORY mode 0700,
token/store/address modes 0600 and owner uid; `address` is the printed link's authority with
no secret, persisted by ctl.
```rust
#[test]
fn a_signed_in_call_starts_a_chat_and_waits_for_its_reply() {
    let ran = run(chat_scripts(1), &[
        sign_in(D1),
        call(D1, argv!["chat", "--project", "1", WORDS, "--key", "first"]),
        call(D1, argv!["wait", "task", task(0), "--until", "replied"]),
        call(D1, argv!["chat", task(0)]),
    ]);
    assert!(ran.calls.iter().all(|c| c.exit == Exit::Code(0)), "{ran:?}");
    assert!(holds(&ran.calls[2].stdout, REPLY), "the scripted reply arrived");
    assert!(ran.llm.iter().any(|o| matches!(o, Observation::Query { .. })));
    assert!(ran.killed.is_empty());
}
```
Proof: world, `cargo nextest run -p temper-real-loop-world --test tools -E
'test(starts_a_chat)'`, under 1 s: real pipes, credential/store files, call exits and
smith's reply, with the full drain checks.
Don't: fake the lock or Saved/Stored; inspect engine or client domain state.
Size: at most 300 lines of harness; no production change.
Removes: nothing.

### 08.5 The forge host's delivery meets the engine's listener
Scope: domain/testing.md 2.4; the first joint proof of temper 07.3.
Needs: the webhook address (08.1); the hosted calls (08.4).
Files (forge wiring, no fake implementation change): `tests/real-loop/src/forge.rs`,
`tests/real-loop/src/lib.rs`, `tests/real-loop/src/fixtures.rs`,
`tests/real-loop/src/service.rs`, `tests/real-loop/src/world.rs`,
`tests/real-loop/src/check.rs`, `tests/real-loop/tests/deliveries.rs`,
`tests/real-loop/Cargo.toml`.
Shape: host temper 07's `ForgeHost` on port 0 with its shared `Forge`. `forge::fixture`
returns a recipe, constructed inside its owner's span: one-process's `stories::forge`
setup, acme/app, id 2, worker write permission, now hooked. Use one webhook secret. Once
`Service::hooks_address` is Some, set `Forge::deliver_to`; only then introduce a fixture
push hook through the domain's existing scenario API. Capture completed writes on the
accepted webhook connection in `ServiceHost`: a signed delivery must elicit HTTP 204 from
the engine. Polling cannot manufacture that response. This needs no adopted repository;
09.6 also needs the tools' adopt for adoption and the landing's domain effects.
`run_forge(forge, configure, scripts, steps)` shares the normal runner; `configure` changes
the fixture document after the forge binds. Default `forge.client.poll_ms`, `poll_max_ms`,
`slow_ms` are 10, 50, 100, respectively, beside the shipped values and reasons in fixtures;
startup checks their combination. Keep all other teardown times. Meter shared-domain
construction, scenario changes, observation drains and final destruction under the
forge-family owner, with its bound including `temper_fake_forge_domain::worst_case` and the
host. Drop all factory handles before dropping that owner's last handle. 09 adds git's host
bound to the same owner; no allocations move unaccounted between it and the service. `Seen`
retains outside forge observations for 09's referee.
Test shape: `tests/deliveries.rs`, counting allocator; the existing protocol world's push
fixture supplies the hook's values, not its bytes.
```rust
#[test]
fn the_forge_host_s_signed_delivery_is_accepted_by_the_engine() {
    let seen = forge::deliver_push();
    assert_ne!(seen.forge.port(), 0);
    assert_ne!(seen.hooks.port(), 0);
    assert_eq!(seen.responses, [204]);
    assert!(seen.drained);
}
```
Proof: joint, `cargo nextest run -p temper-real-loop-world --test deliveries`, under
0.5 s: the host's signed bytes cross the real socket to the engine's component; all owners
and descriptors release.
Don't: change fake forge/client/component files; take 09's fake-git probe or landing;
assume HTTP 204 proves an adopted repository changed.
Size: at most 230 lines of harness; no production change.
Removes: nothing.

### 08.6 Another client's act satisfies a running wait
Scope: client/tools.md 11.3's concurrent clients; finish the reusable caller.
Needs: the hosted calls (08.4); the tools' waits.
Files: `tests/real-loop/src/{calls.rs,service.rs,lib.rs}`; `tests/real-loop/tests/tools.rs`.
Shape: `Start`, `Watching` and `Join` let a call remain in flight. `Watching` waits for the
first task delivery on that call's native wire, observed in completed service writes and
decoded with the existing jig wire codec; it observes no private root state. Start a wait
for `ended` on D1's replied chat, observe its first not-ended delivery, then close that chat
from D2, signed in as the same party. The wait exits 0 with the ended document. No timing
race is made into a sleep.
Test shape: tools.rs, 08.4's fixtures and counting allocator:
```rust
#[test]
fn another_clients_close_meets_a_running_wait() {
    let ran = run(chat_scripts(1), &[
        sign_in(D1), same_party(D2, D1),
        call(D1, argv!["chat", "--project", "1", WORDS, "--key", "waiting"]),
        call(D1, argv!["wait", "task", task(0), "--until", "replied"]),
        Step::Start { at: D1, call: 2, argv: argv!["wait", "task", task(0), "--until", "ended"] },
        Step::Watching { call: 2 },
        call(D2, argv!["close", task(0), "--key", "close-waiting"]),
        Step::Join { call: 2 },
    ]);
    assert!(ran.calls.iter().all(|c| c.exit == Exit::Code(0)), "{ran:?}");
    assert!(holds(&ran.calls[2].stdout, b"ended"));
    assert!(ran.calls[2].spawned < ran.calls[3].spawned);
    assert!(ran.calls[2].ended > ran.calls[3].spawned);
}
```
Proof: world, `cargo nextest run -p temper-real-loop-world --test tools -E
'test(another_clients)'`, under 1 s, including drain.
Don't: change the tools' waits or cli host to tell the harness readiness.
Size: at most 140 lines of harness; no production change.
Removes: nothing.

## Interface sketches

`Ran` owns copied observations after owner release, reaping and the teardown check.

```rust
// crates/temper-engine-protocol-forge/src/component.rs; service/src/system.rs
impl Component { pub fn hooks_address(&self) -> Option<core::net::SocketAddr>; }
impl Service { pub fn hooks_address(&self) -> Option<core::net::SocketAddr>; }

// tests/real-loop/src/lib.rs: Script, Observation, Exit, Killed are re-exports.
/// Scratch files owned by the story; kept on failure, reusable between runs.
pub struct Scratch { /* .. */ }
impl Scratch { pub fn new() -> Self; pub fn path(&self) -> &std::path::Path; }
pub fn run(scripts: Box<[Script]>, steps: &[Step]) -> Ran;
pub fn run_on(scratch: &Scratch, scripts: Box<[Script]>, steps: &[Step]) -> Ran;
pub fn run_forge(forge: forge::Fixture, configure: fn(&mut serde_json::Value, SocketAddr), scripts: fn(SocketAddr) -> Box<[Script]>, steps: &[Step]) -> Ran;
/// Frozen results; call indices include Starts and exclude sign-ins.
pub struct Ran { pub calls: Vec<Called>, pub killed: Vec<Killed>, pub llm: Vec<Observation>, pub forge: Seen }
impl Ran { pub fn number(&self, call: usize) -> String; }
/// A waited-for invocation, including both pipes and monotonic times.
pub struct Called { pub exit: Exit, pub stdout: Vec<u8>, pub stderr: Vec<u8>, pub spawned: Time, pub ended: Time }
// tests/real-loop/src/forge.rs: Config and Domain are the existing fake forge's.
/// A recipe for shared state constructed and destroyed under the forge family's meter.
pub struct Fixture { pub config: Config, pub seed: u64, pub setup: fn(&mut Domain, &Config) }

// tests/real-loop/src/calls.rs
/// A fixture directory, D1 or D2, below Scratch; never an open descriptor.
pub struct Dir(pub u8);
pub const D1: Dir = Dir(1); pub const D2: Dir = Dir(2);
/// Ordered argv words; literals, a prior task number or a prior tree's delegate.
pub struct Argv { /* .. */ }
pub fn task(call: usize) -> Arg;
pub fn delegate(call: usize) -> Arg;
pub fn sign_in(at: Dir) -> Step;
pub fn same_party(at: Dir, from: Dir) -> Step;
pub fn call(at: Dir, argv: Argv) -> Step;
/// Caller instructions; 09 adds Held, Probe, Release, Kill, KillHeld, JoinOther.
pub enum Step {
    /// Consume the printed link on stdin and wait for sign-in to finish.
    SignIn { at: Dir },
    /// Copy a settled sign-in, keeping the destination's request store empty.
    SameParty { at: Dir, from: Dir },
    /// Spawn and wait for this call and its output pipes.
    Call { at: Dir, argv: Argv },
    /// Spawn without waiting; the id is its index in Ran.calls.
    Start { at: Dir, call: u8, argv: Argv },
    /// Wait for the started call's first subject delivery on its native wire.
    Watching { call: u8 },
    /// Reap a started call and finish draining its output pipes.
    Join { call: u8 },
}
```

`argv!` accepts strings, byte slices and `Arg`; helpers `holds`, `rows`, `numbers`,
`kept_lines` decode the tools' published text/JSON, and `Seen` supplies `merged_green` and
`main_file` from fake-forge observations. 09 supplies its new `landing`, `repository`, `git`
and `cue` modules. Its Kill uses `Op::Signal { pidfd, signal: Signal::Kill, to:
Target::Child }`, then `Op::Wait`; `Running::kill` and `Outcome.killed` already support it.

## Questions to raise when reached

- A profile admission or owner ledger fails: report its inputs and the first failing
  allocation span; never switch memory checks off.
- The signed delivery has no 204: retain both socket traces and the forge's hook
  observation; fix at the lowest tier that reproduces it.
- A story exceeds 1 s or the suite's time aim: profile it unoptimized, report the cause and
  times before moving it to `fuzzy_*`.

## Done when

Each in `~/src/rust/temper`, after `git fetch`:

- `git grep -q -F 'hooks_address' origin/main -- crates/temper-service crates/temper-engine-protocol-forge`
- `git grep -q -F 'real::World' origin/main -- tests/real-loop`
- `git grep -q -F 'Span::start' origin/main -- tests/real-loop`
- `git grep -q -F 'temper_client_shell_cli' origin/main -- tests/real-loop`
- `git grep -q -F 'Op::Wait' origin/main -- tests/real-loop`
- `git grep -q -F 'deliver_to' origin/main -- tests/real-loop`
- `git grep -q -F 'forge.client.poll_ms' origin/main -- tests/real-loop`
- `test -z "$(git grep -n -E '127\.0\.0\.1:[1-9][0-9]+' origin/main -- tests/real-loop)"`
