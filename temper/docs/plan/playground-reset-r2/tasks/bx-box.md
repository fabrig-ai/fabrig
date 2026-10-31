+++
id = "bx-box"
part = "box-groups"
kind = "feature"
needs = ["bx-steps", "bx-machine"]
files = [
  "temper/crates/temper-box/src/host.rs",
  "temper/crates/temper-box/src/groups.rs",
  "temper/crates/temper-box/src/records.rs",
  "temper/crates/temper-box/src/logs.rs",
  "temper/crates/temper-box/src/sink.rs",
  "temper/crates/temper-box/src/lib.rs",
  "temper/crates/temper-box/src/tests.rs",
  "temper/crates/temper-box/src/tests/host.rs",
  "temper/crates/temper-box/src/tests/groups.rs",
  "temper/crates/temper-box/src/tests/fixtures.rs",
  "temper/crates/temper-box/src/tests/restart.rs",
  "temper/crates/temper-box/src/tests/logs.rs",
  "temper/crates/temper-box/src/tests/boundary.rs",
  "temper/crates/temper-box/tests/real_loop.rs",
  "temper/crates/temper-box/Cargo.toml",
  "temper/crates/temper/src/box_command.rs",
  "temper/crates/temper/src/box_command/**",
]
effort = "high"
minutes = 90
+++

## Goal

The box reads its groups from `<data>/box/groups/N` and removes them
(`notes/contracts.md`, 3; `temper/docs/design/draft/playground-reset.md`,
3 and 4): at its start, each group file is built by its kind
(`temper_forge_setup::group::build`) into a slot, those whose
`started/N` exists starting as recorded groups do today; `start N` of a
group it does not hold reads its file through the ring; `remove N`
stops the group, runs its removal step, removes `box/started/N` then
`box/groups/N`, and answers `removed N`; a group or file already gone
is removed all the same. The binary's `temper box` builds no steps from
the document's forges: its own always steps (the fake-backends' peers
when given, then serve) and the group files. While the document still
names `application.setup`, the binary writes those entries' group files
at the box's start (a bridge `bx-process` removes), so a deployment
runs as today.

Acceptance, `temper-box`'s lib target, the simulated host world
(`src/tests/fixtures.rs`'s `Harness` on `skein_sim` and
`skein_fake_machine`), new module `tests::groups` unless a test fits
an existing module better:

- `a_start_of_a_group_not_held_reads_its_file_and_starts_it`: a
  `box/groups/7` item (a fake group, `compose`d) and no group at
  construction: `start 7` gives `step 7 prepare`, `step 7 run`,
  `started 7`, the children's argv those of `group::build`, and
  writes `box/started/7`.
- `a_start_without_a_group_file_answers_not_asked`.
- `an_invalid_group_file_answers_unreadable_with_one_notice`.
- `a_group_naming_a_port_the_box_does_not_hold_is_unreadable`: the
  notice names the address.
- `groups_given_at_the_start_with_a_record_start_before_serve` and
  `one_without_a_record_stays_dormant_until_asked` (the startup slots).
- `remove_stops_the_group_runs_its_removal_then_removes_its_records`:
  `remove 7` of a started group: the group's children signalled in
  reverse, then `forge remove DIRS 7` spawned once, exit 0, then
  `box/started/7` and `box/groups/7` gone (in that order), `removed 7`
  answered, a `LABEL: removed` line on the terminal.
- `remove_of_a_group_whose_file_is_gone_answers_removed`, and with only
  `started/7` left, that record removed too.
- `a_failed_removal_keeps_both_files_and_answers_failed`.
- `a_removed_slot_takes_another_group_on_the_same_port`: after
  `removed 7`, `start 8` (another fake group naming the same fake
  forge port) starts on the listener the box still holds: removal
  closes no held listener.
- `a_restarted_serve_can_remove_what_it_asked`: across a serve restart
  (fresh pipes, as `tests::host` restarts it), `remove N` still works.
- Every existing test of `tests::host`, `tests::restart`,
  `tests::logs`, `tests::terminal` and `tests::machine` passes
  (fixtures adapt to the new `Service::new`).
- `tests::boundary`: the box names of `temper_forge_setup` only `Step`,
  `steps`, `Steps`, `Group`, `Stage`, `Kind` and the `group` module's
  `build`, `read`, `Built`, `Machine`, `GroupFile`, `path`,
  `FILE_BYTES`, `STEPS`, `NAME_BYTES`, `LABEL_BYTES`; its dependency
  check is unchanged.
- `temper`'s lib target, `box_command::tests` (new):
  `the_bridge_writes_each_owned_and_fake_entry_once_and_records_the_unasked`
  (owned asked: file only; owned not asked and fake not the demo's:
  file and `started/N`; the demo's: file with `"playground": true`, no
  record; managed: nothing; an existing file is kept as it is);
  `startup_reads_each_group_file_and_skips_an_invalid_one_with_a_line`;
  `the_machine_ports_are_the_entries_and_the_demo_llm_listens`.
