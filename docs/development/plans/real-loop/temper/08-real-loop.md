# Session 08: the real-loop world

`tests/real-loop`, the crate `temper-real-loop-world`: skein's fake LLM
peer on the port the kernel assigns, the standalone service on a scratch
directory with its smith agents hosted in the loop, and `temper ctl`'s
host hosted as one program per call, on one checked real world
(testing.md, 2.4; tools.md, 11.3). The deployment's credential is a
`handed_in` file (decisions.md, design question 1). No forge peer runs
here: the forge's stories are temper 13's (decisions.md, 1 and 2).
temper 09 and 13 extend this world and its fixtures, never redefine them.

Needs at start: nothing outside this session; 08.1 and 08.2 build on
`origin/main` as it is. 08.3 needs temper 05's reads (decisions.md, 5),
with its `sign-in` and its `chat` act; 08.4 needs temper 05's waits and
its `close` act. temper 05's names below are those `05-ctl.md` sketches, created by
temper 05 (`temper-client-shell-cli`'s `host::{OneShot, Startup,
Invocation}`, `arguments::parse`, `limits`): check each as landed.
Steps: `temper-08.1`, `temper-08.2`, `temper-08.3`, `temper-08.4`.

## Read first

- temper's `docs/design/domain/testing.md`: 2.4, 6.1; `docs/design/client/tools.md`:
  5.2 to 5.4, 5.6, 6.1, 6.3, 8.1 to 8.3, 11.3.
- skein's `docs/foundation/development.md`, 5.3 (a new world's cost);
  `testing-strategy.md`: 2.8, 4.1, 6, 7, 8, 10; `programming-model.md`:
  2.1, 4.3, 6.3, 9, 10.2; `kernel.md`, 6.2; `fake-llm.md`, 6.
- This plan's `decisions.md` (design questions 1 and 2; 1, 2, 5) and
  `temper/05-ctl.md` (the host's `Startup` and `Invocation`).
- The code: skein-world's `real` (`World::new_controlled_checked`,
  `spawn_signals`, `host_roots`, `CheckedOutcome`, `Controls::signal`),
  skein's `tests/world/tests/real_memory.rs`, `skein_fake_peers::llm::
  Peer`; smith's `tests/agent-process/src/fake.rs` (`limits`,
  `configured`) and `lib.rs` (`AgentErrors::pipe`); jig's
  `tests/channel/src/fixtures.rs`; temper's `crates/temper/src/lib.rs`
  (`prepare_in`), `crates/temper/src/host.rs`, `crates/temper-service/
  src/system.rs` (`links`, `adopt_signals`, `end`),
  `crates/temper/tests/support/configuration.rs` (`documents`),
  `tests/one-process/src/world.rs` (`Proc`, `agent`, `agent_root`),
  `fixtures.rs` (`provider`, `agent_limits`), `referee/client.rs`.

## What the design asks

- **2.4, what runs:** the service as standalone runs it, smith's agents
  joined by real pipes, the tools' root over jig's native transport,
  skein's fake LLM peer with latency zero, a real scratch directory; and
  "A story never uses a fixed port." 08.1 to 08.3.
- **2.4's stories and 11.3:** sign-in, a scripted chat and reply (08.3);
  a wait satisfied by another client (08.4); a drained shutdown, after
  each story (08.2). A restart, a lost answer and a
  second party are temper 09's (ruling 130); the forge's, temper 13's.
- **11.3, each call a hosted program:** `temper ctl`'s host, one per
  call, on its DIRECTORY; DIRECTORY's files and the call's exit on the
  real kernel (8.1, 8.2). 08.3.
- **5.2, provisional text:** an agent process gives none, so `chat`
  prints none; its words show as the turn commits. 08.3.
- **6.1 and development.md 5.3, the cost:** a story aims at 1 s; a new
  world costs about 1 s focused. 08.2 measures (Questions).

## Choices this plan makes

- **The service is standalone's own preparation.** A root host of the
  world, admitted before the caller (`spawn_signals`), waits until the
  referee hands it the peer's bound address, then writes the scratch's
  files and calls `temper::prepare_in`, as `temper serve` does, and
  adopts its signal pipe (`Service::adopt_signals`). It is admitted
  with the envelope that preparation gives (ruling 104): its spawn
  closure writes the documents with the held forge listener's address
  standing in for the peer's, and takes `temper::envelope_in`'s
  `worst_case` and `operations`, which its `Host` methods return
  throughout; the prepared service's own must equal them. Termination is the
  referee's `Controls::signal(.., ServiceSignal::Terminate)` on that
  pipe; the signal from the OS is the process tier's
  (`temper_serve_prints_its_link_and_drains_on_a_signal`).
- **The link as `serve` prints it.** `ServiceHost` writes each of
  `Service::links()` and a newline to `<scratch>/link` (0600), as
  `crates/temper/src/host.rs` prints it, and sign-in reads that file
  (tools.md 5.6, `sign-in FILE`); `sign-in -` is temper 10's.
- **The forge address is a held loopback listener.** Startup refuses a
  configuration without a forge (`startup/application.rs`). A
  `std::net::TcpListener` on `127.0.0.1:0`, held by `ServiceHost` and
  never accepted, names the forge until temper 13 hosts skein's forge
  peer there. No story here reaches the forge.
- **Plaintext on loopback** (fake-llm.md, 6; startup allows it there).
  TLS to the fake is the process tier's (`support::fake_llm`).
- **The documents are the process tier's, copied.** `fixtures::documents`
  builds `agent.json` and `config.json` as `support::configuration::
  documents` does, with the peer's bound address, plaintext, the
  clients' listener on port 0, the held forge address and the agent's
  program `fixtures::AGENT`. `support` is `crates/temper`'s test module,
  which another crate cannot import; 08.2's commit body names the copy
  for the README's last step.
- **Construction inside the world.** Each process builds its state (the
  peer's domain, the steps, the service) in its spawn closure, factory
  or first turn, so the checked outcome sees each release exactly what
  it allocated; `run` takes constructors, never values. `Ran` borrows
  from the outcome, which verifies release when it drops.
- **The order:** the peer, `ServiceHost`, the caller, then hosted
  children in spawn order, as temper 09's cue needs.

## Rules that bind it

- **Nothing upstream, nothing of `tests/one-process`:** followed, never
  edited or depended on (README, Hotspots); no edit to `crates/temper`
  but 08.2's `envelope_in` (ruling 104).
- **Limits from the builders:** the peer's from smith's `fake::limits`
  and skein-llm-world's `fake::limits`; the agent's operations from the
  shipped profile's builder, as one-process's `agent_limits`; `ctl`'s
  from temper 05's `limits`. None typed.
- **No sleeps for order:** a cue or an `Until`; every story ends by the
  world's `PATIENCE`.

## Increments

### 08.1 The scripted LLM peer on its kernel-assigned address

Scope: the crate, and skein's fake LLM peer on `127.0.0.1:0`, built from
the peer's own types (decisions.md, design question 2).
Needs: nothing outside this session.
Files: `tests/real-loop/{Cargo.toml, src/lib.rs, src/llm.rs,
src/scripts.rs, src/fixtures.rs, tests/peers.rs}` (new, `fixtures.rs`
with `TOKEN` and `SEED` alone, the rest 08.2's; its `Cargo.toml` on the root's
workspace lines `skein-fake-peers`, `skein-fake-llm-domain`,
`skein-fake-llm-protocol`, `skein-llm`, `skein-llm-world`, `skein-world`,
`smith-agent-process-world` and `jig-channel-world`); root `Cargo.toml` (member
`tests/real-loop`) and `Cargo.lock` (hotspot: forge kit).
Shape:
- `[package]` inherits the workspace's fields, `license.workspace =
  true` among them once temper's workspace names a license
  (`code-organization.md`, 1).
- `llm::Recipe` holds `Peer::new`'s arguments after its address, each
  in the peer's own type, so that several credentials, failures per
  credential and the observations the accounts and live-text plans add
  pass through unchanged.
- `llm::scripted(scripts)`: `Transport::Plaintext`; smith's
  `fake::limits()`; `skein_llm_world::fake::config()` and
  `fake::limits(&skein_llm_world::limits())`, adjusted as smith's
  `configured` does; its `provider::Config`; one credential,
  `fixtures::TOKEN`; `domain::Domain::try_scripted(&config, 17,
  scripts)`. Never smith's fixed-port `configured`.
- `llm::peer(recipe)`: `Peer::new` at `127.0.0.1:0`.
- `llm::bound(recipe)`: the peer alone in a checked world; its
  `address()` once io reports it listening, then `shutdown()`.
- `scripts::chat_scripts(n)`: n scripts, each jig's `one_chat(WORDS)`
  answered as one-process's `read_then_wait_provider` answers it, a
  turn of `Text(REPLY)` then `wait({})`, so that the chat stays open for
  its reads (tools.md, 5.4; ruling 118), copied (nothing of
  `tests/one-process` is depended on); `WORDS`, `REPLY`, `EARLY` (REPLY's first word).
Test shape: `tests/peers.rs`, skein-world's `domain::heap::Counting` as
its global allocator; under 0.5 s.
```rust
#[test]
fn the_scripted_peer_listens_on_a_kernel_assigned_port() {
    use temper_real_loop_world::{llm, scripts};
    let bound = llm::bound(|| llm::scripted(scripts::chat_scripts(1)));
    assert!(bound.ip().is_loopback() && bound.port() != 0, "{bound}");
}
```
Proof: world, no higher than its test: nothing calls the peer until
08.2's service does, which proves it.
Don't: edit skein's peer or its domain; copy smith's fixed ports.
Size: at most ~120 lines of `tests/real-loop/src`, by the plan's
command over that directory alone (`git diff -M --numstat
origin/main...HEAD -- tests/real-loop/src`, summed by awk); 0 by part
1's count, which leaves out `tests/`.
Removes: nothing.

### 08.2 The standalone service, checked, drained on termination

Scope: the world, its scratch, `ServiceHost` and the drained shutdown
every story ends with (testing.md, 2.4; tools.md, 11.3).
Needs: nothing outside this session.
Files: `tests/real-loop/src/{fixtures.rs, world.rs, service.rs,
agent.rs, calls.rs, lib.rs}`, `tests/real-loop/tests/lifecycle.rs`,
`tests/real-loop/Cargo.toml` (`temper`, `temper-service`,
`skein-scratch`, `skein-{io,lib,shell}`, `smith-agent-shell`,
`serde_json`); root `Cargo.toml` (workspace entries `temper` and
`skein-scratch`), `Cargo.lock`; `crates/temper/src/{lib.rs,
tests.rs}` (`envelope_in`, ruling 104).
Shape:
- `temper::envelope_in(directory, config) -> Result<(u64, u32),
  String>`: `prepare` of `config` in `directory`, as `prepare_in` runs
  it, then the service's `worst_case()` and `operations()`, the service
  dropped and every descriptor `prepare` opened closed. A unit test in
  `src/tests.rs`: equal to a prepared service's, nothing left open.
- `run_on(scratch, recipe, steps)`: `new_controlled_checked` with the
  story's referee; spawns `Proc::Llm`, `Proc::Service` and
  `Proc::Caller`; `run(&Clock, PATIENCE)`. `run` is `run_on` on a new
  `skein_scratch::Scratch` with `llm::scripted(scripts())`.
- The referee hands `ServiceHost` the peer's address once `address()`
  is `Some`; releases the caller once `<scratch>/link` is written;
  signals the service's Terminate once the caller is done; shuts the
  peer once the service is empty. It allocates nothing of theirs.
- `fixtures::write(scratch, llm, forge)`: the documents, the worker's
  and client parents, `accounts/chat.json` handed in (`TOKEN`,
  `expires_at` 4102444800: jig's `ACCOUNT` expiry has passed); private
  files 0600, directories 0700.
- `ServiceHost` follows `crates/temper/src/host.rs`: it forwards
  completions after observing them, prints the link, ends empty.
- `fixtures::AGENT` hosted as one-process's `world::agent`: config by
  `smith_agent_shell::config::read` of its argv's `agent.json`,
  `AgentErrors::pipe(stderr)`, roots as `agent_root` names them at the
  scratch's worker root. A story's first task spawns it (08.3).
- `Caller` with no steps only waits for its release.
- After every story `run` asserts `End::Drained` with the service's
  `Debug` end shown, and that nothing was killed.
Test shape: `tests/lifecycle.rs`, the counting allocator.
```rust
#[test]
fn a_started_service_drains_on_termination() {
    use temper_real_loop_world::{run, scripts, End};
    let ran = run(|| scripts::chat_scripts(1), Vec::new);
    assert_eq!(ran.served(), Some(&End::Drained), "link {:?}", ran.link());
}
```
The first story: run alone, unoptimized, three times (`cargo
nextest run -p temper-real-loop-world --profile measure -j 1 -E
'test(=a_started_service_drains_on_termination)'`), the median in the
commit body, with the sum, under the same profile, of the packages
testing.md 6.1 counts below the domain. The README's aim for the real
loop and the user's decision on development.md 5.3's ~1 s rest on that
number (Questions).
Proof: joint (the real service, peer and kernel), no higher than its
test: the layer above, the executables, is temper 10's process tier.
Don't: call `crates/temper`'s private `host`; configure an empty
`forges`; name a port; change `[profile.*]`.
Size: at most ~300 lines of `tests/real-loop/src`, counted as 08.1.
Removes: nothing.

### 08.3 The meeting point: a sign-in and a chat to its reply

Scope: `temper ctl`'s host hosted per call: a sign-in, a `chat` act
whose task runs on a hosted agent, then reads until the reply, with no
provisional text before it (tools.md, 5.2, 5.3, 5.6, 8.1, 11.3).
Needs: temper 05's reads (decisions.md, 5), its `sign-in` and its `chat`
act (05.2 to 05.4), temper-05.s1's FILE root (ruling 111), and
`temper-service` taking a peer's end, during a call it handed jig's
engine end, as the call's end (`client/README.md`, section 8, from the
designer's answer of 2026-10-19): temper-08.s1, before 08.3
(ruling 123). Each call's stream is then free once its process exits, so the
story's calls, one after another, stay within the small profile's two
streams.
Files: `tests/real-loop/src/{calls.rs, world.rs, fixtures.rs, lib.rs}`,
`tests/real-loop/tests/tools.rs`, `tests/real-loop/Cargo.toml`;
`tests/client/tests/names.rs`, whose allowed list gains
`temper-real-loop-world` beside `temper-one-process-world` (ruling 127).
Shape:
- `fixtures::CTL` hosted with `host_roots`: the root `parent` is
  DIRECTORY's parent, from the call's argv, and for a call that names
  a FILE, the root `file_parent` is `arguments::file_root(..)`
  (temper-05.s1, ruling 111); the factory makes
  `Startup { parent, file_parent, stdin, stdout, stderr, signals, user:
  skein_shell::effective_user(), seed: fixtures::SEED }` from the
  inherited pipes and signal, `Invocation { directory: DIRECTORY's leaf,
  parsed: arguments::parse(..) }`, then `OneShot::new`. The host opens
  DIRECTORY (temper 05.1's `OpenPrivate`). Instances `CALLS_AT_ONCE` (2);
  operations from temper 05's `limits`.
- The caller spawns each call by `Op::Spawn` (pipes 0, 1, 2), reads 1
  and 2 to EOF, then `Op::Wait`; one step at a time.
- `Step::SignIn { at }`: `sign-in <scratch>/link`. `Step::Call`.
  `Step::Until { at, argv, holds, early }` reruns the call as each ends
  until stdout holds `holds`; `Called.tries` counts the runs and
  `Called.early` those before it that held `early`.
- `Arg::Task(i)` is the task number call i's `done: task N` printed.
Test shape: `tests/tools.rs`, the counting allocator; each story aims at
1 s, measured as 08.2's.
```rust
#[test]
fn a_signed_in_chat_shows_its_reply_only_once_committed() {
    use temper_real_loop_world::{argv, calls::{call, occurrences, sign_in, task, until, D1}, run, scripts, Exit};
    let ran = run(|| scripts::chat_scripts(1), || vec![
        sign_in(D1),
        call(D1, argv!["chat", "--project", "1", "--key", "first", scripts::WORDS]),
        until(D1, argv!["chat", task(1)], scripts::REPLY, scripts::EARLY),
    ]);
    let [signed, chat, read] = ran.calls() else { panic!("three calls") };
    let exits = [signed.exit, chat.exit, read.exit];
    assert_eq!(exits, [Some(Exit::Code(0)); 3], "{}", String::from_utf8_lossy(&chat.stderr));
    assert_eq!((read.early, occurrences(&read.stdout, scripts::REPLY)), (0, 1), "{} reads", read.tries);
}
```
`a_signed_in_directory_is_private_to_its_user`: `sign_in(D1)` alone;
DIRECTORY is 0700, `token` and `address` 0600, each owned by the
effective user (tools.md, 8.1).
Proof: joint, as 08.2's. Reads that finish but never show the reply
are diagnosed in at most 3 world runs and 30 minutes, then asked
(ruling 125).
Don't: pass a descriptor of DIRECTORY to the host; retry a lock or
end a call in the caller (temper 05's); read the link from argv; raise
`CALLS_AT_ONCE`, the profile's streams or the heartbeat, or wait in the
caller, for a stream an ended call held (ruling 118).
Size: at most ~200 lines of `tests/real-loop/src`, counted as 08.1.
Removes: nothing.

### 08.4 Another client's act satisfies a running wait

Scope: a `wait` already watching ends when another call closes the chat
(tools.md, 5.3, 5.4, 8.3, 11.3).
Needs: temper 05's waits (05.6) and its `close` act (05.5).
Files: `tests/real-loop/src/{calls.rs, service.rs, world.rs}`,
`tests/real-loop/tests/tools.rs`.
Shape:
- `Step::Start { at, argv }` starts a call and goes on; `Step::Join {
  call }` waits for its end, kept at its index. Two calls at once on D1:
  a wait takes no lock (8.3).
- `Step::Watching { call }`: the referee arms `ServiceHost` once call is
  started; `ServiceHost` decodes its sends on connections accepted after
  arming, with a decoder after one-process's `referee/client.rs::Wire`,
  until a task delivery; the referee then releases the caller.
Test shape: `tests/tools.rs`, beside 08.3's story, the counting
allocator.
```rust
#[test]
fn another_clients_close_meets_a_running_wait() {
    use temper_real_loop_world::{argv, calls::{call, join, sign_in, start, task, watching, D1}, run, scripts, Exit};
    let ran = run(|| scripts::chat_scripts(1), || vec![
        sign_in(D1),
        call(D1, argv!["chat", "--project", "1", "--key", "waiting", scripts::WORDS]),
        call(D1, argv!["wait", "task", task(1), "--until", "replied"]),
        start(D1, argv!["wait", "task", task(1), "--until", "ended"]),
        watching(3),
        call(D1, argv!["close", task(1), "--key", "close-waiting"]),
        join(3),
    ]);
    let (wait, close) = (&ran.calls()[3], &ran.calls()[4]);
    assert_eq!([wait.exit, close.exit], [Some(Exit::Code(0)); 2], "{}", String::from_utf8_lossy(&wait.stderr));
    assert!(wait.ended > close.spawned, "the wait ended at {:?}, before the close was asked", wait.ended); // ruling 134: two exits after one commit have no order
}
```
Proof: joint, as 08.2's.
Don't: order the close by time or a sleep; take the lock for a wait.
Size: at most ~150 lines of `tests/real-loop/src`, counted as 08.1.
Removes: nothing.

## Interface sketches

What temper 09 and 13 extend: 09 adds `D2`, `same_party`, `number`,
its steps (`SameParty`, `Held`, `Kill`, `Probe`, `Release`, `Contended`; ruling 130)
and its cue on `ServiceHost`; 13 the forge peer at `fixtures::write`'s
forge address, after forge-kit's `skein-05.5` and `temper-05.2`.
```rust
// tests/real-loop/src/lib.rs; `Exit`, `Time`, `End` and `Scratch` are re-exports.
pub fn run(scripts: fn() -> Box<[api::Script]>, steps: impl FnOnce() -> Vec<Step>) -> Ran;
pub fn run_on(scratch: Scratch, recipe: impl FnOnce() -> llm::Recipe, steps: impl FnOnce() -> Vec<Step>)
    -> Ran;
