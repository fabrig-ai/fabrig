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
  "jig/crates/jig-domain-people/src/domain/removal.rs",
  "jig/crates/jig-domain-people/src/tests/removal.rs",
  "jig/tests/core/src/removal.rs",
  "jig/crates/jig-domain-people/src/domain.rs",
  "jig/crates/jig-domain-people/src/tests/moving_chat.rs",
  "jig/testing/jig-conformance/src/harness.rs",
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

### 2026-11-01: Should pd-delete repair retained AccountChange::SetUses answers that name a deleted project (recommended), or should this task's ownership expand to repair the people deletion and its reference audit? (playground-reset-r4/pd-conformance-2, foreman playground-reset-r4/pd-conformance/foreman-2)

The retained AccountChange::SetUses answer is a question for pd-delete’s family; pd-conformance’s ownership does not expand.
The spec’s Report instruction settles ownership: “any row the check found that no deletion step erases (with its key), which is a question for `pd-delete`’s family.”
Step 2 requires “every row … whose key or value names the project” to be a violation, except the specified ClosedLedger rows.
Both `jig/crates/jig-domain-people/src/domain/removal.rs` and `jig/tests/core/src/removal.rs` classify every AccountChange as project-free; neither file belongs to this task.
Route the retained-answer defect and reference-audit omission to pd-delete’s family.
After that repair, pd-conformance resumes its required checks, then commits and pushes. Acceptance remains unmet; this is not an acceptance as done.

### 2026-11-01: Should fab land pd-delete's retained AccountChange::SetUses answer and reference-audit repair, then restart pd-conformance (recommended), or supply a base already containing that repair? The assigned base still lacks the prerequisite required by the spec's 2026-11-01 answer. (playground-reset-r4/pd-conformance-3, foreman playground-reset-r4/pd-conformance/foreman-3)

Supply a base containing pd-delete’s retained-answer and reference-audit repair, then restart pd-conformance.
The binding answer says: “Route the retained-answer defect and reference-audit omission to pd-delete’s family.”
It then says: “After that repair, pd-conformance resumes its required checks, then commits and pushes.”
Both omissions remain in `jig/crates/jig-domain-people/src/domain/removal.rs` and `jig/tests/core/src/removal.rs`.
The next run implements the pending scenario and transcript accessor within its existing ownership, completes checks, commits and pushes.
Acceptance remains unmet; this is not acceptance as done.

### 2026-11-01: Supply a base containing pd-delete’s retained AccountChange::SetUses answer and reference-audit repair, then restart pd-conformance (recommended), or explicitly revise the prerequisite and ownership? The supplied base still lacks the repair required by the binding answers. (playground-reset-r4/pd-conformance-4, foreman playground-reset-r4/pd-conformance/foreman-4)

Supply a base containing pd-delete’s retained-answer and reference-audit repair, then restart pd-conformance.
The latest binding answer explicitly says: “Supply a base containing pd-delete’s retained-answer and reference-audit repair, then restart pd-conformance.”
Ownership remains settled: “Route the retained-answer defect and reference-audit omission to pd-delete’s family.”
Inspection confirms both predicates still classify AccountChange as project-free in `jig/crates/jig-domain-people/src/domain/removal.rs` and `jig/tests/core/src/removal.rs`.
After the repaired base is supplied, pd-conformance implements its pending scenario and transcript accessor within existing ownership, completes checks, commits and pushes.
Acceptance remains unmet; this is not acceptance as done.

### 2026-11-01: the retained `AccountChange::SetUses` answer, after four runs parked on it (playground-reset-r4/pd-conformance-4)

This answer replaces the three above that wait for a repaired base.
`pd-delete` and `pd-people-gone` have both landed, so no task of theirs
will make the repair. It is this task's now, as neighbour edits. The
code to change is the one match arm named below, in two files:

- `jig/crates/jig-domain-people/src/domain/removal.rs` `names_project`:
  take `Ask::AccountChange { .. }` out of the `false` arm and add
  `Ask::AccountChange { change: crate::AccountChange::SetUses { uses:
  Some(uses), .. }, .. }`, which is true when any `Use::Project(named)`
  in `uses` equals `project`. Every other `AccountChange` stays
  `false` (follow `domain.rs`'s `SetUses { uses: Some(uses), .. }`
  arm). `gone` then erases that retained answer at `ProjectGone`, as
  the draft's section 5 ("an act's retained answer that named it")
  asks.
- `jig/tests/core/src/removal.rs` `ask_project`: the same arm, so that
  `rows_naming` finds such an answer.
- Its test: in `jig/crates/jig-domain-people/src/tests/removal.rs`, add
  `a_gone_project_erases_a_retained_set_uses_answer_naming_it`. Save an
  `AccountChange::SetUses` answer whose uses name P, a second whose
  uses name only another project, and a `SetUses { uses: None }`. After
  `ProjectGone { P }`, the first is erased and the other two stay. Add
  one `mod` line if that file needs it.

