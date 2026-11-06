+++
id = "held-waiting"
part = "held-waiting"
kind = "feature"
needs = ["hold-word", "person-answered"]
files = ["temper/crates/temper-client-tools/src/jig/render/document.rs", "temper/crates/temper-client-tools/src/jig/render/holds.rs", "temper/crates/temper-client-tools/src/tests/jig/task_words.rs"]
effort = "high"
minutes = 25
+++
# A held task says it's held

## Goal

`task show` on a held task that awaits nothing else prints `waiting on: Nothing
awaited` beside its hold, which reads as if nothing were wrong. It should say what
it waits on: its hold.

Which form it prints is **not yours to choose**: see step 1. This departs from
your run's aim of no questions, because the request asks for it.

Acceptance:

- The `waiting on:` table in `temper/docs/plan/plainer-panels/notes/contracts.md`
  holds for all three rows, each by a test.
- The form printed is the one the user's answer names.
- `task show --json` prints what it prints today, `waits_on` included.

## Inputs

- `temper/docs/design/client/tools.md`, 6.1 (`waiting on:`) and 15 (its open
  question).
- `jig/docs/design/client/domain/client.md`, 19 (`words`).
- `temper/docs/plan/plainer-panels/inputs/request.md`, `Left open`.
- `temper/docs/plan/plainer-panels/notes/common.md` and `notes/contracts.md`.

## Owns and may change

Owns, in `temper/crates/temper-client-tools/src/`:

- `jig/render/document.rs`: `waiting` and its call in `task_with_specification`.
- `jig/render/holds.rs`: `reason`'s visibility, only if the answer is `held: REASON`.
- `tests/jig/task_words.rs`: new tests.

Options:

- A golden of a held task's `task show` that changes only on its `waiting on:`
  line follows from this change: regenerate it with its own tool and list it. A
  search of `tests/golden/` finds `^held:` only in `landing_held.txt`, so none is
  expected.
- `jig-client-view` and `temper-client-view` print their own wait: leave them.

## Steps

1. **Ask, before you write any code,** unless your run already carries the user's
   answer. Write your report with this as its first line, and stop:

   ```
   Question: What should `waiting on:` print for a held task that awaits nothing else: `held` alone, or `held: REASON`, which may be long on a narrow screen? temper/docs/plan/plainer-panels/inputs/request.md, Left open, gives this to the person who wrote the draft, so it goes to the user, and an answer call does not settle it. Recommendation: `held` alone, because the same panel's `held:` and `way out:` lines already print the reason and what to do.
   ```

   An answer that tells you to choose is no answer: ask again. An answer naming a
   third form that `WaitingWord::Held` can print is the answer. One that needs
   another word in jig is a `Question:` to the same person.

2. Build the form answered. Sketch:

   ```rust
   // jig/render/document.rs: waiting takes the task, for its hold.
   fn waiting(task: &Task, vocabulary: &dyn crate::jig::render::Parts, me: Option<&Me>, book: &Phrasebook, out: &mut Text);
   ```

   `WaitsOn::Party` prints as today. `Dependencies`, `Delegates` and `News` print
   `Word::waiting` as today. `Nothing` looks at `task.hold`: with `None`,
   `Word::waiting` as today; with `Some`, `Word::Waiting(WaitingWord::Held)`.

   - **`held`:** the word alone. Drop `vocabulary` from the sketch.
   - **`held: REASON`:** the word, `: `, then `super::holds`' `reason`, which the
     `held:` line prints, made `pub(crate)`.

3. Tests in `tests/jig/task_words.rs`, beside
   `a_held_tasks_text_names_its_reason_and_release_revision`, with `f::held_task`:

   - `a_held_task_that_awaits_nothing_else_waits_on_its_hold`:
     `f::held_task(f::goal(f::TASK, c::WaitsOn::Nothing), c::HoldCategory::Effects)`
     prints the form answered on its `waiting on:` line, whole, and no `Nothing
     awaited`.
   - `a_held_task_a_delegate_keeps_prints_its_delegates`: `c::WaitsOn::Delegates`
     prints `waiting on: Waiting for delegates`. Do the same for
     `f::waiting_on_me()`, a party's wait.
   - `a_task_with_no_hold_keeps_nothing_awaited`: `c::WaitsOn::Nothing` and no hold
     prints `waiting on: Nothing awaited`.
   - `a_held_tasks_json_keeps_its_wait`: `f::TASK_JSON_CALL` prints the same
     `waits_on` with the change as a task with no hold does.

Run:

```
heavy cargo nextest run -p temper-client-tools --lib -E 'test(waits_on_its_hold) | test(keeps_prints) | test(keeps_nothing_awaited) | test(keeps_its_wait)'
```

Then the checks your run's text gives. Commit and push.

## Report

As your run's text says. Quote the answer you built, and who your run says gave it.
