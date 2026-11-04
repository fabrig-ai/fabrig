+++
id = "kv-snapshot-file"
part = "kv-review"
kind = "feature"
needs = ["kv-charges"]
files = ["skein/crates/skein-kv/src/store.rs", "skein/crates/skein-kv/src/store/compaction.rs", "skein/tests/kv/tests/suite/store.rs"]
effort = "high"
minutes = 60
+++

# skein-kv: a snapshot's file is closed on every way out

## Goal

The review's Part A, finding 2. A failed log write, sync or rollover close during a snapshot reaches
`recover`, which sets `compaction = Idle` and drops the `Header`, `Chunk`, `Trailer`, `Sync` or `Close`
state with no `Close` for its file. That breaks kv.md 4's "settles its I/O" and 5.4's rule on handles.

**The cut read one way further than the review names:** a failed rollover create or directory sync
reaches `open_failed` (`Phase::Creating`, `Phase::Announcing`; `kv.status.md`, 4), which drops `dir` and
its `Compaction` the same way. This task closes the file on both. It does not answer those commits:
that departure stays.

Acceptance:

- Tests in `skein/tests/kv/tests/suite/store.rs` fail on today's code and pass after it, each asserting
  `world.io.open_files() == 0` at the end: a failed write and a failed sync at each snapshot stage that
  holds the file, and a failed rollover create and directory sync with the file held.
- `compaction::failed`'s or-pattern over the states holding a file is `Compaction::file`, one list.
- `skein/tests/kv`'s other tests, its golden files and `kv.status.md` do not change.
- The note's `What a task runs` passes.

## Inputs

- `temper/docs/plan/debt-6/notes/common.md`, `temper/docs/plan/debt-6/notes/skein-kv.md`, `AGENTS.md`,
  `skein/AGENTS.md`.
- `skein/docs/foundation/programming-model.md`, 5.4; `skein/docs/design/kv.md`, 4, 5 and 7;
  `skein/docs/design/kv.status.md`, 4.

## Owns and may change

- Owns: `skein-kv/src/store.rs`'s `commits_failed`, `recovery_closed`, `open_failed` and its call sites,
  `src/store/compaction.rs`'s `Compaction` and `failed`; `skein/tests/kv/tests/suite/store.rs`, where it
  adds tests and changes none.
- `open_failed`'s callers pass `step` where they pass `step.above`: call sites only.
- `Phase::RolloverClosing`'s `Failed` cannot be reached by a fault: the sim lets a `Close` go at once and
  never fails it. Its arm reaches the same function as the others; leave it untested and say so.
- If a stage does not hold the file when a commit can be started against it, leave it out of the sweep
  and name it in the report.

## Steps

1. Write the tests first and see them fail. `commits_race_fuzzy_snapshot_chunks` shows how: `limits(32)`
   and `fire_once` walk a snapshot a step at a time, and `failed_sync_reports_uncertain_commit_then_recovers`
   shows how a fault lands on one step.

   ```rust
   // A snapshot `fires` steps along, its file open, in `Phase::Ready`.
   fn snapshot_at(limits: Limits, fires: u32) -> World;

   #[test]
   fn a_commit_failing_mid_snapshot_closes_the_snapshot_s_file() {
       // For each stage and for a failed write and a failed sync: Failed, Recovered, Close, Closed,
       // then open_files() == 0. The commit's Failed and the Recovered that follows stay as they are.
   }

   #[test]
   fn a_rollover_failing_mid_snapshot_closes_the_snapshot_s_file() {
       // A small limits.segment makes a second commit roll over with the snapshot running. For a
       // failed create and a failed directory sync, the store ends, and open_files() == 0.
   }
   ```

2. `Compaction::file`:

   ```rust
   /// The snapshot's open file, in each state that holds one.
   pub(super) fn file(&self) -> Option<Token>;
   ```

   `failed` asks it first, and sends what it returns to `failure_closing`; its match keeps `Create`, the
   states with no file, and `Idle | Due`.
3. `recovery_closed` closes that file before it recovers, one request at a time:

   ```rust
   fn recovery_closed(store: &mut Store, mut dir: Dir, step: &mut Step<'_>) -> State {
       match mem::replace(&mut dir.compaction, Compaction::Idle).file() {
           Some(file) => {
               step.below.push(file::Request::Close { owner: IO_OWNER, file });
               dir.at(Phase::RecoveryClosing)
           }
           None => store.recover(dir, step.below, &step.env.limits),
       }
   }
   ```

   `commits_failed`'s `None` arm calls it instead of `recover`. `open_failed` does the same through
   `open_failed_file`, so `OpenFailureClosing`'s answer comes back to it with `compaction` `Idle`.
4. What the note's `What a task runs` names; the citations check. Commit and push.

## Report

The common note's table: each new test's failure on today's code. Which stages the sweep holds. The
note's line counts; this task may add about 12 lines of code.