pub struct Ran { /* the checked outcome and the scratch */ }
impl Ran {
    pub fn calls(&self) -> &[Called];
    pub fn served(&self) -> Option<&End>;
    pub fn link(&self) -> &[u8];
    pub fn llm(&self) -> &[skein_fake_peers::llm::Observation];
    pub fn killed(&self) -> &[skein_world::real::Killed];
    pub fn path(&self) -> &std::path::Path;
    /// Verifies release, then keeps the scratch for another run (09's restart).
    pub fn into_scratch(self) -> Scratch;
}
/// One call, every step's in step order: sign-ins, Calls, Untils' last run and Starts.
pub struct Called {
    pub exit: Option<Exit>, pub stdout: Vec<u8>, pub stderr: Vec<u8>,
    pub spawned: Time, pub ended: Time, pub tries: u32, pub early: u32,
}

// tests/real-loop/src/llm.rs
pub struct Recipe {
    pub transport: Transport, pub limits: Limits, pub config: provider::Config, pub credential: skein_llm::Credential,
    pub protocol_limits: provider::Limits, pub domain: domain::Domain, pub domain_limits: domain::Config,
}
pub fn scripted(scripts: Box<[api::Script]>) -> Recipe;
pub fn peer(recipe: Recipe) -> Result<skein_fake_peers::llm::Peer, skein_fake_peers::Error>;
pub fn bound(recipe: impl FnOnce() -> Recipe) -> std::net::SocketAddr;

