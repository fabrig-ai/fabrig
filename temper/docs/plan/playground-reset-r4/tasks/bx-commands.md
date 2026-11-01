+++
id = "bx-commands"
part = "box-groups"
kind = "feature"
needs = ["bx-kinds"]
files = [
  "temper/crates/temper/src/setup.rs",
  "temper/crates/temper/src/setup/tests.rs",
  "temper/crates/temper/src/forge.rs",
  "temper/crates/temper/src/forge/tests.rs",
  "temper/crates/temper/src/forge_sign_in.rs",
  "temper/crates/temper/src/fake.rs",
  "temper/crates/temper/src/fake/tests.rs",
  "temper/crates/temper/src/fake/check.rs",
  "temper/crates/temper/src/fake/check/tests.rs",
]
effort = "medium"
minutes = 40
+++

## Goal

The commands a group runs find forge N's entry when the document's setup
has none (contracts, section 3): `temper forge fetch / prepare / run DIRS N`,
`temper forge sign-in DIRS N`, `temper fake forge DIRS N` and `temper fake
check`'s forge read entry N from `<data>/box/groups/N` when the document's
`setup` has no entry N, and refuse, naming the file, when neither has it.
The document's entry wins where it exists (D1).

Acceptance:

- One function in the binary's `setup.rs` gives forge N's entry and whether
  it is the playground's; every one of the commands above calls it in place
  of `checked.setup.entry(N)`.
- With a document naming no forge 7 and a file `box/groups/7` (owned):
  `temper forge prepare DIRS 7` prepares the file's directory as it prepares
  a document's forge (the process tier's `forge_prepare_makes_an_owned_forge_then_remakes_nothing`
  shape, here in a focused test of the binary on a scratch directory with
  a stub Forgejo, or as the existing focused forge tests do it: see
  `temper/crates/temper/src/forge/tests.rs`); with a file for a fake forge,
  `temper fake forge DIRS 9` serves the file's fake.
- With neither: the refusal `temper forge prepare: forge 7: the setup has no
  entry for it and <data>/box/groups/7 is absent` (the path absolute, as the
  binary names files), and its kin for the other commands.
- A file the reader refuses is refused, naming the file and its problem.
- `temper forge reconcile DIRS N` is unchanged (a records-only forge is
  refused already, and a file's forge is records-only).
- The tests below pass, and the binary's existing forge and fake tests.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (section 3, D1).
- `temper/docs/design/protocol/setup.md`, 6.3-6.6 (the owned commands' inputs)
  and 8.4 (the fake forge's deployment form).
- bx-kinds' report and `temper_forge_setup::file` (`read`, `path`, `BYTES`).
- The code: `temper/crates/temper/src/forge.rs` (`start`, `reconcile_settings`),
  `temper/crates/temper/src/forge_sign_in.rs` (`run`), `temper/crates/temper/src/fake.rs`
  (`forge_document`'s `Source::Setup`), `temper/crates/temper/src/fake/check.rs`
  (the forge's mode check), `temper/crates/temper/src/setup.rs`.

## Owns and may change

- Owns the files in the header (`setup/tests.rs` exists after `bx-machine`;
  add to it).
- Neighbours: none.
- Options:
  - Reading the file: `skein_shell::read_private_file` with the effective
    user, as `fake.rs` reads the handover (default: the file is serve's,
    0600), bound `temper_forge_setup::file::BYTES`.
  - If `forge_sign_in.rs` needs only an owned entry's `Owned`, pass it the
    found entry rather than `checked` (default), keeping its signature
    otherwise.

## Steps

1. In `setup.rs`:

   ```rust
   /// Forge `number`'s setup entry: the document's, else its group file's
   /// (contracts, 3), with whether it is the playground's.
   pub(crate) fn entry(
       directories: &super::Directories,
       checked: &temper_forge_setup::check::Checked,
       number: u16,
   ) -> Result<(temper_forge_setup::setup::Entry, bool), String>;
   ```

   The document's entry gives `false`. The file's path is
   `directories.data` joined with `temper_forge_setup::file::path(number)`.
2. Call it in `forge::start` (fetch, prepare, run, sign-in), `fake.rs`'s
   `Source::Setup`, and `fake/check.rs`'s forge read; each command's
   refusal words keep their prefix (`Refused::line`, `prefixed`).
3. Tests:
   - `setup/tests.rs`: `an_entry_comes_from_the_document_before_its_file`,
     `an_entry_absent_from_both_names_the_file`,
     `a_refused_group_file_is_named`.
   - `forge/tests.rs`: `prepare_reads_a_forge_from_its_group_file`
     (an owned file on a scratch data directory; the preparation refused or
     run as the existing focused tests drive `start`: assert at least that
     the entry is found and the directory made 0700).
   - `fake/tests.rs`: `the_fake_forge_reads_its_entry_from_its_group_file`.
4. `cargo fmt`; `heavy cargo clippy -p temper --all-targets -- -D
   warnings`; `heavy cargo nextest run -p temper --lib`. Commit, push.

## Report

Fab's sections; under `## Interfaces`: `setup::entry` and each command's new
refusal words.

## Answers

2026-11-01. May this task update `fake/check/tests.rs`'s `no-entry` row
for the new refusal? Yes: the file joins this task's files (no other task
owns it). Change only that row's expected refusal to the new words,
`forge 1: the setup has no entry for it and <data>/box/groups/1 is absent`
with the path absolute, built as the row's neighbours build theirs
(`path("no-entry", "data/box/groups/1")` if that is the case's data
directory); no other assertion there changes.
