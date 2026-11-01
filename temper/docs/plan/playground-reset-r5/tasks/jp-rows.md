+++
id = "jp-rows"
part = "jig-remove-project"
kind = "feature"
needs = ["jp-row", "jp-children", "jp-people"]
files = ["jig/crates/jig-domain-core/src/removal/rows.rs", "jig/crates/jig-domain-core/src/tests/removed_rows.rs", "jig/tests/core/tests/suite/removed_rows.rs", "jig/tests/core/tests/suite/main.rs"]
effort = "high"
minutes = 100
+++

## Goal

Give a project's deletion its last steps (draft
`temper/docs/design/draft/playground-reset.md`, section 5, "then its own
rows" and "what outlives it"; part note section 2, step 4): once its
tasks are erased, its closed ledgers are folded into the deployment's and
the parties' spend totals and erased, its notes erased, then every other
row naming it, and last the deletion's own row. Each step resumes at a
start from its beginning.

Acceptance:

- `removal::rows::begin(core, env, work, out, project)` (`jp-row`'s stub,
  filled; signature as `jp-row`'s code has it): at a row not yet past
  `Erasing`, saves it at `Step::Spend` and loads the first page of
  `reads::Scope::Spend { project, from: None }` through `removal::load`;
  at `Spend`, `Notes` or `Own` (a start's resumption), runs that step
  from its beginning.
- `Spend`: `removal::rows::loaded` hands each page's closed ledgers to
  the tasks child as `tasks::Event::ForgetLedgers { project, ledgers }`
  (`jp-children`), stepping it directly (`tasks::step` with a follow
  queue whose other requests are routed as `routing/tasks.rs`' `Ended`
  arm routes its `follow`); each `tasks::Request::SpendForgotten { party,
  spent }` is added to `core.spent` (`Spender::Deployment` for `None`,
  `Spender::Party(p)` for `Some(p)`, checked addition) and saved as
  `CoreRecord::Spent`; the next page follows; the last moves to `Notes`.
- `Notes`: the project's notes loaded (`Scope::Notes { project }`, and
  `Scope::ScopedNotes` for each goal or resource scope naming it, if
  `Scope::Notes` does not cover them: read `jig-domain-notes`' `Scope`);
  each index line whose scope names the project erased, and each entry no
  line of another scope still names; the last page moves to `Own`.
- `Own`, one decision: `tasks::Event::ForgetProject { project }` stepped
  as above (its `SpendForgotten` folded the same way);
  `accounts::Event::ForgetProject { project }` and
  `people::Event::ProjectErased { project }` through `work`;
  `Write::Erase(Key::People(people::Key::Policy(project)))`; the
  authority's policy dropped (`authority::Event::Dropped`, or the event
  `jig-domain-authority` has for it: `domain.rs`, 45); the core's
  `permission_roles` and `template_overrides` entries, its projections
  naming the project (`projections.rs`) and any period or watch state for
  it dropped; then `Write::Erase(Key::Core(CoreKey::Removal(project)))`
  and the `core.removals` entry gone.
- Afterwards no durable row names the project, and the `Spent` rows hold
  its spend: the deployment's total grown by the project's whole spend,
  each party's by what its pools spent.
- Tests: `jig-domain-core` unit tests, new `src/tests/removed_rows.rs`
  (the fold's arithmetic, the `Own` decision's writes);
  `jig-core-world`'s `suite` binary, new `removed_rows.rs` (its `mod` line
  in `main.rs` between `refusals` and `restart`): a store holding a kept
  `Removal { step: Spend }` for a project with no task left, a closed
  period and a person's pool with spend, notes in its scopes, an account
  whose uses name it and another project, a retained answer naming it;
  started: the `Spent` rows as above, and no durable key or value names
  the project (scan as `jig/tests/core/tests/suite/erasing.rs` does);
  another project untouched; a crash after each step's commit resumed to
  the same end.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (section 1).
- `temper/docs/plan/playground-reset-r5/notes/jig-remove-project.md`,
  sections 1, 2, 4 and 6.
- `tasks/jp-children.md`, `tasks/jp-people.md` (the plan directory), for
  the events you send; read their merged code.
- `jig/docs/design/domain/tasks.md`, sections 2 and 3 (funding);
  `jig/docs/design/protocol/store.md`, 4.4 (the families).
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s4-jig-projects.md`,
  section 1 (every project-bearing family) and its spend paragraph.

## Owns and may change

- Owns the files in `files`.
- May change: `jig/tests/core/src/**` for a fixture that seeds a kept
  `Removal` row, closed ledgers or notes, if `jp-steps`' seeding helper is
  not yet merged or does not fit (add your own beside it, under another
  name).
- Not yours: `removal.rs`, `removal/close.rs`, `removal/steps.rs`,
  `routing/tasks.rs`, `routing/deployment_acts.rs`, `reads*`, `stored.rs`,
  `policy.rs` (tasks `jp-close` and `jp-steps` run beside you). A row
  naming the project that only a file you do not own can erase goes under
  `## Left`.
- Options: if a page of `Scope::Spend` carries closed ledgers' totals but
  not their records, load the `ClosedLedger` rows by the funders it names
  and say so. If the people child's party limit bounds `core.spent` too
  low for a test's parties, raise the test's limit, not the bound.

## Steps

1. `begin` and the `Spend` step; unit tests for the fold.
2. `Notes`, then `Own`.
3. The core world's cases: `heavy cargo nextest run -p jig-core-world
   --test suite -E 'test(/removed_rows/)'`; `heavy cargo nextest run -p
   jig-domain-core --lib -E 'test(/removed_rows/)'`.
4. `cargo fmt`; `heavy cargo clippy -p jig-domain-core --all-targets --
   -D warnings`; the focused tests of `jig-domain-core`, `jig-core-world`.
   Commit and push.

## Report

As `notes/common.md` says; list every family the `Own` step erases and
any row naming the project you found left.
