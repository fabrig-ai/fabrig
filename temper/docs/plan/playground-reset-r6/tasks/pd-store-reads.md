+++
id = "pd-store-reads"
part = "pd"
kind = "feature"
needs = ["pd-vocabulary"]
files = [
  "jig/crates/jig-protocol-store/src/reads.rs",
  "jig/crates/jig-protocol-store/src/reads/**",
  "jig/crates/jig-protocol-store/src/intervals.rs",
  "jig/crates/jig-protocol-store/src/tests/**",
  "jig/tests/store/**",
]
effort = "high"
minutes = 90
+++

## Goal

Serve the two store read scopes the core's erasure of a deleted project pages through
(`pd-vocabulary` added them; contracts J4 Erasing): `ProjectRows { project }`, which finds a
project's ended tasks and decided person proposals, and `TaskRows { task }`, which finds every
row filed under one task's key prefixes. Both are raw, unauthenticated reads for the core
alone; a party read never names them (the core does not admit them).

Acceptance:
- `Scope::ProjectRows { project }` pages, in key order, the tasks family's `Ended` rows
  (tag 6) and then its `DecidedProposal` rows (tag 8), and answers those whose record names
  `project` (`TaskRecord::project`; the decided proposal's project). A page scans at most
  `range.rows` stored rows and `range.bytes` bytes, whether or not they match, and returns the
  matches among them; `next` is the position after the last row scanned (`None` at the end),
  so a page may be empty with a `next`. Each `Row` carries `value` (the record as
  `jig_domain_core::Record::Tasks(..)`), `owner: None` and no joins.
- `Scope::TaskRows { task }` pages, in key order, every row under the task's prefixes: tasks
  `History` `[1,1,9,task]`, `Milestone` `[1,1,10,task]`, `Activity` `[1,1,11,task]`,
  `EndedDelegate` `[1,1,17,task]`, `Delegate` `[1,1,18,task]`; core `Terminal` `[1,0,5,task]`,
  `ProposalDecision` of `Proposer::Task(task)` `[1,0,6,0,task]`, `EscalationDecision`
  `[1,0,7,task]`; people `EndedRead` `[1,2,7,task]`. Each as its `jig_domain_core::Record`,
  `owner: None`, no joins; bounded and continued as above. Transcript turns are not rows here
  (the core drops them with `Write::DropTranscript`).
- A read of either scope after the rows it returned were erased in a later commit returns the
  rest (continuation by position, not by count), and a read from the start returns only what
  remains.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, J4.
- `jig/docs/design/protocol/store.md`, 4.2 and 4.4 (key layout), 7.3 (reads and their bounds).
- The code: `src/intervals.rs` `scope` (~140-217, `numbered_prefix`), `src/reads.rs` (dispatch
  ~40-70), `src/reads/paged.rs` (`history`, `spend` pages), `src/keys/{tasks,core,people}.rs`.

## Owns and may change

Owned: the `files` above. Nothing in other crates: the scopes' shapes are `pd-vocabulary`'s.

Options, pre-approved:
- If `intervals::scope` cannot express a filtered scan, serve `ProjectRows` from a new
  function in `reads/paged.rs` over the two family prefixes, filtering after decode.
- If the read's position type cannot name a key across two families, encode the position as
  the last scanned key, as the existing pages do.

## Steps

1. `intervals.rs`: the ranges of both scopes (prefixes above, in key order).
2. `reads/paged.rs`: `project_rows` and `task_rows`, bounded as stated; `reads.rs` dispatch.
3. Tests in `src/tests/reads/` (a new file beside the existing read tests, registered as they
   are):
   - `project_rows_find_a_projects_ended_tasks_and_decided_proposals` (two projects' ended
     rows interleaved by number; only project 2's come back, in key order, across pages of
     one row).
   - `a_project_rows_page_scans_at_most_its_rows_and_may_return_none`.
   - `task_rows_find_every_row_under_the_tasks_prefixes` (history, milestone, activity,
     delegate edges, a terminal, a task proposal decision, an escalation decision and an
     ended read of task 7, and the same of task 8: only task 7's come back).
   - `task_rows_after_an_erase_return_the_rest`.
   Commands: `heavy cargo nextest run -p jig-protocol-store --lib -E 'test(/project_rows|task_rows/)'`,
   then `heavy cargo nextest run -p jig-protocol-store --lib` and
   `heavy cargo nextest run -p jig-store-world --test suite`.
4. `cargo fmt`; `heavy cargo clippy -p jig-protocol-store -p jig-store-world --all-targets`;
   old forms; commit and push.
