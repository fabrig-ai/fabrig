+++
id = "bx-wire"
part = "box-groups"
kind = "feature"
needs = ["bx-kinds", "bx-sweep"]
files = [
  "temper/crates/temper/src/box_command.rs",
  "temper/crates/temper/src/box_command/groups.rs",
  "temper/crates/temper/src/box_command/tests.rs",
]
effort = "high"
minutes = 50
+++

## Goal

`temper box` hands the box its groups from the group files and its kinds'
groups from the machine's members (contracts, sections 3 and 6): at its
start each `<data>/box/groups/N` whose N the document's setup does not name
is an asked group before serve, built by its kind's builder from the file's
entry; and the box holds an owned Forgejo's group (from `application.owned`)
and the playground's (from `application.demo.fake` and `application.demo.llm`),
each numbered `u16::MAX`, in `Settings.vacant`, their listeners held from
the box's start.

Acceptance:

- A box started with `box/groups/7` (owned, `"start": "asked"`) and a
  document naming no forge 7 has forge 7's group, asked, recorded, before
  serve; with `box/started/7` it runs in its place before serve, as a
  recorded asked group of the document runs; a playground file is dormant
  until asked and writes no record.
- A file whose N the document's setup names is not read: the document's
  entry is the group (D1).
- A file that `temper_forge_setup::file::read` refuses refuses the box's
  start, naming the file's path and the problem, as a document's problem
  refuses it. A name in `box/groups` that is not a canonical forge number
  is named on stderr and skipped. An absent `box/groups` is none.
- `Settings.vacant` holds the owned kind's group when `Deployment.owned` is
  some, the playground's when the demo has a `fake`, each built by
  `temper_forge_setup::group` / `temper_fake::steps::group` for the
  template's `entry(u16::MAX)`; neither, when the document has no machine
  member (every document today), so the box behaves as it does today.
- Listeners: every step's listener is opened, the kinds' groups' and the
  file groups' among them; one address named by steps of two groups (asked,
  file or kind) is one listener given to each; one named by an always step
  and any other step is refused, as today.
- The owned Forgejo's started notice (`owned_notices`) is set for each
  owned group from its own entry: a document's, a file's, the owned kind's
  (its label `Forge 65535 (Forgejo): `, which `Group::numbered` renumbers).
- The tests below pass, and the binary's existing box tests.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (sections 3, 6).
- `temper/docs/design/protocol/setup.md`, 7.1 ("The listeners are the
  box's"), 7.3 ("The binary hands it the steps").
- The reports and code of `box-pipe` (`Settings.vacant`), `bx-sweep`
  (`Settings.directories`), `bx-machine` (`Deployment.owned`,
  `temper_fake::demo::Demo::fake`, `setup::Template::entry`) and `bx-kinds`
  (`temper_forge_setup::group`, `temper_fake::steps::group`,
  `temper_forge_setup::file`).
- The code: `temper/crates/temper/src/box_command.rs` (`run_configuration`,
  `open_listeners`, `owned_notices`), `temper/crates/temper/src/setup.rs`.

## Owns and may change

- Owns `temper/crates/temper/src/box_command.rs` and its new child modules
  `box_command/groups.rs` (the files' and kinds' groups) and
  `box_command/tests.rs`.
- Neighbours: none. `temper/crates/temper/src/fake_backends.rs` calls
  `run_configuration` and keeps doing so unchanged (its first steps precede
  every group, the files' and the kinds' included).
- Options:
  - Placement: every file group and kind's group goes before serve, the
    last always step (`before` = `always.len() - 1`), after the document's
    asked groups at that point, files by ascending number (default).
  - Reading the files: `std::fs` beneath `directories.data`, each through
    `skein_shell::read_file` with `temper_forge_setup::file::BYTES` (default),
    or `read_private_file` if the binary reads other private files so; the
    files are 0600 (serve writes them).

## Steps

1. `box_command/groups.rs`:

   ```rust
   /// The groups of `<data>/box/groups`, but the document's numbers, by
   /// their kinds' builders; each placed before serve (contracts, 3).
   pub(super) fn files(
       directories: &super::super::Directories,
       checked: &temper_forge_setup::check::Checked,
       demo: Option<&temper_fake::demo::Demo>,
       flags: &[Box<[u8]>],
       forgejo_listens: bool,
   ) -> Result<Vec<(temper_forge_setup::Group, temper_forge_setup::setup::Entry)>, String>;
   /// The kinds' groups, numbered `u16::MAX`, with the entries they were
   /// built from (for their notices).
   pub(super) fn kinds(
       owned: Option<&temper_forge_setup::setup::Template>,
       demo: Option<&temper_fake::demo::Demo>,
       flags: &[Box<[u8]>],
       forgejo_listens: bool,
   ) -> Vec<(temper_forge_setup::Group, temper_forge_setup::setup::Entry)>;
   ```

   A playground file's group takes the demo's LLM `listen` (`llm` of
   `temper_fake::steps::group`); with no demo, a playground file refuses
   the start (`application.demo` names the fake LLM).
2. `run_configuration`: the file groups appended to `steps.asked`, placed;
   the kinds' groups into `Settings.vacant`, placed alike; `open_listeners`
   over all of them, with the sharing rule; `owned_notices` per group from
   its own entry.
3. Tests, `box_command/tests.rs`, on a scratch directory (as the binary's
   other file tests make theirs) and the init template's document:
   - `a_group_file_is_an_asked_group_before_serve_and_the_documents_entry_wins`.
   - `a_refused_group_file_refuses_the_start_naming_its_path`.
   - `no_machine_member_gives_no_kinds_group`.
   - `the_machine_members_give_the_owned_and_playground_kinds_numbered_65535`
     (a document with `application.owned` and `application.demo.fake`: the
     owned kind's `numbered(7)` equals the file group for 7).
   - `groups_may_share_a_listener_and_an_always_step_may_not`.
   - `each_owned_group_takes_its_own_entrys_notice`.
4. `cargo fmt`; `heavy cargo clippy -p temper --all-targets -- -D
   warnings`; `heavy cargo nextest run -p temper --lib`. The process tier is
   not run (common.md). Commit, push.

## Report

Fab's sections; under `## Interfaces`: `groups::files`, `groups::kinds` and
the listener rule as built; any refusal words a person sees.
