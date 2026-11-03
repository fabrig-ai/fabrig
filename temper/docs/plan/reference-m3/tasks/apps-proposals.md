+++
id = "apps-proposals"
part = "appsettings"
kind = "feature"
needs = ["apps-change"]
files = ["jig/crates/jig-domain-core/src/routing/settings/dispatch.rs", "jig/crates/jig-domain-core/src/settings.rs", "jig/crates/jig-domain-core/src/settings/application.rs", "jig/crates/jig-domain-core/src/documents/settings_review.rs", "jig/crates/jig-domain-core/src/tests/settings/application_proposals.rs", "jig/crates/jig-domain-core/src/tests/settings.rs", "jig/docs/design/domain/engine.status.md", "jig/crates/jig-domain-tasks/src/person_proposals.rs", "jig/crates/jig-domain-tasks/src/tests/settings_proposals.rs"]
effort = "high"
minutes = 100
+++

# apps-proposals: a proposal of an application's settings, kept and decided

## Goal

A proposal of settings naming items of an application's kinds hands their values to the
kinds' connectors in the decision that makes it, keeps only their edits and sizes in the
core's set, erases them at the connector when it is withdrawn (stale included) or rejected,
and, accepted, is judged from the values the connector kept and turned into the change,
as `apps-change` decides a party's.

Acceptance:

- The unit tests below pass; `jig-domain-core`'s focused tests and `jig/tests/core`'s
  settings-proposal stories pass unchanged (a change of meaning of the proposal decisions:
  run them).
- The **Not yet** of `jig/docs/design/domain/engine.status.md` loses "17.6" from "an
  application's kinds in 17.6 and 17.7" (leaving 17.7).

## Inputs

- `jig/docs/design/domain/engine.md`, 17.5 (**Kept in the same decision**, its proposal
  sentences), 17.6 (all).
- `jig/docs/design/domain/connectors.md`, section 17 (**Kept in the same decision**).
- `temper/docs/plan/reference-m3/notes/contracts.md`, C2 (`KeepProposalSettings`,
  `DropProposalSettings`, `JudgeSettings`' `proposal`, `KeepSettings`' `proposal`).
- `temper/docs/plan/reference-m3/notes/common.md`.

## Owns and may change

- Owned: the `files` above.
- Neighbour: `jig/crates/jig-domain-core/src/settings/restart.rs`, the restore of a
  proposal's set only (its `applications`, below), so a restart keeps it. Every
  `settings::Proposal` literal in jig and temper gets `applications: Box::default()` (the
  field alone; in `jig-protocol-store`'s codec until `apps-store` stores it).
- Options, pre-approved:
  - If a proposal's number is not unique across proposers (`Key::Proposal` keys by
    proposer and number), send the connector the number the tasks child gave the proposal
    as it is (`u64::from(number)`), as `KeepProposal` for effects does, and say in the
    report whether two proposers can share a number.
  - If the review document (`documents/settings_review.rs`) cannot give an application
    item's size without a client-domain change, leave the review's application items out
    and say so: `apps-wire` carries them.
  - **Who may accept** a set naming a project's items (17.6: that project's owners too):
    if `proposal_check`'s owner test and the tasks child's settings waiting can name the
    projects the set's items name (the projects `apps-change` passes for *not theirs*),
    require an owner in each (default); if the tasks child has no place for them, keep
    deployment owners as the accepters, say so in the report, and leave "17.6: a
    proposal naming a project's items waits for the deployment's owners alone" in the
    status entry in place of the removed text.

## Steps

1. `settings::Proposal` gains `applications: Box<[application::Recorded]>` (its `items`
   keep jig's kinds; a set's application items as their `SizedEdit`s, a set's size its
   bytes' length).
2. **Proposed** (`prepare_proposal`, `proposal_check`): the core's checks of `apps-change`
   step 2 run on the application items; in the decision that keeps the set, each kind's
   connector gets one `Ask::Effect(connector::Ask::KeepProposalSettings { proposal, items })`
   with its items (`Edit::Set(bytes)`, `Remove`, `Reset`). No judge: the connector checks
   its kinds' bounds alone and keeps them.
3. **Erased**: the decision that withdraws (stale, in `withdraw_stale`, or by its proposer)
   or rejects a pending set naming application items sends each connector named
   `DropProposalSettings { proposal }` once.
4. **Accepted** (`accept`): the change of `apps-change` runs with `JudgeSettings {
   proposal: Some(proposal), .. }`, each `Set` item's bytes empty (the connector judges the
   value it kept), and, once committed, `KeepSettings { proposal: Some(proposal), .. }`. An
   acceptance refused (judged, out of shape, full) leaves the proposal pending and its kept
   values where they are.
5. **Review** (`documents/settings_review.rs`): an application item shows the setting, its
   edit, its state now (`Follows`) and its size now and after, never a value.
6. **Tests** (`src/tests/settings/application_proposals.rs`, the scripted connector of
   `apps-change`'s tests): a proposal kept (the `KeepProposalSettings` items; the core's set
   sizes only); withdrawn by a later revision (one `DropProposalSettings`); rejected (one);
   accepted (a `JudgeSettings` with `proposal`, then `KeepSettings` with `proposal`); an
   acceptance judged refused leaves it pending, no drop sent; a restart keeps the set's
   applications.
7. Narrow tests: `heavy cargo nextest run -p jig-domain-core --lib -E 'test(/settings/)'`;
   `heavy cargo nextest run -p jig-core-world --test suite -E 'test(/settings_proposal/)'`
   (check the count). `cargo fmt`; `heavy cargo clippy --workspace --all-targets`. Edit the
   status entry. Commit and push.

## Report

The options taken; how the accepted proposal's judge distinguishes kept values.

## Answers

2026-11-03. **Q:** may this task relax `jig-domain-tasks`' `person_proposals.rs::valid_settings`
to admit a settings proposal with no jig heads, or must a separate task first add
application heads to the tasks vocabulary?

**A:** Relax it, in this task (now in `files`). `engine.md`, 17.6 (**The owner keeps the
value**): the tasks child keeps the proposal's proposer, reason, holder and state; the core
keeps its set. The tasks child's heads are jig's kinds only, and a set naming only an
application's items has none of them, so no application head is added to the tasks
vocabulary, and no jig head is invented.

- `valid_settings` admits an empty head list (the upper bound `SETTINGS_ITEMS`, the zero
  numbers and the duplicates stay refused); its doc says the core checks that the whole set,
  application items included, is non-empty. Both callers (`valid_action`,
  `proposals.rs::proposal_scope`) keep using it unchanged.
- The core refuses a set with neither jig items nor application items, as it does today for
  an empty or unchanged set; keep (or add) that refusal in this task's tests.
- `src/tests/settings_proposals.rs` (jig-domain-tasks): one test that a deployment-scope
  settings proposal with no heads is kept pending at the deployment's owners. Run
  `heavy cargo nextest run -p jig-domain-tasks --lib -E 'test(/settings_proposals/)'` (check
  the count) and add `jig-domain-tasks` to the clippy run. The store's codec encodes empty
  heads already (it bounds the list above only): change nothing there.
- `tests::settings::application::application_removed_states_do_not_exhaust_the_declared_active_value_bound`:
  run it once on `apps-change`'s commit alone. Failing there, it is not this task's: leave it
  under `## Left` with both logs, and this task's acceptance holds without it. Passing there,
  this task's change broke it: fix it here.
- The options already taken stand (deployment owners as accepters with the 17.6 limitation
  kept in the status; application review items left to `apps-wire`; the tasks child's number
  as the connector's proposal, which `apps-proposal-key` later replaces with a key).