- `temper-box/tests/real_loop.rs`, real ring and real files, synthetic
  children: `a_removed_group_leaves_no_file_behind`: a group file and
  its record, `remove N` from the synthetic serve, `removed N` read
  back, both files gone.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md` (what you may
  change without asking).
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 3 and 4
  (`application.ports`, for the bridge's end).
- `temper/docs/design/draft/playground-reset.md`, 3, 4.
- `temper/docs/design/protocol/setup.md`, 7.1 (the pipe, records,
  listeners held from the start), 7.2, 7.3, 7.5 (the fake backends'
  first steps), 8.4 (the demo's group).
- `temper/docs/plan/playground-reset-r2/tasks/bx-machine.md` and
  `bx-steps.md` (the merged code is the truth: `Machine::with_slots`,
  `install`, `vacant`, `slot`, the `Load`/`Unrecord`/`Vacate` orders
  and their inputs, `layout::slotted`/`fill`/`clear`, `group::*`).
- Code: `temper/crates/temper-box/src/host.rs` (`Service::new`,
  `order`, `take_control_line`, `spawn_pipes`, `adopt_listeners`,
  `io_limits`, the arms marked `bx-box gives this its io`),
  `records.rs` (its phases are the model for the new file io),
  `logs.rs` (`Logs::new`, per-step log names), `terminal.rs`
  (`max_bytes`, `failure_bytes`), `tests/fixtures.rs`;
  `temper/crates/temper/src/box_command.rs` whole,
  `temper/crates/temper/src/fake_backends.rs` (`run`, which calls
  `run_configuration` with its first steps).

## Owns and may change

- Owns the files above; `src/groups.rs` is new (the group file's read
  through the ring), as is `src/tests/groups.rs` and
  `temper/crates/temper/src/box_command/tests.rs` (with `#[cfg(test)]
  mod tests;` in `box_command.rs`).
- May change `src/machine.rs`, `src/layout.rs`, `src/terminal.rs`,
  `src/pipe.rs` where wiring needs a small addition (an accessor, a
  bound); a change of the policy's behavior there is a question.
- May change `temper/crates/temper/src/fake_backends.rs` only at its
  call of `run_configuration` if the signature changes; its
  `checked_peers` keeps calling `temper_fake::steps`, which stays.
- `temper-fake/src/steps.rs` stays (fake-backends' port check uses it);
  `bx-process` retires it.
- Not yours: `setup.rs`, `lib.rs`, `init.rs` of the binary
  (`forge-records`'), `temper-service`.

## Steps

**The service's surface** (`src/host.rs`):

```rust
pub struct Settings { /* today's */, pub groups: temper_forge_setup::group::Machine, pub slots: u32 }
impl Service {
    /// `steps` the box's own (no group); `groups` built from the files at its start.
    pub fn new(limits: &Limits, steps: Steps, groups: Box<[temper_forge_setup::group::Built]>, settings: Settings)
        -> Result<Self, Unfit>;
}
```

- Layout: `layout::slotted(steps, groups as (group, remove) pairs,
  settings.slots, group::STEPS + 1)`; `Machine::with_slots`. Every
  per-step collection (children, logs, terminal tails) has one entry
  per position, vacant ones included; output bounds use
  `group::NAME_BYTES` and `group::LABEL_BYTES` for slot positions, so
  a group installed later never exceeds them; io limits count all
  positions.
- `Order::Load { forge }`: `groups.rs`, a phase machine like
  `records.rs` (tokens of its own, one load at a time): open
  `box/groups/N` beneath the root (read), read at most
  `group::FILE_BYTES + 1` bytes (more is invalid), close; NotFound
  anywhere on the path is `NotLoaded { missing: true }`. Then
  `group::read` and `group::build` with `settings.groups`; each step's
  listener must be one the box holds (`settings.listeners`), else
  invalid. Invalid: one notice line `box: forge N: WHY` and
  `NotLoaded { missing: false }`. Valid: `machine.vacant()`'s slot,
  `layout::fill`, `Machine::install`, its children and log entries made
  for the slot's positions, then `Input::Loaded`.
- `Order::Unrecord { forge }`: in `records.rs` beside the record's
  write: remove `started/N` in `box/started`, then `N` in `box/groups`
  (each directory opened beneath the root, `Op::Remove`, closed);
  NotFound at any step is success. Then `Unrecorded`; another error is
  a notice line (as a record's diagnostic) and `NotUnrecorded`.
- `Order::Vacate { slot }`: `layout::clear`; the slot's children and
  log entries returned to vacant (their log files closed; the log
  files themselves stay with the launch's logs).
- `take_control_line`: `remove N` is already mapped (bx-machine).
- Listeners: the box holds every listener of `settings.listeners` from
  its start to its end, as it holds a dormant group's today; a group's
  steps get theirs by address (`spawn_pipes`), and removing a group
  closes none.

**The binary** (`temper/crates/temper/src/box_command.rs`):

- `run_configuration` keeps its signature. It no longer calls
  `temper_fake::steps` nor `owned_notices` (deleted: `group::build`
  sets the notice). Its steps: `Steps { always: first + [serve],
  asked: [] }`, serve by `temper_forge_setup::step::serve`.
- `fn bridge_groups(directories, deployment: &Deployment) -> Result<(), String>`,
  called first: for each `deployment.checked.setup` entry, owned or
  fake, whose `<data>/box/groups/N` does not exist, write it (0700
  `box/groups/`, 0600 file, written to a temporary name in the same
  directory then renamed, as contract 3 says serve writes it):
  `{"setup": ENTRY, "playground": P}`, `ENTRY` the entry's object bytes
  as the document has them (located with `skein_json`, the binary's
  dependency), `P` whether the entry is `deployment.demo`'s forge; and
  for an entry not `asked` and not the demo's (the forges today's
  `steps` runs always), an empty `box/started/N` (0600, in a 0700
  `box/started/`) if absent, so it starts at every box start as it does
  today. Managed entries: nothing. Its doc says it goes with the
  document's `application.setup` (`bx-process`).
- `pub(crate) fn machine_ports(deployment: &Deployment) -> Vec<std::net::SocketAddr>`:
  the addresses the box holds for groups: each owned entry's `listen`
  on Linux (`cfg!(target_os = "linux")`; on macOS Forgejo binds its own,
  as today), each fake entry's `listen`, and the demo's LLM `listen`.
  `forge-records` changes its body to `application.ports` (contract
  4) when the document loses its forges; keep it one small function.
- `open_listeners` opens or adopts (held, inherited) the listeners of
  `machine_ports` and serve's, as it does today for every step's; an
  inherited listener neither serve nor `machine_ports` names is refused
  with today's words; held ones no step and no machine port names are
  closed as today.
- `fn startup_groups(directories, machine: &group::Machine) -> Box<[group::Built]>`:
  lists `<data>/box/groups/` (absent: none), each entry whose name is a
  canonical decimal forge number, read with
  `skein_shell::read_private_file` (bound `group::FILE_BYTES`),
  `group::read` then `group::build`; a file that fails prints one line
  `temper box: box/groups/N: WHY` on stderr and is skipped (a later
  `start N` refuses it again); a temporary file of an interrupted write
  is skipped silently. More files than slots: the box refuses to start,
  naming the count.
- `Settings.groups`: `group::Machine { flags: directories.flags(),
  directories: directories.jig(), limits: deployment.checked.limits,
  llm: the demo's LLM listen, forgejo_listens: cfg!(target_os =
  "linux") }`; `Settings.slots`: `const GROUP_SLOTS: u32 = 4` (the
  forge component's `forges` default, contract 4), raised to the
  number of startup groups if more (never refusing a deployment that
  runs today).

**Tests.** The host world as `tests::host` drives it (`Harness` with
`skein_fake_machine::Item`s for the files; extend `fixtures.rs` with a
constructor taking group files and `Built` groups, and a helper that
writes a `compose`d file item). Real loop: extend `real_loop.rs`'s
synthetic world with a group file in its data directory; the removal
step is a synthetic once program exiting 0. Binary: `box_command::tests`
in a scratch directory with documents from
`crate::tests::configuration` where it has one.

Then: `cargo fmt`; `heavy cargo clippy -p temper-box -p temper
--all-targets -- -D warnings`; narrow: `heavy cargo nextest run -p
temper-box --lib -E 'test(/groups::|host::|boundary::/)'`, `heavy cargo
nextest run -p temper-box --test real_loop`, `heavy cargo nextest run
-p temper --lib -E 'test(/box_command::/)'` (check the counts); then the
focused tests: `heavy cargo nextest run -p temper-box -p temper` (the
`temper` suite runs `temper box` through the bridge). Old forms from
`temper/`: `python3 -I -B scripts/old_forms.py`. Commit and push your
branch.

## Report

- How slot positions are sized (children, logs, output bounds) and any
  bound the slots raised.
- The order of the removal's file effects and what a crash between
  them leaves (a retry must end).
- What the bridge writes for each kind of entry, and whether any
  process-tier story (`temper/crates/temper/tests/process.rs`, not in
  the gate) depends on behavior the bridge does not keep (the legacy
  managed `forge reconcile` step is gone from the box).

## Answers

2026-10-31.

- **May `temper-box` take dev-dependencies to build a
  `group::Machine` in its fixtures and `real_loop`?** Yes, option 1:
  add `jig-protocol-config.workspace = true` and
  `skein-forge-setup.workspace = true` to `temper-box`'s
  `[dev-dependencies]` (`Cargo.toml` is yours for these two lines; its
  `[dependencies]` stay as they are). Both edges point the allowed way
  (temper to jig and skein), both crates already sit beneath
  `temper-forge-setup`, and no other crate's surface changes;
  `temper-forge-setup` gains no constructor. Build `directories` with
  `jig_protocol_config::Directories::given(config, data, cache)` (it is
  public; `working()` too) on the fixtures' own paths, and `limits` as a
  `skein_forge_setup::limits::Limits` literal (its fields are public).
  `tests::boundary`'s dependency check stays as it is: it bounds the
  box's temper dependencies only. Say the two lines in the report.
