+++
id = "bx-machine"
part = "box-groups"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-box/src/machine.rs",
  "temper/crates/temper-box/src/layout.rs",
  "temper/crates/temper-box/src/pipe.rs",
  "temper/crates/temper-box/src/terminal.rs",
  "temper/crates/temper-box/src/tests/machine.rs",
  "temper/crates/temper-box/src/tests/pipe.rs",
  "temper/crates/temper-box/src/tests/terminal.rs",
]
effort = "high"
minutes = 90
+++

## Goal

The box's policy (`temper-box`'s `Machine`, its layout, its pipe words
and terminal facts) learns groups installed at runtime and their
removal (`temper/docs/design/draft/playground-reset.md`, 3 and 4;
`notes/contracts.md`, 3): a fixed number of group *slots* placed before
serve, a group loaded into a vacant slot when `start N` or `remove N`
names a group the box does not hold, and `remove N`, which stops the
group, runs its removal step, has the host remove its records, vacates
the slot and answers `removed N`. The host's io for loading and
removing is `bx-box`'s; here the host answers the new orders at once
so that today's behavior is unchanged.

Acceptance, `temper-box`'s lib target:

- `tests::pipe`: `remove_is_a_canonical_command` (`remove N\n` is
  `Command::Remove(N)`, with the same number rules as `start`/`stop`;
  `remove 01\n`, `remove\n`, `remove 65536\n` are unknown);
  `removed_has_the_wire_spelling` (`pipe::removed(7)` is `removed 7\n`;
  each new `Why` word fits `failed N WHY` within the line bound).
- `tests::machine`, with steps laid out by `layout::slotted`:
  - `slots_sit_before_serve_and_recorded_groups_join_the_start_chain`:
    two slots, one group given at construction and recorded: the start
    chain runs the always steps, the recorded group's steps, then serve;
    a vacant slot spawns nothing.
  - `start_of_a_group_not_held_loads_it_then_starts_it`: `Ask` of an
    unknown forge with a vacant slot orders `Load { forge }`; after the
    test installs the group (`layout::fill`, `Machine::install`) and
    gives `Loaded { forge }`, the group starts as an asked group does
    today (`step N ...`, `started N`).
  - `start_of_a_group_whose_file_is_missing_is_not_asked`: `NotLoaded {
    forge, missing: true }` answers `failed N not asked`; `missing:
    false` answers `failed N unreadable`.
  - `a_start_with_every_slot_held_is_refused_full`.
  - `a_second_load_while_one_is_pending_is_busy`.
  - `remove_of_a_started_group_stops_it_runs_its_removal_then_unrecords`:
    reverse-order signals as a stop, then `Spawn` of the slot's removal
    step; its exit 0 orders `Unrecord { forge }`; `Unrecorded { forge }`
    orders `Vacate { slot }`, says `removed N`, and the slot is vacant
    again (a later `Ask` of another forge loads into it).
  - `remove_of_a_dormant_group_runs_only_its_removal`.
  - `remove_of_a_group_not_held_loads_it_first`; and when its file is
    missing (`NotLoaded { missing: true }`) orders `Unrecord` at once
    and answers `removed N` (a kept `remove` sent again ends).
  - `a_removal_step_that_fails_answers_failed_and_keeps_the_group`:
    exit 1 answers `failed N removal exited 1`; the group stays held
    and dormant; a second `Remove` runs the removal again.
  - `remove_while_another_group_starts_is_busy`, and during the box's
    stop answers `failed N stopping`, as stop's refusals today.
  - `not_unrecorded_answers_failed_and_keeps_the_group`.
  - `the_removal_step_never_joins_a_start_chain`: a recorded group at
    construction starts without its removal step running; a box stop
    never spawns it.
  - Every existing test in `tests::machine` passes unchanged in
    meaning (only `Placed` literals change, if `Placed` gains a field).
- `tests::terminal`: `group_removed_uses_its_label`: `Fact::GroupRemoved
  { forge }` prints `LABEL: removed` as `GroupStopped` prints
  `LABEL: stopped` today.

## Inputs

- `temper/docs/plan/playground-reset-r3/notes/common.md` and
  `temper/docs/plan/playground-reset-r3/notes/scope.md` (what you may
  change without asking; it widens this section).
