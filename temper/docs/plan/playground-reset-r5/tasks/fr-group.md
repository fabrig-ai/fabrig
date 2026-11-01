+++
id = "fr-group"
part = "forge-records"
kind = "interface"
needs = []
files = ["temper/crates/temper-forge-setup/src/group.rs", "temper/crates/temper-forge-setup/src/lib.rs", "temper/crates/temper-forge-setup/src/setup.rs", "temper/crates/temper-forge-setup/src/step.rs", "temper/crates/temper-forge-setup/src/tests/**", "temper/crates/temper-fake/src/steps.rs", "temper/crates/temper-fake/src/tests/steps.rs"]
effort = "high"
minutes = 90
+++

## Goal

The box's group file: its codec, one module of `temper-forge-setup`
(`group`), and the steps of one group built from it by its kind, owned
(`temper-forge-setup`) or fake (`temper-fake`). Written by serve and read
by the box and the forge's commands in later tasks (`fr-svc-route`,
`fr-box`, `fr-shell`); nothing reads it yet. Today's document-driven
`steps` stay as they are.

Acceptance:

- `temper_forge_setup::group` (new `src/group.rs`, `pub mod group` in
  `lib.rs`) with exactly the shapes of `notes/forge-records.md`, section
  4: `GroupFile { forge: u16, kind: GroupKind, entry: setup::Entry }`,
  `GroupKind::{Owned, Fake { llm: bool }}`, `GROUP_BYTES: u32 = 8192`,
  `encode(&GroupFile) -> Box<[u8]>` (the JSON of the note: `forge`,
  `kind` `"owned"`/`"fake"`, `llm` for fake alone, `setup` the entry as
  `application.setup[]` writes it, number and directory included) and
  `decode(bytes: &[u8], limits: &Limits, directories:
  &config::Directories) -> Result<GroupFile, Problems>`, which reads
  `setup` through the same entry parser and path resolution
  `Setup::read` uses (factor that resolution out of `Setup::read` into a
  function both call), and refuses: more than `GROUP_BYTES`; a `forge`
  disagreeing with the entry's; `owned` with a non-owned mode, `fake`
  with a non-fake mode, `llm` on `owned`; any other member; trailing bytes.
- The encoder writes what the decoder reads back: an entry encoded then
  decoded under the same directories is equal (paths written as the
  resolved absolute paths, which `resolve_path` keeps).
- `temper_forge_setup::step::group(file: &GroupFile, flags: &[Box<[u8]>],
  listeners: Box<[core::net::SocketAddr]>, forgejo_listens: bool) ->
  Group` for `GroupKind::Owned`: the group `steps` builds today for one
  owned entry with `asked` (fetch, prepare, forgejo, runner), `recorded:
  true`, `before: 0`; `steps` itself calls it for each asked owned entry so
  the two never differ. Panics (an owner's error) on a fake kind.
- `temper_fake::steps::group(file: &GroupFile, demo: Option<&Demo>,
  flags: &[Box<[u8]>], listeners: Box<[core::net::SocketAddr]>) -> Group`
  for `GroupKind::Fake`: with `llm: true` today's playground group (fake
  llm, forge prepare, fake forge; label `Playground`; `recorded: false`),
  its llm step from `demo` as today (`None` with `llm: true` is an owner's
  error); with `llm: false` today's two steps of a non-demo fake forge, as
  one group, `recorded: false`, label the forge's. Today's `steps` builds
  its playground group through it.
- Tests: `temper-forge-setup`'s library tests (a new `src/tests/group.rs`):
  `an_owned_group_file_round_trips_with_its_resolved_paths`,
  `a_fake_group_file_round_trips_with_and_without_the_llm`,
  `a_group_file_refuses_a_disagreeing_number_a_wrong_mode_and_trailing_bytes`,
  `a_group_file_past_its_bound_is_refused`,
  `an_owned_group_file_builds_the_steps_its_document_entry_builds`;
  `temper-fake`'s `src/tests/steps.rs`:
  `a_playground_group_file_builds_todays_playground_group`,
  `a_fake_group_file_without_the_llm_builds_the_fake_forges_two_steps`.
  Today's step tests pass unchanged.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (section 4),
  `temper/docs/plan/playground-reset-r5/notes/forge-records.md` (section 4).
- `temper/docs/design/protocol/setup.md`, 7.1 to 7.3 (the box, its
  groups, its terminal labels) and 8 (the setup entries).
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s2-box.md`,
  section 1 (`temper-forge-setup`'s `step.rs`, `temper-fake`'s
  `steps.rs`) and section 4 (the setup entry's parser).

## Owns and may change

- Owns: `temper-forge-setup`'s `src/group.rs` (new), `src/lib.rs`,
  `src/setup.rs` (the shared resolution only; the entry parser made
  `pub(crate)` if it is not), `src/step.rs`, `src/tests/` (a new
  `group.rs` and its `mod` line); `temper-fake`'s `src/steps.rs` and
  `src/tests/steps.rs`.
- Nothing else. `temper-box` is not touched: its boundary test allows it
  only `Step`, `steps`, `Steps`, `Group`, `Stage`, `Kind` of the setup
  crate, and it never reads the group file's codec (it gets steps from
  its caller, `fr-box`).
- Options: if the setup entry has no JSON encoder today, write one in
  `group.rs` for the entry's forms (`managed` is never a group's); if an
  owned entry's `"fetched"` binaries resolve to versioned cache paths that
  cannot be written back as `"fetched"`, encode the resolved paths and say
  so.

## Steps

1. `group.rs` codec and its tests.
2. `step::group`, `steps` through it; `temper_fake::steps::group`, the
   playground group through it; their tests.
3. Narrow: `heavy cargo nextest run -p temper-forge-setup --lib -E
   'test(/group/) | test(/steps/)'`; `heavy cargo nextest run -p
   temper-fake --lib -E 'test(/steps/)'`.
4. `cargo fmt`; `heavy cargo clippy -p temper-forge-setup -p temper-fake
   --all-targets -- -D warnings`; focused tests of both and `heavy cargo
   check -p temper -p temper-box --tests`. Old-forms from `temper/`.
   Commit and push.

## Report

As `notes/common.md` says; the JSON your encoder writes for each kind, in
one example each.
