+++
id = "story-rows"
part = "story"
kind = "feature"
needs = ["story", "conflicts"]
files = ["temper/tests/system/**"]
effort = "high"
minutes = 150
+++

## Goal

Complete the merge job's story in the composed system world
(`temper/docs/design/domain/testing.md`, 2.1) with the rows the request
names beside task `story`'s two: a clean merge, a base commit only on a
branch the job's checkout has not fetched, and more files in conflict
than the checkout's `conflicts`. The last is the fault the cut found by
reading (`temper/docs/design/protocol/git.md`, 4.6); with task
`conflicts` landed, the world's fake follows the git layer's rule and the
row shows the job settling every file.

Acceptance:

- `Row::Clean`, `Row::OtherBranch` and `Row::PastTheBound` in
  `temper/tests/system/src/composed/merge_job.rs`, and their rows in
  `a_changes_merge_job_runs_on_its_worker`'s table
  (`notes/contracts.md`, section 3); no second test. Each row keeps
  `story`'s common checks (the hello's kind, the command's path and
  request, the payload limits, the job's task finished with no failure
  counted).
- The composed world's checkout operations go through
  `temper_worker_checkout_world::translate::perform_within` with its
  worker's `limits.checkout.conflicts` (`notes/contracts.md`, section 4);
  `story`'s rows and the world's other stories stay green.
- `Row::Clean`: a catch-up of main into the goal's branch (the goal's
  `change` call naming `"merge":"<main's tip>"`, as the engine world's
  `catch_up` makes one), main advanced in a file the goal's branch never
  touched. The command sees no file in conflict, writes
  `{"settled":[]}` and exits 0; the catch-up's branch is pushed as a
  commit whose parents are the goal branch's tip and main's tip, holding
  both sides' files; the catch-up goes on from it.
- `Row::OtherBranch`: the base is a commit that exists only on a branch
  the job's checkout has not fetched when the job starts: the change's
  checkout cached by its workstream, the base made afterwards on the goal
  branch (or another branch the checkout's fetch does not name). The
  report shows the base absent from the cached clone on the fake disk
  before the job's preparation and the preparation's fetch of the base by
  its id; then as `Row::Settled`.
- `Row::PastTheBound`: the change's update conflicts with its base in
  `conflicts + 1` files. The job prepares (the merge in progress holds
  every file marked); the command settles and stages all of them, writes
  `{"settled":[...]}` within the 512-byte result bound and exits 0; the
  merge is pushed with both parents; the job's task ended finished with
  no transient failure counted.
- `temper/tests/system/README.md`'s sentence for the story names its
  five rows.

## Inputs

- `temper/docs/plan/merge-job-runs-2/notes/common.md`;
  `temper/docs/plan/merge-job-runs-2/notes/contracts.md`, sections 3 and 4.
- `temper/docs/design/domain/testing.md`, 2.1 (the merge job's story);
  `temper/docs/design/domain/forge.md`, 8.2 (step 1: a catch-up's
  producer is the merge job), 8.4 (*The merge job*), section 10 and 11.1
  (a goal's catch-ups);
  `temper/docs/design/domain/worker.md`, 4.4 (a merge job past the
  checkout's `conflicts`) and section 5 (*A merge in progress*; the cache
  keyed by workstream); `temper/docs/design/protocol/git.md`, 4.3 (a fetch
  by commit) and 4.6.
- Task `story`'s world and table (`temper/tests/system/src/composed/merge_job.rs`,
  `temper/tests/system/tests/suite/merge_job.rs`) and its report.
- A catch-up made and checked in the engine world:
  `temper/tests/engine/domain/tests/suite/forge_routes/catch_up.rs`
  (`goal_world`, `catch_up`,
  `a_catch_up_merges_mains_commit_into_a_branch_at_the_goal_branchs_tip`).
- What the fake clone fetches: `skein/crates/skein-fake-checkout/src/git.rs`
  (clone imports every branch tip; a fetch imports its source's history),
  and `temper/tests/worker/checkout/src/translate.rs`'s `Op::Fetch`.

## Owns and may change

- **Owns:** `temper/tests/system/**`.
- **Options:**
  - `Row::Clean`: if the composed world's goal cannot ask a catch-up
    before main advanced (the call names main's tip), build the goal's
    script after advancing main. If the engine makes no merge job for a
    catch-up with nothing to settle, that is a production fault: write
    the row first, failing, and stop with a `Question:` naming the step.
  - `Row::OtherBranch`: if the job's workstream holds no cached
    checkout (the job's checkout is a fresh clone, which imports every
    branch tip), make the job reuse one as production would (a second
    conflict on the same change, after a first job left its workspace
    cached), and if that cannot be had in the world, keep the row on a
    fresh clone, assert the preparation fetched the base by its id, and
    say so in the report.
  - `Row::PastTheBound`: the world's worker keeps production's
    `conflicts`; if the composed world lowers it, the row's file count
    follows the world's value plus one.
- **A production fault** a row shows: write the row first, failing, and
  commit it; a fix local to one function of temper's worker or engine
  crates, with a narrow test in its crate, is yours and said in the
  report; anything wider, or in jig, is a `Question:`.

## Steps

1. Switch the world to `perform_within`; run
   `heavy cargo nextest run -p temper-system-world --test suite -E 'test(=merge_job::a_changes_merge_job_runs_on_its_worker)'`.
2. Add each row, running the table alone after each.
3. Bite: call `perform` instead of `perform_within` and see
   `Row::PastTheBound` fail (a panic at the checkout's admission or a
   held job); make the clean row's command exit 1 and see it fail with a
   failure counted; restore both.
4. `cargo fmt`; `heavy cargo clippy -p temper-system-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper-system-world --test suite`; the same
   with `--profile fuzzy`.
5. Commit and push.

## Report

`Done:`, `Failed:` or `Question:` first. Then: each row's setup and what
it asserts; whether `Row::OtherBranch` ran on a cached checkout; any
production fault a row showed, where and how it was fixed; the bites;
implementation and check minutes apart.
