+++
id = "jp-steps"
part = "jig-remove-project"
kind = "feature"
needs = ["jp-acts", "pg-jig-uses", "jp-erase", "jp-row"]
files = ["jig/crates/jig-domain-core/src/removal/steps.rs", "jig/crates/jig-domain-core/src/routing/deployment_acts.rs", "jig/crates/jig-domain-core/src/routing/tasks.rs", "jig/crates/jig-domain-core/src/reads/vocabulary.rs", "jig/crates/jig-protocol-store/src/reads.rs", "jig/crates/jig-protocol-store/src/reads/**", "jig/crates/jig-protocol-store/src/intervals.rs", "jig/crates/jig-protocol-store/src/tests/project_ended.rs", "jig/tests/core/tests/suite/project_removal_steps.rs", "jig/tests/core/tests/suite/main.rs"]
effort = "high"
minutes = 110
+++

## Goal

Make a deleted project's tasks end and go (draft
`temper/docs/design/draft/playground-reset.md`, section 5, "its tasks
ended, then erased"; part note section 2, steps 2 and 3): the
deletion's `Ending` step cancels each live task of the project as a
conversation's close cancels it and waits for each to end, and its
`Erasing` step finds every ended task of the project by a paged load and
erases each whole through `jp-erase`'s entry, then hands over to the
rows' step. Both resume at each start from the kept row.

Acceptance:

- `removal::steps::begin(core, env, work, out, project)` (`jp-row`'s
  stub, filled; signature as `jp-row`'s code has it), for the kept row's
  `Ending`: each live task of the project whose requester is not a task
  and which is neither closing nor ended is cancelled as
  `Act::Cancel` cancels one (`jp-acts`' shared compound construction in
  `routing/deployment_acts.rs`, made `pub(crate)` if it is not), by
  `Party::Deployment { project }`, its live delegates at once and the task
  with the grace: a conversation's is the current settings' conversation
  template's `grace` (as `temper-engine-domain/src/route/demo.rs`'s
  `send_end_act` picks it), any other task's none; each with the reason
  `b"Project deleted"`. Batches stay within the compound's bounds and the
  continuation's room; what does not fit goes in the next decision that
  runs `begin` or `ended`. A compound's answer goes to no one: a
  continuation that drops it (a reserved token as `make_project_request`
  uses for `OpenPeriod`, or a `PersonTaskRoute` variant; pick and say).
  If no live task of the project is left, it moves to `Erasing` in the
  same decision.
- `removal::steps::ended(...)`, called from the `tasks::Request::Ended`
  arm of `routing/tasks.rs` when the ended task's project is
  `core.removing(project)` (read the project before the row leaves): runs
  what `begin` left, and when no live task of the project is left saves
  the row at `Step::Erasing` and loads the first page.
- `Erasing`: a new `reads::Scope::ProjectEnded { project: u32, after: Option<u64> }`
  (core's `reads/vocabulary.rs`; the store's side in `jig-protocol-store`'s
  `reads`): the tasks family's `Ended` rows in task order past `after`,
  only those whose record's project is `project`, within the range's row
  and byte caps counting rows examined, `next` naming the last task
  examined (a page may be empty with a `next`). Loaded through
  `removal::load`; `removal::steps::loaded` erases each page's tasks with
  `erase::erase_task` in one decision, then loads the next page past the
  last examined task (`Held::ReadLoad` follows the commit); a page with no
  `next` calls `removal::rows::begin`. `Busy` loads again at the next
  decision; `Gone` starts the scan again from `None`. A task
  `erase_task` refuses stays (it fails the referee later) and the scan
  goes on.
- `routing/deployment_acts.rs`: `Act::RemoveProject` routed to
  `crate::removal::close::close(...)` (the stub refuses `NotOffered`, as
  today: no change of behaviour until `jp-close`); `Act::Close` and
  `Act::Cancel` naming a project that is `core.removing` refused
  `Unknown`.
- At start, `removal::resume` (`jp-row`'s hook) reaches `begin` for
  `Ending` and `Erasing`; `begin` at `Erasing` loads the first page from
  `None`.
- Tests: `jig-core-world`'s `suite` binary, new
  `project_removal_steps.rs`: a store holding a kept `Removal { step:
  Ending }` for a project with a live conversation (with a delegate), a
  change task and an ended task; started: each cancelled (the
  conversation with its template's grace, the grace's end fired on the
  world's clock), ended, then every task of the project erased (no
  durable key or value names one of its tasks: scan as `jp-erase`'s
  `jig/tests/core/tests/suite/erasing.rs` does), and the row gone through
  `rows::begin`'s stub; another project's tasks untouched; a crash after
  the first cancel's commit and after the first page's erase, each
  resumed to the same end; `Act::Cancel` on a removing project's task
  refused `Unknown`. `jig-protocol-store` library tests, new
  `src/tests/project_ended.rs`: the scope's pages over a store of ended
  tasks of two projects, `after` and the caps honoured.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (section 1).
- `temper/docs/plan/playground-reset-r5/notes/jig-remove-project.md`,
  sections 1, 2 and 4.
- `jig/docs/design/domain/tasks.md`, section 7 (cancelling closes down
  the tree; grace; the deployment's compound); `jig/docs/design/protocol/store.md`,
  7 (loads and pages), 11.1.
- `tasks/jp-acts.md`, `tasks/jp-erase.md` (the plan directory), for the
  shared compound and `erase_task`; read their merged code.
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s4-jig-projects.md`,
  sections 2, 3 and 6.

## Owns and may change

- Owns the files in `files`: `removal/steps.rs`; in
  `routing/deployment_acts.rs` the `RemoveProject` arm, the removing
  guards and the shared compound's visibility; in `routing/tasks.rs` the
  call from the `Ended` arm (and from the compound's answer, if the
  dropped continuation lands there); the `Scope` variant and its store
  side; the tests named above.
- May change: compiler-forced arms over `reads::Scope` anywhere in jig
  (`reads/rows.rs`, `reads/indexed.rs`, the client and ops worlds'
  readers) and temper (if any matches it); `jig/tests/core/src/**` for a
  fixture that seeds a kept `Removal` row (a builder beside the world's
  others), nothing else there.
- Not yours: `removal.rs`, `removal/close.rs`, `removal/rows.rs`,
  `routing/people.rs`, `policy.rs`, `projects.rs` (tasks `jp-close` and
  `jp-rows` run beside you).
- Options: if the store cannot filter `Ended` rows by project without
  decoding each value, decode each (it is a deletion's one-off scan); if
  a task of another project names an ended task of this one so that
  `erase_task` refuses, leave it and name the case in the report.

## Steps

1. `begin` and `ended`; the `Ended` arm's call; the act arms.
2. The scope, core side then store side, with its library tests:
   `heavy cargo nextest run -p jig-protocol-store --lib -E
   'test(/project_ended/)'`.
3. `loaded`; the core world's cases: `heavy cargo nextest run -p
   jig-core-world --test suite -E 'test(/project_removal_steps/)'`.
4. `cargo fmt`; `heavy cargo clippy -p jig-domain-core -p
   jig-protocol-store --all-targets -- -D warnings`; `heavy cargo check
   --tests` for each package whose `Scope` arms you added; the focused
   tests of `jig-domain-core`, `jig-protocol-store`, `jig-core-world`,
   `jig-store-world`. Commit and push.

## Report

As `notes/common.md` says; name the dropped-continuation choice, how the
store filters the scope, and any task-bearing row the erasure left.
