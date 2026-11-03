+++
id = "apps-proposals"
part = "appsettings"
kind = "feature"
needs = ["apps-change"]
files = ["jig/crates/jig-domain-core/src/routing/settings/dispatch.rs", "jig/crates/jig-domain-core/src/settings.rs", "jig/crates/jig-domain-core/src/settings/application.rs", "jig/crates/jig-domain-core/src/documents/settings_review.rs", "jig/crates/jig-domain-core/src/tests/settings/application_proposals.rs", "jig/crates/jig-domain-core/src/tests/settings.rs", "jig/docs/design/domain/engine.status.md"]
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
