+++
id = "bx-pipe"
part = "fs"
kind = "interface"
needs = []
files = ["temper/crates/temper-box/**"]
effort = "high"
minutes = 120
+++

## Goal

The box takes groups it was not built with and removes groups: a `start N` for a group it
does not hold asks its shell to load group N (the load seam: the shell reads
`<data>/box/groups/N` and builds its steps), installs the steps into a free group slot sized
at construction, and starts it; `remove N` stops group N, asks the shell to remove its files,
and answers `removed N`. temper-box alone: the binary's shell answers the seam in a later
task (bx-compose).

Acceptance:
- `pipe::command` parses `remove N\n` (`Command::Remove(u16)`), with `start`/`stop`'s rules
  for digits; `pipe::removed(forge) -> Box<[u8]>` formats `removed N\n`.
- `Limits` gains `groups: u32` (`box.groups`): the groups the box can load after its
  construction, held at once; `small()` gives 4; `check` refuses 0 and more than 64
  (`Unfit::Groups`, names `[b"box.groups"]`); `apply` takes `box.groups`.
- `start N` for a group the box holds behaves as today. For a group it does not hold: with
  no free slot, `failed N too many groups`; otherwise the box asks the shell to load N
  (`Service::take_load`), and on the shell's answer (`Service::loaded`): a group installs
  into a free slot and starts as an asked group does today (its `step`/`started`/`failed`
  lines, its record written when `recorded`); `Loaded::Absent` answers
  `failed N no settings`; `Loaded::Refused(words)` answers `failed N WORDS` (cut to the line
  bound as `pipe::failed` cuts); a group that does not fit its slot (below) answers
  `failed N settings unfit`. Nothing is installed on a failure, and a later `start N` asks
  a load again.
- `remove N`: refused as `stop N` is today (`failed N busy` while any group starts or
  stops, `failed N stopping` while the box stops). For a started group the box holds, the
  box stops it exactly as `stop N` does (no `stopped N` line), then, once dormant, asks the
  shell to remove its files (`Service::take_removal`); for a dormant group, or one the box
  does not hold, it asks at once. On `Service::removed(N, Ok(()))`:
  the terminal says `LABEL: removed` (held groups only), the group is forgotten (its slot
  free again when it was a loaded group; a group the box was built with is never placed
  again, a later `start N` loads it as any unknown group), and the box answers `removed N`.
  On `Err(words)`: the terminal gets the notice `box: remove N: WORDS`, the group stays
  held and dormant, and the box answers `failed N not removed`.
- While a load or a removal is with the shell, the box reads no further pipe line.
- Every existing temper-box test passes unchanged, but for the expected `not asked` of a
  `start` for an unknown group, which now asks a load (change those tests to answer
  `Loaded::Absent` and expect `failed N no settings`).

## Inputs

