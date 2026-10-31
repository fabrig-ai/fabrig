+++
id = "jp-erase"
part = "project-delete"
kind = "feature"
needs = ["jp-close", "jp-planner", "jp-scopes", "jp-people", "jp-referee"]
files = [
  "jig/crates/jig-domain-core/src/routing/removal/erase.rs",
  "jig/tests/core/tests/suite/removal_erase.rs",
]
effort = "high"
minutes = 85
+++

## Goal

Build the deletion's `Erasing` stage (playground-reset.md, 5, "Its tasks
ended, then erased"): every ended task of the project erased whole, one
task per decision, as retention would erase it (jig's `protocol/store.md`,
11.1): every row the task names, its transcript dropped
(`Write::DropTranscript`), every party's read of it and its chat
forgotten (`people::Event::ForgetTask`), and the core's own memory of it
let go. The row keeps its place (`Erasing { after }`), so a restart goes
on from the last task erased. At the end it enters `Notes`.

Acceptance (`jig-core-world`, new `tests/suite/removal_erase.rs`):

- `every_ended_task_of_the_project_is_erased_whole`: a project with a
  person's conversation run to a reply and a result, a delegate it made,
  a deployment task, and a live conversation cancelled by the deletion,
  each with history, milestones, activity and a transcript; a second
  project with an ended task of its own. After the deletion's `Erasing`
  stage, the naming walk (`jp-referee`'s `observations::naming`) counts
  for the project only rows the later stages own (its roster, policy,
  notes, account uses, decided proposals) and none of its tasks'; each
  task's transcript file is gone; the second project's rows are all
  there, unchanged.
- `each_task_is_erased_in_one_decision`: the commits of the stage each
  erase exactly one task's rows (or a page of them) and the last commit
  of a task saves the row at `Erasing { after: Some(task) }`.
- `a_party_reads_nothing_of_an_erased_task`: the person who held the
  conversation reads no chat, result or read position of it; the inbox
  holds nothing of it.
- `the_stage_pages_past_other_projects`: with ended tasks of other
  projects before, between and after the project's in task order, each
  page's other tasks are skipped and the stage ends.
- `a_large_task_is_erased_in_pages`: a task with more history rows than
  one page bound erases over several pages, its `Ended` row last.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md`.
- `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, section 1
  (`Write::DropTranscript`).
- `temper/docs/design/draft/playground-reset.md`, section 5.
- `jig/docs/design/protocol/store.md`, 7.1 (pages), 8.5 (a transcript
  dropped), 11.1 (a task's full records).
- `temper/docs/plan/playground-reset-r2/tasks/jp-close.md` (the module
  layout, `Work`, `enter`, `load_token`), `jp-planner.md` (`record_keys`,
  `row_keys`), `jp-scopes.md` (`EndedTasks`, `TaskRows`).

## Owns and may change

Owns `routing/removal/erase.rs` (replacing `jp-close`'s stub) and the
new test file. May change, as `notes/scope.md` says: `routing/removal.rs`
where `advance` and `loaded` dispatch to `erase` (an arm each, `jp-close`
keeps the rest; `jp-finish` edits `rows.rs` at the same time, not these);
every core map keyed by a task that an ended task can still occupy (rg
`Map<u64` in `jig-domain-core/src/lib.rs`: `archived_chats`,
`ended_marks`, `dependency_results`, `ending_positions` and their kin),
through a `pub(crate)` forget of the task in the module that owns each;
the world's `tests/suite/main.rs` (`mod` line) and its test helpers;
jig's test and example roots (`jig/testing/jig-test-domain/src/**`,
`jig/tests/core/src/**`, `jig/tests/conformance/src/**`,
`jig/examples/ops/domain/src/**`): where `jig-acts` left the core's
`Write::DropTranscript` a no-op there, carry it to the store's
`jig_protocol_store::Write::DropTranscript { task }` as temper's root
does.

Not yours: notes, decided proposals and the project's own rows
(`jp-finish`); resuming at a start (`jp-delete`).

## Steps

1. `erase::start(core, env, project, after, out)`: set
   `removal_work[project] = Work::EndedPage { after }` and emit
   `Held::ReadLoad { load: load_token(project), range: Range { party: 0,
   scope: Scope::EndedTasks, after: after.map(|task|
   Position::Owner(Key::Tasks(tasks::Key::Ended(task)))), backwards:
   false, rows, bytes } }`, with the page bounds the core's own loads use
   (`result_reads` or the census; name the limits you used).
2. `erase::loaded`, for `EndedPage`: walk the rows in order; at the
   first `Ended` record of the project, `Work::TaskRows { record }` and
   load `Scope::TaskRows { task }` from its start; with none of the
   project on the page, load the next page from `next` (the cursor held
   in memory only) or, with `next: None`, `enter(.., RemovalStage::Notes, ..)`.
3. For `TaskRows`: in this decision, erase each row's `row_keys` (with
   `Some(record.requester)`, the held record's),
   deduplicated within the decision (a key erased twice in one commit is
   one erase); with `next: Some`, load the next page; with `next: None`,
   in the same decision also erase `record_keys(record)` (`Ended` last),
   emit `Write::DropTranscript { task }`, step `people::Event::ForgetTask
   { task }`, let go of the core's maps of the task, and
   `enter(.., Erasing { after: Some(task) }, ..)`, which saves the row and
   loads the next `EndedTasks` page after it.
4. Each load answer is one decision, ending as the core's other load
   answers end (`Decided` once its children's handoffs complete); a
   `Loaded::Busy` answer is asked again as the core's other loads ask
   again; `Gone` or `TooLinked` cannot answer these scopes: debug-assert,
   and in release ask the same page again.
5. Tests: the acceptance's. Then, one at a time:
   - `heavy cargo nextest run -p jig-core-world --test suite -E 'test(/removal_erase/)'`;
   - `heavy cargo nextest run -p jig-domain-core -p jig-core-world`;
   - `heavy cargo nextest run -p jig-conformance-world -p jig-store-world -p jig-ops-world`;
   - `heavy cargo check --tests -p temper-engine-domain -p temper-service`
     and `heavy cargo nextest run -p temper-engine-domain` (its root now
     carries `DropTranscript` for real);
   - `cargo fmt`; clippy over the packages you edited; old forms from `jig/`.
6. Commit and push.

## Report

The standard report, plus: the page bounds used; every core map you
taught to forget a task; any row the naming walk still found for an
erased task, and why.

## Answers

2026-10-31. May this task repair the missing `engine::Change::RemoveProject`
arm in `jig/tests/ops/src/client/transport.rs` (`change_facts`, E0004 in
`jig-ops-world`)? Yes. `jp-client` (3fa8fa1d83) added the variant to
`jig-client-scripted-engine`'s `Change` and left this exhaustive match
behind; it is merged, so the trunk itself fails to build `jig-ops-world`'s
tests, and this task's step 5 builds them. Add the one arm,
`engine::Change::RemoveProject { .. }`, to the group that observes nothing
(`=> {}`, beside `Tree(_)` and `Seen { .. }`): the ops world's observations
watch tasks and cards, not project documents, and its scripts never delete
a project. Nothing else in that file or crate. Commit it on its own, first,
saying it repairs the trunk after `jp-client`, and name it in the report.
