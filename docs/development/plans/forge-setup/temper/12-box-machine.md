# Session 12: the box's machine and its host

`crates/temper-box` is made: the box's policy as a small machine (steps, exits and timers in;
spawns, signals and closes out), the `box.` limits, and the boundary test that keeps it blind
to the setup (setup.md 7.2, 7.3, 11). 12.1 puts those in place, tested with hand-fed inputs.
12.2 hosts the machine on skein-io's processes: the children's tokens and pipes, their lines on
the box's two output streams, and `skein_shell::Host`. The binary (temper 13.1) hands the host
the steps, the box's environment and the descriptors it opened.

## Read first

- temper `docs/design/`: `setup.md` 7, 11, 13.1; `protocol.md` 4, 5 (the `temper-box` row);
  `configuration.md` 7 (the `box.` rows, the cross-checks), 8. This plan: `decisions.md` 5, 6, 9,
  16; `findings/skein-io.md` 1, 2, 6; `findings/configuration-and-binary.md` 2, 5, 6.
- skein `docs/design/`: `io.md` 6, 7; `shell.md` 12; `simulator.md` 3.2. `docs/foundation/`:
  `programming-model.md` 2, 3, 5.4, 7, 9, 10; `testing-strategy.md` 2.1, 2.7. temper `workflow.md`
  2. Bare skein paths below are `~/src/rust/skein/crates/<crate>/src/`: skein-io, skein-shell,
  skein-sim (`sim.rs`, `sim/{processes,files}.rs`, `trace.rs`), skein-fake-machine under
  `testing/` (`face.rs`, `fs.rs`).
