+++
id = "jp-children"
part = "jig-remove-project"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-tasks/src/**", "jig/crates/jig-domain-accounts/src/**", "jig/tests/tasks/tests/suite/forget_project.rs", "jig/tests/tasks/tests/suite/main.rs", "jig/tests/accounts/tests/suite/**"]
effort = "high"
minutes = 80
+++

## Goal

Give jig's tasks and accounts children what a deleted project's last step
asks of them (draft `temper/docs/design/draft/playground-reset.md`,
section 5, "then its own rows" and "what outlives it"): the tasks child
forgets a project's funding, reporting the spend it folds so the core can
keep it in the deployment's and the parties' totals, and the accounts
child drops the project from every account's uses. Nothing calls them yet
but the tests; task `jp-rows` calls them from the deletion.

Acceptance:

- `jig_domain_tasks::Event::ForgetProject { project: u32 }`: for a
  project with no live task (assert it: the deletion only asks once none is
  left), in one step: every live funding record whose funder names the
  project (`Funder::Period`, `Funder::Pool`, `Funder::Recurring` with that
  project) erased (`Request::Erase` of its `Key`), its pending person
  proposals (`PersonProposal` with that project) and placed holds
  (`PlacedHold` with that project) erased, its period schedule (what
  `periods::observe` keeps for it) dropped, and its in-memory state for the
  project gone (`funding(funder)` is `None` for each). It emits the spend
  it folded as `Request::SpendForgotten { party: Option<u64>, spent: u64 }`:
  one with `party: None` for the project's whole spend (its `Period`
  ledgers' `spent + spent_below`), and one per person whose `Pool` ledgers
  spent anything (`party: Some(person)`, their `spent + spent_below`),
  none with `spent == 0`.
- `jig_domain_tasks::Event::ForgetLedgers { project: u32, ledgers: Box<[FundingRecord]> }`:
  for closed ledgers the core loaded (`Stored::ClosedLedger` rows read on
  demand), each naming the project (assert), erases each one's
  `ClosedLedger` key and emits `SpendForgotten` as above for them. It
  holds nothing in memory.
- `jig_domain_accounts::Event::ForgetProject { project: u32 }`: every
  account whose uses hold `Use::Project(project)` loses it and is saved
  (`Request::Save` of its `Stored::Account`); an account whose uses become
  empty keeps an empty explicit list only if the accounts design allows
  one, otherwise goes back to the default uses (read
  `jig/docs/design/domain/accounts.md`, 4.2 and pick; say which).
  Accounts naming other projects are untouched.
- Every exhaustive match over `tasks::Event`, `tasks::Request` and
  `accounts::Event` compiles; in `jig-domain-core` the new
  `tasks::Request::SpendForgotten` arm ignores it for now (a later task
  keeps it).
- Tests: `jig-tasks-world`'s `suite` binary, a new `forget_project.rs`:
  a project's current period and a person's pool with spend, forgotten,
  their rows erased and `SpendForgotten` naming the deployment's and the
  person's totals; another project's funding untouched; closed ledgers
  forgotten through `ForgetLedgers`; a restart after the forget restores
  nothing of the project. In the accounts child's tests (its `src/tests`
  or `jig-accounts-world`'s `suite`, whichever already tests uses), one
  case for `ForgetProject`.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (section 1).
- `temper/docs/plan/playground-reset-r5/notes/jig-remove-project.md`,
  sections 1 and 2 (step `Own`, `Spender`).
- `jig/docs/design/domain/tasks.md`, sections 2 and 3 (funders, periods,
  pools); `jig/docs/design/domain/accounts.md`, 4.2 (uses).
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s4-jig-projects.md`,
  section 1 (the tasks family's financial identities, `retire.rs`, placed
  holds, person proposals; accounts' `Use::Project`).

## Owns and may change

- Owns: `jig-domain-tasks` (new module `forget_project.rs`, the two
  events and the request in `boundary.rs`, their dispatch in `domain.rs`);
  `jig-domain-accounts` (the event in `boundary.rs`, its body beside
  `changes.rs`); `jig/tests/tasks/tests/suite/forget_project.rs` (new, its
  `mod` line in `main.rs`); the accounts tests named above.
- May change: compiler-forced arms only, in `jig-domain-core` (the
  `tasks::Request` match in `src/routing/tasks.rs` and any other: the new
  arm ignores the request, one line), `jig/testing/*`, `jig/tests/*`. No
  store codec changes: `SpendForgotten` is not stored by this task.
- Options: if a funding record cannot be erased while a closed task's
  stub or an ended ledger names it, erase it all the same (the project's
  ended tasks are already erased when this runs) and say so. If the
  period schedule lives in the core rather than the tasks child, leave it
  and name it under `## Left`.

## Steps

1. Read `funders.rs`, `retire.rs`, `periods.rs`, `placed.rs`,
   `person_proposals.rs` for the project-keyed state; write
   `forget_project.rs` with both events.
2. The accounts event.
3. Forced arms; `heavy cargo check -p jig-domain-core --tests`.
4. Tests: `heavy cargo nextest run -p jig-tasks-world --test suite -E
   'test(/forget_project/)'`; the accounts case's narrow run.
5. `cargo fmt`; `heavy cargo clippy -p jig-domain-tasks -p
   jig-domain-accounts -p jig-domain-core --all-targets -- -D warnings`;
   the focused tests of `jig-domain-tasks`, `jig-domain-accounts`,
   `jig-tasks-world`, `jig-accounts-world`; `heavy cargo check --tests -p
   jig-core-world -p jig-conformance-world`. Commit and push.

## Report

As `notes/common.md` says; name the uses rule you picked, and every
project-keyed state of the tasks child you found that the list above
misses.
