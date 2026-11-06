+++
id = "chosen-ports"
part = "ports"
kind = "feature"
needs = []
files = [
  "temper/testing/temper-fixtures/src/lib.rs",
  "temper/testing/temper-fixtures/src/ports.rs",
  "temper/testing/temper-fixtures/src/ports/tests.rs",
  "temper/crates/temper/src/fake_backends.rs",
  "temper/crates/temper/src/fake_backends/tests.rs",
  "temper/crates/temper/src/tests/fake_box_ports.rs",
  "temper/crates/temper/tests/support/mod.rs",
  "temper/crates/temper/tests/support/boxed.rs",
  "temper/crates/temper/tests/support/forge_land.rs",
  "temper/crates/temper/tests/suite/fake.rs",
  "temper/crates/temper/tests/suite/ports.rs",
  "temper/crates/temper/tests/suite/main.rs",
  "temper/tests/client-browser/src/configuration.rs",
  "temper/tests/client-browser/src/lib.rs",
  "temper/tests/real-loop/src/service.rs",
  "temper/crates/temper/Cargo.toml",
  "temper/tests/client-browser/Cargo.toml",
  "Cargo.lock",
  "temper/docs/design/domain/testing.status.md",
  "temper/docs/design/protocol/setup.status.md",
]
effort = "high"
minutes = 50
+++

# A port a test listens on again is its own

## Goal

A story's box lost its retained port on a restart, at three process-tier threads: `port 45949 (127.0.0.1:45949): cannot listen: AddressInUse`. The port came from `support::own_listener_files`, a kernel draw released at once, so it was in the ephemeral range and nobody held it while the box was stopped. Make a port a test lets go of and listens on again come from below that range, from a block its process holds for its whole life, so that no other test process on the machine can draw it. A fake box takes its ports from the story the same way. A person's box and a person's fake box stay as they are.

Acceptance:

- `temper_fixtures::ports` exists as sketched, and its tests below pass.
- `support::free_addresses` and `support::reserve_listeners` keep their signatures and draw from it; no caller of theirs changes.
- Every site in *Sites that change* is changed.
- A story's fake box gets its nine ports through `TEMPER_FAKE_BOX_PORTS`; with the variable unset, `prepare` behaves as it does now (`a_fresh_fake_box_writes_none_of_the_persons_preferred_ports` passes unchanged but for its new argument).
- `heavy cargo check --tests` passes for `temper`, `temper-real-loop-world` and `temper-client-browser-world`. No process, browser or live story runs (`notes/common.md`).
- The two **Not yet** entries of `2026-11-06` naming `temper_fixtures::ports` (`testing.status.md`) and `TEMPER_FAKE_BOX_PORTS` (`setup.status.md`) are gone: this task's one departure from *Not yours*, each file edited only for that.

## Inputs

- `temper/docs/design/domain/testing.md`, 2.6, *Three at a time*; 2.4, *The clients' listener's address is fixed*.
- `temper/docs/design/protocol/setup.md`, 7.5.
- `temper/docs/plan/process-ports/notes/common.md`.
- `temper/docs/plan/process-ports/plan.md`, *What the survey found*: why neither a nextest slot nor a lock file under `target/`.

## Owns and may change

Owns: the `files` above.

May also change, and how:

- `temper/crates/temper/tests/process.rs`, `support/sign_in.rs`, `support/restart.rs`, `suite/startover.rs`, `suite/forge_land.rs`: a `support::own_listener_files` call only. Each `temper init --box` a story then starts a box on must be followed by one. The survey found these so: `process.rs`'s two, `support/restart.rs`'s two, `suite/forge_land.rs`, `suite/startover.rs`, and `forge_land::fake_deployment` through `sign_in::patch_configuration`. `forge_land::live_deployment` isn't, and is in *Sites that change*. If you find another that isn't, add the call and say so.
- `temper/crates/temper/tests/suite/box_listeners.rs` and `temper/crates/temper/src/init/tests.rs`: untouched unless the compiler demands it. They test what init chooses for a person.

Options, each with its default:

- If a site in *Sites that stay* does let go of its port and listen on it again, change it as the others, and say so.
- If a port in `temper_fixtures::ports` proves to need more than a `Mutex`-guarded static to be shared across a test's threads, keep the `Mutex`: the process tier's stories draw on one thread, and a poisoned lock may panic.
- If `temper/crates/temper/tests/support/boxed.rs` has a second start of a fake box than the one at `dying_with_test(Command { .. })` under `let fake_backends = ..`, it gets the same variable.
- `landing-fixes-2/port-reservation`, not yet on main when this was cut, changes the body of `reserve` in `fake_backends.rs` and adds `reserve_from`. Leave `reserve`'s body alone: add beside it, so the two merge. If it landed before you start, use what it left: the same rule.

Not yours: `temper/crates/temper/src/init.rs` (a person's choice of ports), `.config/nextest.toml`, jig's and skein's own tiers, `temper/tools/**`.

## Steps

### 1. `temper_fixtures::ports`

`temper/testing/temper-fixtures/src/ports.rs`, with `pub mod ports;` in `lib.rs`, ordinary Rust on `std` alone:

```rust
//! Loopback ports a test lets go of and listens on again (temper/docs/design/domain/testing.md, 2.6).

/// `N` distinct listeners, none at an address in `taken`, on ports below the
/// kernel's ephemeral range, in blocks this process holds until it ends.
pub fn reserve<const N: usize>(taken: &[SocketAddr]) -> [TcpListener; N];

/// As [`reserve`], with the listeners let go: nothing listens at these
/// addresses now, and no other test process will draw them.
pub fn addresses<const N: usize>(taken: &[SocketAddr]) -> [SocketAddr; N];

/// The first port of the kernel's ephemeral range. Reads
/// `/proc/sys/net/ipv4/ip_local_port_range`; where that file doesn't exist
/// (macOS, temper/docs/design/domain/testing.md, 2.8), 49152,
/// `net.inet.ip.portrange.first`'s default.
pub fn ephemeral_first() -> u16;
```

Inside, a private chooser the tests can make more than one of, standing for more than one process, since what excludes them is a kernel socket:

```rust
struct Blocks { claims: Vec<TcpListener>, next: Range<u16>, first: u16, end: u16 }
impl Blocks {
    fn new(ephemeral_first: u16, start: u32) -> Self;
    fn listener(&mut self, taken: &[SocketAddr]) -> TcpListener;
}
static BLOCKS: Mutex<Option<Blocks>> = Mutex::new(None);
```

- **Range:** `LOWEST = 10_000` (above the sign-in callback's 1455 and init's preferred 8367 to 8371) up to `ephemeral_first()`, exclusive. Parse the file in a pure `fn first_of(text: &str) -> Option<u16>`. Fewer than two blocks in that range panics, naming the file and what it read; a machine with such a range wants saying, not working around.
- **A block** is `BLOCK = 64` consecutive ports. A chooser claims one by binding a listener on its first port and pushing that listener onto `claims`, never dropped: the kernel releases it when the process ends, however it ends, and `std`'s listeners are close-on-exec, so a box the test starts doesn't inherit it. A first port someone holds moves to the next block; running off the end wraps; a full circle with no block panics, naming the range.
- **The first block** is `start % blocks`, and the process's `start` is `std::process::id()`, so that test processes starting together don't all contend for one block.
- **A draw** binds the block's next port; one it can't bind is skipped (a leftover of a dead test, or a service on the machine), one in `taken` is skipped too. A block used up claims another. No port is handed out twice in a process's life, so the old "every listener is held until all `N` are drawn" isn't needed.

Tests, `src/ports/tests.rs` (`#[cfg(test)] mod tests;`), each with its rule broken once by hand:

- `the_range_is_read_from_the_kernels_file`: `first_of("32768\t60999\n") == Some(32768)`, `first_of("20000\t60999\n") == Some(20000)`, and garbage gives `None`.
- `a_chosen_port_is_below_the_ephemeral_range`: through the public `reserve`, three blocks' worth: every port is at least `LOWEST` and below `ephemeral_first()`.
- `two_processes_choose_no_port_in_common`: two `Blocks` on the same `start`, three blocks' worth each, disjoint.
- `a_released_port_is_no_other_processs_and_listens_again`: one `Blocks` draws a port and drops the listener; a second draws three blocks' worth and never that port; `TcpListener::bind` on it then succeeds.
- `a_block_is_free_once_its_process_ends`: drop a `Blocks`; a new one on the same `start` claims the same first port.
- `a_port_someone_holds_is_passed_over`: hold a listener on a block's next port by hand; the next draw is another port.
- `a_range_too_narrow_says_so`: `#[should_panic(expected = "ip_local_port_range")]` with `Blocks::new(LOWEST, 0)`.

### 2. The story's support

`temper/crates/temper/Cargo.toml`, `[dev-dependencies]`: `temper-fixtures.workspace = true`. `temper/crates/temper/tests/support/mod.rs`:

```rust
/// `N` distinct loopback addresses that no listener holds now, none of them
/// in `taken`, which a story may let go of and listen on again
/// (temper/docs/design/domain/testing.md, 2.6). Two equal listeners fail
/// temper-forge-setup's `check.rs:112`.
pub fn free_addresses<const N: usize>(taken: &[SocketAddr]) -> [SocketAddr; N] {
    temper_fixtures::ports::addresses(taken)
}
pub fn reserve_listeners<const N: usize>(taken: &[SocketAddr]) -> [std::net::TcpListener; N] {
    temper_fixtures::ports::reserve(taken)
}
```

`own_listeners`' comment says "a kernel-drawn address": say "a chosen address", no more.

### 3. Sites that change

| Site | Now | Becomes |
| --- | --- | --- |
| `temper/crates/temper/tests/suite/fake.rs`, `prepared_forge` | binds 0, drops, the owned forge listens there | `support::free_addresses::<1>(&[])[0]` |
| `temper/crates/temper/tests/support/forge_land.rs`, `live_deployment` | init's own ports, a kernel draw if another box holds 8367, and the live story's sittings restart on them | `super::own_listener_files(&paths[0]);` directly after `initialize(paths);`, before its edits, as `fake_deployment` has through `sign_in::patch_configuration`; compiled, not run |
| `temper/tests/client-browser/src/lib.rs`, `reserved` | binds 0, dropped at its return, the server listens and restarts there | `temper_fixtures::ports::addresses::<1>(&[])[0]` |
| `temper/tests/client-browser/src/configuration.rs`, the four demo sockets | binds 0 four times, dropped at its return | `temper_fixtures::ports::addresses::<4>(&[])`; its comment on keeping reservations goes |
| `temper/tests/real-loop/src/service.rs`, `clients_listener` | binds 0, kept across the bootstrap, dropped at drain, and a restart listens there again | `temper_fixtures::ports::reserve::<1>(&[])`, destructured |

`temper/tests/client-browser/Cargo.toml`: `temper-fixtures.workspace = true` under `[dependencies]`. `temper-real-loop-world` has it already.

### 4. Sites that stay

Each holds its listener from bind to last use, or takes a new kernel port at each start with nothing naming the old one: the survey read them, and they don't change.

`suite/fake.rs` (`no_route`, the inherited listeners, the `:0` documents); `suite/startup.rs`; `suite/relay.rs`; `suite/box_listeners.rs`; `support/relay.rs`; `support/relay_managed.rs`; `support/account_transfer.rs`; `support/story_socket.rs`; `support/fetch.rs`; `support/configuration.rs`, `support/sign_in.rs`, `support/case.rs`, `support/owned.rs` (their `:0` documents); `temper/tests/real-loop/src/service.rs`'s `forge`, `world.rs`'s `reservation`, `world/teardown.rs`, `llm.rs`; `temper/tests/client-browser/src/lib.rs`'s issuer and GitHub listeners, adopted by their peers; `bin/playground-serve.rs`; `temper/crates/temper-forge-setup-shell/tests/suite/fetch.rs`.

### 5. The fake box

`temper/crates/temper/src/fake_backends.rs`, as `crates/temper/src/restart_fault.rs` reads its process-tier switches:

```rust
/// A new sandbox's nine ports, the process tier's (protocol/setup.md, 7.5).
const CHOSEN_PORTS: &str = "TEMPER_FAKE_BOX_PORTS";
struct Chosen { listeners: [u16; 5], peers: [u16; 4] }
fn chosen_ports(value: &OsStr) -> Result<Chosen, String>;
fn prepare(source_dirs: &Directories, bundle: Option<&Path>, adapter: &Path, chosen: Option<&Chosen>) -> Result<Prepared, String>;
```

- `run` reads `std::env::var_os(CHOSEN_PORTS)` and hands `prepare` what `chosen_ports` makes of it; a refusal is its error.
- `chosen_ports` refuses anything but nine comma-separated nonzero ports, naming the variable.
- The new-sandbox path gives `write_with_program` `chosen.listeners` for its `[0; 5]`. Init falls back to a kernel port on a taken one, so compare `written.listeners`' ports with what was named, and refuse a difference, naming the port and the variable, before anything else is written.
- Each peer's `reserve(&excluded)?` becomes a small function beside `reserve`: with a chosen port, a bind of exactly it, its failure naming the port and the variable; without, `reserve(excluded)`. `checked_peers` already refuses a port that isn't distinct. `reserve` keeps its body.
- The retained path never reads `chosen`: a retained sandbox keeps its ports.

`temper/crates/temper/tests/support/boxed.rs`, at the fake box's start: with `fake_backends`, push `("TEMPER_FAKE_BOX_PORTS", ..)` onto the environment, nine ports from `super::free_addresses::<9>(&[])` joined by commas. Every sitting draws nine, and a retained sandbox ignores them; a story's sittings are few.

Tests in `temper/crates/temper/src/fake_backends/tests.rs` (add `None` to `prepare`'s calls there and in `src/tests/fake_box_ports.rs`):

- `a_new_sandbox_takes_the_nine_ports_its_test_named_and_a_restart_keeps_them`: nine from `temper_fixtures::ports::addresses`; `prepare` with them; the five listeners' pointers in `temper.json` and the four `peers` are those ports, in the order of `setup.md`, 7.5; drop it; `prepare` again with nine others: the same document's ports, and all nine bind afterwards.
- `a_named_port_held_elsewhere_refuses_a_new_sandbox_naming_it`: two rows, a listener's and a peer's slot, each with a `TcpListener` holding that port: an error holding the port and `TEMPER_FAKE_BOX_PORTS`.
- `chosen_ports_refuses_all_but_nine_nonzero_ports`: eight, ten, a zero, a word, non-UTF-8.

### 6. A focused test of the support

`temper/crates/temper/tests/suite/ports.rs`, `mod ports;` in `suite/main.rs`:

- `own_listener_files_moves_each_listener_below_the_ephemeral_range`: `temper init --box` as `suite/box_listeners.rs`' `command` does on a scratch, `support::own_listener_files`, then each of the five pointers `support::own_listeners` names is below `temper_fixtures::ports::ephemeral_first()`.

### 7. Checks

Narrowest first:

```
heavy cargo nextest run -p temper-fixtures --lib
heavy cargo nextest run -p temper --lib -E 'test(/fake_backends|fake_box_ports/)'
heavy cargo nextest run -p temper --test suite -E 'test(/^ports::/)'
```

**A change of meaning runs its readers' tests:** what `free_addresses` returns changes, so before the push run the focused tests of its readers and of `reserve`'s: `heavy cargo nextest run -p temper-fixtures -p temper -p temper-real-loop-world`. Compile what doesn't run here: `heavy cargo check -p temper --tests`, `heavy cargo check -p temper-real-loop-world --tests`, `heavy cargo check -p temper-client-browser-world --tests`. Then your standing `cargo fmt`, `old_forms.py` and `heavy cargo clippy -p temper-fixtures -p temper -p temper-real-loop-world -p temper-client-browser-world --all-targets -- -D warnings`, commit and push.

## Report

Beyond the usual: which sites you changed and any of *Sites that stay* you moved; the rule each new test bit on and the failure seen; and the figure read from `/proc/sys/net/ipv4/ip_local_port_range` on the builder you ran on.
