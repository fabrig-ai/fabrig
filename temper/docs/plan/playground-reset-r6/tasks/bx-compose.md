+++
id = "bx-compose"
part = "fs"
kind = "feature"
needs = ["bx-pipe", "bx-steps"]
files = ["temper/crates/temper/src/box_command.rs", "temper/crates/temper/src/box_command/**", "temper/crates/temper/src/forge.rs", "temper/crates/temper/src/fake.rs", "temper/crates/temper/src/fake/check.rs", "temper/crates/temper/tests/suite/startup.rs", "temper/crates/temper/tests/suite/forge.rs"]
effort = "high"
minutes = 110
+++

## Goal

The binary is the box's shell for its groups (notes contracts T3): it answers the box's
load seam from `<data>/box/groups/N`, removes a group's files when the box asks, loads the
recorded groups at the box's start so they run before serve, and `temper forge
fetch|prepare|run ... N` and `temper fake forge N` read their entry from `box/groups/N`.
The document's setup entries still work beside them (they leave in a later task).

Acceptance:
- The box runs inside a shell host (`Boxed`) that answers `Service::take_load` with
  `Loaded::Group` built from `box/groups/N` (owned or fake; the owned Forgejo step carries
  the started notice `owned_notices` gives today), `Loaded::Absent` when the file is absent,
  `Loaded::Refused(words)` otherwise; and `Service::take_removal` by removing, in order,
  `box/started/N`, the directory `forge-N` whole, `box/groups/N` (each absent is fine; the
  cache is never touched), answering `Err(words)` naming the path and the error on the
  first failure.
- At the box's start, before the box is built: each `box/started/N` (a decimal name) whose
  `box/groups/N` exists and loads is given in `Steps.asked`, recorded, placed before serve;
  one whose file is absent is removed, and the terminal says so once
  (`box: group N: no settings; its started record is removed`); one whose file is refused is
  left and named on the terminal (`box: group N: WORDS`), dormant. A number that is also a
  group the document builds is left to the document's group.
- `temper forge fetch|prepare|run [DIRECTORIES] N` and `temper fake forge [DIRECTORIES] N`
  (and `temper fake forge check`) take forge N's entry from `box/groups/N` when it exists,
  else from the document's `application.setup` as today; every other input (fetch bytes, the
  webhooks listener, sign-in) stays the document's.
- The box holds each listener address once, however many steps name it (a fake group from
  a file names the same machine ports as the document's demo group).
- Every existing test of package `temper` (default profile) passes.

## Inputs

- `temper/docs/design/protocol/setup.md`, 7.1 (the box's start, its records and pipe), 7.3,
  6.6 (the cache), 8.4 (the demo's group).
- Draft `temper/docs/design/draft/playground-reset.md`, sections 3 and 4.
- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, T3.
- The seam as bx-pipe landed it: `temper/crates/temper-box/src/host.rs` (`Loaded`,
  `take_load`, `loaded`, `take_removal`, `removed`) and its report; the readers bx-steps
  landed: `temper_forge_setup::setup::{Entry::group, GROUP_BYTES, directory}`,
  `temper_fake::steps::group`.
- The precedent for a shell answering its service between iterations:
  `temper/crates/temper/src/host.rs` (`impl skein_shell::Host for Host`, `drain`).

## Owns and may change

- Owns: `temper/crates/temper/src/box_command.rs` and a new `src/box_command/` (module
  `groups.rs` and its `tests.rs`); `src/forge.rs`, `src/fake.rs`, `src/fake/check.rs` for
  where the entry comes from; `tests/suite/startup.rs` and `tests/suite/forge.rs` for the
  tests below.
- May change: `src/fake_backends.rs` call sites of `box_command::run_configuration` only, if
  its signature moves. Not `src/host.rs` or `src/lib.rs` (serve's host; sv's tasks edit
  them), but for a `mod` line or a call site.
- Options, pre-approved:
  - The context the loader needs (data path, `jig_protocol_config::Directories`, the
    directory flags, the setup limits `checked`'s parse used, the demo LLM's listener,
    `forgejo_listens`) is gathered once in `run_configuration`; its type is inside.
  - The started notice: default, refactor `owned_notices` into a function of one owned entry
    and apply it to both the document's groups and loaded ones. Keep its words.
  - Reading the file: `skein_shell::read_private_file(path, GROUP_BYTES, effective_user)`
    (a file open to others is refused, its words naming the path); the refusal words are the
    first problem line of `Entry::group`'s problems, `box/groups/N: ` before it.
  - An entry whose forge is not N, a managed entry, and a fake entry with no demo LLM are
    refused with words saying which (`forge M's settings`, `a managed forge runs no group`,
    `no playground LLM`).

## Steps

