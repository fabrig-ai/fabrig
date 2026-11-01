+++
id = "bx-kinds"
part = "box-groups"
kind = "feature"
needs = ["box-pipe", "bx-machine"]
files = [
  "temper/crates/temper-forge-setup/src/step.rs",
  "temper/crates/temper-forge-setup/src/file.rs",
  "temper/crates/temper-forge-setup/src/lib.rs",
  "temper/crates/temper-forge-setup/src/setup.rs",
  "temper/crates/temper-forge-setup/src/tests.rs",
  "temper/crates/temper-forge-setup/src/tests/steps.rs",
  "temper/crates/temper-forge-setup/src/tests/file.rs",
  "temper/crates/temper-fake/src/steps.rs",
  "temper/crates/temper-fake/src/tests/steps.rs",
]
effort = "high"
minutes = 50
+++

## Goal

The steps of a kind from one entry, and the group file's one reader
(contracts, section 3): `temper-forge-setup` gives an owned Forgejo's group
from its entry, `temper-fake` the playground's (or a fake forge's) group
from its entry, each the group `steps` gives today for the same entry; and
`temper_forge_setup::file::read` reads `<data>/box/groups/N` into an entry,
the reader every user of a group file calls (the box's start, `temper forge
... DIRS N`, `temper fake forge DIRS N`: `bx-wire` and `bx-commands`). A
group renumbered with `Group::numbered` is the group built for that number,
started notice included.

Acceptance:

- `temper_forge_setup::group`, `temper_fake::steps::group` and
  `temper_forge_setup::file::{read, path, GroupFile}` exist (below);
  `temper_forge_setup::steps` and `temper_fake::steps` build their groups by
  them, and every existing test of both crates passes unchanged.
- For every owned entry shape (fetched or not, with or without a runner,
  Forgejo listening or not) and the playground's: `group(entry for 65535)
  .numbered(N) == group(entry for N)`; with a started notice set on the
  Forgejo step naming the label, the notice renumbers too.
- `file::read` reads a file of contracts 3's form into the same `Entry` that
  `Setup::read` gives for the same entry in a document (paths resolved
  against the data and cache directories), and refuses, at a path rooted at
  the file's members (`setup.mode...`, `playground`): an entry whose `forge`
  is not N; an owned entry without `"start": "asked"`; `playground: true`
  with an entry that is not fake; any other top-level member.
- The tests below pass.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (section 3).
- `temper/docs/design/protocol/setup.md`, 7.1 (the groups), 7.3 (what the
  box names: `Group`, `label`), 8 (a setup entry), 8.4 (the demo's group).
- box-pipe's and bx-machine's reports (`## Interfaces`), and their code:
  `Group::numbered` (`temper-forge-setup/src/step.rs`), `setup::Template`
  and the entry reader it shares (`temper-forge-setup/src/setup.rs`).
- The code: `temper/crates/temper-forge-setup/src/step.rs` (`steps`,
  `forge`), `temper/crates/temper-fake/src/steps.rs` (`steps`, `command`),
  `temper/crates/temper/src/box_command.rs` (`owned_notices`: the notice's
  words, which `bx-wire` sets on these groups).

## Owns and may change

- Owns the files in the header; `file.rs` and `tests/file.rs` are new.
- In `setup.rs`, only: a `pub(crate)` per-entry read with its paths resolved
  (what `Setup::read` does for one list item), for `file::read` to call;
  `Setup::read` calls it, its behaviour unchanged.
- Neighbours: none. The binary's wiring is `bx-wire`'s and `bx-commands`'.
- Options:
  - The file's document bounds: `jig_protocol_config::document` with
    `DocumentLimits { bytes: file::BYTES, tokens: file::BYTES, depth: 32 }`,
    `pub const BYTES: u32 = 65_536` (default; the box reads at most this).
    If box-pipe or bx-sweep fixed another bound in `temper_box::limits`, use
    the same number and say so.
  - A fake entry with `playground: false` (a seeded fake forge, D1, whose
    document the box read without it): its group is `prepare N` then `fake
    forge N`, labelled `Forge N (fake)`, not recorded (default). If a test
    of today's steps shows those two must stay always steps for a
    document's fake forge, keep `steps` as it is for the document and use
    the group for files alone.

## Steps

1. `temper-forge-setup/src/step.rs`:

   ```rust
   /// An owned Forgejo's group from its entry, as `steps` places an asked
   /// one: fetch when fetched, prepare, Forgejo (ready at, and on Linux
   /// listening on, its `listen`), its runner; labelled `Forge N (Forgejo)`,
   /// recorded, `before` 0 (the caller places it). None for another mode.
   #[must_use]
   pub fn group(entry: &Entry, flags: &[Box<[u8]>], forgejo_listens: bool) -> Option<Group>;
   ```

   re-exported from `lib.rs` beside `steps`; `steps` builds an asked owned
   entry's group with it, and an unasked one's steps from the same list.
   `Group::numbered` also renumbers each step's started notice, word by
   word as the name (box-pipe kept it as is: an owned Forgejo's notice
   begins with its label, `Forge N (Forgejo): `, which `bx-wire` sets).
2. `temper-fake/src/steps.rs`:

   ```rust
   /// A fake entry's group: with `llm`, the playground's (the fake LLM
   /// listening on `llm`, then `prepare N`, then `fake forge N` on the
   /// entry's `listen`), labelled `Playground`; without, the fake forge's
   /// two, labelled `Forge N (fake)`. Never recorded; `before` 0. None for
   /// another mode.
   #[must_use]
   pub fn group(entry: &Entry, llm: Option<core::net::SocketAddr>, flags: &[Box<[u8]>]) -> Option<Group>;
   ```

   `steps` builds the demo's group with it, `llm` the demo's LLM `listen`.
3. `temper-forge-setup/src/file.rs`, names this task creates:

   ```rust
   /// One forge's group file, `<data>/box/groups/N` (contracts, 3).
   #[derive(Clone, PartialEq, Eq, Debug)]
   pub struct GroupFile { pub entry: Entry, pub playground: bool }
   /// The file's bound.
   pub const BYTES: u32 = 65_536;
   /// `box/groups/N`, beneath the data directory.
   #[must_use]
   pub fn path(forge: u16) -> Box<[u8]>;
   /// Reads forge `forge`'s file whole: `{"setup": ENTRY, "playground": BOOL}`.
   pub fn read(
       bytes: &[u8],
       forge: u16,
       limits: &Limits,
       directories: &jig_protocol_config::Directories,
   ) -> Result<GroupFile, Problems>;
   ```

   `Problems` is the crate's `problem::Problems`, printed by `problem::lines`
   as the setup check's are.
4. Tests:
   - `temper-forge-setup/src/tests/steps.rs`:
     `an_owned_entrys_group_is_its_asked_group_in_steps` (each entry shape);
     `a_group_built_for_one_number_and_numbered_is_the_group_for_another`
     (each shape, a notice `Forge 65535 (Forgejo): started\n...` set on the
     Forgejo step becomes `Forge 7 (Forgejo): started\n...`).
   - `temper-forge-setup/src/tests/file.rs`:
     `a_group_file_reads_as_its_entry_in_a_document` (the init template's
     forge-3 entry and forge-1 entry, each in a file and in a document);
     `a_group_file_refuses_another_number_an_unasked_owned_entry_and_a_playground_not_fake`;
     `a_group_file_refuses_another_member_and_bytes_past_its_bound`.
   - `temper-fake/src/tests/steps.rs`:
     `the_playground_group_is_the_demos_group_in_steps`;
     `a_playground_group_numbered_is_the_group_for_that_number`;
     `a_fake_forges_group_without_llm_has_its_two_steps`.
5. `cargo fmt`; `heavy cargo clippy -p temper-forge-setup -p temper-fake
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-forge-setup --lib` and `-p temper-fake --lib`; dependents build:
   `heavy cargo check -p temper-box -p temper-forge-setup-shell
   -p temper-service -p temper --tests`. Commit, push.

## Report

Fab's sections; under `## Interfaces`: `temper_forge_setup::group`,
`temper_fake::steps::group`, `file::{GroupFile, BYTES, path, read}`, the
`setup.rs` function you exposed, and `Group::numbered`'s added notice rule.
