+++
id = "pd-project-rows"
part = "project-deletion"
kind = "feature"
needs = ["pd-close", "pd-erase-task"]
files = [
  "jig/crates/jig-domain-core/src/project_rows.rs",
  "jig/crates/jig-domain-core/src/tests/project_rows.rs",
  "jig/crates/jig-domain-tasks/src/periods.rs",
  "jig/crates/jig-domain-tasks/src/tests/project_gone.rs",
  "jig/crates/jig-domain-accounts/src/project_gone.rs",
  "jig/tests/accounts/tests/suite/project_gone.rs",
  "jig/crates/jig-domain-notes/src/project_gone.rs",
]
effort = "high"
minutes = 60
+++

## Goal

A deleted project's own rows, once its tasks are erased (draft 5,
"Then its own rows"): its pools and its period in the current period
closed, so their spend stays in the period's totals, its recurring
ledgers posted into the period and erased (spend is never per task),
its per-project
state gone from the tasks child, its notes erased, every account's use
naming it dropped, and the core's per-project maps cleared. One
function the deletion calls (`pd-delete`), with its own completion
hook; nothing else calls it yet. (The roles and policy copy went at the
deletion's decision, `pd-close`/`pd-people-gone`; the connectors'
rows go by `ReleaseProject`, `pd-delete`.)