- The model, `crates/temper-worker-protocol-git` (a component on io's processes): `lib.rs:2-4`,
  `component.rs:148-160`, `operation.rs:107-117`, `:332-360`, `:435-457`, `boundary.rs:61-67`,
  `limits.rs:116-124`, `tests/fixtures.rs:91-150`. Limits: `crates/temper-service/src/limits.rs:247-262`,
  `limits/names.rs:4-11`; the passes: `crates/temper-service/src/system.rs:371-448`.

## What the design asks

- **setup.md 7.2 (answers 23-25):** order; a restart runs the once steps just before the step,
  after a backoff counted from its exits within `box.window_ms` (from `box.backoff_ms` again
  after a window without one); a give-up naming the step; a stop of every long step, last first:
  signal, `box.grace_ms`, second signal, `box.grace_ms`, close; `temper serve`'s stdout as is on
  the box's stdout, every other line of a step on its stderr as `<step>: `, the box's own as
  `box: ` (`box: <step> gave up after <n> restarts`); every step the box's environment. **7.3:**
  a machine of steps, exits and timers; no `temper-*` dependency but `temper-forge-setup`, and
  only `steps` and `Step` of it. **configuration.md 7:** five `box.` limits; each backoff at most
  its maximum, `backoff_max_ms` below `window_ms`, `restarts` at most 64.

## Choices this plan makes

- **The machine speaks the box's own words,** `Input` and `Order`, by a step's index and
  nothing of io; the host maps them (7.3: "a small machine of its own").
- **One crate, one lint regime** (answer 22): `[lints] workspace = true`, `no_std` plus alloc as
  the git component; the host stays inside the subset (`skein_shell::Host` by delegation,
  drive.rs:12-49), and all std is the binary's (13.1).
- **A step is over at its child's `Closed`:** the exit is taken at `Exited` and `Close` ordered
  then (it kills the group, io.md 6); next step, restart and give-up follow `Closed`. Signals
  go to the process (`Target::Child`). A `NotStarted` is an exit.
- **The backoff is derived from the window:** with `n` exits within `box.window_ms`, this one
  included, the wait is `min(backoff_max, backoff * 2^(n-1))`, and `n > restarts` gives up. The
  exits are a queue of `restarts + 1` times, so `restarts` is at most `MAX_RESTARTS` (64).
- **A give-up runs the stop sequence, then exits 1;** a stop asked for exits 0. `GaveUp` carries
  the restarts for the box's line, `0` for a once step (setup.md 7.2 has no `n` for it): one pattern
  for every give-up. **`Refused` is the box's own** (7.3). **A constant, not a limit:** `LINE_BYTES`
  4096 (a longer line is written in pieces, each prefixed).
- **The box's stdout is a step's, found by name:** `Settings.plain` (the binary passes `serve`,
  the name `steps` gives `temper serve`); no step of that name, no plain stdout.

## Rules that bind it

- **7.3 is a test:** among `temper-*` crates the box depends on `temper-forge-setup` only, and
  names `steps` and `Step` of it alone, tests included. Children are skein-io's: `std::process`
  and file locks are temper 08.1's module, whose boundary test scans this crate's `src` too.
- **Step code under the subset:** the machine reads no clock and no file; an entry point emits at
  most `max_out()` orders, into a queue reserved for it. Limits come from `limits::small()`,
  another value named beside it with its reason.
- **Ceilings are aims:** a test over its ceiling does not fail; a miss is noted in the landing report,
  read from nextest's `PASS [ t ]` lines in the test log.

## Increments

### 12.1 The machine, the `box.` limits and the boundary test
Scope: `crates/temper-box`: `limits` (configuration.md 7), `machine` (setup.md 7.2), the test of 7.3.
Needs: temper 07.1 (`temper_forge_setup::{Step, steps}`). 7.3 lets the box name those two alone, so
`Step` hides its kind (07.1): `is_long()` gives once-or-long, `Step::new(name, arguments, long)` builds one
for the fixtures. Ready check: `git grep -c -E 'pub (const )?fn (new|is_long)' --
crates/temper-forge-setup/src/step.rs` prints `crates/temper-forge-setup/src/step.rs:2`, else 12.1 adds both.
Files: new `crates/temper-box/{Cargo.toml,src/{lib,limits,machine,tests}.rs}` and
`src/tests/{fixtures,machine,limits,boundary}.rs` (dependencies `temper-forge-setup`, `skein-lib`, `skein-io`,
`skein-shell`; dev `jig-boundary`; `[lints] workspace = true`); root `Cargo.toml` (**hotspot with temper 07.1
and 08.1**: `members`, `temper-box = { path = "crates/temper-box" }` in `[workspace.dependencies]`);
`Cargo.lock` (**hotspot**: origin/main's, then `cargo check -p temper-box --tests`, no `cargo update`).
Users: none (`git grep -n -E 'temper_box|temper-box' -- crates tests testing tools` prints nothing at the base).
Shape:
- `limits.rs`, sketch 1. `apply` takes the whole name (`box.backoff_ms`), as temper-service's (limits.rs:259);
  durations are milliseconds (names.rs:4-9); a value past `u32` is `Range`, any other name `Unknown`.
  `small()`: 500 ms, 30 s, 5, 300 s, 10 s. `check`: `Backoff`, `Window`, `Restarts`, in that order.
  `worst_case`: per step its state, `restarts + 1` times, two timers.
- `machine.rs`, sketch 2; rules by index and once-or-long alone, no name:
  - **start:** `Spawn` step 0; a once step's `Closed` after `Exited(Code(0))` spawns the next, a long step's
    `Started` does; another exit of a once step (in a lane too), or its `NotStarted`, gives up (`restarts: 0`).
  - **restart:** a long step's `Closed` outside a stop records an exit at `now`, drops those older than
    `window`, gives up on more than `restarts`, else arms the wait (Choices). Due, its lane runs: the once
    steps just before it, in order, then it, ending at its `Started`; lanes are disjoint.
  - **stop** (`Terminate`, or a give-up): disarm backoffs, start nothing, stop the live steps last
    first, one at a time: `Signal` and arm the grace; its `Exited` orders `Close`; the grace due
    once orders a second `Signal` and arms again, due twice orders `Close`; its `Closed` moves to
    the previous live step. A step still starting is stopped once started. A second `Terminate`
    (or one during a give-up's stop) orders `Close` for every live step at once.
Test shape: unit tests in `src/tests/{machine,limits,boundary}.rs`, limits from `limits::small()`; the cap
test takes `restarts: 8` beside its 5 (five restarts give up after waits of 0.5 to 8 s, before 30 s).
`src/tests/fixtures.rs` is made here, 12.2 extends it: `owned_steps()` (setup.md 7.1's tree: prepare once,
forgejo long, runner long, two reconciles once, serve long; constants `PREPARE`, `FORGEJO`, `RUNNER`,
`RECONCILE_1`, `RECONCILE_2`, `SERVE`); `step(name, long)`, the one place a `Step` is built; `Rig`, the machine
with its `Queue<Order>` and `now`: `running()`, `started`, `exited`, `closed`, `completed(step)` (a once step
run: started, exit 0, closed), `advance(by)` (calls `fire`), `terminate()`, `orders()` (drains, as `Below`,
fixtures.rs:91-150). Reference test:
```rust
// crates/temper-box/src/tests/machine.rs
use super::fixtures::{FORGEJO, PREPARE, Rig};
use crate::machine::Order;
use skein_io::kernel::Exit;
#[test]
fn a_long_step_that_exits_runs_its_once_steps_again_before_it() {
    let mut rig = Rig::running();
    rig.exited(FORGEJO, Exit::Code(1));
    assert_eq!(&*rig.orders(), &[Order::Close { step: FORGEJO }], "an exit is closed at once");
    rig.closed(FORGEJO);
    rig.advance(skein_lib::Duration::from_millis(500));
    assert_eq!(&*rig.orders(), &[Order::Spawn { step: PREPARE }], "the once step before it first");
    rig.completed(PREPARE);
    assert_eq!(&*rig.orders(), &[Order::Close { step: PREPARE }, Order::Spawn { step: FORGEJO }], "then it");
}
```
Tests: `machine.rs`: `a_once_step_must_exit_zero_before_the_next_starts`,
`a_long_step_lets_the_next_start_once_it_has_started`, the reference,
`a_long_step_that_exits_runs_every_once_step_just_before_it_again` (`SERVE`: `RECONCILE_1`, `RECONCILE_2`, it),
`a_long_step_after_a_long_step_restarts_alone` (`RUNNER`: no once step before it, `FORGEJO` untouched),
`the_backoff_doubles_and_stops_doubling_at_its_maximum`, `the_backoff_starts_again_after_a_window_without_an_exit`,
`more_exits_than_restarts_within_the_window_give_up_naming_the_step` (`GaveUp { restarts: 5 }`),
`a_once_step_that_fails_gives_up_and_stops_what_runs` (`GaveUp { restarts: 0 }`),
`a_stop_signals_the_long_steps_last_first_one_at_a_time`,
`every_long_step_is_signalled_twice_with_a_grace_after_each`,
`a_stop_during_a_backoff_starts_nothing`, `a_step_still_starting_is_stopped_once_started`,
`a_second_termination_signal_closes_every_child_at_once`. `limits.rs` (literals):
`the_small_profile_has_the_five_values_of_the_plan`, `a_backoff_above_its_maximum_names_both_limits`,
`a_maximum_not_below_the_window_names_both_limits`, `restarts_above_sixty_four_name_their_limit`,
`a_name_that_is_no_box_limit_is_unknown`, `a_value_past_u32_is_out_of_range`. `boundary.rs` uses
jig-boundary at the rev `Cargo.lock` locks (755e02e at temper eab91b13; `manifests`, `manifest.rs:106`; normal and target
dependencies) as `tests/client/tests/names.rs:3-9` does; tests run from `crates/temper-box`:
```rust
// crates/temper-box/src/tests/boundary.rs
use jig_boundary::manifest::manifests;
#[test]
fn the_box_depends_on_no_temper_crate_but_the_setup() {
    let root = std::path::Path::new("../..").canonicalize().expect("temper's root");
    let mut found = false;
    for manifest in &manifests(&root) {
        if manifest.name.as_deref() == Some("temper-box") {
            found = true;
            for dependency in &manifest.dependencies {
                if dependency.starts_with("temper-") {
                    assert_eq!(dependency, "temper-forge-setup", "no other temper crate");
                }
            }
        }
    }
    assert!(found, "temper-box's manifest is found");
}
```
and `the_box_names_only_steps_and_step_of_the_setup`: it reads each `.rs` under `src/` (its own file
excepted: its needle is a string) and, at each `temper_forge_setup`, requires `::` then `steps`, `Step` or a
`{..}` of those two alone (a bare `use`, an alias or another item is a breach). Loops, no closures (the
crate's lints). Ceilings: units under 20 ms.
Proof: unit, `cargo nextest run -p temper-box -E 'test(/tests::(machine|limits|boundary)::/)'`:
the orders of a start, a restart (with its lane), a give-up and a stop as literals, the five limits and
the box's edges to the setup; 22 pass. Not higher: 12.2 first hosts the machine on io (joint).
Don't: read a clock, a file or the environment; name a token, a byte or a step in the machine; use `std`.
Size: at most 280 lines of production code. Removes: nothing.

### 12.2 The host on skein-io's processes
Scope: `host` (`Settings`, `Service`, `io_limits`, `skein_shell::Host`), `lines` (a child's bytes
into prefixed lines, and where each pipe goes) and `sink` (a write stream with room), joining the
machine to io (setup.md 7.2, last bullets).
Needs: 12.1; dev `skein-sim` and `skein-fake-machine`, git dependencies as
`tests/one-process/Cargo.toml:44-45` (already in `Cargo.lock`).
Files: new `src/{host,lines,sink}.rs`, `src/tests/{host,lines,sink}.rs`; `src/lib.rs`,
`src/tests.rs`, `src/tests/fixtures.rs` (12.1's, plus `Harness`); the crate's `Cargo.toml`;
`Cargo.lock` (**hotspot**; the new dev edges only).
Shape:
- `Service::new` checks the limits, makes `Io` from `io_limits(steps.len())` (sketch 3), adopts
  the signalfd (skein-io `layer.rs:198`) and the two descriptors as write pipes without a
  timeout (`:167`), makes the machine. The passes are `system.rs:371-448`'s for io alone, then
  the machine's `fire`, its orders as `down` and `reclaim`, each reserving `MAX_OUT` first.
  `Host` by delegation: `exit()` is `Code(0)` after `Stopped`, `Code(1)` after `GaveUp` or a
  failed output; `is_empty()` once the machine is idle and the sinks and signalfd are closed
  (`system.rs:426-433`).
- **Orders:** `Spawn` is `Request::Spawn` (records.rs:43) with `settings.program`, the step's
  arguments, `settings.environment` (the box's own, whole), `settings.root`, `dir` empty, pipes
  for child fds 1 and 2 (`Way::Out`) and an owner token fresh per spawn, so a stale event of an
  old child is ignored. `Signal` is `Request::Signal`, `Terminate`, `Target::Child`; one sent
  while another is in flight is dropped (`process.rs:237-250`), the grace spaces them. `Close`
  is `Request::Close`. **Events:** `Spawned` is `Started` and demands each pipe's first read;
  `Exited`; the child's `Closed` is `Closed`, or `NotStarted` after a failed spawn; `Shutdown`
  is `Terminate`.
- **A child's bytes:** one chunk in flight per pipe, `Read::Arrived { max: LINE_BYTES }`, read
  again once the sink took it; `Close` on the pipe at `End` or `Failed` (`operation.rs:332-360`).
  `lines::route(plain, step, child_fd)` sends the stdout of the step named `plain` to the stdout
  sink unchanged and every other pipe through `Lines`, which cuts at LF (a longer line in pieces,
  an unterminated last fragment at `End`) and prefixes each piece with the step's name and `: `,
  for the stderr sink. When the machine is idle after a give-up, the host writes `box: <step>
  gave up after <n> restarts` there too (`0` for a once step), before the sinks close.
- **`Sink`:** `Demand { read: Nothing, room }`, on `Room` one `Send` within it
  (`operation.rs:435-457`); a bounded queue refusing a push when full, each line or piece one
  entry, so lines of two pipes never mix; `Up::Failed` is reported once and stops the box, exit
  1 (as `crates/temper/src/host.rs:78-92`). `io_limits`: `sockets` three per step and three
  adopted, `intake` `LINE_BYTES`, `output` that plus the longest step name and 3 bytes (the
  box's line is shorter), `receive` 1, `names` 0, `refusals` the steps, `sends` 1 (one `Send` on each `Room`),
  `accepts` 1 and `backlog` 1 (io's minimum, `Limits::is_usable`, `limits.rs:71-79`; the box never listens),
  `close_timeout` 5 s and `retry` 10 ms (as `temper-client-shell-terminal/src/limits.rs:90-94`).
Test shape: `Harness` in fixtures runs `Service` over `skein_sim::Sim` as `skein_shell::drive`
(drive.rs:53-75) runs it over the ring: `Sim::new(SEED, Config::calm())` (sim.rs:150),
`spawn_process` (:203), `open_signal_source` (:310), two `open_inherited_write` (processes.rs:273),
a root from `Machine::lay(&[])` (fake machine fs.rs:316; it gives an `Opened`) through
`Sim::root(Handle::new(opened.raw()))` (files.rs:53); each
turn `reap`, `iterate`, `drain`, `submit` (sim.rs:362, 346), every call waiting answered by
`skein_fake_machine::step`, one ask at a time, never `serve`: `step` drops a spawn's environment (`face.rs:19`,
`env: _`), so the harness records each spawn, its environment and `Sim::now` (sim.rs:191) from its `Ask::Spawn`
before it steps it; and when
idle `advance_to` the earlier of `next_due` and the host's deadline (:428, :449). Programs are the
fake machine's: `process_fixture` with the step's arguments `exit 0`, `exit 1` or `never`
(face.rs:76-105), which ignore a last argument: `once` and `long` append the step's name there,
and the harness reads it. `stderr()` is `peer_drain` (processes.rs:306), `terminate()` is
`deliver_service_signal` (sim.rs:320). Limits: `limits::small()` and `io_limits`. Reference test
(joint):
```rust
// crates/temper-box/src/tests/host.rs
use super::fixtures::{Harness, long, once};
use skein_io::kernel::Exit;
use skein_lib::Duration;
const GAPS: [u64; 5] = [500, 1_000, 2_000, 4_000, 8_000];
#[test]
fn a_long_step_that_keeps_exiting_is_given_up_naming_it() {
    let steps = [once(b"prepare", &[b"exit", b"0"]), long(b"forgejo", &[b"exit", b"1"])];
    let mut harness = Harness::new(&steps);
    assert_eq!(harness.run_to_end(), Exit::Code(1));
    let spawns = harness.spawn_times(b"forgejo");
    assert_eq!(spawns.len(), 6, "the first run and five restarts");
    for (pair, gap) in spawns.windows(2).zip(GAPS) {
        assert!(pair[1].saturating_since(pair[0]) >= Duration::from_millis(gap), "the backoff doubles");
    }
    assert_eq!(harness.spawn_times(b"prepare").len(), 6, "each restart runs prepare again");
    let line = b"box: forgejo gave up after 5 restarts\n";
    assert!(Harness::contains(&harness.stderr(), line), "the give-up names the step");
}
```
Tests: `host.rs`: `the_steps_start_in_order_and_a_stop_leaves_no_child` (exit 0; the sim's
`assert_quiescent` and `assert_no_open_fds`, sim.rs:483, 493), `a_stop_signals_each_long_step_twice_last_first`
(the `Signal` entries of `Sim::trace`, sim.rs:517, trace.rs:238-246, matched to the spawn
completions' pidfds), the reference, `a_once_step_that_fails_ends_the_box_with_exit_one` (`box: prepare gave up after 0 restarts`),
`every_step_is_spawned_with_the_environment_the_box_was_given`,
`the_io_limits_are_usable_for_one_six_and_sixty_four_steps`. `lines.rs` and `sink.rs` (unit,
hand-fed bytes and `Up` events, literals as in 12.1): `a_line_is_written_with_its_step_s_name_before_it`,
`a_line_longer_than_the_cap_is_written_in_pieces_each_named`,
`an_unterminated_last_fragment_is_written_when_the_pipe_ends`,
`only_the_plain_steps_stdout_is_written_as_is`, `a_sink_sends_no_more_than_the_room_it_was_granted`,
`a_failed_sink_is_reported_once`. Ceilings: units under 20 ms, joint under 0.5 s (the sim's time
moves by `advance_to`).
Proof: joint, `cargo nextest run -p temper-box -E 'test(/tests::(host|lines|sink)::/)'`: the
machine and host on the real io over the simulator: steps start in order with the box's
environment, a stop signals each long step twice and leaves no child and no descriptor, a step
that keeps exiting is given up after five restarts with spawns at least 0.5, 1, 2, 4 and 8 s
apart, and the box exits 1 with `box: forgejo gave up after 5 restarts` on stderr; 12 pass. Not process: real children, signals and a child's bytes through pipes first run in
temper 13.1, whose Proof names this increment (sim programs write no bytes).
Don't: use `std` (print, process, thread, time, locks); write to stdout or stderr but through the
two sinks; read a child's output without a bound; add a second loop beside the passes.
Size: at most 220 lines of production code; it lands at up to 850 by the ruling of 2026-10-16T23:00:52Z's block. Removes: nothing.

## Interface sketches

`temper-box` owns all three; 13.1 uses `Service::new`, `Settings`, `limits::{small, apply, check}` and
`Unfit::names`. `Step` and `steps` are temper 07.1's, never redefined.

Sketch 1, 12.1:
```rust
// crates/temper-box/src/limits.rs
use skein_lib::Duration;
/// The box's limits (configuration.md, 7): `box.backoff_ms`, `box.backoff_max_ms`, `box.restarts`,
/// `box.window_ms` and `box.grace_ms`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub backoff: Duration,
    pub backoff_max: Duration,
    pub restarts: u32,
    pub window: Duration,
    pub grace: Duration,
}
/// The most exits one step remembers: configuration.md 7's bound on `box.restarts`.
pub const MAX_RESTARTS: u32 = 64;
/// Why `check` refuses limits; `names` gives the limits the failure names.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Unfit {
    Backoff,  // `box.backoff_ms` above `box.backoff_max_ms`
    Window,   // `box.backoff_max_ms` not below `box.window_ms`
    Restarts, // `box.restarts` above `MAX_RESTARTS`
}
/// Why `apply` refuses an override: `Unknown`, not a `box.` limit; `Range`, a value past the field's type.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refused { Unknown, Range }
impl Unfit {
    /// Two limits for a cross-check, the one for a bound.
    #[must_use] pub fn names(self) -> &'static [&'static [u8]] { todo!() }
}
#[must_use] pub fn small() -> Limits { todo!() }
pub fn apply(limits: &mut Limits, name: &[u8], value: u64) -> Result<(), Refused> { todo!() }
pub fn check(limits: &Limits) -> Result<(), Unfit> { todo!() }
#[must_use] pub fn worst_case(limits: &Limits, steps: u32) -> Option<u64> { todo!() }
```
Sketch 2, 12.1 (`Step` is 07.1's):
```rust
// crates/temper-box/src/machine.rs
use crate::limits::{Limits, Unfit};
use alloc::boxed::Box;
use skein_io::kernel::Exit;
use skein_lib::{Queue, Time};
use temper_forge_setup::Step;
/// What the host tells the machine; `step` indexes the steps it was made from.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Input {
    Started { step: usize },            // the child exists (io's `Spawned`)
    NotStarted { step: usize },         // no child could be made, nothing is left
    Exited { step: usize, exit: Exit }, // the child exited (io's `Exited`); not closed yet
    Closed { step: usize },             // the child's pipes and group are gone
    Terminate,                          // the termination signal: the first stops, a second closes all
}
/// What the machine asks of the host.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Order {
    Spawn { step: usize },  // run the step's command as a new child
    Signal { step: usize }, // send the child a termination signal, to the process alone
    Close { step: usize },  // close the child, which kills its group
}
/// Why the machine ended, once `is_idle` holds.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    Stopped, // a stop the box was asked for: exit 0
    /// A once step failed (`restarts` 0) or a long one exited too often (`restarts` is the
    /// limit): exit 1, and the box's line `box: <step> gave up after <restarts> restarts`.
    GaveUp { step: usize, restarts: u32 },
}
/// The box's policy: steps, exits and timers in; spawns, signals and closes out.
#[derive(Debug)]
pub struct Machine {
    longs: Box<[bool]>, // and each step's phase, its exits and its timers
}
impl Machine {
    pub fn new(limits: &Limits, steps: &[Step]) -> Result<Machine, Unfit> { todo!() }
    /// The most orders one entry point emits: one per step.
    #[must_use] pub fn max_out(&self) -> u32 { todo!() }
    pub fn start(&mut self, now: Time, orders: &mut Queue<Order>) { todo!() }
    pub fn input(&mut self, now: Time, input: Input, orders: &mut Queue<Order>) { todo!() }
    pub fn fire(&mut self, now: Time, orders: &mut Queue<Order>) { todo!() }
    #[must_use] pub fn next_deadline(&self) -> Option<Time> { todo!() }
    #[must_use] pub fn outcome(&self) -> Option<Outcome> { todo!() }
    #[must_use] pub fn is_idle(&self) -> bool { todo!() }
}
```
Sketch 3, 12.2:
```rust
// crates/temper-box/src/host.rs
use crate::limits::{Limits, Unfit};
use alloc::boxed::Box;
use skein_io::kernel::Fd;
use temper_forge_setup::Step;
/// What the binary (13.1) hands the host.
#[derive(Debug)]
pub struct Settings {
    pub program: Box<[u8]>,     // the `temper` executable every step runs: a path as given, no search (decisions.md, 9)
    pub environment: Box<[Box<[u8]>]>, // the box's own, `NAME=value` as the binary read it; every step gets it whole
    pub plain: Box<[u8]>,       // the name of the step whose stdout is the box's as is: `serve`
    pub root: Fd,               // the directory every step starts in, from `skein_shell::open_root`
    pub signals: Fd,            // the signalfd of `skein_shell::open_termination_signals`
    pub stdout: Fd,             // descriptor 1
    pub stderr: Fd,             // descriptor 2
}
/// The longest piece of a child's output held at once; a longer line is cut.
pub const LINE_BYTES: u32 = 4096;
/// The box as a host's service: io, the machine and the two output streams.
#[derive(Debug)]
pub struct Service {
    io: skein_io::Io, // and the machine, the children's tokens and pipes, the sinks
}
impl Service {
    /// A box over `steps`; `Unfit` when `limits` break their rules.
    pub fn new(limits: &Limits, steps: Box<[Step]>, settings: Settings) -> Result<Service, Unfit> { todo!() }
}
#[must_use] pub fn io_limits(steps: u32) -> skein_io::Limits { todo!() }
```

## Questions to raise when reached

None open. A once step's give-up line is the plan's choice (Choices).

## Done when

- `cargo nextest run -p temper-box` passes, at least 34 tests, and `cargo nextest run
  -p temper-box -E 'test(/tests::boundary::/)'` passes (7.3's test, which the gate runs).
- `sed -n '/^\[dependencies\]/,/^\[/p' crates/temper-box/Cargo.toml | grep -c '^temper-'` prints 1,
  and `! git grep -n -E 'std::process|try_lock|CommandExt' -- crates/temper-box` succeeds: one
  temper crate, no process or lock of std.
- From the parent of the first `Step: temper-12.1` commit to `origin/main` (the worktrees are gone, so no
  `origin/main...HEAD`): `git diff -M --numstat $(git log origin/main --all-match --grep='^Plan: forge-setup$'
  --grep='^Step: temper-12\.1$' --format=%H | tail -1)^ origin/main -- crates/temper-box ':!*/tests.rs'
  ':!*/tests/*' | awk '{a+=$1} END {print a}'` prints at most 1324 (12.1's 474 landed, and 12.2's 850).
