+++
id = "box-pipe"
part = "box-groups"
kind = "interface"
needs = []
files = ["temper/crates/temper-box/**", "temper/crates/temper-forge-setup/src/step.rs", "temper/crates/temper-forge-setup/src/tests/steps.rs"]
effort = "high"
minutes = 60
+++

## Goal

The box takes a group it was not built with: `start N` for a forge with no
group reads `<data>/box/groups/N` and runs one of its kinds' groups for N;
and the pipe's new `remove N` stops a group, removes `box/started/N` and
`box/groups/N`, and answers `removed N` (contracts, section 3). The forge's
directory is not removed here: `bx-sweep` adds it.

Acceptance:

- `temper_box::pipe::command` knows `remove N\n` as `Command::Remove(N)`,
  canonical decimal only, as `start` and `stop`; `pipe::removed(N)` formats
  `removed N\n`.
- `temper_box::host::Settings` has a new field `vacant: Box<[Group]>`, the
  kinds' groups (below); with none, every behaviour of today holds and every
  existing temper-box test passes unchanged but for the new field in its
  `Settings` literal.
- `start N` for a forge with no group, with a group file and a free kind's
  group of its kind, runs that group numbered N as an asked group runs
  (`step`, `started`, `failed` lines, its record when `recorded`); with no
  file, `failed N not asked` as today; with no free group of the kind,
  `failed N busy`.
- `remove N` answers `removed N` once the group's steps are closed (if they
  ran) and both files are gone, a file already absent counting as removed;
  for a forge with no group and no file, `removed N` at once. A kind's group
  is free again after its forge's `remove`; any other group is gone (a later
  `start N` reads the file, absent, and answers `failed N not asked`).
