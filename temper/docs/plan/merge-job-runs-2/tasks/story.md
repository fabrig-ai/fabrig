+++
id = "story"
part = "story"
kind = "feature"
needs = []
files = ["temper/tests/system/**", "temper/crates/temper-service/src/limits.rs", "temper/crates/temper-service/src/assignment.rs", "temper/crates/temper-service/src/route.rs", "temper/crates/temper-service/src/worker_route.rs"]
effort = "high"
minutes = 180
+++

## Goal

Make the composed system world run a change's merge job from end to end
(`temper/docs/design/domain/testing.md`, 2.1, the merge job's story): the
engine root makes the job, the worker root that declared the kind as
production does takes it through production's translation of a job,
prepares the head with the base merged in on the fake disk, runs the
world's command, delivers or not, and the engine goes on from the branch.
This is the meeting point of the plan: the lowest tier that holds the
engine, a worker over the link and the merge job. Its first two rows are
a conflict the command settles and one it leaves.

Acceptance:

- `temper/tests/system/tests/suite/merge_job.rs`,
  `a_changes_merge_job_runs_on_its_worker`: one table over `Row::Settled`
  and `Row::Left` (`notes/contracts.md`, section 3), each row built by
  `World::merge_job(seed, row)` and run by `run_merge_job`, its
  expectations a column of the table.
- Both rows, from the report:
  - the hello the engine took declared kind 3 with spec 4,096, result
    512 and deadline 600 s, the worker root built with
    `temper_service::limits::job_kinds()`, and the engine's declaration
    `temper_service::limits::merge_jobs` of the world's jig limits;
  - the spawn's program is the checkout's `<repository>/.temper/merge`
    under the job's directory, and that file is on the fake disk with the
    repository's content (`command_found`); its arguments are tooling.md
    2's (the program, `merge`, the request file's path); the request
    file holds `{"version":1,"head":"HEAD","base":"BASE"}` exactly,
    lowercase hexadecimal, as tooling.md 4.1 says;
  - the job's items and spec crossed by `assignment::job_down` and were
    decoded by `worker_route::job_items` and `job_spec` with the payload
    limits a production worker derives,
    `temper_service::worker_settings::payload_limits` of the world
    worker's limits, not `v1::CEILINGS`;
  - the job's task ended finished, with no failure counted.
- `Row::Settled`: the change's update conflicts with its base in one
  file; the command sees that file marked and `MERGE_HEAD` at the base,
  writes the settled content, stages it, writes `{"settled":["FILE"]}`
  and exits 0. The change's branch on the fake forge moves to a commit
  whose parents are the change's head and the base, whose tree holds the
  settled file; no resolution is made, and the change goes on from that
  head as from a resolution's push (`forge.md`, 8.4).
- `Row::Left`: the same conflict; the command writes
  `{"settled":[],"left":["FILE"]}` and exits 3. The branch does not
  move; the change makes a resolution of the same base whose words carry
  the job's result (`resolution` in the report). The resolution need not
  run.
- If a row fails for a fault in production code rather than the world's,
  see *Owns and may change*.

## Inputs

- `temper/docs/plan/merge-job-runs-2/notes/common.md`;
  `temper/docs/plan/merge-job-runs-2/notes/contracts.md`, sections 1 to 3.
- `temper/docs/design/domain/testing.md`, 2.1: the composed world and its
  merge job's story (the paragraph dated by this plan).
- `temper/docs/design/domain/worker.md`, 4.4 (the merge kind, the spec, the
  delivery); `temper/docs/design/domain/forge.md`, 8.4, *The merge job*;
  `temper/docs/design/protocol/tooling.md`, 2 and 4.1.
- `temper/tests/system/README.md`: what the composed world is and plays.
- Patterns to follow:
  - a goal whose two changes land on its branch, the second moving it
    under the first: `temper/tests/system/src/composed/parked.rs`
    (`parked_edits`, `continue_parked_edits`, and
    `advance_goal_for_producer` in `composed.rs`). The first change's
    producer writes FILE; the second lands a different FILE on the goal,
    so the first change's update conflicts;
  - the engine's merge kind and authority in a world's configuration:
    `temper/tests/engine/domain/src/forge_routes.rs`, the `merge_kind`
    and `merge_authority` blocks (lines 801 to 820), and `with_merge_job`
    (line 1237);
  - a scripted merge command on the fake disk, its result read and its
    process gone: `temper/tests/worker/domain/src/whole/merge.rs`,
    `command`;
  - production's job translation: `temper-service/src/route.rs`,
    `job_assignment`, and its two tests in
    `temper-service/src/tests/workers.rs` (734 to 783);
  - the composed world's link translation of a run:
    `temper/tests/system/src/composed/link.rs`, `assignment` and `parts`.
