+++
id = "rf-box"
part = "remove-forge"
kind = "feature"
needs = ["fr-box"]
files = ["temper/crates/temper-box/**"]
effort = "high"
minutes = 70
+++

## Goal

The box's `remove FORGE` (contracts, section 4; `notes/remove-forge.md`,
section 6; draft `temper/docs/design/draft/playground-reset.md`, section
4, "Its box group stopped and removed"): the group stopped if it runs,
its group file, its `started` record and its directory deleted, the cache
kept, answered `removed FORGE`.

Acceptance:

- `pipe::Command::Remove(u16)`, parsed from `remove N\n` with the canonical
  number rule of `start`/`stop`; `pipe::removed(N)` writes `removed N\n`.
- On `remove N`: a running or starting group is stopped as `stop N` stops
  it (reverse order, restarts disarmed), then the box deletes
  `<data>/box/groups/N`, `<data>/box/started/N` and the group's directory
  (the decoded group's entry `directory`, which the box takes from its
  caller: `Groups` gains `fn directory(&self, forge: u16, file: &[u8]) ->
  Result<Box<[u8]>, Box<[u8]>>`, the data-relative or absolute path),
  recursively, never following a symbolic link out of it and never
  deleting anything outside `<data>`; then forgets the runtime group and
  answers `removed N`. A number with no group file (and no group) answers
  `removed N` at once. A document group (from the startup steps) is
  refused `failed N not removable` (the document's forges go at the
  cut-over). `busy` and `stopping` refusals as `stop`'s.
- A deletion that fails (an errno other than not-found) answers `failed N
  <why>` and leaves the group stopped and dormant; a later `remove N` tries
  again from the start.
- The removed number's room in the runtime group table is free for a new
  group.
- Tests (library tests): `src/tests/pipe.rs`
  `remove_lines_parse_canonically_and_removed_replies_spell_their_number`;
  `src/tests/host.rs`
  `remove_stops_a_running_group_then_deletes_its_file_record_and_directory`,
  `remove_keeps_the_cache_and_everything_outside_the_directory`,
  `remove_of_a_number_without_a_group_file_answers_removed_at_once`,
  `remove_of_a_document_group_is_refused`,
  `a_failed_deletion_fails_and_a_second_remove_finishes`,
  `a_removed_groups_room_takes_a_new_group`.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (section 4),
  `temper/docs/plan/playground-reset-r5/notes/remove-forge.md` (section 6).
- `temper/docs/design/protocol/setup.md`, 7.1 (the pipe).
- `tasks/fr-box.md` (the runtime groups and `Groups`).

## Owns and may change

- Owns: `temper/crates/temper-box/**`.
- May change: the `Groups` implementations outside the crate
  (`temper/crates/temper/src/groups.rs` if `fr-shell` has merged, else the
  refusing ones beside `Service::new`'s callers), only to add
  `directory`.
- Options: if skein-io offers no recursive removal, walk the directory
  with `Op::List` and remove depth first, bounded by a constant you name
  (entries per directory, depth), failing past it.

## Steps

1. The command and reply.
2. Stop then delete; the forgotten group's room.
3. Tests. Narrow: `heavy cargo nextest run -p temper-box --lib -E
   'test(/remove/)'`.
4. `cargo fmt`; `heavy cargo clippy -p temper-box --all-targets -- -D
   warnings`; focused tests of `temper-box`; `heavy cargo check -p temper
   --tests`. Old-forms from `temper/`. Commit and push.

## Report

As `notes/common.md` says; the deletion's bounds if you walked.
