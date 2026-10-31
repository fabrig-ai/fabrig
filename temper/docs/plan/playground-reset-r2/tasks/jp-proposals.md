+++
id = "jp-proposals"
part = "project-delete"
kind = "interface"
needs = ["jp-scopes"]
files = [
  "jig/crates/jig-domain-core/src/reads/**",
  "jig/crates/jig-protocol-store/src/reads/**",
  "jig/crates/jig-protocol-store/src/intervals.rs",
  "jig/tests/store/tests/suite/erasure_reads.rs",
]
effort = "medium"
minutes = 30
+++

## Goal

Add the third core-only erasure read: every decided person proposal, in
proposal order (tasks' `DecidedProposal` rows, which live only in the
store and name their project), so a project's deletion erases its
decided proposals with the rest (playground-reset.md, 5). It is
`jp-scopes`' `EndedTasks` again for another family, but for one join:
each row carries its `CoreRecord::ProposalDecision` as `related`, as the
point read of a party's proposal does (`reads/point.rs`, `proposal`), so
the deletion can plan the proposer's `Update`, keyed by the decision's
time, without another read (`jp-planner.md`, Answers).

Acceptance (`jig-store-world`, `tests/suite/erasure_reads.rs`, which
`jp-scopes` made):

- `decided_proposals_page_every_decided_proposal_in_order`: three decided
  proposals of two projects and one pending: pages of one row give the
  three `DecidedProposal` records in proposal order, `next` after each,
  `None` at the end; the pending one is not among them; each row's
  `related` is its saved `ProposalDecision` record, and `None` for one
  saved without it.
- A party's read of the scope is refused, as `jp-scopes`' are (one case
  beside `a_party_read_of_an_erasure_scope_is_refused`).

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md`.
- `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/design/draft/playground-reset.md`, section 5.
- `temper/docs/plan/playground-reset-r2/tasks/jp-scopes.md` (the shape
  this repeats).

## Owns and may change

Owns the files in the header, which `jp-scopes` owned before (it has
merged). May change, as `notes/scope.md` says: every exhaustive match on
`reads::Scope` the variant forces, in jig and temper, as `jp-scopes`'
report lists them, each arm as `EndedTasks`'.

## Steps

1. `reads/vocabulary.rs`'s `Scope`: `DecidedProposals`, doc as
   `EndedTasks`' (the core's own read, for a project's deletion).
2. `intervals.rs`'s `scope()`: the prefix `[JIG, 1, 8]` (tasks'
   `DecidedProposal`; check the tag against `keys/tasks.rs`). Each page
   row is built with `joins::row`'s `related` key
   `CoreKey::ProposalDecision { proposer: Proposer::Party(row.proposer),
   proposal: row.number }`, as `point::proposal` builds it.
3. Tests: the acceptance's. Then, one at a time:
   - `heavy cargo nextest run -p jig-store-world --test suite -E 'test(/erasure_reads/)'`;
   - `heavy cargo nextest run -p jig-protocol-store -p jig-domain-core`;
   - `heavy cargo nextest run -p jig-store-world -p jig-core-world -p jig-client-world -p jig-ops-world`;
   - `cargo fmt`; clippy over the packages you edited; old forms from `jig/`.
4. Commit and push.

## Report

The standard report, plus the forced arms outside the owned files.

## Answers

2026-10-31. Q (asked by `jp-planner`): how does a project's deletion
learn an accepted proposal's decision time, which its row does not keep?
A: This read joins each row's `ProposalDecision` record as `related`
(Goal, acceptance and step 2 amended); no stored format changes.

