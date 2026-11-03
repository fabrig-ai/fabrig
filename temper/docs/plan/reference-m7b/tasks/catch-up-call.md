+++
id = "catch-up-call"
part = "catch-up"
kind = "feature"
needs = ["change-merge-route"]
files = ["temper/crates/temper-engine-smith/src/calls.rs", "temper/crates/temper-engine-smith/src/tests.rs", "temper/crates/temper-engine-domain/src/boundary.rs", "temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/tests/engine/domain/tests/suite/forge_routes/catch_up.rs", "temper/tests/engine/domain/tests/suite/forge_routes.rs"]
effort = "high"
minutes = 140
+++

## Goal

A coordinator catches its goal's branch up with main: `change` takes `merge`, a commit, and the
change it makes has the merge job for its producer, which merges that commit into a branch made
at the landing branch's tip (`temper/docs/design/domain/agent.md, 4.3.1`;
`temper/docs/design/domain/forge.md, 8.1`, 8.2 step 1, section 10). The steps are
`change-merge-steps`' and the job's making `change-merge-route`'s; this task is the call, its
refusals, parameter 16 and the stories.

Acceptance:

- `change`'s plan schema offers `merge` (a string: "A catch-up: the commit to merge into a
  branch made at the landing branch's tip, as 40 or 64 hexadecimal digits; the merge job, not
  an agent, produces it."); the plain schema does not; the call's `merge` reaches
  `engine::ChangeCall::merge` (C6);
- C6's refusals, each naming what it lacks, in the root's words for a call's refused member:
  from a run without jig's control family (as the other plan members are refused); a project
  naming no `merge_job`; a run whose authority does not delegate to `Job(3)`; a commit not 40 or
  64 hexadecimal digits;
- the change's spec carries parameter 16, and `start_change` sets `ChangeRow::catch_up` from it;
- engine domain world stories in `forge_routes/catch_up.rs`, each citing
  `temper/docs/design/domain/forge.md, 10`:
  - `a_catch_up_merges_mains_commit_into_a_branch_at_the_goal_branchs_tip`: a run whose task
    holds a goal branch calls `change` with `landing: "goal"` and `merge` naming main's tip;
    a job of kind 3 is assigned whose item starts a merge of that commit at the goal branch's
    tip, pushing the change's own branch; the world pushes the merge and answers the job; the
    change opens its pull request into the goal branch and merges, and the goal branch then
    contains main's commit;
  - `a_catch_up_the_merge_job_cannot_settle_goes_to_a_resolution`: the job answers with no
    push; a resolution is assigned, starting from the goal branch's tip with the commit merged
    in, its words carrying the job's result; after its push the change opens and merges;
  - `a_catch_up_already_contained_ends_with_no_change`: `merge` names a commit the goal
    branch already contains; the change ends with no change, its words saying so, no job made;
  - `a_catch_up_is_refused_without_a_merge_job`, `..._without_the_merge_kind`,
    `..._for_a_malformed_commit`: each refusal's words.

## Inputs

- `temper/docs/design/domain/agent.md, 4.3.1` (`merge`, the change's authority);
  `temper/docs/design/domain/forge.md, 8.1`, `8.2` (step 1), `8.4`, section 10 ("A catch-up").
- `temper/docs/plan/reference-m7b/notes/contracts.md`, C2, C3.1, C5, C6; `notes/common.md`.

## Owns and may change

- `temper/crates/temper-engine-smith/src/calls.rs`: `PLAN_CHANGE`'s schema and the `change`
  arm's parse of `merge` (hexadecimal parsed there into C6's `[u8; 32]`, a SHA-1 commit padded as
  the forge pads one; the malformed refusal may be the parse's, naming `merge`); its tests in
  `src/tests.rs`.
- `temper/crates/temper-engine-domain/src/boundary.rs`: `ChangeCall::merge`.
- `temper/crates/temper-engine-domain/src/route/forge_route.rs`: `change_batch` (the refusals),
  the change's parameters (16), `start_change` (parameter 16 into `catch_up`).
- `temper/crates/temper-engine-domain-forge/src/items.rs`, only if a resolution of a catch-up
  whose branch is not made does not start from its landing branch's tip: make `Resolve`'s item
  start there when the change has never pushed (`last_head` none), as `Merge { from: None }`'s
  does, and say so.
- Tests: `temper/tests/engine/domain/tests/suite/forge_routes/catch_up.rs`, registered in
  `forge_routes.rs`, reusing `goal_branch.rs`' and `light_rules.rs`' helpers (move a helper to the
  world's `src/` only if two files need it).
- Readers: `engine::ChangeCall { .. }` literals in temper (about eight, `rg 'ChangeCall \{'`)
  gain `merge: None`, field only.
- Not this task's: the forge-change crate, the help page (`merge-help`), the reference's files.

## Steps

1. The schema and parse; `ChangeCall::merge`; the refusals in `change_batch`; parameter 16.
2. The stories; `temper-engine-smith` unit tests for the parse
   (`a_plan_change_reads_merge_as_a_commit`, `a_malformed_merge_is_refused_naming_it`).
3. `heavy cargo nextest run -p temper-engine-smith --lib -E 'test(/merge|change/)'`,
   `heavy cargo nextest run -p temper-engine-domain-world -E 'test(/catch_up/)'`, then the
   packages' tests (`-p temper-engine-smith`, `-p temper-engine-domain --lib`,
   `-p temper-engine-domain-world`); `heavy cargo check -p temper-service --tests`; `cargo fmt`;
   `python3 -I -B temper/scripts/old_forms.py`; clippy on the three packages,
   `--all-targets -- -D warnings`.
4. Commit and push.

## Report

As the implementer's text says.

## Answers

### 2026-11-03: Should fab resume this task after change-merge-route supplies its typed job-workspace handoff (recommended), or assign that unfinished prerequisite here? (reference-m7b/catch-up-call-1, foreman reference-m7b/catch-up-call/foreman-1)

Resume catch-up-call after change-merge-route supplies its typed job-workspace handoff; do not assign that prerequisite here.
The spec declares `needs = ["change-merge-route"]` and says “the job's making `change-merge-route`'s; this task is the call, its refusals, parameter 16 and the stories.”
`temper/docs/plan/reference-m7b/tasks/change-merge-route.md`, Answers: “This task builds it” assigns workspace preparation, claim, typed delivery and cleanup to change-merge-route.
That answer also states: “a catch-up's `Merge { from: None }` head stays `catch-up-call`'s.”
Acceptance remains incomplete; the pushed work is not accepted as done.
After the prerequisite lands, resume catch-up-call to finish the item-start and push assertions, complete both landing stories, and run its required final checks.
