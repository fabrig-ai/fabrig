+++
id = "own-question-parks"
part = "asker-finish"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-tasks/src/run/ends.rs", "jig/crates/jig-domain-tasks/src/run.rs", "jig/crates/jig-domain-tasks/src/admission.rs", "jig/crates/jig-domain-tasks/src/tests/kept.rs", "jig/tests/tasks/tests/suite/questions.rs", "jig/tests/core/tests/stories/kept.rs"]
effort = "high"
minutes = 40
+++

## Goal

A run that asked its requester a question and then calls `finish` with form `failure`
ends its task today: the question reaches no one. A done finish already parks. Make a
run's own finish park whatever its form, while its task's own question is open. No
try is spent, and the answer wakes the next run, which can finish.

Acceptance:

- A task with its own question open: `End::Finished` with `TaskResult::Failure` parks,
  `tries` none, its `Line::Kept` names the question, no activation.
- The answer wakes it, and its next run's finish, failure included, ends it.
- As ever: `End::Failed(_)`, `End::Exceeded(_)` and a cancel end or count as they do
  today; a procedure's `ProcedureEnd::Result(Failure)` ends a procedure kept by its own
  question (a change's); a failure under a pending dependency or an owed question ends.

## Inputs

- `jig/docs/design/domain/tasks.md`, 5.7, *Its own question keeps it too*.
- `jig/docs/design/domain/engine.md`, 7.4, *Finished done while kept, it parks*.
- `temper/docs/plan/producer-failure/notes/common.md`.

## Owns and may change

Owns the `files` above. No other file of jig changes, and no store or wire type.

This branch stays in jig. It changes what a shared rule means, and two temper tests
end an asker with a finished failure:
`temper/tests/engine/domain/tests/suite/forge_routes/questions.rs`,
`an_asker_that_ends_drops_its_question`, and `.../foremen.rs`,
`an_ended_asker_cancels_its_foreman`. They go red here and `asker-left` owns them. List
them under `## Left`; any other temper red is a question.

Options: the method's name inside `ends.rs` is yours. If the admission and the commit
can share one call, share it.

## Steps

Sketch, `run/ends.rs`, beside `parks`, which stays as it is because `procedure.rs` reads it:

```rust
/// A run's own finish (5.7): as `parks`, and kept by its own question a failure parks too.
pub(crate) fn run_parks(self, end: &End) -> bool {
    self.parks(end) || (self.question.is_some() && finished(end))
}
```

`finished` lists every `End` variant: `Finished` and `FinishedWithFollowups` true, the
rest false. `run.rs`, `let parked = kept.parks(&end);`, and `admission.rs`,
`valid && kept.parks(&end)`, call `run_parks`. A `Failure` past its bytes still becomes
`End::Failed(Class::Invalid)` first, in `run.rs`'s order.

Read `smith/crates/smith-domain-session/src/session/restore.rs` and confirm a window
ending in an accepted `finish` resumes as it does for a done finish. A failure's turn
should not differ. If it does, stop and ask.

Tests, failing first:

1. `jig/tests/core/tests/stories/kept.rs`, beside
   `a_requester_question_parks_the_run_then_its_answer_resumes_it`, the story with a
   failure: `a_failure_finished_under_its_question_parks_and_the_answer_resumes_it`.
   Resumed, it finishes and ends.
2. `jig/tests/tasks/tests/suite/questions.rs`: `a_question_keep_permits_failure` becomes
   `a_question_keep_parks_a_finished_failure`, asserting as
   `a_task_with_its_own_question_unanswered_parks_on_finish` does; after the answer, a
   finished failure ends it `Ending::Failed`. Add
   `a_question_keep_lets_a_failing_run_count`: `End::Failed(Class::Run)` counts its try.
3. `jig/crates/jig-domain-tasks/src/tests/kept.rs`: rows for the new method: question
   and failure park; dependencies alone, or owing alone, and failure do not; `parks`
   with a question and failure is still false, for the procedure.

Each new test bites: remove the rule once, see it fail, restore.

Then `cargo fmt`, clippy on `jig-domain-tasks`, and the readers: `jig-domain-tasks`,
`jig-tasks-world` and `jig-core-world` under `default` and `--profile fuzzy` for the
worlds. Run `heavy cargo nextest run -p temper-engine-domain-world --test suite -E
'test(forge_routes::questions) | test(forge_routes::foremen)'` once to name what goes
red. Commit, push.

## Report

What the resume check found; the temper tests that went red, by name.