- `temper_forge_setup::Group::numbered` exists and its test passes.
- temper-box's tests below pass; `the_box_names_only_steps_and_step_of_the_setup`
  and `the_box_depends_on_no_temper_crate_but_the_setup` still pass.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (section 3 is
  this task's shape; read it first).
- `temper/docs/design/protocol/setup.md`, 7.1 (the pipe, its one-ask-at-a-time
  rule, the record), 7.2 (stopping a group), 7.3 (what the box names).
- `temper/docs/design/draft/playground-reset.md`, sections 3 ("The box knows
  kinds, not forges") and 4 ("Its box group stopped and removed").
- `temper/docs/plan/playground-reset-r4/notes/surveys/s4-box.md`, section 1
  (file and line facts for the box at the base).
- The code: `temper/crates/temper-box/src/{pipe,layout,machine,records,host}.rs`,
  `temper/crates/temper-forge-setup/src/step.rs`; the worker's file steps in
  `temper/crates/temper-worker-protocol-git/src/files.rs` as a model of
  kernel file operations beneath a root.

## Owns and may change

- Owns: `temper/crates/temper-box/**` (sources, `src/tests/`, `tests/real_loop.rs`,
  `Cargo.toml`); in `temper/crates/temper-forge-setup`, `src/step.rs` (the
  new method only) and `src/tests/steps.rs` (its test).
- Neighbour, `temper/crates/temper/src/box_command.rs`: the `Settings`
  literal's new field, `vacant: Box::new([])`, and nothing else (the binary's
  kinds' groups are `bx-wire`'s).
- Allowed new dependency of temper-box: `skein-json` (skein's), and
  `jig-protocol-config` if its `read` helpers serve for the file's one member;
  no `temper-*` crate (the boundary test).
- Departures from the design to state in the commit: setup.md 7.1's pipe
  gains `remove`/`removed` and the group files (contracts, 3; the draft's
  section 10 names this amendment).
- Options:
  - The kinds' groups in `Settings` (default), or in `Steps` as a third
    field: keep `Settings`; `Steps` and `Group` gain no field (their literals
    are in many files of other tasks).
  - The group file is read by the box with kernel operations beneath
    `Settings.root`, as `records.rs` reads `box/started/N`. If reading it
    whole needs a bound, use `Limits.line_bytes`-like sizing from
    `limits.rs`: add `group_file_bytes` (default 65536) to `temper_box::limits::Limits`
    if a bound is needed, and say so in the report; a larger file is
    `failed N group file too long`, named on stderr.
  - If `Machine` cannot grow its groups at runtime without moving step
    indices, keep indices fixed: the kinds' groups are placed at start
    (flattened with the asked groups by their `before`), and a fill replaces
    a placed group's steps in place. This is the default, not an option to
    avoid.

## Steps

1. **`Group::numbered`** (temper-forge-setup, `step.rs`):

   ```rust
   impl Group {
       /// The same group for forge `forge`: every argument that is this
       /// group's forge number in decimal, and every space-separated word of
       /// a step's name and of the label that is, becomes `forge`'s; the rest
       /// is kept (contracts, 3: the box knows kinds, not forges).
       #[must_use]
       pub fn numbered(&self, forge: u16) -> Group;
   }
   ```

   `ready`, `listeners`, `started_notice`, `kind`, `Stage`s, `recorded` and
   `before` are kept. Test in `src/tests/steps.rs`:
   `an_asked_group_numbered_again_is_the_group_set_up_for_that_number`: a
   `Setup` with one owned entry, `"start": "asked"`, read once as forge
   65535 and once as forge 7 (the existing fixtures of `tests/steps.rs`);
   `steps(..).asked[0].numbered(7)` equals the forge-7 `steps(..).asked[0]`,
   with and without a runner and with fetched binaries.

2. **The pipe** (`pipe.rs`): `Command::Remove(u16)` parsed as `start` and
   `stop` are (canonical decimal, LF, no leading zero); `pub fn removed(forge:
   u16) -> Box<[u8]>`. Extend `only_canonical_start_and_stop_lines_are_known`
   (rename it `only_canonical_start_stop_and_remove_lines_are_known`) with
   `remove` cases.

3. **The kinds' groups** (`host.rs` `Settings`):

   ```rust
   /// Each kind's group the box can run for a forge it was not started
   /// with, numbered `u16::MAX`: an owned Forgejo's (`recorded`), the
   /// playground's (not `recorded`). Placed by `before` as an asked group;
   /// its steps' listeners are among `listeners` (contracts, 3).
   pub vacant: Box<[Group]>,
   ```

   A kind is told by `recorded` alone: the file's `"playground": true` takes
   a group that is not `recorded`, `false` one that is. The box holds each
   kind's group from its start, its terminal and log bounds sized from it
   (its number is five digits, the widest), dormant and named by no forge
   until a fill.

4. **`start N` with no group** (`machine.rs`, `host.rs`, a new
   `src/groups.rs` for the file operations): the machine, on `Ask { forge }`
   for a forge it has no group for and with a kind's group free, orders the
   file read (`Order::Read { forge }`, a name the task creates); the host
   opens `box/groups/N` beneath the root, reads it whole, closes it, takes
   the top-level member `playground` (a JSON `true` or `false`, the rest of
   the file the steps' own: `temper forge prepare DIRS N` reads it) and
   answers the machine (`Input::Read { forge, playground: Option<bool> }`,
   `None` for an absent file). An unreadable or malformed file is named on
   stderr (`box: box/groups/N: ...`, as `records.rs`'s diagnostics) and
   answers `failed N group file unreadable`. The ask is the one ask in
   flight while the file is read: another `start`, `stop` or `remove`
   answers `failed M busy`, as during a group's start. On a kind: the host
   puts that kind's group's `numbered(N)` steps in its place (each step's
   name, arguments, label; its log file's name follows the step's name), the
   machine's group takes the forge N, and the ask goes on as for a dormant
   asked group. A filled group's record is not probed (it has none: a group
   with a record at the box's start is the binary's, from its file).
   A `start N` for a group one of whose listeners another group not dormant
   holds answers `failed N busy` (a kind's group and a document's group may
   name the machine's one port).

5. **`remove N`** (`Input::Remove { forge }`, `Said::Removed { forge }`, names
   the task creates): refused as `stop` is (`failed N stopping` while the box
   stops; `failed N busy` while any group starts, fails or stops, or a file
   is read or removed); else the group's running steps are stopped as at
   `stop N` (7.2), with no `stopped` line; then the host removes
   `box/started/N`, then `box/groups/N` (`Op::Remove`, beneath opened `box`
   and its subdirectories; `NotFound` at any part is removed), and answers
   the machine (`Input::Removed { forge }`, or `Input::NotRemoved { forge }`
   on another error, named on stderr, answering `failed N files not
   removed`). On `Removed`: a kind's group filled by N is free again (its
   steps stay as they were, dormant, until the next fill); another group of
   N is gone, so that no `start N` or `stop N` finds it (`failed N not
   asked`); the terminal says `<label>: removed`, as `stop` says
   `<label>: stopped` (`terminal.rs`'s `Fact`, a new `GroupRemoved`); then
   `removed N`. For a forge with no group, the files are removed and
   `removed N` answers, so a `remove` sent again after a restart answers.
   The started-record set (`records.rs`'s `recorded`) drops N.

6. **Tests** (temper-box's own; the machine's in `src/tests/machine.rs` with
   `Rig`-style inputs, the host's in `src/tests/host.rs` on its simulated
   kernel, with the group file and records as files of the simulated root):
   - `a_start_for_no_group_reads_its_file_and_runs_its_kinds_group_numbered`:
     settings with an owned kind's group (fetch, prepare, forgejo with ready,
     runner) and serve; a file `box/groups/7` `{"setup":{},"playground":false}`;
     `start 7` gives `step 7 prepare`, the children spawned with
     `forge prepare ... 7` arguments, `started 7`, the record `box/started/7`
     written 0600.
   - `a_start_for_no_group_and_no_file_is_not_asked_and_a_taken_kind_is_busy`:
     no file: `failed 7 not asked`; files 7 and 8, owned: `start 7` started,
     then `start 8` `failed 8 busy`.
   - `the_playground_file_takes_the_unrecorded_kind_and_writes_no_record`.
   - `a_malformed_group_file_is_named_and_fails_its_start`.
   - `remove_stops_a_started_group_removes_both_files_and_frees_its_kind`:
     after `started 7`, `remove 7`: the group's long steps signalled in
     reverse, `box/started/7` and `box/groups/7` gone, the terminal's
     `Forge 7 (Forgejo): removed`, `removed 7`; then a file 8 and `start 8`
     runs the kind's group as forge 8.
   - `remove_for_no_group_and_no_file_answers_removed_and_again_after_a_restart`.
   - `remove_of_a_document_group_leaves_no_group_for_that_forge`: a group of
     the given `Steps` (forge 3), `remove 3`, then `start 3` `failed 3 not
     asked` and `stop 3` `failed 3 not asked`.
   - `remove_is_busy_while_a_group_starts_and_stopping_while_the_box_stops`.
   - `a_file_error_at_remove_is_named_and_fails_with_files_not_removed`.
   - The machine's: a fill and a removal by inputs alone, as
     `asked_groups_follow_their_chains_records_failures_and_stop` drives a
     group (setup.md 7.3, "The machine's step tests").
7. `cargo fmt`; `heavy cargo clippy -p temper-box -p temper-forge-setup
   --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-box` and
   `-p temper-forge-setup --lib`; their direct dependents build:
   `heavy cargo check -p temper -p temper-fake --tests`. Commit, push.

## Report

Fab's sections; under `## Interfaces` every public item added or changed
(`Group::numbered`, `pipe::Command::Remove`, `pipe::removed`,
`host::Settings::vacant`, machine `Input`/`Order`/`Said`/`Why` variants,
any `Limits` field), each with its path and signature; the group file's
member the box reads and the `failed` words it adds (`group file
unreadable`, `files not removed`, and any other), so `bx-sweep`, `bx-wire`
and serve's box link (the engine's `fr-make-forge`, `rm-remove-forge`) can
be checked against it.