- Where the world refuses jobs today: `temper/tests/system/src/composed.rs`,
  `Delivery::AssignedJob` among the panicking deliveries (around line
  1038), `worker::Request::Job` among the panicking requests (around line
  1203), and `worker::Domain::new(.., Box::new([]))` (lines 260 and 463).

## Owns and may change

- **Owns:** `temper/tests/system/**`: `composed.rs` (the job's delivery,
  the job's operations, the worker's kinds, the engine's configuration),
  `composed/link.rs` (a job's translation), the new
  `composed/merge_job.rs` and its `mod` line, the new
  `tests/suite/merge_job.rs` and its `mod` line in `tests/suite/main.rs`,
  and `README.md`'s list of focused cases (one sentence for this story).
- **temper-service, the surfaces of `notes/contracts.md`, sections 1 and
  2 alone:** `limits.rs` (`merge_bound` and `merge_jobs` made `pub`,
  `job_kinds` added under the `worker` feature), `worker.rs`'s
  `worker_root` alone (calls `job_kinds`; task `lines` edits other parts
  of `worker.rs`), `assignment.rs` (`job_down`), `route.rs`'s
  `job_assignment` alone (calls `job_down`), `worker_route.rs`'s
  `job_items` and `job_spec` made `pub`. No behaviour of the service
  changes; its existing tests hold it.
- **The engine's configuration in the world** follows production:
  `config.core.jobs` from `merge_jobs`, the project's landing naming
  `merge_job = Some(b".temper/merge")`, and `Executor::Job(3)` in the
  project's delegation kinds as the engine world's `merge_authority`
  block puts it. If the world's jig limits do not hold the merge bound
  (spec 4,096, result 512), raise the world's own limits to the small
  profile's, never production's.
- **The repository's command:** the world's repository holds
  `.temper/merge` (any content; the world's command stands for it). The
  world serves the worker's `Request::Job` operations itself, as the
  worker world's `whole/jobs.rs` does: `Write` to the fake disk,
  `Spawn` checked and answered by the row's command (which edits and
  stages through the fake disk and the fake's index as `whole/merge.rs`
  does, then `Exited`), `Read` from the fake disk, `Kill` and `Gone`.
  A spawn whose program is not on the fake disk exits as a failed spawn
  does in production (`Exit::Signal`, then `Gone`), so a wrong path is a
  red row, never a panic.
- **Options:** if the engine's procedure makes no merge job for this
  conflict (for instance the authority or kind does not reach the
  change), find why in the world's configuration first; if production
  refuses what the design says it makes, stop with a `Question:` naming
  the step. If a row fails for a fault in production code (temper's
  worker or engine crates) and the fix is local to one function, write
  the row first, failing, commit it, then fix it with a narrow test of
  its own in the crate that holds it, and say so in the report; a fix
  wider than one function, or in jig, is a `Question:`. A wrong path
  for the command, a spec or result the worker refuses, a hello without
  the kind or a transient failure counted on the job are such faults.
- **Neighbours, compiler-forced only:** other `temper/tests/system`
  stories that build the world's configuration or worker.

## Steps

1. Read the inputs; run
   `heavy cargo nextest run -p temper-system-world --test suite -E 'test(=parked::a_parked_producers_edits_survive_a_clean_merge_and_a_contained_resume)'`
   once to see the goal's two changes run.
2. Add contracts 1 and 2 in temper-service; run
   `heavy cargo nextest run -p temper-service --lib -E 'test(=tests::workers::a_jobs_forge_workspace_becomes_worker_items) | test(=worker::merge_tests::the_production_worker_declares_the_merge_kind)'`
   (check the count: 2).
3. Build the world: both roots declare the kind, `Delivery::AssignedJob`
   crosses through `job_down` and `job_items`/`job_spec`, the worker's job
   operations are served, the job's answer crosses back with
   `link::parts`. Write `Row::Settled`, run it alone, then `Row::Left`.
4. `cargo fmt`; `heavy cargo clippy -p temper-system-world -p temper-service --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper-system-world --test suite`;
   `heavy cargo nextest run -p temper-system-world --test suite --profile fuzzy`;
   `heavy cargo nextest run -p temper-service --lib`.
5. Bite: remove the kind from the worker's root (`Box::new([])`) and see
   both rows fail for no job placed; restore. Point the world's command
   at a path without the repository's name and see the row fail on
   `command_found`; restore.
6. Commit and push.

## Report

`Done:`, `Failed:` or `Question:` first. Then: what crossed and how the
world plays each side; what each row asserts; any production fault found,
where (file and line) and what fixed it; what the world departs from
production in (the command, the disk); the bites and what failed;
implementation and check minutes apart.
