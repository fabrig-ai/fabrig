+++
id = "mine-forge"
part = "mine"
kind = "feature"
needs = ["mine-procedure", "goal-release"]
files = ["temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/items.rs", "temper/crates/temper-engine-domain-forge/src/tests/held_branch.rs", "temper/crates/temper-engine-domain-forge/src/tests.rs"]
effort = "high"
minutes = 90
+++

# mine-forge: the forge's side of a change on its requester's held branch

## Goal

A change on its requester's held branch (L on g, g held by its goal G) holds no branch: G's
hold covers g. The forge must still treat L as g's change where a change's branch matters:
drift on g reaches L as well as G; the update L asks for is never drift; L's repair and
resolution push g under G's hold; and the procedure learns that L's branch is its
requester's (C3's `requester_branch`). Contract C5 has each rule.

Acceptance (pure domain tests in the forge crate):
- A hold of g by task G and a change row for L (task L, `branch` g's name, live): a drift
  found on g sets G's hold's drift as today, and sets L's `ChangeRow.drift` and emits
  `Request::Drift { task: L, .. }` as for a change holding its branch.
- A pending update (or prepare) entry of L on g suppresses drift on g, as the holder's own
  does today.
- `workspace_item` for L's repair and for its resolution names `holder: Some(G)` (g's
  holder) and pushes `row.branch`; for a change that holds its branch, `holder` is the change
  as today.
- `change_facts` gives `requester_branch: true` for L, `false` for a change holding its own
  branch and for one whose branch has no known hold.
- Every test of `temper-engine-domain-forge` passes, and `temper-engine-forge-world`'s
  `simulation` stories.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.1 ("A change on its requester's held branch"), 8.4
  (a repair, a resolution) and 8.5 (its paragraph "On its requester's held branch").
- `temper/docs/plan/reference-m2/notes/contracts.md`, C3 and C5.
- `temper/docs/plan/reference-m2/notes/common.md`.

## Owns and may change

Owns the files in the header; in `domain.rs`, only `drift`, `updating_entry`, `change_facts`
and helpers they call (forgejo-connect and the wave-2 plans change other parts of it).

- `domain.rs`, `drift`: after the holder's handling, for each change row (`self.changes`, or
  the map `Domain::change` reads) in the same repository whose `branch` is the drifted
  branch's name, whose task is not the hold's, and whose state is not ended: the same
  `row.drift` and `Request::Drift { task: row.task, resource }`, unless that change's own
  update or delete entry is pending. Keep the holder's path byte for byte.
- `domain.rs`, `updating_entry` (or its caller in `drift`): the holder's check also passes
  when a live change on the held branch (as above) has an update or prepare entry pending.
- `domain.rs`, `change_facts`: `requester_branch` as C3 says, replacing the `false` task
  `mine-procedure` put there.
- `items.rs`, `workspace_item`: for `Repair` and `Resolve`, `holder` is the forge's hold of
  `row.branch` when it is another task's (`self.holds`/`Domain::hold`), else `row.task` as
  today. `Produce` is unchanged (a change on its requester's branch has no producer).
- If `claimed_writes`/`claim` (the run writer slot on g's hold) refuses L's repair because
  the slot's holder is G (read `claim` near `domain.rs:2190`): make the least change that
  admits a run of a task below the holder, say so in the report, and add a test.
- New test module `src/tests/held_branch.rs`, declared in `src/tests.rs` (one `mod` line).

## Steps

1. Tests first in `src/tests/held_branch.rs`, with the crate's fixtures (`fixtures::domain`,
   `adopted`, `outputs`, the goal branch helpers of `src/tests/goal_branch.rs` made
   `pub(super)` if needed, visibility only):
   `drift_on_a_goal_branch_reaches_the_change_landing_it`,
   `the_landing_changes_own_update_of_the_goal_branch_is_not_drift`,
   `a_landing_changes_repair_writes_the_goal_branch_under_its_goals_hold`,
   `facts_say_whether_a_changes_branch_is_its_requesters`.
   Model the drift ones on `src/tests/change.rs`'s `a_pending_prepare_rebase_is_not_branch_drift`.
2. The code, as above.
3. Narrow: `heavy cargo nextest run -p temper-engine-domain-forge --lib -E
   'test(/held_branch::/)'` (check the count); then `heavy cargo nextest run -p
   temper-engine-domain-forge`; then `heavy cargo nextest run -p temper-engine-forge-world
   --test suite -E 'test(/simulation::/)'`.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain-forge --all-targets -- -D warnings`; `heavy cargo check -p
   temper-engine-domain --tests`. Commit ("forge: a change on its requester's held branch
   hears its drift and writes it under its holder") and push.

## Report

As the implementer's text says.