- `temper/docs/design/protocol/setup.md`, 7.1 (the pipe's lines and its one-ask rules), 7.2
  (the box's stop and groups), 7.3.
- Draft `temper/docs/design/draft/playground-reset.md`, sections 3 ("The box knows kinds")
  and 4 ("Its box group stopped and removed").
- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, T3 (the pipe's `remove N`,
  `removed N`, `failed N no settings`, and the files' order).
- `skein/docs/foundation/programming-model.md`, sections 1, 6.1 and 10 (no closures, traits
  or `dyn`: the seam is a pull and an answer, as temper-service's handover reads are,
  `temper/crates/temper-service/src/system.rs` `take_handover_read`/`handover_read`, and
  their shell `temper/crates/temper/src/host.rs` `drain`).

## Owns and may change

- Owns: `temper/crates/temper-box/**` (src, its tests, `tests/real_loop.rs`).
- May change, compiler-forced only: `temper/crates/temper/src/setup.rs` and
  `temper/crates/temper/tests/suite/startup.rs` if a `Limits` literal or match needs the new
  field or variant (`Unfit::Groups`). Nothing else outside temper-box.
- `temper_forge_setup`'s `Step`, `Group`, `Steps`, `Stage`, `Kind` are read as they are; the
  box's boundary test (`src/tests/boundary.rs`) stays as it is.
- Options, pre-approved:
  - Slot layout is inside. Default: the flattened layout keeps today's placement of the
    construction's `Steps` (always and asked by `before`) and appends `limits.groups` slots
    of `GROUP_STEPS` positions each after them, sized at construction (machine states,
    children, logs, io limits, terminal and reply bounds all count them); a slot's unused
    positions are never spawned and name no group. If keeping one placement for built and
    loaded groups is simpler, do that and say so.
  - A loaded group's `before` is ignored and it never joins the start chain (it is started by
    `start N` only). A group built at construction keeps today's start-chain behaviour; the
    shell gives such groups (the recorded groups loaded at the box's start) in
    `Steps.asked`.
  - Where a step's log file is opened for a loaded group is inside; default: when the group
    is installed, before its first step spawns, by the logs machine as at the box's start
    (a refusal there is that step's log refusal, as today).

## Steps

1. **Pipe** (`src/pipe.rs`): `Command::Remove(u16)` beside `Start`/`Stop`, same digit rules;
   `pub fn removed(forge: u16) -> Box<[u8]>`. Tests in `src/tests/pipe.rs`:
   `remove_parses_as_start_and_stop_do` (`remove 7\n`, `remove 0\n`, `remove 07\n` unknown,
   `remove 65536\n` unknown, no LF unknown), `removed_formats_its_line`.
2. **Limits** (`src/limits.rs`): `pub groups: u32`, `small()` 4, `apply` `box.groups`,
   `check` 1..=64 else `Unfit::Groups`; `pub const GROUP_STEPS: u32 = 4` (an owned forge's
   fetch, prepare, forgejo, runner); `pub const GROUP_NAME_BYTES: u32 = 64` (a step's name and
   a group's label at most). Tests in `src/tests/limits.rs`: `box_groups_applies_and_checks`.
3. **The seam** (`src/host.rs`, public):

   ```rust
   /// The shell's answer for `<data>/box/groups/N` (draft playground-reset.md, 3).
   #[derive(Debug)]
   pub enum Loaded {
       /// Its steps, built by the group's mode; `forge` is N.
       Group(temper_forge_setup::Group),
       /// No `box/groups/N`.
       Absent,
       /// Unreadable or not a group's settings: the words of `failed N WORDS`.
       Refused(Box<[u8]>),
   }
   impl Service {
       /// The group whose settings the shell reads and builds; answered by `loaded`.
       pub fn take_load(&mut self) -> Option<u16>;
       pub fn loaded(&mut self, forge: u16, loaded: Loaded);
       /// The group whose files the shell removes, in this order: `box/started/N`, the
       /// directory `forge-N` and everything in it, `box/groups/N` (the cache stays);
       /// answered by `removed`.
       pub fn take_removal(&mut self) -> Option<u16>;
       pub fn removed(&mut self, forge: u16, removed: Result<(), Box<[u8]>>);
   }
   ```

   At most one load or removal is with the shell at a time (the pipe is not read meanwhile).
   After `loaded` or `removed`, `work_pending` is true until the box has acted on it. An
   answer for a forge the box did not ask, or after its stop began, is ignored. The module
   documentation of `host.rs` says what the shell does for each, citing the draft's
   section 3 and 4.
4. **Fit** (checked in `loaded`, nothing installed when it fails: `failed N settings unfit`):
   `group.forge == N`; 1 to `GROUP_STEPS` steps; no `Kind::Link` step; each step's name and
   the label at most `GROUP_NAME_BYTES`; each step's started notice at most
   `1024 + 2 * data.len()` bytes (the owned forge's notice names its directory twice); each
   step's listeners among `Settings.listeners` (the shell holds the machine's ports from
   the box's start).
5. **Machine** (`src/machine.rs`): an `Input` and `Order` for each new edge; the names are
   inside. A suggestion: `Input::Install { forge, slot }`, `Input::Remove { forge }`,
   `Input::Removed { forge, ok: bool }`; `Order::Load { forge }`, `Order::RemoveFiles
   { forge }`; `Said::Removed { forge }`; `Why::NoSettings`, `Why::TooMany`, `Why::Unfit`,
   `Why::NotRemoved` (terminal words `no settings`, `too many groups`, `settings unfit`,
   `not removed`); `Fact::GroupRemoved { forge }` (`LABEL: removed`). A `Refused(words)`
   answer's words are held by the host for its one `failed` line. A removal of a stopping
   group continues from `stop_group_next`'s end instead of saying `stopped`.
6. **Host** (`src/host.rs`): `take_control_line` routes `Command::Remove`; installs a loaded
   group into the free slot (its `Placed` entries, `Child::new` for each step, the machine's
   states and group range, its log files, its label); frees a removed loaded group's slot;
   the record writer (`src/records.rs`) writes `box/started/N` for any forge it is told.
7. **Test harness** (`src/tests/fixtures.rs`): the harness plays the shell: after each
   `drain` it answers `take_load` from a table it holds (`Harness::settings(forge, Loaded)`;
   `Absent` for a forge not set) and `take_removal` with `Harness::removal`
   (`Ok(())` unless a test sets an `Err`), recording each forge asked in `loads` and
   `removals`. A fixture `loaded_group(forge)` builds `prepare N` (once, `exit 0`) and
   `forgejo N` (long, `never`), labelled `Forge N (Forgejo)`, recorded.
8. **Tests** in `src/tests/host.rs` (each ends with `terminate`, `run_to_end` exiting 0,
   `assert_quiescent`, `assert_no_open_fds`, no open handles, as the existing group tests
   do):
   - `a_start_for_an_unknown_group_loads_it_and_starts_it`: always `[serve]`; `start 7` →
     `loads == [7]`; replies `step 7 prepare\nstep 7 run\nstarted 7\n`; `box/started/7`
     exists; a second `start 7` answers `started 7` with no second load.
   - `a_start_with_no_settings_answers_failed_no_settings_and_loads_again`.
   - `a_refused_load_answers_failed_with_the_shells_words`.
   - `an_unfit_group_answers_failed_settings_unfit` (five steps; a forge of 8 for `start 7`).
   - `a_start_past_the_groups_answers_too_many_groups` (`groups: 1`; 7 loaded; `start 8` →
     `failed 8 too many groups`, no load asked).
   - `a_remove_stops_a_started_group_then_removes_its_files_and_answers_removed`
     (stubborn group as in `a_pipe_stop_closes_only_its_group_keeps_the_record_and_can_be_started_again`;
     no `stopped 7`; `removals == [7]` only after its steps closed; reply `removed 7\n`;
     stderr has `Forge 7 (Forgejo): removed\n`).
   - `a_remove_of_a_group_not_held_answers_removed_at_once`.
   - `a_failed_removal_answers_not_removed_and_keeps_the_group` (then `start 7` starts it
     with no load).
   - `a_freed_slot_takes_the_next_group` (`groups: 1`: start 7, remove 7, start 8 loads and
     starts 8).
   - `a_built_group_removed_loads_on_its_next_start` (`grouped_steps()`'s forge 3).
   - `a_remove_while_a_group_starts_answers_busy`.
   - `no_pipe_line_is_read_while_a_load_is_with_the_shell` (hold the harness's answer:
     `start 7\n` then `stop 3\n` written together; `stopped 3` follows the load's answer).
   - Machine tests in `src/tests/machine.rs` for the remove edges of a starting, started,
     failing and dormant group, as the file's asked-group cases do.
9. Narrow runs, one at a time, e.g.
   `heavy cargo nextest run -p temper-box --lib -E 'test(=tests::host::a_start_for_an_unknown_group_loads_it_and_starts_it)'`;
   then `heavy cargo nextest run -p temper-box` (all its tests, `real_loop` with them);
   `heavy cargo check -p temper` (its box call sites); `cargo fmt`;
   `heavy cargo clippy -p temper-box --all-targets`. Old forms
   (`python3 -I -B scripts/old_forms.py` in `temper/`), never raised. Commit and push your
   branch.

## Report

- The slot layout you chose and the bounds that size it (positions per slot, name, label
  and notice bytes), and what the shell must do for each seam call, in two lines each: the
  composition task (bx-compose) builds on them.
- What `temper/docs/design/protocol/setup.md` 7.1 and 7.2 need to say (the `remove`/`removed`
  rows, the new `failed` words, `box.groups`), for the lead.
