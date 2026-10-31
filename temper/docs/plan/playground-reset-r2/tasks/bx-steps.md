+++
id = "bx-steps"
part = "box-groups"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-forge-setup/src/group.rs",
  "temper/crates/temper-forge-setup/src/lib.rs",
  "temper/crates/temper-forge-setup/src/setup.rs",
  "temper/crates/temper-forge-setup/src/step.rs",
  "temper/crates/temper-forge-setup/src/tests.rs",
  "temper/crates/temper-forge-setup/src/tests/group.rs",
]
effort = "high"
minutes = 60
+++

## Goal

`temper-forge-setup` gains the box group's file and its steps by kind
(`temper/docs/design/draft/playground-reset.md`, 3, "The box knows kinds,
not forges"; `notes/contracts.md`, 3): a codec for `<data>/box/groups/N`
(`compose` writes it, `read` checks it into a setup entry) and a builder
that turns one group file into the group's steps, an owned Forgejo's or
the fakes' (the playground's fake LLM among them), plus the group's
removal step. Nothing calls them yet: the box (`bx-box`), the child
commands (`bx-children`) and serve (`bx-serve`) do. Today's
`steps(setup, ...)` and `temper_fake::steps` stay as they are.

Acceptance, all in `temper-forge-setup`'s lib target, module
`tests::group` (new):

- `an_owned_group_fetches_prepares_then_runs_forgejo_and_its_runner`:
  an owned entry with both binaries "fetched" and a runner builds, in
  order, `forge fetch DIRS N` (Fetch, once), `forge prepare DIRS N`
  (Prepare, once), `forge run DIRS N forgejo` (Run, long, ready at its
  `listen`, listening on it when `forgejo_listens`, its started notice
  set), `forge run DIRS N runner` (Run, long); label `Forge N (Forgejo)`,
  `recorded` true.
- `an_owned_group_without_fetched_binaries_or_runner_has_neither_step`.
- `forgejo_listens_false_leaves_forgejo_its_own_bind` (no listener on
  the Forgejo step; still ready at `listen`).
- `the_owned_started_notice_names_the_api_the_person_and_its_headers`:
  the notice bytes are exactly those `temper/crates/temper/src/box_command.rs`'s
  `owned_notices` makes today for the same entry (copy its expected
  lines from that function: `Forge N (Forgejo): started`, `... api
  http://ADDR/api/v1`, `... person LOGIN, read header DIR/person_read.header,
  write header DIR/person_write.header`, each LF-ended).