No other line of those files changes; the `files` list above now names all three. The next run builds the scenario,
the transcript accessor (the first answer) and the tests on its base.
Run 2 built them but did not commit, so it left nothing to start from.
Then it makes the repair above and runs step 4's checks, adding
`heavy cargo nextest run -p jig-domain-people --lib -E
'test(=tests::removal::a_gone_project_erases_a_retained_set_uses_answer_naming_it)'`
and `-p jig-core-world --test suite -E 'test(/removal_stages/)'`. It
commits and pushes. A row the scenario finds that the deletion does
not erase, other than this one, is still a question. A boundary this
answer already grants is not asked again.

### 2026-11-01: the test connector's `Made` row naming a deleted task (playground-reset-r4/pd-conformance-5)

Neither option: the `Made` row outlives the deletion, and the check
excepts it as it excepts `ClosedLedger` rows. No connector or root repair,
and no ownership beyond this spec's `files`.
A `Made` row is the connector's record of an object it made on its system,
by the effect's key (jig's `domain/connectors.md`, 4.4: "The connector
records every object it made, by key, in the store: the ownership of
section 11"). The object itself stays on the system: a deletion undoes
nothing there (draft 4, "On the forge itself"). Its key carries the task's
number as part of the object's identity, and task numbers are never reused.
It refers to no task row. If the record were erased, a later adoption of the
same resource would find this deployment's own objects with "no making"
recorded and refuse them as another deployment's (`connectors.md`, 12, "One
deployment's"). `ReleaseProject` also could not choose which `Made` rows to
erase: their keys name tasks, not the project. Ordinary retention does not
erase `Made` rows either.

- `jig/tests/conformance/src/scenarios/removal.rs`'s check: a
  `connector::Record::Made` row is not a violation, even when its key's
  `task` is a task recorded before the removal. Every other connector
  record keeps its arm. The report lists the `Made` rows the scenario
  built under "each family's rows" as outliving the deletion.
- `jig/tests/core/src/removal.rs` `names`: move
  `connector::Record::Made { .. }` into the `false` arm, beside `Pool`.
  `rows_naming`'s doc comment then names it: "except the project's closed
  period and pool totals and the connector's records of objects it made,
  which outlive deletion". No other line of that file changes beyond the
  `ask_project` arm the answer above grants. Run the core world's
  `removal_stages` tests again, as above.
- Do not change the held effect's release or its settlement: the scenario
  keeps them, because they are how a `Made` row comes to exist.

A row the scenario finds that the deletion does not erase, other than the
`SetUses` answer and the `Made` rows, is still a question.

### 2026-11-01: the former requester's reply-index row after `Ask::Move` (playground-reset-r4/pd-conformance-6)

