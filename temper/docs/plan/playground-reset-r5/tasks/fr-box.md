+++
id = "fr-box"
part = "forge-records"
kind = "feature"
needs = []
files = ["temper/crates/temper-box/**"]
effort = "high"
minutes = 110
+++

## Goal

The box runs groups from group files (draft
`temper/docs/design/draft/playground-reset.md`, section 3, "The box knows
kinds, not forges"; contracts, section 4): `start N` for a forge the
startup steps do not name reads `<data>/box/groups/N`, has its caller
build the group's steps from it, and runs it; at its start the box runs
each group whose `started` record exists from its group file. The
document's groups keep working beside them until the cut-over
(`notes/forge-records.md`, section 0). The box stays wiring only: it never
reads the group file's JSON.

Acceptance:

- `temper_box::host::Groups` (new trait in `host.rs`, or a new
  `groups.rs` re-exported there):
  ```rust
  /// Builds a group's steps from its group file's bytes (contracts, 4).
  pub trait Groups: core::fmt::Debug {
      fn group(&self, forge: u16, file: &[u8]) -> Result<temper_forge_setup::Group, Box<[u8]>>;
  }
  ```
  `Service::new(limits, steps, settings, groups: Box<dyn Groups>)`; its
  callers in temper (`temper/crates/temper/src/box_command.rs` and the
  fake-backends path) pass a `Groups` that refuses every file (`Err(b"no
  group files yet")`) until `fr-shell` gives the real one; the crate's test
  fixtures pass a scripted one.
- **`start N`** for a number with no group among the steps: the box reads
  `<data>/box/groups/N` (at most 8192 bytes, `temper_forge_setup`'s
  `GROUP_BYTES` written here as a local constant with a comment naming
  it), calls `groups.group(N, bytes)`, and runs the group as an asked
  dormant group runs today (its stages reported `step N ...`, `started
  N`, its record written once it stands if `recorded`). No file, an
  unreadable one, or a refused build answers `failed N <why>`. A group so
  built stays known until the box stops (a later `stop N` / `start N`
  acts on it as on a document group).
- **At the box's start:** it lists `<data>/box/started/` (skein-io's
  `Op::List`) and for each `N` whose group the steps do not name, reads
  its group file and builds it as above; those groups join the startup
  chain exactly as recorded document groups do today (`Machine::start`).
  A record whose file is missing or refused is a startup fact naming it,
  and the box goes on without it.
- **Room:** runtime groups are bounded by a new `Limits.groups: u32`
  (default 2: the machine's one owned and one fake port) of at most
  `GROUP_STEPS = 4` steps each; the machine, children, logs, records and
  terminal bounds are sized with that room at `new`, so a runtime group
  never grows a table. A `start N` past the room answers `failed N box
  full`.
- The boundary tests (`src/tests/boundary.rs`) hold unchanged: the box
  names only `Step`, `steps`, `Steps`, `Group`, `Stage`, `Kind` of the
  setup crate.
- Tests (the crate's library tests): `src/tests/host.rs`
  `a_start_for_an_unknown_forge_runs_the_group_built_from_its_file`,
  `a_start_without_a_group_file_or_with_a_refused_one_fails_naming_it`,
  `a_recorded_group_file_group_restarts_at_the_next_box_start`,
  `a_start_past_the_group_room_fails_box_full`; `src/tests/machine.rs`
  `runtime_groups_stop_in_reverse_and_go_dormant_as_document_groups`.
  Every existing test passes unchanged.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (section 4),
  `temper/docs/plan/playground-reset-r5/notes/forge-records.md`
  (sections 0 and 4).
- `temper/docs/design/protocol/setup.md`, 7.1 to 7.3 (the box, its pipe,
  its records and terminal).
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s2-box.md`,
  section 1 (the box's machine, pipe, records, restart) and its readers'
  list.

## Owns and may change

- Owns: `temper/crates/temper-box/**`.
- May change: the calls of `temper_box::host::Service::new` in
  `temper/crates/temper/src/box_command.rs` and
  `temper/crates/temper/src/fake_backends.rs` (the new argument, a
  refusing `Groups` defined beside the call), and `Limits` literals
  (`groups: 2`).
- Options: if the tree's lints refuse `Box<dyn Groups>`, make `Service`
  generic over `G: Groups` and say so. If `Op::List` is not reachable from
  the box's file layer, probe `box/started/N` for each `N` named by a file
  under `box/groups/` listed instead, or, failing both, ask (a question):
  do not probe a number range.

## Steps

1. `Groups`, the new argument, the refusing callers.
2. Runtime groups in the machine, records and terminal; their room.
3. The start's listing; `start N` from a file.
4. Tests on the crate's fixtures (`src/tests/fixtures.rs`, a scripted
   `Groups`).
5. Narrow: `heavy cargo nextest run -p temper-box --lib -E 'test(/host/) |
   test(/machine/) | test(/boundary/)'`.
6. `cargo fmt`; `heavy cargo clippy -p temper-box --all-targets -- -D
   warnings`; focused tests of `temper-box`; `heavy cargo check -p temper
   --tests`. Old-forms from `temper/`. Commit and push.

## Report

As `notes/common.md` says; how the box lists its records, and the room
figures you sized.
