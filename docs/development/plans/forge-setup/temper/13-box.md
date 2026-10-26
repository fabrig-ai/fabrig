# Session 13: `temper box`

temper 12 built the box's machine and its host (`temper-box`); this
session puts them behind a command and proves them on real processes.
`temper box CONFIG.json` reads the document, checks it, asks
`temper-forge-setup` for its steps, and runs `temper-box`'s host over them
until a stop. Then the process tier runs the real binary on an owned
Forgejo: the sign-in link on its stdout, each other line on its stderr
under its step's prefix, Forgejo answering, a stop that leaves no
process (13.1); Forgejo killed and restarted with the engine still
serving (13.2). Needs at start: temper 12.2, 10.1 and 10.2 (`tier_configuration`, `binaries()`).

## Read first

- **Design:** setup.md 7.1 to 7.3 (what the box runs, does and names), 8
  (what `temper box` checks and does not read), 6.3 (a later preparation
  takes Forgejo's lock alone), 13.4 (the process tier's `temper box`
  bullet); domain/testing.md 2.6 (its fifth test); configuration.md 2 and 8
  (every command that reads the document checks it, the box's limits
  among it); protocol.md 4 (stdout carries sign-in links only).
- **Decisions:** 5 (the `box.` values), 6 (how the setup is checked), 9
  (`current_exe`, CONFIG made absolute), 16 (no live service), 22 (13.2's
  Size counts the tier's support), 24 (`box.` overrides are unknown until
  13.1).
- **Findings:** configuration-and-binary.md 4 (the binary), forge-and-tiers.md 4
  (the process tier today).
- **Code:** `crates/temper/src/lib.rs:22-75` (`USAGE`, `Command`,
  `configure`, `run`), `:352-396` (`close`, `prepare_in`, `serve`: the
  model to copy); `tests/support/mod.rs:82-111` (`serve`,
  `printed_link`, `Ended`), `:197-335` (`Process`, `Judge`, `run_signal`);
  skein-world at temper's pin 8a10437: `end_to_end.rs:140`
  (`start_with_tree`), `:266` (`tree_status`), `tree.rs:61-68`
  (`Expectation`), `:83-93` (`TreeStatus`).
- **skein:** programming-model.md 2.1 (no `std::process` in step or shell
  code); testing-strategy.md 2.8 (a process tree that ends), 2.9 (end to
  end), 7 (the referee).
- **Other sessions, by capability:** `Step` and `steps` (temper 07.1); the
  binary's check of the document, which every command calls (07.3);
  `temper forge prepare` (08.1), `run` (08.2) and `reconcile` (10.1), the
  commands the steps name; `temper-box`'s `box.` limits (`small`, `apply`,
  `check`) and machine (12.1) and host (`Settings`, `Service`; 12.2); the
  owned configuration the process tier writes, apart from its start of
  Forgejo (10.2, `support::owned::tier_configuration`, in 08.1's
  `support/owned.rs`).

## What the design asks

- **setup.md 7.1, 7.2:** the tree of children and the policy (order,
  restarts, give-up, stop, streams, process groups) are temper 12's. This
  session shows them on real processes and adds none.
- **7.3, 8, configuration.md 2, 8:** the binary checks the document as
  section 8 says (structure, members, limits, `setup`; no private file
  but CONFIG.json), the box's limits among them, as every command does;
  asks for `steps`; hands them to `temper-box`; names nothing else of the
  setup.
- **7.2's streams:** `temper serve`'s stdout, as it is, is the box's;
  every other line goes to the box's stderr, prefixed `<step>: ` or, the
  box's own, `box: `.
- **13.4, domain/testing.md 2.6:** the box's test: the link on stdout; Forgejo
  killed and restarted, the engine still serving; a stop that leaves no
  process the box started.

## Choices this plan makes

- **The module is `box_command`** (`box` is a keyword). `configure` takes
  `box` with one path, as `serve` and `agent` (lib.rs:42-62).
- **Every command checks the box's limits** (decision 6; configuration.md
  2): 13.1 adds them to the check 07.3 made for every command, so that
  `temper serve` and `temper box` refuse the same document. `temper box`
  has no check of its own. Until 13.1 the check refuses a `box.` override
  as unknown (decision 24); 13.1 is the increment that wires temper-box's
  checks into it, and accepts `box.` overrides from then on.