- `temper/docs/plan/playground-reset-r3/notes/contracts.md`, 3.
- `temper/docs/design/draft/playground-reset.md`, 3 ("The box knows
  kinds, not forges"), 4 ("Its box group stopped and removed").
- `temper/docs/design/protocol/setup.md`, 7.1 (the pipe, `started`
  records), 7.2 (what the box does: chains, stop, failures), 7.3 (labels).
- Code: `temper/crates/temper-box/src/machine.rs` whole (its `ask`,
  `stop_group`, `stop_group_next`, `advance_group`, `refused`,
  `stop_range`), `layout.rs`, `pipe.rs`, `terminal.rs`; the uses of
  `Order`, `Said`, `Input` and `Command` in `src/host.rs`
  (`order_ready`, `order`, `orders_runnable`, `take_control_line`, the
  `Said` formatting near line 620).

## Owns and may change

- Owns `src/machine.rs`, `src/layout.rs`, `src/pipe.rs`,
  `src/terminal.rs` and their tests `src/tests/{machine,pipe,terminal}.rs`.
- May change `src/host.rs` only to keep it compiling and today's
  behavior: `take_control_line` maps `Command::Remove(forge)` to
  `Input::Remove { forge }`; the new `Order`s get arms in
  `order_ready`/`orders_runnable` (ready at once) and in `order`, where
  `Load { forge }` answers `Input::NotLoaded { forge, missing: true }`,
  `Unrecord { forge }` answers `Input::Unrecorded { forge }` and
  `Vacate { .. }` does nothing; `Said::Removed` formats with
  `pipe::removed`; the new `Why`s format their words. Mark each such
  arm with a comment `bx-box gives this its io`. Any other edit
  `host.rs` needs to compile and keep today's behavior is yours too
  (`notes/scope.md`); its io for loading and removing is `bx-box`'s.
- May change `src/tests/fixtures.rs`, `src/tests/host.rs` and any
  other test or crate where a `Placed` literal, a `Machine::new` call
  or an exhaustive match on the new variants no longer compiles.
- Not yours: `temper-forge-setup` (`bx-steps` changes it at the same
  time: name nothing of it but `Group`, `Step`, `Steps`, `Stage`, `Kind`,
  which keep their shapes), the `temper` binary.

## Steps

Layout (`src/layout.rs`), its doc citing
`temper/docs/design/draft/playground-reset.md, 3`:

```rust
/// Runtime groups' places: `count` slots of `width` positions each,
/// starting at `first`, just before the last always step (serve).
/// A slot's last position is its group's removal step; the positions
/// before it hold the group's steps, the rest vacant.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Slots { pub first: usize, pub count: u32, pub width: u32 }
impl Slots {
    pub const NONE: Self; // count 0: today's layouts
    pub fn slot(&self, position: usize) -> Option<u32>;
}
/// The always steps, then `count` slots (the given groups filling the
/// first, each with its removal step), then the last always step.
/// Each given group's `before` is ignored. Panics if a group has more
/// than `width - 1` steps or more groups are given than `count`.
pub fn slotted(steps: Steps, groups: Box<[(Group, Step)]>, count: u32, width: u32) -> (Box<[Placed]>, Slots);
/// Writes one group and its removal step into a vacant slot's positions.
pub fn fill(placed: &mut [Placed], slots: &Slots, slot: u32, group: Group, remove: Step);
/// Returns a slot's positions to vacant placeholders.
pub fn clear(placed: &mut [Placed], slots: &Slots, slot: u32);
```

`flatten(steps)` stays as it is for today's callers. A vacant
position's `Placed` is a placeholder no order ever names
(`Step::new(b"", Box::new([]), Kind::Once)`, `asked: None`, `label:
None`); if the machine needs to tell placeholders, removal steps and
group steps apart from `Placed` alone, add `pub role: Role` to `Placed`
(`enum Role { Always, Group, Removal, Vacant }`) and update the
literals; otherwise derive it from `Slots`. Say which in the report.

Policy (`src/machine.rs`):

```rust
impl Machine {
    pub fn new(limits: &Limits, steps: &[Placed]) -> Result<Self, Unfit>; // as today: Slots::NONE
    pub fn with_slots(limits: &Limits, steps: &[Placed], slots: Slots) -> Result<Self, Unfit>;
    /// A vacant slot, if any (the host fills it on `Load`).
    pub fn vacant(&self) -> Option<u32>;
    /// The slot holding forge N's group.
    pub fn slot(&self, forge: u16) -> Option<u32>;
    /// Takes the group the host just wrote into `slot` (`layout::fill`):
    /// its step states, dormant; called before `Input::Loaded`.
    pub fn install(&mut self, slot: u32, steps: &[Placed]);
}
pub enum Input { /* today's */, Remove { forge: u16 }, Loaded { forge: u16 },
    NotLoaded { forge: u16, missing: bool }, Unrecorded { forge: u16 }, NotUnrecorded { forge: u16 } }
pub enum Order { /* today's */, Load { forge: u16 }, Unrecord { forge: u16 }, Vacate { slot: u32 } }
pub enum Said { /* today's */, Removed { forge: u16 } }
pub enum Why { /* today's */, Full, Unreadable, Removal { step: usize, exit: Exit }, NotRemoved }
```

- **Load:** `Ask` or `Remove` of a forge no slot holds: refused as
  today's refusals say first (box stopping, a group failing or
  stopping: busy); then `Why::Full` with no vacant slot; `Why::Busy`
  while another load is pending (one at a time); otherwise the machine
  reserves the vacant slot it reports, remembers what follows (start or
  remove) and orders `Load { forge }`. `Loaded` continues with the
  start (today's `ask`) or the removal; `NotLoaded { missing: true }`:
  a start answers `Why::NotAsked`, a removal orders `Unrecord` and
  then answers `Removed`; `NotLoaded { missing: false }` answers
  `Why::Unreadable`. A reserved slot not filled is vacant again.
- **Remove of a held group:** refused as `stop_group` refuses (another
  group starting: busy; box stopping); otherwise the group is stopped
  as `stop_group` stops it (disarm backoff, withdraw unsent spawns,
  reverse-order signals, grace), without saying `stopped N`; once its
  steps are closed (or at once if dormant), the slot's removal step is
  spawned outside any chain (no successor; not `Hold`; no `step N`
  reply). Its `Exited` with code 0 orders `Unrecord { forge }`; any
  other exit, or `NotStarted`, answers `Failed { why: Removal { .. } }`
  or `NotStarted` and the group stays held, dormant. `Unrecorded`
  orders `Vacate { slot }`, a terminal `Fact::GroupRemoved { forge }`
  and says `Removed`; the machine's slot is vacant (its step states
  reset to dormant, exit history cleared). `NotUnrecorded` answers
  `Why::NotRemoved`, the group held and dormant.
