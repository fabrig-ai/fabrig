+++
id = "pd-conformance"
part = "pd"
kind = "feature"
needs = ["pd-core-release"]
files = [
  "jig/testing/jig-conformance/**",
  "jig/tests/conformance/**",
  "jig/tests/core/src/observations.rs",
  "jig/tests/core/src/observations/**",
  "jig/crates/jig-domain-core/src/projections.rs",
  "jig/crates/jig-domain-core/src/deletion/**",
  "jig/tests/core/tests/suite/deletion.rs",
]
effort = "high"
minutes = 120
+++

## Goal

jig's conformance world proves a project's deletion (draft section 8, "jig's conformance
world"; contracts J4): one story fills every family of jig's store with rows naming a project,
deletes it by the deployment's act, and its check finds no durable row naming the project but
what J4 keeps; the same story cut by a crash after any commit ends the same.

Acceptance:
- `jig_conformance::Scenario::ProjectDeleted`, in `Scenario::ALL` (its length raised), with its
  program: ready; fill; `RemoveProject` by the granted connector; drain until the deletion row
  is gone (advancing the clock past the conversation's grace).
- **Fill** (before the act), rows naming project 2 (project 1 is the control, filled alike in
  part and never deleted), each family the world can reach: tasks `Live`, `Ledger` (a period and
  a pool with spend), `PersonProposal` and `DecidedProposal`, `Ended`, `History`, `Milestone`,
  `Activity`, `Update`, `Result`, `EndedDelegate`, `Delegate`, `EndedByParty`, `EndedByProject`,
  `PlacedHold`; core `Terminal`, `ProposalDecision`, a transcript with turns; people `Roles`,
  `Policy`, `ReadConversation`, `EndedRead`, a party's `Answer` (a rename), the connector's
  `Acted` rows; notes in the project's, a goal's and a resources scope; an account with
  `Some([Use::Project(2)])`; the test connector's adoption, task and proposal rows. Before the
  act, the check asserts each listed family holds a row naming project 2 (a family the world
  cannot reach is named in the report with why, and left out of this assertion only).
- **Check** (`ScenarioApplication::check` for the scenario, and at the end of every cut run):
  every row of `world.store.rows`, matched exhaustively over `root::Record` and every child
  record (no wildcard: the crate denies `wildcard_enum_match_arm`), names project 2 or one of
  its tasks only if it is a people `Acted` or `Answer` row (retained answers) or a tasks
  `ClosedLedger` row (settled spend); no transcript file of project 2's tasks remains (the
  store's readback or files); project 1's rows are all still there; the deletion row is gone and
  project 2's name may be made again.
- **Referee.** The referee learns of the deletion from what the world observes (the act's
  answer and the row's erase): project 2 leaves `policy.projects` once deleted, so a later
  decision naming it is a violation; erased tasks' receipts are cleared (`pd-core-erase`'s
  fact).
- **Cuts.** `every_shared_adversarial_story_runs_on_the_testing_application` and the fuzzy
  every-commit test run the new story; a focused test cuts it after the deciding commit, after
  the first `Erasing` commit and after the first `Releasing` commit
  (`replay::chosen_cuts_traced`), each ending as the uncut run; a fuzzy test
  `fuzzy::deletion::project_deleted_survives_every_cut` uses `replay::every_cut` and checks
  every commit was cut.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, J4, J5.
- `temper/docs/design/draft/playground-reset.md`, sections 5 and 8.
- `jig/docs/design/domain/testing.md` (the conformance world).
- `skein/docs/foundation/testing-strategy.md`, section 1 and 8.1 (budgets).
- The code: `jig/testing/jig-conformance/src/{scenarios.rs,referee.rs,harness.rs}`,
  `jig/tests/conformance/src/{lib.rs,scenarios.rs,replay.rs}` and an existing scenario module
  (`src/scenarios/lifecycle.rs`), `jig/tests/conformance/tests/suite/{scenarios.rs,fuzzy/scenarios.rs}`,
  `jig/tests/core/src/observations.rs` (`durable_at`), `jig/tests/core/tests/suite/deployment_acts.rs`
  (sending a deployment act).

## Owns and may change

Owned: the `files` above. Nothing else in the domain crates: a row the deletion leaves that it
should not is a failure to report, never a domain change here, except the projection repair
the Answers section gives this task.

Options, pre-approved:
- The world grants `remove_project` (and what the fill needs) to connector 1 in its `Config`
  for this scenario only.
- `root::Delivery::Core(Held::Acted)` is dropped by the world today (`src/lib.rs` ~643):
  collect the act's answer for the referee if it needs it.
- If a family cannot be filled in this world without a domain change, leave it out of the fill
  assertion and name it in the report.

## Steps

1. `jig-conformance`: the scenario, its actions (`FillForDeletion`, `RemoveProject`), its
   program, and the referee's project drop.
2. The world: `src/scenarios/deletion.rs` (fill, act, check), wiring in `src/scenarios.rs`
   and `src/lib.rs`; observer facts in `jig/tests/core/src/observations*.rs` if the referee
   needs them.
3. Tests: `tests/suite/scenarios.rs` `project_deleted_chosen_stage_cuts_end_the_story`;
   `tests/suite/fuzzy/deletion.rs` (registered in `fuzzy/mod.rs` or `main.rs` as its
   neighbours are) `project_deleted_survives_every_cut`.
   Commands: `heavy cargo nextest run -p jig-conformance-world --test suite -E 'test(/project_deleted/)'`,
   `heavy cargo nextest run -p jig-conformance-world --test suite`,
   `heavy cargo nextest run -p jig-conformance-world --profile fuzzy --test suite -E 'test(/fuzzy::deletion::/)'`,
   `heavy cargo nextest run -p jig-conformance --lib`,
   `heavy cargo nextest run -p jig-core-world --test suite` (observer changes).
   Read each tier's timing: over budget is red (AGENTS.md, 6); say the fuzzy test's time.
4. `cargo fmt`; `heavy cargo clippy -p jig-conformance -p jig-conformance-world -p jig-core-world --all-targets`;
   old forms; commit and push.

## Report

The families filled and those the world cannot reach (with why); the fuzzy test's time and
the number of cuts; any row the check found that the deletion should have erased.

## Answers

2026-11-01. **Q: a core `Projection` Goal row of project 2 (closing, `pending: [1, 2]`)
survives the deletion after cut 22; who repairs it?** This task, widened (its own acceptance
cannot hold without it; `pd-core-release` is already in the trunk). A tracked goal's
projection is the project's own row, so the deletion's *Releasing* stage retires it (J4,
"the project's own rows"):

- In `deletion/release.rs` `release` (the one decision, also redone at each start), forget
  every entry of `core.projections` whose `goal.project` is the deleted project, through
  `projections.rs`'s `forget` made `pub(crate)` (or a `pub(crate) fn forget_project(core,
  project, out)` beside it): its task, milestone and goal rows erased and
  `Ask::ForgetProjection { goal }` to every connector. A late settle for it (`route_projection_settled`) then
  changes nothing (`settled` returns on an absent goal), and the start's restore never meets
  a projection whose project has no policy.
- Leave `eligible`/`restore_goal`'s general behaviour (a closing feed restored with its
  cohort pending is not fed again) as it is: it is not this task's, and the deletion no longer
  depends on it. Name it in the report under `## Left`, with the cut that shows it, for the
  domain owner.
- A core test in `jig/tests/core/tests/suite/deletion.rs`,
  `releasing_forgets_the_projects_projections` (a tracked goal ended with its closing feed
  unsettled by a connector, then the project deleted: no `Projection` row names it, and each
  connector was asked `ForgetProjection`), run with
  `heavy cargo nextest run -p jig-core-world --test suite -E 'test(/deletion::/)'` and
  `heavy cargo nextest run -p jig-domain-core --lib`.
- Then rerun the fuzzy commands from Steps 3 (keep the exhaustive check as it is, no
  exception). If a further row of another family remains at some cut, repair it the same way
  only if it is inside the files above; otherwise stop and report it with its family and cut.