- `the_playground_group_runs_the_fake_llm_then_prepares_and_runs_its_forge`:
  a fake entry with `playground: true` builds `fake llm DIRS` (Run,
  long, listening on the machine's LLM port), `forge prepare DIRS N`
  (Prepare, once), `fake forge DIRS N` (Run, long, listening on the
  entry's `listen`); label `Playground`, `recorded` false; the same
  command words and order as `temper/crates/temper-fake/src/steps.rs`
  gives the demo's group today.
- `a_playground_group_without_an_llm_port_is_refused`.
- `a_fake_group_outside_the_playground_prepares_and_runs_its_forge_recorded`:
  label `Forge N (fake)`, `recorded` true, no fake LLM step.
- `a_managed_entry_builds_no_group` (refused with a line naming the forge).
- `every_group_has_its_removal_step`: `forge remove DIRS N`, once, name
  `remove N`, for each kind.
- `compose_then_read_gives_the_entry_numbered_and_resolved`: `compose`
  of an owned and of a fake `S` (the setup entry as
  `temper/crates/temper-init/template/temper.json` has them today, less
  its `forge` member) then `read` gives the `Entry` `Setup::read`
  gives for the same entry in a document, its relative `directory`,
  `forgejo`, `runner.binary` and `fake.git` resolved the same way, and
  the `playground` bit.
- `read_refuses_another_forge_an_unknown_member_and_an_oversized_file`:
  a file whose `setup.forge` is not N; a member beside `setup` and
  `playground`; `playground` not a boolean; a file longer than
  `FILE_BYTES`; not JSON.
- `compose_refuses_a_setup_that_is_not_an_object_or_names_its_forge`.
- `names_and_labels_fit_their_bounds`: for forge 65535, every step name
  is at most `NAME_BYTES` and every label at most `LABEL_BYTES`, and
  each group has at most `STEPS` steps (the box sizes its slots from
  these three).

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 2 (the
  definition's `setup` member) and 3 (the group file, the steps by
  kind, the commands).
- `temper/docs/design/draft/playground-reset.md`, 3 ("The box knows
  kinds, not forges") and 4 ("Its box group stopped and removed").
- `temper/docs/design/protocol/setup.md`, 7.1, 7.2, 7.3 (the commands,
  groups and labels as they are), 8 (the setup members), 8.4 (the
  demo's group).
- Code: `temper/crates/temper-forge-setup/src/step.rs` (`steps`, `forge`),
  `src/setup.rs` (`Setup::read`, `entry`, `resolve_path`), `src/check.rs`
  (`checked`, which calls `given::values`/`given::check`),
  `temper/crates/temper-fake/src/steps.rs` (the demo's group and its
  `command` helper), `temper/crates/temper/src/box_command.rs`
  (`owned_notices`).

## Owns and may change

- Owns `temper/crates/temper-forge-setup/src/group.rs` (new) and its
  test module `src/tests/group.rs` (new, with its `mod group;` line in
  `src/tests.rs`).
- May change `src/lib.rs` (`pub mod group;`), `src/setup.rs` (factor the
  per-entry path resolution `Setup::read` does into a private function
  both it and `group::read` call, and make `entry` reachable from
  `group.rs` as `pub(crate)`; no change of what `Setup::read` returns),
  `src/step.rs` (make the private `forge` helper `pub(crate)` for
  `group.rs`; add `pub fn serve(flags: &[Box<[u8]>], listeners:
  Box<[core::net::SocketAddr]>) -> Step`, the serve step `steps` ends
  with today, and have `steps` call it).
- Not yours: `temper-fake` (its `steps.rs` stays as it is; `bx-box`
  retires it), `temper-box`, the `temper` binary. `Group`, `Steps`,
  `Stage`, `Kind` and `Step` keep their shapes: no field is added, so
  no literal elsewhere changes.

## Steps

Module `temper/crates/temper-forge-setup/src/group.rs`, its doc citing
`temper/docs/design/draft/playground-reset.md, 3` and
`protocol/setup.md, 7.3`:

```rust
/// The file's bound, the box's read buffer (contract 3).
pub const FILE_BYTES: u32 = 65_536;
/// Most steps a group runs: an owned Forgejo's fetch, prepare, forgejo, runner.
pub const STEPS: u32 = 4;
/// Longest step name and label a group gives; the box sizes its output from them.
pub const NAME_BYTES: u32 = /* computed for forge 65535, at least 24 */;
pub const LABEL_BYTES: u32 = /* likewise */;
/// The directory beneath the data directory, and one group's file in it.
pub const DIRECTORY: &[u8] = b"box/groups";
#[must_use]
pub fn path(forge: u16) -> Box<[u8]>; // b"box/groups/N"

/// One group's file, checked: the forge's setup entry, numbered, its paths resolved.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GroupFile {
    pub entry: crate::setup::Entry,
    pub playground: bool,
}
/// What the box supplies to build any group: the children's directory
/// flags, the directories that resolve an entry's paths, the setup
/// limits, the machine's fake LLM port, and whether Forgejo takes its
/// listener from the box (Linux) or binds it itself.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Machine {
    pub flags: Box<[Box<[u8]>]>,
    pub directories: jig_protocol_config::Directories,
    pub limits: skein_forge_setup::limits::Limits,
    pub llm: Option<core::net::SocketAddr>,
    pub forgejo_listens: bool,
}
/// A group and the once step that removes its forge's directory.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Built {
    pub group: crate::step::Group,
    pub remove: crate::step::Step,
}

/// `{"setup": S + "forge": N, "playground": P}`, S one JSON object without `forge`.
pub fn compose(setup: &[u8], forge: u16, playground: bool) -> Result<Box<[u8]>, Box<[u8]>>;
/// Checks a file's bytes as forge N's group; a refusal is one line, no LF.
pub fn read(bytes: &[u8], forge: u16, machine: &Machine) -> Result<GroupFile, Box<[u8]>>;
/// The group's steps by its kind; a managed forge has none.
pub fn build(file: &GroupFile, machine: &Machine) -> Result<Built, Box<[u8]>>;
```

- `compose` checks `setup` parses as one JSON object (with
  `jig_protocol_config::document` and the bounds `Setup::read`'s callers
  use) that has no `forge` member, then writes
  `{"setup":{"forge":N,` followed by S's bytes after its opening brace
  (an empty S gives `{"forge":N}`), then `,"playground":true|false}`.
  It changes nothing else of S: its `listen`s are already the machine's
  ports (contract 2: the definition is composed from the machine's
  document). Longer than `FILE_BYTES` is refused.
- `read` parses the bytes (at most `FILE_BYTES`), requires an object of
  exactly `setup` and `playground` (boolean), reads `setup` with
  `setup.rs`'s `entry` at path `setup`, resolves its paths as
  `Setup::read` does (data for `directory` and `fake.git`, cache for
  `forgejo` and `runner.binary`, the same function), requires
  `entry.forge == forge`, then runs `given::check(&given::values(&entry,
  None, &machine.limits))` as `check::checked` does per entry. The
  refusal line names the file (`box/groups/N`) and the path and reason,
  in the words `problem`'s lines give a document's problems.
- `build`, by `entry.mode`:
  - `Owned`: the steps `steps` builds for an owned entry today, in the
    same order and with the same `Stage`s, the Forgejo step's listener
    given only when `machine.forgejo_listens`, and its started notice
    set to the bytes `box_command.rs`'s `owned_notices` builds (move the
    composition here, from `entry.directory`, `owned.listen` and
    `owned.person.login`; `bx-box` deletes it there). Label `Forge N
    (Forgejo)`, `recorded: true`.
  - `Fake` with `playground`: `fake llm DIRS` (Run, long, listening on
    `machine.llm`, refused when `machine.llm` is none), then `forge
    prepare DIRS N` (Prepare, once), then `fake forge DIRS N` (Run,
    long, listening on `fake.listen`). Label `Playground`, `recorded:
    false`: as the demo's group today, it writes no `started` record, so
    a box start never starts it by itself (setup.md, 8.4).
  - `Fake` without `playground`: `forge prepare DIRS N`, `fake forge DIRS
    N`, the same kinds; label `Forge N (fake)`, `recorded: true`.
  - `Managed`: refused, `forge N is managed: the box runs no group for it`.
  - `before` is 0 in every group: the box places its groups itself
    (`bx-machine`).
  - `remove`: `Step::new("remove N", ["forge", "remove", FLAGS..., "N"],
    Kind::Once)`, built by `step.rs`'s `forge` helper.
  - Command lines are built as `temper-fake/src/steps.rs`'s `command`
    builds them (words, then the flags, then the number); copy that
    helper into `group.rs` (no closures: the crate's code uses explicit
    loops and matches).
- Tests in `src/tests/group.rs`, plain unit tests on fixed entries
  (reuse `src/tests/fixtures.rs` where it has an owned or fake entry;
  write the JSON inline otherwise). Directories as in
  `src/tests/steps.rs` (`/config`, `/data`, `/cache`).

Then: `cargo fmt`; `heavy cargo clippy -p temper-forge-setup
--all-targets -- -D warnings`; `heavy cargo nextest run -p
temper-forge-setup --lib -E 'test(/group::/)'` (check the count is the
test list above); then the focused tests of the package and its direct
dependents: `heavy cargo nextest run -p temper-forge-setup -p temper-box
-p temper-fake -p temper-forge-setup-shell -p temper-service -p temper`
(nothing outside `temper-forge-setup` changes meaning; `steps` gives the
same `Steps`). Old forms: `python3 -I -B scripts/old_forms.py` from
`temper/`. Commit and push your branch.

## Report

- The values of `NAME_BYTES` and `LABEL_BYTES` and how you derived
  them.
- Whether `read`'s refusal lines reuse `problem`'s formatting, and an
  example line.
- Anything of `given::check` that a single entry cannot satisfy without
  the document's forges (`check::members` needs them, and is not run).
