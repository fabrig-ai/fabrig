+++
id = "merge-kind"
part = "merge-kind"
kind = "feature"
needs = []
files = ["temper/crates/temper-service/src/limits.rs", "temper/crates/temper-service/src/worker.rs", "temper/crates/temper-service/src/system.rs", "temper/crates/temper-service/src/startup.rs", "temper/crates/temper-service/src/startup/checks.rs", "temper/tests/worker/domain/**"]
effort = "high"
minutes = 120
+++

## Goal

temper's first job kind: the worker declares kind 3, `merge`, delivering, and the engine
declares its bound to jig's core (`notes/contracts.md`, C1). A merge job runs the project's
merge command in a merge in progress and, on exit 0, the worker commits the merge with both
parents and pushes it; on exit 3 it delivers nothing.

Acceptance:

- the production worker's hello states kind 3 with C1's bounds, and the engine's core
  configuration declares the same `JobBound`;
- in the worker's world, with the real root, hub, capability and checkout: a merge job whose
  item starts as a merge in progress (`wire::Start::Merge { branch, base }`, the two sides
  conflicting in one file), whose command settles that file and stages it, answers `Finished`
  with the command's result, after one push whose commit has the head and the base as its
  parents; a job whose command exits 3 answers `Finished` with its result and pushes nothing;
  a job whose command exits 0 but leaves the file marked answers `Failed(Undelivered(_))` and
  pushes nothing.

## Inputs

- `temper/docs/design/domain/worker.md, 4.4` (the kinds' table, "The merge kind"), section 5
  (a merge in progress); `temper/docs/design/protocol/tooling.md, 4.1`.
- `temper/docs/plan/reference-m7b/notes/contracts.md`, C1; `notes/common.md`.

## Owns and may change

- `temper/crates/temper-service/src/limits.rs`: C1's constants and `merge_bound()`.
- `temper/crates/temper-service/src/worker.rs`: the root's kinds, `[tooling::Kind {
  bound: merge_bound(), verb: b"merge", delivers: true }]` in place of `Box::new([])`.
- `temper/crates/temper-service/src/system.rs` and `startup/checks.rs`: the core's `jobs` set to
  `[merge_bound()]` after `jig_protocol_config::core(..)` builds it, in both places.
- `temper/tests/worker/domain/**`: the merge stories, in a new `src/whole/merge.rs` beside
  `whole/jobs.rs` (reusing its world where it can: its kind is a parameter today) and their
  tests in `tests/suite/jobs.rs`, or a `tests/suite/merge.rs` registered in the suite's
  `main.rs`.
- Readers: a temper-service test that asserts the hello's kinds are empty, or the core's jobs
  empty, now expects kind 3; change the expectation only. If one of the service's other worlds
  (`temper/tests/*`) asserts no kinds, the same.
- **If the checkout refuses a job's `Start::Merge` item, or the delivery does not commit with
  both parents** (`temper-worker-domain/src/workspace.rs`, `stage_job`;
  `temper-worker-domain-checkout`'s `push_from`): fix it in `temper-worker-domain`'s job path
  so a job's merge in progress is a run's, and say so in the report. Changing
  `temper-worker-domain-checkout` beyond that is a question.

## Steps

1. The constants, the worker's kind, the engine's declaration.
2. The world stories, each citing `temper/docs/design/domain/worker.md, 4.4`:
   - `a_merge_job_settles_its_conflict_and_pushes_a_merge_with_both_parents`;
   - `a_merge_job_that_leaves_conflicts_pushes_nothing`;
   - `a_merge_job_that_leaves_a_file_marked_fails_undelivered`;
   - `the_production_worker_declares_the_merge_kind` (a temper-service lib test, where the
     worker's root is built, or the nearest place that reads its hello's kinds).
   The fake command is the world's scripted tooling operation, as `whole/jobs.rs` scripts
   `check`: it writes the settled file on the fake disk and stages it, then exits.
3. `heavy cargo nextest run -p temper-worker-domain-world -E 'test(/merge_job/)'` (the
   package's name is in `temper/tests/worker/domain/Cargo.toml`), then the package's tests and
   `-p temper-service --lib`; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-service -p <the world's package> --all-targets -- -D warnings`.
4. Commit and push.

## Report

As the implementer's text says; say whether the job's merge in progress needed any change in
the worker.

## Answers

2026-11-03. **Extend the task to align temper's task byte limits, startup checks and fixtures
with C1, or a separate task?** Neither: C1's sizes were wrong, and are revised
(`notes/contracts.md`, R2; `temper/docs/design/domain/worker.md, 4.4`). Raising
`tasks.result_bytes` past 512 is jig's settings schema, a stored format, and not this plan's.
So: `MERGE_SPEC_BYTES` is 4,096 and `MERGE_RESULT_BYTES` 512 (the small profile's tasks child);
the worker's hello states `merge_bound()` as it does now; the engine's core declares
`[merge_bound()]` only where the jig limits it is built with hold it (`spec <= tasks.spec_bytes`,
`result <= tasks.result_bytes`), else no jobs, through one helper in `limits.rs` that
`system.rs` and `startup.rs` both call (`notes/common.md`, "Corrections found after release";
`startup.rs` added to `files`). No limit, profile, startup check or service fixture changes:
the service's fixtures (128 and 32) declare no kind 3, so their tests pass
unchanged. Acceptance's "the engine's core configuration declares the same `JobBound`" holds on
the small profile's own limits: add a temper-service lib test
`the_engine_declares_the_merge_kind_where_its_tasks_child_holds_it` beside the hello's test,
the helper giving `[merge_bound()]` for the small profile's limits and nothing for limits with
`tasks.result_bytes` 32. Keep the worker world's merge stories within the 512-byte result.