- **Two process tests, not the design's one,** so that 13.1 proves itself
  (it has the command, not the kill) and each test shows one behaviour
  (the Don't list): their sum is domain/testing.md 2.6's fifth test. Cost: a
  second box start, about 30 s of the process tier (an estimate: nothing
  times a box start yet).
- **The tier's owned configuration, not the example.** domain/testing.md 2.6 says
  "on the owned example", which is temper 14's, after this session: the
  tests run on 10.2's `tier_configuration`, and 14.1 moves its writer onto
  the example, so that both tests run on it from then.
- **The box's tree is the referee's evidence:** 13.1 asserts
  `live_at_leader_exit` is empty; 13.2 reads `live_pids` to find Forgejo
  and the engine by `/proc/<pid>/exe` and their arguments.
- **"Forgejo answers"** is `GET /api/v1/version` giving 200 (the
  throwaway's readiness, `instance.rs:69`); **"the engine serves"** is the
  clients' listener (the link's address) answering `GET /` with any status
  line, since the design fixes no route's answer.
- **The streams are asserted on real pipes** (12.2's simulator writes no bytes): 13.1 requires each non-empty
  stderr line to start with a step's prefix, `prepare 1: `, `forgejo 1: `, `runner 1: `, `reconcile 1: `,
  `serve: ` or `box: ` (the tier's one forge is number 1, 10.2; the names are 07.1's), and one `runner 1: ` line
  at least. The author reads the first run's stderr to confirm the runner daemon writes one; if it writes
  none, that assertion goes and the landing report says so.
- **The box's binary** is started with `Expectation::Supervise`, a stderr
  bound of 262,144 bytes, not `serve`'s 65,536 (mod.rs:82-94): its stderr
  carries every step's lines, which a failure shows. A harness bound, not
  a profile limit. Its environment is empty, as `serve`'s: the box hands
  each step its own (setup.md 7.2) and `temper forge run` clears it for
  Forgejo (6.4).
- **The kill** is `/usr/bin/kill -KILL <pid>` through `std::process`
  in `tests/support/boxed.rs` (precedent: `/usr/bin/git`, `support/live.rs:140`).
- **Support in a module of its own,** `tests/support/boxed.rs`, a child of
  `support`, so it uses `Process`, `StdoutReader` and `printed_link`
  without widening them. `Judge` (mod.rs:253) stays: it watches
  `temper serve` alone.

## Rules that bind it

- **The binary starts nothing and locks nothing:** no `Command::new`,
  `try_lock` or `exec` in `crates/temper/src`; temper-box's host runs the
  children on skein-io, and 08.1's module holds the lock and the exec
  (decision 7; its boundary test scans `crates/*/src`).
- **stdout carries sign-in links only** (protocol.md 4): the box prints
  nothing of its own there.
- **The process tests take their limits from the shipped profile.** They
  set no `box.` or `setup.` override: the restart waits `box.backoff_ms`
  500 and a stop's grace is `box.grace_ms` 10,000 after each of its two
  signals (decision 5). They need both binaries, exported on a line of their own before the command
  (Proof); `tier_configuration` takes them from 10.2's `binaries()`, which panics naming `TEMPER_FORGEJO`
  or `TEMPER_FORGEJO_RUNNER` when unset, so a test refuses at once, not at a prepare 120 s later.
- **Hotspots** (client-api, findings/client-api.md 2): `src/lib.rs` (its 05.2,
  to about 10-18 14Z), `crates/temper/Cargo.toml` (its 07.4, about 10-18 16Z to
  20Z), `tests/process.rs` and `tests/support/*` (to about 10-18 20Z).
  `Cargo.lock` is never merged by hand: take origin/main's and build again.

## Increments

### 13.1 `temper box CONFIG.json`
Scope: the command (setup.md 7.3, 8; protocol.md 4); the box's limits in the check every command makes
(decision 6); its process proof.
Needs: the session's (12.2's host; 10.1's `temper forge reconcile`, with 08.1's `prepare` and 08.2's `run`
behind it); 07.1's `steps` and 07.3's check; for the process proof only, 10.2's `support::owned::tier_configuration`
(apart from its start of Forgejo) and the `binaries()` it calls, which reads both variables and panics naming the
missing one. Ready check: `grep -n 'pub fn tier_configuration(directory: &Path) -> SocketAddr' crates/temper/tests/support/owned.rs`
prints one line, and `git grep -n 'fn binaries' -- crates/temper/tests/support` prints one.
Files (under `crates/temper/`): `src/lib.rs` (**hotspot, client-api 05.2**: `USAGE` :22, `Command` :26, `configure`
:42, `run` :63, `pub mod` :19); `src/box_command.rs` (new); the file of 07.3's check; `Cargo.toml` (**hotspot,
client-api 07.4**: `temper-box.workspace = true`, 12.1's entry); `src/tests.rs` (the two `USAGE` literals, :7 and
:14, and two tests); `tests/startup.rs`; `tests/process.rs` and `tests/support/{mod.rs,boxed.rs}` (**hotspot**;
`mod.rs` gains `pub mod boxed;`); the root `Cargo.lock`.
**Corrections against main (ruling 134, at temper 4764383ae):** Files gain `crates/temper/src/forge.rs` (:165 and :185
take `super::setup::check(..)`'s result: `(_, checked, _)` once it is a triple); the box's values are
`crates/temper-box/src/limits.rs:40-48`, not the service's `small` profile; `checks.rs:14` is
`temper-service/src/startup/checks.rs:14`; `forge.rs:244-246` is `temper-service/src/forge.rs:233` (the `SecretBytes`
arm) and :262-265; readiness is `instance.rs:107` (`GET /api/v1/version`, called at :75), not :69; skein-world is
pinned at `8a51e10b` (Cargo.lock:1237); `tests/support/mod.rs` takes `pub mod boxed;` first in its group (rustfmt).
Lines: `lib.rs` `USAGE` :24, `Command` :27, `configure` :45-68, `run` :69-82, `pub mod` list :18-22; `support/mod.rs`
`configuration` :20, `Scratch::new` :50, `Process` :199, `Judge` :255.
Shape:
- `USAGE` ends ` | temper box CONFIG.json` (protocol.md 4's order, after the `temper forge` forms of 08.1, 08.2 and
  10.1). `Command::Box { configuration: PathBuf }`, taken in `configure`'s `serve` branch; `run`: `Command::Box {
  configuration } => box_command::run(&configuration).map_err(|why| prefixed("temper box", &why))?`, into the
  shared exit tail (lib.rs:71-74).
- **The box's limits:** 07.3's check hands on the `box.` overrides undecoded and refuses them as unknown (decision
  24). 13.1 wires temper-box's checks in: the check keeps `box.` names from `checked` (whose other names are a
  problem), applies them to `temper_box::limits::small()` (`apply`), then `check`s. `apply`'s `Unknown` and `Range`
  take problems.rs:76-77's words. A breach of two limits (`Unfit::names`) is one line worded as temper-service's
  `against` (checks.rs:14): `application.limits.overrides.box.<first>: must agree with box.<second>`; of one
  (`Restarts`), `...box.restarts: past its bound`, as 05.2's `forge::problems` words `hook_secret_bytes` alone
  (forge.rs:244-246, problems.rs:87, at the base). The check's result carries the `Limits`, which `run` alone reads.
- `box_command::run` (sketch 1), so that everything that can refuse runs before the first process: read CONFIG.json
  (`skein_shell::read_file`, lib.rs:370); the check; the cwd, CONFIG made absolute (`std::path::absolute`, lib.rs:358),
  the program (`std::env::current_exe`, decision 9) and the environment (`std::env::vars_os`, as `NAME=value`: every
  step gets it); `steps` (07.1); then as `serve` (lib.rs:381-395): `open_termination_signals`, the cwd's `open_root`,
  12.2's `Settings` (`plain` `serve`, descriptors 1 and 2) into `Service::new`, `Kernel::open`, `skein_shell::drive`,
  `close(owned)` on each refusal.
- `tests/support/boxed.rs` (sketch 2): `run` starts `temper box config.json` in the scratch directory with
  `Binary::start_with_tree(.., Mode::Pipes, 262_144, Expectation::Supervise, false)`, under a `real::World` whose
  referee waits for the link (as `Judge::act`, mod.rs:260), probes Forgejo, sends `Terminate`, and settles when every
  process is empty; a non-empty `live_at_leader_exit` gets the tree killed (`signal_tree`), so that a failing test
  leaves nothing behind.
Test shape: focused, in `src/tests.rs` and `tests/startup.rs` (precedent `problems_reach_stderr_one_line_each`,
startup.rs:50, with `temper::run(Command::Box { .. })`); the document from `support::configuration(dir,
&Forge::Unreached)` (mod.rs:19), whose limits are the shipped `small` profile (`temper-service/src/limits.rs:247-257`)
plus the fixture's overrides (`support/configuration.rs:43-44`); a test that needs another value names it. Ceilings:
configure rows under 20 ms, the others under 0.5 s; a miss is noted in the landing report, read from nextest's
`PASS [ t ]` lines in the test log. Tests:
1. `box_with_one_path_is_the_box_command`
2. `box_without_exactly_one_path_prints_the_usage` (none, two; the new `USAGE` as a literal)
3. `a_box_configuration_that_is_absent_is_refused_naming_it`: `temper box: configuration absent.json: absent\n`
4. `a_box_configuration_missing_members_is_refused_one_line_each`: `{}` gives `temper box: engine: missing\ntemper box:
   workers: missing\ntemper box: application: missing\n`
5. `a_setup_naming_no_forge_refuses_the_box_before_it_starts`: setup.md 8's managed entry with `"forge": 7`; the
   line starts `temper box: application.setup[0].forge: ` (07.1's reason) and no `forge-7` directory appears.
6. `a_box_limit_the_check_refuses_is_refused_in_one_line`, rows (an override, then the line after `temper box:
   application.limits.overrides.`): `box.backoff_max_ms` 100, below `small()`'s `box.backoff_ms` 500 (12.1):
   `box.backoff_ms: must agree with box.backoff_max_ms`; 300000, `small()`'s `box.window_ms`: `box.backoff_max_ms:
   must agree with box.window_ms`; `box.restarts` 65: `box.restarts: past its bound`; 4294967296: `box.restarts:
   outside its type`; `box.nope` 1: `box.nope: no limit of temper's has this name`. Each line ends `\n`.