- **Start chain:** recorded groups given at construction are chained
  as today (`start(now, recorded, ...)`); removal positions and vacant
  positions are never in a chain, never spawned by the box's start or
  restarts, never signalled by the box's stop unless live (a removal
  live at the box's stop is stopped as any once step is).
- `refused`, `group`, `group_starting` and the group list see only
  held slots and today's groups; a vacant slot names no forge.
- Pipe (`src/pipe.rs`): `Command::Remove(u16)` decoded as `start`/`stop`
  are; `pub fn removed(forge: u16) -> Box<[u8]>`; the failure words of
  the new `Why`s: `full`, `unreadable`, `removal exited N` / `removal
  killed by signal N` (as `exited` words a step), `not removed`;
  wherever `Why`'s words are formatted today, the new ones join them.
- Terminal (`src/terminal.rs`): `Fact::GroupRemoved { forge }`, printed
  `LABEL: removed` as `GroupStopped` is; `max_bytes` and the label
  lookups count slot positions as they count steps.
- Tests: in `src/tests/machine.rs` drive the policy as its tests do
  today (orders out, inputs in, no host); build steps with
  `layout::slotted` and `super::fixtures::step`; one helper
  `install(machine, placed, slots, slot, group, remove)` that calls
  `layout::fill` then `Machine::install`. Names as in the Goal.

Then: `cargo fmt`; `heavy cargo clippy -p temper-box --all-targets --
-D warnings`; narrow runs `heavy cargo nextest run -p temper-box --lib
-E 'test(/machine::|pipe::|terminal::/)'` (check the count); then the
focused tests of the package and its dependent: `heavy cargo nextest
run -p temper-box -p temper`. Old forms from `temper/`: `python3 -I -B
scripts/old_forms.py`. Commit and push your branch.

## Report

- Whether `Placed` gained `role`, and the slot geometry the machine
  keeps (how `install` resets step states).
- How the removal's stop reuses `stop_group`'s code, and any case where
  a group `Failing` or `Recording` meets `Remove` (answered busy, as
  stop does, unless you found a simpler rule).
- The host arms you added, each, for `bx-box`.
