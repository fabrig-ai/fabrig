+++
id = "goal-release"
part = "release"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-domain-forge/src/goal_branch.rs", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/tests/goal_branch.rs", "temper/docs/design/domain/forge.status.md"]
effort = "high"
minutes = 60
+++

# goal-release: a goal's own branch is deleted as its goal closes once a change landed it

## Goal

As a goal closes, its hold's release keeps its own branch g whatever way it ended
(`continue_release`, the `goal_branch::is_goal` arm, in
`temper-engine-domain-forge/src/domain.rs`). The design deletes g when a change landed it
and keeps it otherwise (`temper/docs/design/domain/forge.md`, section 11 and 11.1). Decide it
from the change rows the forge keeps, as contract C2 says: no new read, field or effect.

Acceptance:
- A goal whose branch a landed change (a change row in g's repository, its `branch` g's
  name, landed) merged at g's last known tip: its release deletes g with
  `Write::DeleteBranch`, as a change's branch is deleted, and is `Released` once the delete
  is made; for each of the endings done, failed and cancelled.
- A goal whose branch no change landed, or whose tip moved past the landed head, or whose
  tip is unknown: its release keeps g as today, its hold erased, nothing retained.
- A delete the forge refuses for good fails the release (`ReleaseFailed`), as any branch's;
  a branch already gone counts as deleted (the client's idempotent `DeleteBranch`).
- Every test of `temper-engine-domain-forge` passes.

## Inputs

- `temper/docs/design/domain/forge.md`, section 11 ("Deleted as their tasks close", the goal's
  own branch's sentence) and 11.1 ("Landed whole").
- `temper/docs/design/domain/after-landing.md`, section 6 (the goal's landing holds none).
- `temper/docs/plan/reference-m2/notes/contracts.md`, C2.
- `temper/docs/plan/reference-m2/notes/common.md`.
- `temper/docs/design/domain/forge.status.md`: its **Not yet** "11, 11.1: a goal's own branch
  deleted as its goal closes once a change has landed it: ...", which this task removes.

## Owns and may change

Owns the files in the header; in `domain.rs`, only `continue_release`'s goal-branch arm and
what it calls (forgejo-connect and the wave-2 plans change other parts of that file).

- `goal_branch.rs`: a `pub(super) fn landed(d: &Domain, name: &Name) -> bool`: true when a
  change row in `d` (the map `Domain::change` reads) has the name's repository, `branch` equal
  to the branch the name names, a landed state (`landed_commit(&row.change.state)` is
  `Some`), and its landed head equal to `d.branch_head(name)`. The landed head is the head
  the change merged at: its row's `result.head` (the head its checks last read, which a
  change merges at exactly: forge.md, 8.2 step 7). If the code keeps the merged head more
  exactly elsewhere on the row, use that and say so. The module's first comment line says
  what is true now: "A goal's taking makes its branch; its release deletes it once a change
  landed it, and keeps it otherwise".
- `continue_release`: in the `is_goal` arm, when `goal_branch::landed(d, &name)`, fall
  through to the branch deletion below (the `to_client` and `enqueue_release` path), never
  to the failed-and-not-root retention (a goal's branch is never handed to its root);
  otherwise keep the arm as it is.
- If `d.branch_head` of g is not kept current by the moves L's own repairs make (the forge
  saves owned advances of a held branch: read where `BranchHead` is saved), say so in the
  report under `## Left` with the function that would need it; do not widen this task.

## Steps

1. Tests first, in `src/tests/goal_branch.rs`, using its helpers (`fresh`, `take`, `name`,
   `outputs`, `fixtures::answer_next`, `GOAL`, `OTHER`, `CLOSING`, `TIP`, `GOAL_BRANCH`):
   - rename `a_closing_goal_keeps_its_branch_whatever_way_it_ended` to
     `a_closing_goal_keeps_its_branch_when_no_change_landed_it`, unchanged otherwise;
   - `a_closing_goal_deletes_its_branch_once_a_change_landed_it_whatever_way_it_ended`:
     for each ending, restore (or record through the events the forge's tests use for a
     change, `rg 'Stored::Change\(' temper/crates/temper-engine-domain-forge/src/tests`) a
     change row in the same repository whose `branch` is `GOAL_BRANCH`, its state `Landed`
     with a merge commit, its `result.head` `Some(TIP)`; release the goal; assert the next
     call is `Write::DeleteBranch` of `GOAL_BRANCH`; answer it made; assert `Released {
     task: GOAL }` and no hold;
   - `a_closing_goal_keeps_its_branch_moved_past_its_landing`: the same row, its
     `result.head` another commit than g's tip; assert g kept, no `DeleteBranch`;
   - `a_goal_branchs_refused_delete_fails_the_release`: the delete answered with a
     permanent error; assert `ReleaseFailed { task: GOAL, .. }` and the hold kept.
2. The code, as above.
3. Narrow: `heavy cargo nextest run -p temper-engine-domain-forge --lib -E
   'test(/goal_branch::/)'` (check the count); then `heavy cargo nextest run -p
   temper-engine-domain-forge`.
4. The stories that read a goal's end: `heavy cargo nextest run -p temper-engine-domain-world
   --test suite -E 'test(/forge_routes::goal/)'` (`a_goals_end_keeps_its_own_branch` lands
   nothing, so it keeps g and passes unchanged).
5. Remove the **Not yet** entry above from `forge.status.md`. `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-engine-domain-forge
   --all-targets -- -D warnings`. Commit ("forge: delete a goal's branch as it closes once a
   change landed it") and push.

## Report

As the implementer's text says.