Neither option: no repair of `jig/crates/jig-domain-core/src/erase.rs`
in this task. The check excepts this row, keeps the `Ask::Move`, and
records the row as a leak.
A reply's `Update` row is keyed by party first and saved under the
task's requester at the time of the turn (`indexes.rs` `save`).
`Ask::Move` changes the requester and leaves the earlier rows in place
(`jig-domain-tasks` `moving.rs` erases only the old parent's
`Delegate`). After that, nothing durable records who the earlier
requester was. The `Turn` line carries no party, and the task record
names only the current requester. So erasure has no data for a
"targeted repair": `reply_update` can rebuild only the final
requester's key. Finding the other rows takes either a stored-format
change (the former requesters in the record or on the line) or a
family-wide sweep of `Update` rows, like the decided-proposal sweep
(`project_rows.rs`). jig's store design never considered adoption here
(`jig/docs/design/protocol/store.md`, 7.4 and 11.1: "its replies' rows
in its requester's updates"), so ordinary retention leaks the same row.
That choice is for jig's designer and the user, not for this task.
Reads already skip the row (`reads/rows.rs` `reply_update` returns
nothing unless the party is the conversation's requester). It is dead
weight, not a disclosure.

- `jig/tests/conformance/src/scenarios/removal.rs`'s check: a
  `tasks::Stored::Update` row whose object is `UpdateObject::Reply`,
  whose `task` is a task the scenario moved, and whose `party` is that
  task's requester before the move is not a violation. The scenario
  records the moved task and its former requester when it makes the
  move. Do not reconstruct them in the check. Every other `Update` row
  stays a violation: the same task's rows under its final requester,
  and its `Result` and `Proposal` updates. The check does not require
  the excepted row to be present, so a later repair passes unchanged.
  Comment the exception at the arm in one line, citing store.md 7.4,
  as a known leak of adoption.
- The report lists the row under "each family's rows" as a known leak
  that no deletion step erases, with its key, in a section
  `## Known leak` for jig's designer.
- `jig/tests/core/src/removal.rs`, `erase.rs` and `moving.rs` do not
  change.

Then finish step 4's checks, the two added by the `SetUses` answer
included, then commit and push. A row the scenario finds that the
deletion does not erase, other than the `SetUses` answer, the `Made`
rows and this former requester's reply rows, is still a question.

### 2026-11-01: the zero read position a conversation move saves, refused at a cold restart (playground-reset-r4/pd-conformance-7)

(A). The default restore's check is wrong, not the scenario's crash
coverage: the shared sweep keeps cutting every setup commit.
`inbox/moving.rs` `move_chat` saves `ReadConversation { through: 0 }` on
purpose: the new requester starts with nothing read, and the row is how
its position exists at all (`tests/moving_chat.rs`
`a_moved_conversations_new_requester_starts_with_nothing_read`). The
explicit start's `start.rs` `load` already admits that row: its `fits`
checks only occupancy. Only `domain.rs` `restore_default` refuses it,
so one stored row restores on one start and fails the other.

- `jig/crates/jig-domain-people/src/domain.rs` `restore_default`, the
  `Stored::ReadConversation` arm of `valid`: drop `&& *through != 0`.
  `party != 0`, `task != 0`, the duplicate and the capacity checks stay.
  No other line of `domain.rs` changes; `ReadResults` keeps its
  `position != 0` (no writer saves zero there).
- Its test, in `jig/crates/jig-domain-people/src/tests/moving_chat.rs`:
  `a_moved_conversations_zero_position_restores_cold`. Take the row the
  move saved (`moving_chat()`'s `out[1]`), restore it into a fresh
  domain as `tests.rs`'s cold restores do (`Test::new`, then
  `d = Domain::new(..)`, `Event::Restore` of persons 1 and 2 and the
  row, then `Event::Restored`), and assert that no restore output
  refuses it and `conversation_seen(2, 9) == Some(0)`.
- Add `heavy cargo nextest run -p jig-domain-people --lib -E
  'test(=tests::moving_chat::a_moved_conversations_zero_position_restores_cold)'`
  to step 4, then rerun the crash sweep alone, then the rest of step 4.

The `files` list above now names both files. A row the scenario finds
that the deletion does not erase, other than the three excepted above,
is still a question; so is a restore refusal of a row some writer other
than `move_chat` saves.

### 2026-11-01: a resumed snapshot's heap meter, which begins after the snapshot it owns (playground-reset-r4/pd-conformance-8)

Extend this task: the repair is in `jig/testing/jig-conformance/src/harness.rs`,
which the `files` list above now names. No change to `skein-heap`, to the
ceiling (`A::maximum`) or to tracing.
A fresh harness takes its `Meter` before the domain and while its store is
empty, so whatever it later frees was allocated after its base. A resumed
one takes its `Meter` after the snapshot's store, peers, systems, referee
and trace were cloned, then owns and frees them; the deletion is the first
scenario to free enough of the forked store to go below the base. The
meter must begin before anything the resumed harness owns, so it begins
where the snapshot does:

- `ColdSnapshot` gains a field `meter: skein_world::domain::heap::Meter`.
  `cold_snapshot` builds it with `Meter::new()` first, before any clone or
  `fork`, and stores it; `from_snapshot` drops its own `Meter::new()` and
  moves `snapshot.meter` into the new harness. Its doc comment says the
  meter begins at the capture, so the resumed engine's heap counts the
  outside state it took over as well, a conservative bound as the fresh
  harness's count of its peers is. No other line of `harness.rs` changes.
- This counts more than a fresh run (the clones of the systems, peers,
  referee and trace). If a resumed scenario's held heap then exceeds its
  maximum, that is a question with the measured `held` and `maximum`, the
  scenario, the cut and the seed; do not raise the maximum or subtract an
  estimate.
- The regression is the untraced
  `removal::a_deleted_project_cut_at_each_chosen_point_ends_the_same` as it
  stands, through the ordinary untraced replay helper; do not trace it to
  pass. Also run `-E 'test(/^snapshot::/)'` of `jig-conformance-world`'s
  suite, the other users of `from_snapshot`.

Then the rest of step 4: the complete focused suite, `fuzzy` for the
shared sweep, the core world's `removal_stages`, the two people
regressions, `cargo fmt`, clippy on every changed package, and
`heavy cargo check --tests -p jig-ops-world -p temper-conformance-world`.
Commit and push. Runs 2 and 8 built the work and left it uncommitted, so
the next run started from nothing: if a run must stop on a question again,
it first commits what it has and pushes its branch, saying in the report
which checks that commit has not passed.