1. `src/box_command/groups.rs`:

   ```rust
   //! The box's groups from their files (draft playground-reset.md, 3 and 4; notes contracts T3).
   pub(crate) struct Context { /* data dir, jig directories, flags, setup limits, demo LLM listener, forgejo_listens */ }
   /// `box/groups/N` as the box's load seam answers it.
   pub(crate) fn load(context: &Context, forge: u16) -> temper_box::host::Loaded;
   /// A group's files, in T3's order; the cache stays.
   pub(crate) fn remove(data: &std::path::Path, forge: u16) -> Result<(), Box<[u8]>>;
   /// The recorded groups at the box's start, and the terminal lines they leave.
   pub(crate) fn recorded(context: &Context, built: &[u16], before: u32)
       -> Result<(Vec<temper_forge_setup::Group>, Vec<String>), String>;
   /// The box under its shell: the seam answered after each iteration.
   pub(crate) struct Boxed { pub service: temper_box::host::Service, context: Context }
   impl skein_shell::Host for Boxed { /* every method delegates; drain answers loads and removals */ }
   ```

   `recorded` lists `<data>/box/started/` (absent is none), skips names that are not a
   decimal u16 and those in `built`, and gives each loaded group `recorded: true` and
   `before`. `run_configuration` calls it, appends the groups to `Steps.asked` with
   `before` the index of serve in `always`, prints its lines on stderr before the box
   starts, and drives `Boxed`.
2. `src/forge.rs` (`start`) and `src/fake.rs` (`forge_document`, `Source::Setup`), and
   `src/fake/check.rs`: one helper (in `groups.rs`, `pub(crate) fn entry(directories,
   limits, forge) -> Result<Option<Entry>, String>`) reads `box/groups/N`; `None` falls back
   to `checked.setup.entry(N)`.
3. Unit tests, `src/box_command/tests.rs` (scratch directories under the test's scratch
   root, as the crate's other file tests make them):
   - `a_fake_group_file_loads_as_the_playgrounds_steps` (written with
     `temper_forge_setup::setup::group_settings` from a fake body; steps `fake llm`,
     `prepare 7`, `fake forge 7`, label `Playground`, recorded).
   - `an_owned_group_file_loads_with_its_started_notice`.
   - `a_missing_group_file_is_absent`; `a_group_file_open_to_others_is_refused`;
     `a_group_file_for_another_forge_is_refused`.
   - `removal_takes_the_record_the_directory_then_the_settings` (a deep `forge-7` tree; a
     cache directory beside it stays); `a_removal_cut_after_the_directory_ends_on_its_next_ask`
     (only `box/groups/7` left: removed, `Ok`); `a_removal_of_nothing_is_done`.
   - `the_start_loads_recorded_groups_and_drops_records_without_settings` (records 7 and 9,
     settings for 7 only, 3 in `built`: groups `[7]`, `box/started/9` gone, 3 untouched).
   Run each: `heavy cargo nextest run -p temper --lib -E 'test(=box_command::tests::NAME)'`
   (check the selected count; the module path is the one you give the tests).
4. Suite tests (`tests/suite/startup.rs`, binary `suite`, default profile), on a fresh
   `temper init` as `a_box_on_a_fresh_init_reaches_serves_link_with_forge_3_dormant` does:
   - `a_box_runs_a_recorded_group_from_its_file_before_serve`: write `box/groups/7` (the
     init document's fake `application.setup` entry without `forge` and `directory`, through
     `group_settings(body, 7)`) and an empty
     `box/started/7`, 0600 in 0700 directories; run the box (`Plan::Stop`); serve's link
     printed, exit 0, nothing left running, `forge-7/admin_token` exists and the box's logs
     hold `fake-forge-7.log`.
   - `a_box_drops_a_started_record_without_settings` (`box/started/9` alone: gone after the
     run; the box's stderr says so).
   `tests/suite/forge.rs`: `forge_prepare_takes_its_entry_from_the_group_file` (a fake
   `box/groups/7`, no setup entry 7 in the document: `temper forge prepare [DIRS] 7` exits 0
   and makes `forge-7/admin_token`).
   Run each: `heavy cargo nextest run -p temper --test suite -E 'test(=startup::NAME)'`.
5. Then `heavy cargo nextest run -p temper` (default profile: lib, suite and the rest; not
   the process tier); `cargo fmt`; `heavy cargo clippy -p temper --all-targets`. Old forms
   never raised (`python3 -I -B scripts/old_forms.py` in `temper/`). Commit and push.

## Report

- The terminal lines the start's scan prints, and anything of the seam that behaved other
  than bx-pipe's report said.
- Every reader of `checked.setup` left in the binary (file and line), for the task that
  removes the document's setup.
