+++
id = "next-run-tip"
part = "next-run"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-domain-forge/src/items.rs", "temper/crates/temper-engine-domain-forge/src/tests/early.rs"]
effort = "high"
minutes = 60
+++

# A goal branch's change: its next producer merges the branch's tip first

## Goal

A producer of a change into a landing branch whose rules ask no update (a goal's own branch),
starting from the change's pushed branch or from saved work, starts from a merge in progress
with the landing branch's tip as base, as an early-started change's next producer already does
(`temper/docs/design/domain/forge.md`, 8.11, "The next producer merges the landing tip
first"). So a producer going on after an answer that added a dependency, or after an amendment
naming a design revision landed into g, works on g as it now stands.

Acceptance:

- `temper-engine-domain-forge/src/items.rs`, `workspace_item`, where a `Produce` delegate's
  `Start::Branch` or `Start::Saved` becomes `Start::Merge { branch, base: <the landing
  branch's head> }` for `change::Early::Started(_) | change::Early::Spent(_)`: the same also
  when `self.requires(repository, &row.base).up_to_date` is false (`Domain::requires`,
  `src/domain.rs`, the rules covering the landing branch; a branch no rule covers requires an
  update, so nothing changes there). A change with no early start into such a branch, with
  nothing pushed or saved, still starts from the landing branch's tip (`Start::Base`).
- Changes into a branch whose rules ask an update (main, under its rule) keep today's starts.
- Tests in `temper-engine-domain-forge/src/tests/early.rs`, beside
  `an_early_changes_next_producer_merges_the_landing_tip_into_branch_or_saved_work`, built the
  same way:
  - `a_goal_branch_changes_next_producer_merges_the_landing_tip_into_branch_or_saved_work`: a
    change with no early start into a branch covered by a rule with `up_to_date` false (as
    the reference's rule for `temper` branches, `temper/docs/examples/reference/settings.json`'s
    landing rule 2), its branch pushed (and, in a second case, saved work): its producer's
    workspace item is `Start::Merge` with the landing branch's head as base;
  - `a_main_changes_next_producer_starts_from_its_branch`: the same change into a branch whose
    rule asks an update: `Start::Branch` (and `Start::Saved`) as today.
- The focused tests of `temper-engine-domain-forge` and `temper-engine-domain` pass (this
  changes what a workspace item means for goal branches: run `temper-engine-domain`'s too).

## Inputs

- `temper/docs/plan/reference-m10/notes/common.md`.
- `temper/docs/design/domain/forge.md`, 8.11 ("Early starts": "The next producer merges the
  landing tip first") and 8.7 (the landing rules' waits); `temper/docs/design/domain/worker.md`,
  4.1 and section 5 (a start from a merge in progress).

## Owns and may change

- Owns the two files of this spec's header for what acceptance names.
- The test module's fixtures in `temper-engine-domain-forge/src/tests.rs`, to set a landing
  rule with `up_to_date` false where none exists: a helper or a field, nothing else.
- If `workspace_item` cannot reach the landing rules (no `requires` on `self` there), stop and
  ask, naming what it lacks.

## Steps

1. The condition; the two tests; run them alone (`heavy cargo nextest run -p
   temper-engine-domain-forge --lib -E 'test(/goal_branch_changes_next_producer|main_changes_next_producer/)'`).
2. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge --all-targets -- -D
   warnings`; the focused tests of `temper-engine-domain-forge` and `temper-engine-domain`;
   commit and push.

## Report

As the implementer's text asks.