7. `a_box_limit_past_its_check_refuses_temper_serve_too`: row 1's document through `prepare_in`; the same
   line, prefixed `temper serve: `.
8. `a_box_limit_within_its_check_is_accepted_by_temper_serve`: `box.restarts` 3 through `prepare_in` is `Ok`
   (decision 24 refused it before).
9. The process test, the reference of the new kind (below).

Reference test, appended to `crates/temper/tests/process.rs` (which already imports `Exit` and `Duration`, :3-4):
```rust
#[test]
fn temper_box_prints_the_link_and_a_stop_leaves_no_process() {
    let scratch = support::Scratch::new("box_stop");
    let forgejo = support::owned::tier_configuration(scratch.path()); // 10.2's; Forgejo's address
    let ended = support::boxed::run(scratch.path(), forgejo, support::boxed::Plan::Stop, Duration::from_secs(120));
    let link = support::printed_link(&ended.stdout).expect("the box's stdout carries the link");
    assert!(skein_lib::bytes::find(link, b"/jig/v1/sign-in/link?secret=").is_some(), "the sign-in link");
    let alone = |line: &[u8]| line.is_empty() || line.starts_with(b"http://127.0.0.1:");
    assert!(ended.stdout.split(|byte| *byte == b'\n').all(alone), "stdout holds links alone");
    let steps: [&[u8]; 6] = [b"prepare 1: ", b"forgejo 1: ", b"runner 1: ", b"reconcile 1: ", b"serve: ", b"box: "];
    let lines = || ended.stderr.split(|byte| *byte == b'\n').filter(|line| !line.is_empty());
    assert!(lines().all(|line| steps.iter().any(|step| line.starts_with(step))), "each stderr line names its step");
    assert!(lines().any(|line| line.starts_with(b"runner 1: ")), "the runner's lines arrive prefixed");
    assert_eq!(ended.forgejo_at_link, Some(200), "Forgejo answered its version when the link came");
    assert_eq!(ended.exit, Some(Exit::Code(0)), "stopped as asked: {}", String::from_utf8_lossy(&ended.stderr));
    assert_eq!(ended.left, Some(Vec::new()), "no process the box started outlived it");
    assert_eq!(support::boxed::status(forgejo, "/api/v1/version"), None, "Forgejo is gone");
    scratch.remove();
}
```
Proof: process; in one shell, the binaries exported on a line of their own, then the command:
`export TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64`
`sh -c 'export NEXTEST_DOUBLE_SPAWN=0; cargo build -p temper --bin temper && cargo nextest run -p temper --profile process -E "test(=temper_box_prints_the_link_and_a_stop_leaves_no_process)"'`
exits 0: `temper box` on the owned configuration prints the link and nothing else on stdout, every stderr line
carries its step's prefix, Forgejo answered 200, SIGTERM ended it with exit 0 and no process left. Bound 120 s (aim
40 s, read from the test's `PASS [ t ]` line: a miss is noted in the landing report). Process is the highest layer
that exists (no live service, decision 16). 12.2's host, proven at joint over the simulator, meets real children,
signals and bytes through pipes here (12.2's Proof names this increment).
Don't: start a process, lock or exec in `crates/temper/src`; print to stdout; read any file but CONFIG.json; name a
Forgejo request, a `Setup` member or `temper-box` internals; give `temper box` a check of its own.
Size: at most 120 lines of production code.
Removes: nothing.

### 13.2 The box restarts a killed Forgejo
Scope: the middle of domain/testing.md 2.6's box test: Forgejo killed and restarted by the
box, the engine still serving. The restart works because a later preparation takes
Forgejo's lock alone while the runner holds its own (setup.md 6.3); a box that gives up
at the preparation (exit 1, naming it) is a defect to fix here, not a question.
Needs: 13.1.
Files: `crates/temper/tests/process.rs` (**hotspot, client-api, to about 10-18 20Z**),
`tests/support/boxed.rs` (13.1's; **hotspot** directory). No production file unless the
test finds a defect (at most 10 lines, outside this Size; more is Question 1).
Shape: `Plan::KillForgejo` and three `Outcome` fields (sketch 2). After the link the
referee's `act` has three phases (a private `Phase`); `status` blocks the world while it
waits, so it runs at most every 200 ms:
1. *Linked:* once the link is printed, `find` the engine and Forgejo in `live_pids` (a
   helper reading `/proc/<pid>/exe` against the temper binary and the canonical pinned
   path, and `/proc/<pid>/cmdline` for the arguments `serve` and `web`). When both are
   found, `kill` Forgejo and remember both pids.
2. *Killed:* when a Forgejo with another pid is in `live_pids` and `status(forgejo,
   "/api/v1/version") == Some(200)`, set `forgejo_pids`; set `engine_served` to whether
   the engine's pid is unchanged and `status(engine, "/")` is `Some(_)` (the engine's
   address is the link's); then `binary.signal(Terminate)`.
3. *Stopping:* wait for the world to settle; copy `live_at_leader_exit` into `left`.
Test shape: `tests/process.rs`; helpers `support::boxed::run` (13.1), `Scratch::new`
(mod.rs:48), `printed_link`. Limits as 13.1: the profile's alone (restart after
`box.backoff_ms` 500). Test:
1. `temper_box_restarts_a_killed_forgejo_while_the_engine_serves`
```rust
#[test]
fn temper_box_restarts_a_killed_forgejo_while_the_engine_serves() {
    let scratch = support::Scratch::new("box_kill");
    let forgejo = support::owned::tier_configuration(scratch.path());
    let kill = support::boxed::Plan::KillForgejo;
    let ended = support::boxed::run(scratch.path(), forgejo, kill, Duration::from_secs(120));
    let (killed, restarted) = ended.forgejo_pids.expect("Forgejo was killed and came back");
    assert_ne!(killed, restarted, "another Forgejo process answers");
    assert_eq!(ended.engine_served, Some(true), "the same engine process answered while Forgejo was back");
    assert_eq!(ended.exit, Some(Exit::Code(0)), "stopped as asked: {}", String::from_utf8_lossy(&ended.stderr));
    assert_eq!(ended.left, Some(Vec::new()), "no process the box started outlived it");
    scratch.remove();
}
```
Proof: process, the same two lines as 13.1 (the export, then the test command) with `-E "test(=temper_box_restarts_a_killed_forgejo_while_the_engine_serves)"`
exits 0: the kill, a new Forgejo answering and the engine's pid unchanged. Bound 120 s
(aim 60 s, read from the test's `PASS [ t ]` line: a miss is noted in the landing report). Process is the
highest layer; the test is itself at it.
Don't: kill anything but Forgejo's pid; set a `box.` or `setup.` override to speed the
test; use `std::process` outside `tests/support/boxed.rs`.
Size: at most 80 lines of the process tier's support (decisions.md 22: the count of
code-organization.md 6 for `crates/temper/tests/support`; the brief's 10 counted
production lines, of which 13.2 has none).
Removes: nothing.

## Interface sketches

Sketch 1, 13.1 (the only production item):
```rust
// crates/temper/src/box_command.rs
use skein_io::kernel::Exit;
use std::path::Path;

/// `temper box CONFIG.json`: checks the document, asks for its steps and runs temper-box's
/// host over them until a stop. `Err` holds the refusal's lines, which `lib::run` prefixes
/// with `temper box: `; `Ok` is the host's exit (0 after a stop it was asked for, 1 when it
/// gave up). A refusal comes before any process starts.
pub fn run(configuration: &Path) -> Result<Exit, String> {
    todo!()
}
```

Sketch 2, 13.1 (13.2 adds `KillForgejo` and the three marked fields):
```rust
// crates/temper/tests/support/boxed.rs
use skein_io::kernel::Exit;
use skein_lib::Duration;
use std::{net::SocketAddr, path::Path};

/// What the referee does once the link is printed.
#[derive(Clone, Copy, Debug)]
pub enum Plan {
    /// Probe Forgejo, then send a termination signal.
    Stop,
    /// Kill Forgejo, wait for another to answer, check the engine, then stop (13.2).
    KillForgejo,
}
/// What the world saw of one box run.
pub struct Outcome {
    /// The box's exit status.
    pub exit: Option<Exit>,
    /// The box's stdout: `temper serve`'s, the sign-in links.
    pub stdout: Vec<u8>,
    /// The box's stderr: every other line, `<step>: ` or `box: ` before it.
    pub stderr: Vec<u8>,
    /// Status of `GET /api/v1/version` on Forgejo's address when the link was printed.
    pub forgejo_at_link: Option<u16>,
    /// Processes still alive when the box's own process ended; empty is a clean stop.
    pub left: Option<Vec<u32>>,
    /// The killed Forgejo's pid and the pid of the one that answered after it (13.2).
    pub forgejo_pids: Option<(u32, u32)>,
    /// Whether the engine, the same process, answered `GET /` meanwhile (13.2).
    pub engine_served: Option<bool>,
}
/// Starts `temper box config.json` in `directory` and drives it by `plan` within `deadline`.
#[must_use]
pub fn run(directory: &Path, forgejo: SocketAddr, plan: Plan, deadline: Duration) -> Outcome {
    todo!()
}
/// The status of a plain `GET target` on `address`; `None` when nothing answers.
#[must_use]
pub fn status(address: SocketAddr, target: &str) -> Option<u16> {
    todo!()
}
```

## Questions to raise when reached

1. **A defect whose fix passes 10 lines** (only if 13.2's test finds one: the box gives up at a later
   preparation, or the runner's lock is held, against setup.md 6.3). Options: the fix lands here in a commit of
   its own, over the tolerance; or it goes to session 08's or 10's plan as a step of that session. Recommended: the
   first, since the box's test found it. Raise it with the failing log, the file and the lines the fix needs. The
   runner's lock and the steps' environment (7.2: the box's own) are settled by the design.

## Done when

1. Both increments landed: `git log origin/main --grep='^Plan: forge-setup$'
   --format='%(trailers:key=Step,valueonly,separator=)' | sort -u |
   grep -c '^temper-13\.[12]$'` prints 2.
2. The focused tests: `cargo nextest run -p temper -E
   'test(/box/)'` ends `13 tests run: 13 passed`.
3. The process tests, in one shell, the binaries exported first:
   `export TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64`
   then `sh -c 'export NEXTEST_DOUBLE_SPAWN=0; cargo build -p temper --bin temper && cargo nextest run -p temper --profile process -E "test(/^temper_box_/)"'`
   ends `2 tests run: 2 passed`.
4. The binary starts nothing: `grep -rnE 'Command::new|try_lock|\.exec\(' crates/temper/src` prints nothing.
5. The usage names the command: `grep -c 'temper box CONFIG.json' crates/temper/src/lib.rs` prints 1.
6. No forbidden word: `grep -rniE 'actual|genuine|legacy|compat|workaround' crates/temper/src/box_command.rs crates/temper/tests/support/boxed.rs` prints nothing.
7. No file past 800 lines: `wc -l crates/temper/src/lib.rs crates/temper/src/box_command.rs crates/temper/tests/process.rs crates/temper/tests/support/mod.rs crates/temper/tests/support/boxed.rs | awk '$2 != "total" && $1 > 800'` prints nothing.