// tests/real-loop/src/scripts.rs
pub const WORDS: &str = "Hello";
pub const REPLY: &[u8] = b"World answer";
pub const EARLY: &[u8] = b"World";
pub fn chat_scripts(chats: usize) -> Box<[api::Script]>;

// tests/real-loop/src/fixtures.rs
pub const AGENT: &[u8] = b"/real-loop/temper-agent";
pub const CTL: &[u8] = b"/real-loop/temper-ctl";
pub const TOKEN: &[u8] = jig_channel_world::fixtures::TOKEN;
pub const SEED: u64 = jig_channel_world::fixtures::SEED;
pub fn documents(scratch: &Path, llm: SocketAddr, forge: SocketAddr) -> (serde_json::Value, serde_json::Value);
pub fn write(scratch: &Path, llm: SocketAddr, forge: SocketAddr) -> std::io::Result<()>;

// tests/real-loop/src/calls.rs
pub struct Dir(pub u8);
pub const D1: Dir = Dir(1);
pub enum Arg { Lit(&'static str), Task(usize) }
pub struct Argv(pub Vec<Arg>);
pub enum Step {
    SignIn { at: Dir }, Call { at: Dir, argv: Argv },
    Until { at: Dir, argv: Argv, holds: &'static [u8], early: &'static [u8] },
    Start { at: Dir, argv: Argv }, Watching { call: usize }, Join { call: usize },
}
pub fn sign_in(at: Dir) -> Step;
pub fn call(at: Dir, argv: Argv) -> Step;
pub fn until(at: Dir, argv: Argv, holds: &'static [u8], early: &'static [u8]) -> Step;
pub fn start(at: Dir, argv: Argv) -> Step;
pub fn watching(call: usize) -> Step;
pub fn join(call: usize) -> Step;
pub fn task(call: usize) -> Arg;
pub fn occurrences(haystack: &[u8], needle: &[u8]) -> usize;
// `argv!` makes an `Argv` of literals and `task(..)`.

// tests/real-loop/src/service.rs
/// The standalone service as a root host; observes completions before forwarding them.
pub struct ServiceHost { /* path, signal pipe, held forge listener, service, link */ }
```

## Questions to raise when reached

- **08.2's median wall time,** whatever it is: to the supervisor, for the
  README's aim and the user's decision (development.md, 5.3). A story
  over 1 s: profiled, then raised with what it waits on, before any
  merge or fuzzy move. The real loop's stories together past
  testing.md 6.1's 6 s hold no story within its 1 s (README, Rules, "The
  worlds' sums").
- **The held forge stalls the service:** startup, the link, the chat or
  the drain waits on the unanswered forge address. Raise it with the
  times; never a fixed port, never skein's forge peer before temper 13
  without a ruling.
- **`prepare_in` in a root's first turn** fails the checked world's
  exact release, its admission bound or its descriptor ledger: raise it
  with the report, before any check is switched off.
- **temper 05's `OneShot::new` or `limits`** differ from `05-ctl.md`'s
  (`Startup`, `Invocation`): follow the landed code; raise a missing
  capability.

## Done when

Each prints what it says, on `origin/main`, after `git fetch`:

- `git grep -n -F '"tests/real-loop"' origin/main -- Cargo.toml`
  prints a line.
- `git grep -c -E 'fn (the_scripted_peer_listens_on_a_kernel_assigned_port|a_started_service_drains_on_termination|a_signed_in_chat_shows_its_reply_only_once_committed|a_signed_in_directory_is_private_to_its_user|another_clients_close_meets_a_running_wait)\(' origin/main -- tests/real-loop/tests`
  prints counts that sum to 5.
- `git grep -n -F -e 'new_controlled_checked' -e 'temper::prepare_in' -e 'temper_client_shell_cli::' origin/main -- tests/real-loop/src`
  prints at least three lines.
- `git grep -n -E '127\.0\.0\.1:[1-9]' origin/main -- tests/real-loop`
  prints nothing.
