+++
id = "jp-scopes"
part = "project-delete"
kind = "interface"
needs = ["jig-acts"]
files = [
  "jig/crates/jig-domain-core/src/reads/**",
  "jig/crates/jig-domain-core/src/reads.rs",
  "jig/crates/jig-protocol-store/src/reads/**",
  "jig/crates/jig-protocol-store/src/reads.rs",
  "jig/crates/jig-protocol-store/src/tests/reads/**",
  "jig/crates/jig-protocol-store/src/tests/intervals.rs",
  "jig/tests/store/tests/suite/erasure_reads.rs",
]
effort = "high"
minutes = 75
+++

## Goal

Give the core two store reads of its own, which a project's deletion
pages through to find what to erase (playground-reset.md, 5, "Its tasks
ended, then erased"): every ended task in task order, and every row a
task names under a key that starts with the task. Both are `reads::Scope`
variants loaded through `Held::ReadLoad` as party reads are, so every
application's root carries them unchanged (temper's carries `ReadLoad`'s
range opaquely to its store). No party ever reads them: the core's party
read paths refuse them.

Acceptance (`jig-store-world`, new `tests/suite/erasure_reads.rs`, on the
real store with rows written by the core's codecs):

- `ended_tasks_page_every_ended_task_in_task_order`: three ended tasks of
  two projects and a live one; pages of one row each give the three
  `Ended` records in task order, with `next` continuing after the last
  and `None` at the end; a live task is not among them.
- `task_rows_page_every_row_keyed_by_the_task`: one ended task with
  history, milestones, activity, two ended delegates, a stub, a terminal,
  a run proof, a call, an escalation decision, an ended read position and
  a settings proposal of its own; `TaskRows { task }` pages give each
  row once, and none of a neighbouring task (numbers one below and one
  above).
- `task_rows_are_empty_once_erased`: after erasing every key a page gave,
  the same read gives an empty page and no `next`.
- The core refuses either scope from a party read (a case in the core's
  read tests: `a_party_read_of_an_erasure_scope_is_refused`).

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md`.
- `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/design/draft/playground-reset.md`, section 5.
- `jig/docs/design/protocol/store.md`, 4.4 (jig's families and their key
  layouts), 7.1 (pages), 7.3 (parties' reads; these two are not).
- `jig/docs/design/domain/engine.md`, section 11 (reads), for the
  `Scope`/`Range`/`Loaded` vocabulary.

## Owns and may change

Owns the files in the header. May change, as `notes/scope.md` says:

- `jig/crates/jig-protocol-store/src/intervals.rs`: `scope()`'s arms for
  the two new variants (`jp-row` owns the file's `RestoreRange::Core`
  interval and keeps it; if `jp-row` has merged first, rebase on it).
- Every exhaustive match on `reads::Scope` the variants force, in jig and
  temper: in the survey `jig/crates/jig-protocol-store/src/reads/indexed.rs`
  (yours), `jig/tests/client/src/joint/wire/cursor.rs` (its scope
  encoding: give each a new tag after the last), `jig/tests/client/src/joint/wire/ledger_pages.rs`,
  `jig/tests/ops/src/joint/reads.rs`; each arm as its nearest kin
  (`EndedDelegates`, a store range with no party joins), or a refusal
  where the reader serves parties.
- The core's party read admission (in `reads/**`): a party's read naming
  either scope is refused as a malformed or unknown read is today.

Not yours: what the core does with the pages (`jp-delete`); the key
layouts themselves.

## Steps

1. `jig-domain-core/src/reads/vocabulary.rs`, in `Scope`:

   ```rust
   /// Every ended task in task order: the core's own read, for a
   /// project's deletion (temper/docs/design/draft/playground-reset.md, 5).
   EndedTasks,
   /// Every row whose key starts with the task, across jig's families:
   /// the core's own read, for a task's erasure (playground-reset.md, 5).
   TaskRows { task: u64 },
   ```

   Rows come as `Row { value, owner: None, ancestors: [], related: None,
   unjoined: None, ended_read: None, linked: [], inbox: None }`, in key
   order, `Loaded::Page { rows, next }` with `next` the last row's
   `Position::Owner(key)` while more may follow. The range's `party` is
   ignored for both.

2. `jig-protocol-store/src/intervals.rs`'s `scope()`:
   - `EndedTasks`: the prefix `[JIG, 1, 6]` (tasks' `Ended`).
   - `TaskRows { task }`, in this order (tags from `keys/*.rs`; check each
     against the key writers):
     tasks `Stub(task)` (`[JIG,1,1]` + task, a point), `History`
     (`[JIG,1,9]` + task), `Milestone` (`[JIG,1,10]` + task), `Activity`
     (`[JIG,1,11]` + task), `EndedDelegate { requester: task, .. }`
     (`[JIG,1,17]` + task), `Delegate { requester: task, .. }`
     (`[JIG,1,18]` + task); core `Projection` rows of the goal numbered
     `task` (if the projection key starts with the goal; otherwise leave
     them, since a goal's projection rows are erased when it closes, and
     say so), `RunProof(task)` (`[JIG,0,2]` + task), `Call { task, .. }`
     (`[JIG,0,3]` + task), `Terminal { task, .. }` (`[JIG,0,5]` + task),
     `EscalationDecision { task, .. }` (`[JIG,0,7]` + task),
     `ProposalDecision { proposer: Proposer::Task(task), .. }` and the
     settings family's `Proposal { proposer: Proposer::Task(task), .. }`
     (if their keys start with the proposer; otherwise leave them out and
     say so); people `EndedRead { task, .. }` (`[JIG,2,7]` + task).
     Not `Turn` (a transcript's frames, dropped with it by
     `Write::DropTranscript`), nor `Live`/`Ended` (the caller holds the
     record), nor rows keyed by a party first (the caller derives those
     keys from the record).
   The page reader (`reads/paged.rs` or wherever `Scope` intervals are
   paged) walks the intervals in order with the range's `rows` and
   `bytes` caps, as it does for `EndedDelegates`.

3. The core's readers of `Scope` (`reads/rows.rs`, `reads/vocabulary.rs`'s
   sizing, `reads/settings.rs`): an arm each; a party's read never
   carries them.

4. Tests: the acceptance's. Then, one at a time:
   - `heavy cargo nextest run -p jig-store-world --test suite -E 'test(/erasure_reads/)'`
     (the binary name from `jig/tests/store/Cargo.toml`);
   - `heavy cargo nextest run -p jig-protocol-store -p jig-domain-core`;
   - `heavy cargo nextest run -p jig-store-world -p jig-core-world -p jig-client-world -p jig-ops-world`;
   - `heavy cargo check --tests -p temper-engine-domain -p temper-service`
     (no temper edit is expected; if one is forced, its focused tests);
   - `cargo fmt`; clippy over every package you edited; old forms from `jig/`.
5. Commit and push.

## Report

The standard report, plus: the intervals as built, with any family left
out and why; the forced arms outside the owned files.
