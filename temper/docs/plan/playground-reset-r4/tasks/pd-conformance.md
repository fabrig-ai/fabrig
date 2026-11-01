+++
id = "pd-conformance"
part = "project-deletion"
kind = "feature"
needs = ["pd-delete"]
files = [
  "jig/testing/jig-conformance/src/scenarios.rs",
  "jig/tests/conformance/src/scenarios.rs",
  "jig/tests/conformance/src/scenarios/removal.rs",
  "jig/tests/conformance/tests/suite/removal.rs",
  "jig/tests/conformance/tests/suite/main.rs",
  "jig/testing/jig-test-store-files/src/store.rs",
  "jig/testing/jig-test-store-files/src/files.rs",
]
effort = "high"
minutes = 55
+++

## Goal

jig's conformance world tests a project's deletion (draft 5, "Tested
in jig's conformance world", and draft 8): a scenario that gives the
project a row in each of jig's store families, deletes it by the
deployment's act, and is cut by a crash at chosen points; the
scenario's end check, the referee's for this scenario, finds no row
that names the project or one of its tasks, but its `ClosedLedger`
rows, and no `Removal` row.

Acceptance: the tests below pass at their seeds and under the
scenario's crash cuts; the other scenarios are unchanged.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md`,
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (D5).
- `temper/docs/design/draft/playground-reset.md`, sections 5 and 8.
- `jig/docs/design/protocol/store.md`, 4.4 (jig's families: core,
  tasks, people, notes, accounts, settings).
- The model: `Scenario::ConversationEndsWhileRead`
  (`jig/testing/jig-conformance/src/scenarios.rs` `program`;
  `jig/tests/conformance/src/scenarios/conversations.rs`, its actions
  and `check_ending` scanning `world.store.rows`;
  `jig/tests/conformance/tests/suite/scenarios.rs`'s
  `conversation_ends_while_read_chosen_cut_ends_the_story`).
- `pd-delete`'s `jig_core_world::removal::rows_naming` (the predicate
  to match; this world keeps its own over `world.store.rows`).

## Owns and may change

Owned: the `files` above. Neighbours: every other implementer of
`ScenarioApplication`/the kit's `Scenario` and `Action` enums
(`jig/tests/ops/src/conformance/**`, `temper/tests/conformance/src/**`):
the arm alone, the new scenario refused or skipped there as their
other unsupported scenarios are. `jig/tests/conformance/src/lib.rs`:
`Config`'s grant of `ActKind::RemoveProject` to the test connector for
this scenario.

Pre-approved: settings have no project scope in jig today (survey s3,
4): the settings family's case asserts that no settings row names the
project and builds none; if a family cannot be given a row of the
project through this world's application, say which and assert its
absence only.

## Steps

1. Kit: `Scenario::ProjectRemoved` and the `Action`s it needs (e.g.
   `MakeProjectRows` building: a live and an ended conversation with a
   delegate and replies read by a second party, a held effect, a note
   of the project's scope, a pool set, an account whose uses name the
   project, a retained party answer; then `RemoveProject` by the
   deployment's act), with its `program`: drain after each, and its
   chosen crash cuts after the removal's decision and inside its
   erasure.
2. `jig/tests/conformance/src/scenarios/removal.rs`: the actions, and
   `check_ending(world)`: per family, every row of `world.store.rows`
   whose key or value names the project or a task recorded before the
   removal is a violation naming its key, `ClosedLedger` rows of the
   project's period and pools excepted; no `Removal` row; the
   transcript files of the tasks gone (the world's durable files, as
   the store tests read them).
3. Tests (`tests/suite/removal.rs`, `mod removal;` in `main.rs`): one
   per family, `a_deleted_project_leaves_no_core_row`, `..._tasks_row`,
   `..._people_row`, `..._notes_row`, `..._accounts_row`,
   `..._settings_row`, each running the scenario and checking its
   family; `a_deleted_project_cut_at_each_chosen_point_ends_the_same`
   (every cut of the program); `a_deleted_projects_scenario_replays_from_its_seed`.
4. Checks: `cargo fmt`; clippy on `jig-conformance`,
   `jig-conformance-world`; `heavy cargo nextest run -p
   jig-conformance-world --test suite`; `fuzzy` for this world's
   simulation if it draws scenarios (the failing test alone); `heavy
   cargo check --tests -p jig-ops-world -p temper-conformance-world`
   (names from their `Cargo.toml`). Commit, push.

## Report

fab's sections; `## Interfaces`: the kit's new variants; each family's
rows the scenario built; any row the check found that no deletion step
erases (with its key), which is a question for `pd-delete`'s family.

## Answers

2026-11-01. Q: may `Testing::store` build a directory-backed store for
`Scenario::ProjectRemoved` (A), or may `jig-test-store-files` gain a
read-only transcript accessor for memory stores (B)? A: (B). The world
keeps one store construction for every scenario: a directory-backed
store needs a nonempty patch-set fixture, which runs patches at open
and puts this one scenario on another store than the rest. The memory
store already keeps its transcripts in `Files` (its `unread_transcript`
names them `{task >> 12:016x}/{task:016x}`, relative to the transcripts
root; the disk's names carry `transcripts/`). Add, in `store.rs`,
`pub fn transcript_bytes(&mut self, task: u64) -> Option<Vec<u8>>`,
"the task's transcript file's bytes, or `None` when it has none,
whichever store backs the shell", answering from `disk` (by `read` on
the `transcripts/...` name) or from `files` (a `pub(super)` read in
`files.rs` beside `unread_transcript`, opening read-only and closing
what it opens); read-only, no other change to either backing, and
`file_bytes` stays as it is. Its own test: a unit test or a case in the
store tests of `jig-test-store-files`'s package that a written
transcript reads back and a dropped one reads `None`, on a memory
store. `removal.rs`'s check calls it for every task recorded before
the removal and finds `None`. `lib.rs` keeps only the
`ActKind::RemoveProject` grant. The `files` list above now names
both `jig-test-store-files` files; `pd-sweep`, which owned them, has
landed.