Acceptance: after `erase_rows` completes, the store holds no row
naming the project but its `ClosedLedger` rows (the period's and its
pools', the spend that outlives it: draft 5 and D5), and no child's
memory names it; tests below.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md`,
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (D5: spend
  kept in totals).
- `temper/docs/design/draft/playground-reset.md`, section 5 ("Then its
  own rows", "What outlives it").
- `jig/docs/design/domain/tasks.md` (funding: a period's ledgers, a
  party's pool), `jig/docs/design/domain/accounts.md`, 4.2 (an
  account's uses), `jig/docs/design/domain/engine.md`, section 10 (notes' scopes).
- `jig/crates/jig-domain-tasks/src/retire.rs` (`close`: a ledger saved
  as `ClosedLedger`, the model), `funders.rs`, `value.rs` (`Funder`);
  `jig/crates/jig-domain-accounts/src/records.rs` (`Use::Project`);
  `jig/crates/jig-domain-notes/src/boundary.rs` (`Scope`, `Key`);
  `pd-sweep`'s `Sweep::Notes`; `pd-erase-task`'s `erase.rs` (`swept`).

## Owns and may change

Owned: the `files` above (new files, each a module of its crate).
Neighbours, each edit the line or arm named:

- `jig-domain-tasks`: `boundary.rs` (one `Event::ProjectGone { project
  }` variant), `domain.rs` (its dispatch arm), `lib.rs`/`tests.rs`
  (`mod` lines);
- `jig-domain-accounts`: `boundary.rs` (`Event::ProjectGone { project
  }`), `domain.rs` (dispatch), `lib.rs` (`mod` line);
  `jig/tests/accounts/tests/suite/main.rs` (`mod project_gone;`);
- `jig-domain-notes`: `boundary.rs` (`Event::ProjectGone { project }`,
  dropping any cached line or entry of the project's scopes; its rows
  are erased by the core's sweep), `domain.rs`, `lib.rs`;
- `jig-domain-core`: `reads/sweep.rs` (one `Sweep::Recurring { project
  }` variant, its `admits` arm and its test's cases), `lib.rs` (`mod
  project_rows;`, and the field `pub(crate) row_sweeps: Map<Token,
  reads::Sweep>` beside `erasures`, made in `Core::new` with
  `Map::with_capacity(limits.authority.projects)`), `tests.rs` (`mod project_rows;`), `erase.rs` (`swept` hands a page whose owner is not
  an erasure's to `project_rows::swept`), the core's per-project maps'
  owners (`funding.rs` periods and boundaries, `Core.permission_roles`,
  `Core.template_overrides`): one removal each;
- `jig-protocol-store`: `intervals.rs` (the `Sweep::Recurring` arm:
  `numbered_prefix(&[keys::JIG, 1, 7, 2], project)`, the
  `ClosedLedger` family's `Funder::Recurring` tag), `tests/sweep.rs`
  (one test);
- compiler arms for the three new events and the sweep anywhere in
  `jig/**` and `temper/**`.

Pre-approved:

- if the tasks child already has an event that closes a project's
  current period and pools (a period boundary), use it and add none;
- an account whose uses named only this project keeps `Some([])`, used
  by no project, and is not removed (the deployment's acts remove
  accounts: draft 6.2);
- a note entry whose index line names the project is erased by
  `Key::Entry { name }`; one whose line names no project stays.

## Steps

1. `project_rows.rs` (new, step code; module doc citing draft 5):

   ```rust
   /// Erase a deleted project's own rows; completes in `rows_done`.
   pub(crate) fn erase_rows(core: &mut Core, env: &Env<Limits>, work: &mut Queue<Event>,
       out: &mut Queue<Request>, project: u32);
   /// A notes sweep page of the project's lines.
   pub(crate) fn swept(core: &mut Core, env: &Env<Limits>, work: &mut Queue<Event>,
       owner: Token, rows: Box<[Record]>, more: bool) -> Requests;
   /// `pd-delete` continues the deletion here.
   fn rows_done(core: &mut Core, env: &Env<Limits>, work: &mut Queue<Event>,
       out: &mut Queue<Request>, project: u32);
   ```

   `erase_rows`: `tasks::Event::ProjectGone` (the current period's and
   pools' ledgers closed as `retire::close` closes one; before them,
   each live `Funder::Recurring { project, .. }` ledger's spend posted
   into its period as `retire`'s first loop posts it and its `Ledger`
   row erased, no `ClosedLedger` written; the project's recurring and
   standing templates and period state dropped, each row written), `accounts::Event::ProjectGone` (each account's `Use::Project
   (project)` dropped, the account row saved), `notes::Event::ProjectGone`,
   the core's maps; then `Held::Sweep { owner, sweep: Sweep::Notes {
   project } }`. `swept`: erase each line and its entry; sweep again on
   `more`; else `Sweep::Recurring { project }` (the archived
   `ClosedLedger(Funder::Recurring { project, .. })` rows), each erased
   by key, again on `more`; else `rows_done` (empty but for `// pd-delete continues the
   project's deletion here.`).
2. Tests:
   - `jig-domain-tasks` `src/tests/project_gone.rs`:
     `a_gone_projects_period_and_pools_are_closed_and_its_templates_dropped`,
     `a_gone_projects_recurring_ledgers_post_into_its_period_and_go`;
   - `jig-protocol-store` `src/tests/sweep.rs`:
     `the_recurring_sweep_admits_only_its_projects_closed_recurring_ledgers`;
   - `jig-accounts-world` `tests/suite/project_gone.rs`:
     `each_accounts_use_of_a_gone_project_is_dropped`;
   - `jig-domain-core` `src/tests/project_rows.rs`:
     `erasing_a_projects_rows_closes_its_ledgers_and_sweeps_its_notes`,
     `a_notes_page_erases_each_line_and_its_entry`,
     `the_last_notes_page_sweeps_the_closed_recurring_ledgers`.
3. Checks: `cargo fmt`; clippy on the four crates; their `--lib`
   tests; `heavy cargo nextest run -p jig-tasks-world`, `-p
   jig-accounts-world`, `-p jig-notes-world`; `heavy cargo nextest run
   -p jig-protocol-store --lib`; `heavy cargo check --tests -p
   jig-core-world -p jig-conformance-world -p temper-engine-domain`.
   Commit, push.

## Report

fab's sections; `## Interfaces`: the three events, the three functions;
each row kind erased or closed, by key; which options were taken.

## Answers

2026-11-01, from `pd-erase-task`'s question (where are an ended task's
archived recurring ledgers found?): a task's erasure leaves its
recurring ledgers; this task owns them, as the project's period
accounting (`Funder::Recurring { project, task, period }`, keyed
project first). The spec above adds `Sweep::Recurring { project }`
and the steps that post the live ones into the period and erase the
archived ones; the deletion's end check (`pd-delete`,
`pd-conformance`) exempts only the period's and pools' `ClosedLedger`
rows, so none naming a task remains.

2026-11-01, from this task's question (how does `swept` know a page's
project and its sweep?): option A. `Core.row_sweeps: Map<Token,
reads::Sweep>`, beside `erasures` and bounded as it is
(`limits.authority.projects`, one deletion per project), is allowed in
`lib.rs` (Owns, `jig-domain-core`). `erase_rows` takes an owner from
`next_result_request`, as `erase::erase_task` does, and inserts
`Sweep::Notes { project }`; the notes sweep's last page replaces it
with `Sweep::Recurring { project }` under the same owner; the recurring
sweep's last page removes it before `rows_done`. `erase::swept` hands a
page whose owner is not in `erasures` to `project_rows::swept`, which
ignores an owner in neither map, as `erase::swept` does. The notes
design in Inputs is `jig/docs/design/domain/engine.md`, section 10.
