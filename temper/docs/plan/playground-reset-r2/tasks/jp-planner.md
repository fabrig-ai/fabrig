+++
id = "jp-planner"
part = "project-delete"
kind = "feature"
needs = ["jig-acts"]
files = [
  "jig/crates/jig-domain-core/src/erasure.rs",
  "jig/crates/jig-domain-core/src/tests/erasure.rs",
]
effort = "high"
minutes = 60
+++

## Goal

Write the core's erasure planner: pure functions that give every store
key an ended task's erasure must erase, from its `Ended` record and from
the rows a `TaskRows { task }` read gives (playground-reset.md, 5, "erased
whole at once, as its retention would erase it"; jig's
`protocol/store.md`, 11.1, a task's full records). `jp-delete` calls them;
nothing calls them yet.

Acceptance (`jig-domain-core/src/tests/erasure.rs`):

- `a_persons_ended_conversation_plans_every_index_row`: an ended
  conversation of a person, listed, with a result position: the plan
  holds `Result`, the result's `Update`, `EndedByParty`, `EndedByProject`
  and `Ended`, each once, `Ended` last.
- `an_unlisted_task_plans_no_project_index`: no `EndedByProject` key.
- `a_delegate_plans_its_requesters_edges`: a task whose requester is a
  task: `EndedDelegate { requester, task }` and `Delegate { requester,
  task }`, and no person index.
- `a_rows_own_key_is_planned`: for one row of each kind `TaskRows` gives
  (tasks `Stub`, `History`, `Milestone`, `Activity`, `EndedDelegate`,
  `Delegate`; core `RunProof`, `Call`, `Terminal`, `EscalationDecision`;
  people `EndedRead`), its own key.
- `an_activity_reply_plans_its_update`: an activity line the core indexes
  into its requester's updates (`core/indexes.rs`'s `save`) plans that
  `Update` key too, equal to the one `indexes::save` writes for it.
- `the_plan_matches_what_the_core_saved`: a task driven through the core
  in the crate's test fixtures (made, run to one activity reply, ended),
  every key the core saved naming it (collected from the `Write::Save`s
  of those steps) is in the plan of its `Ended` record and of its rows;
  if the fixtures cannot drive that far within the budget, cover the same
  with saved rows built by the core's own `indexes::save` and say so.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md`.
- `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/design/draft/playground-reset.md`, section 5.
- `jig/docs/design/protocol/store.md`, 4.4 (the families), 7.4 (indexes),
  11.1 (retention: what a task's full records are).
- `jig/crates/jig-domain-core/src/forget.rs` (the same derivations for a
  forgotten task: reuse its helpers rather than repeat them).
- `jig/crates/jig-domain-core/src/indexes.rs` (the index rows the core
  writes for a task's rows).

## Owns and may change

Owns `erasure.rs` and its tests. May change, as `notes/scope.md` says:
`jig-domain-core/src/lib.rs` (the `mod erasure;` line),
`jig-domain-core/src/tests.rs` (the `mod erasure;` line), and
`forget.rs` and `indexes.rs` to make a key derivation `pub(crate)` and
shared (a function moved or exposed, its behavior unchanged). `jp-row`
edits `lib.rs` and `tests.rs` at the same time for its own lines: keep
your edit to your lines.

Not yours: emitting any write (`jp-delete`); `Write::DropTranscript`
(`jp-delete` sends it beside the plan).

## Steps

1. `erasure.rs` (module doc: the planner of an ended task's whole
   erasure, citing `temper/docs/design/draft/playground-reset.md, 5` and
   `protocol/store.md, 11.1`):

   ```rust
   /// The keys an ended task's record alone names: its party and project
   /// indexes and its requester's edge, then `Ended` itself, last.
   pub(crate) fn record_keys(record: &tasks::TaskRecord, out: &mut List<Key>) -> Option<()>;
   /// A row a `TaskRows` read gave: its own key, then any index key the
   /// core derived from it when it was saved.
   pub(crate) fn row_keys(row: &Record, out: &mut List<Key>) -> Option<()>;
   /// The most keys one record or row can plan, to size `out`.
   pub(crate) const MOST_KEYS: u32 = /* the count you find */;
   ```

   `record_keys`, from the record (as `forget.rs` derives them): for a
   `Party::Person` requester with a result position, `Result { party,
   position }`, its result `Update` and `EndedByParty`; `EndedByProject`
   if `flags.listed`; for a `Party::Task` requester, `EndedDelegate` and
   `Delegate`; then `Ended(task)`. `None` when `out` is full (the caller
   sizes it with `MOST_KEYS`), never a panic.
   `row_keys`: `row.key()` (the `Record`'s key), then, for each kind
   `indexes::save` derives an index row from, that index row's key by the
   same derivation.

2. Tests: the acceptance's. Then, one at a time:
   - `heavy cargo nextest run -p jig-domain-core --lib -E 'test(/erasure/)'`;
   - `heavy cargo nextest run -p jig-domain-core`;
   - if you changed `forget.rs` or `indexes.rs`:
     `heavy cargo nextest run -p jig-core-world`;
   - `cargo fmt`; clippy over `jig-domain-core`; old forms from `jig/`.
3. Commit and push.

## Report

The standard report, plus: every row kind the plan covers and how its
key is derived; any row naming the task you found no derivation for (the
referee, `jp-referee`, will look for it).
