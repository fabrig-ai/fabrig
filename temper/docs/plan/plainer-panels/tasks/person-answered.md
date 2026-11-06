+++
id = "person-answered"
part = "person-answered"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-tools/src/jig/render.rs", "temper/crates/temper-client-tools/src/jig/render/document.rs", "temper/crates/temper-client-tools/src/jig/render/document/read_sets.rs", "temper/crates/temper-client-tools/src/jig/runner/reads.rs", "temper/crates/temper-client-tools/src/tests/jig/task_words.rs", "temper/tests/client/tests/suite/person_task_tools.rs"]
effort = "high"
minutes = 35
+++
# An answered person task says who answered

## Goal

`task show` on an answered person task prints `taken by: none`. Once an answer
has ended the task, it prints `answered by: PARTY` in that line's place. A
closing person task with no holder prints neither line.

Acceptance:

- The table in `temper/docs/plan/plainer-panels/notes/contracts.md`, under
  `The lines of task show`, holds for all four person-task rows, each by a test.
- `task show --json` prints what it prints today, `taken_by` included.
- `a_person_answers_a_delegated_choice_and_wakes_its_goal` shows the line
  through the real engine.

## Inputs

- `temper/docs/design/client/tools.md`, 5.5 (area 3, the paragraph on `task show
  TASK` and a person task's document).
- `temper/docs/plan/plainer-panels/notes/common.md` and `notes/contracts.md`.

## Owns and may change

Owns, in `temper/crates/temper-client-tools/src/`:

- `jig/render.rs`: a new helper beside `person_task`.
- `jig/render/document.rs`: `person_text` only.
- `jig/render/document/read_sets.rs`: the `person_text` call.
- `jig/runner/reads.rs`: `Reading`, its literal in `reads.rs` and `person_page`.
- `tests/jig/task_words.rs`: new tests.

May change `temper/tests/client/tests/suite/person_task_tools.rs`: the assertion
below, nothing else.

Options:

- The answerer's source is `EndedValue::PersonTask`'s `result.details.chosen_by`.
  If a real ended person task reaches `person_page` with no details or no
  `chosen_by`, print `taken by:` as today, and say so in the report.
- `encode_person_task` stays as it is.

## Steps

Sketch of the outer layer:

```rust
// jig/render.rs, beside person_task.
/// Who answered a retained person task, as its result says; a live document does not say.
pub(crate) fn person_answerer(value: &ReadValue) -> Option<u64>;

// jig/runner/reads.rs: Reading gains
pub person_answerer: Option<u64>,
// set in person_page beside reading.person.

// jig/render/document.rs
fn person_text(person: &PersonTask, answerer: Option<u64>, me: Option<&Me>, book: &Phrasebook, out: &mut Text);
```

`person_text` replaces its `taken by:` lines with this order, in the crate's
strict subset:

1. `answerer` is `Some(party)`: `\nanswered by: `, then `acts::party(party, me, out)`.
2. `taken_by` is `Some(party)`: `\ntaken by: `, then the party, as today.
3. `person.task.phase` is `Closing`: nothing.
4. Otherwise: `\ntaken by: none`.

Tests in `tests/jig/task_words.rs`. Serve `f::TASK_CALL` a task page whose
`person_revision` is `Some`, then the person page it reads next, as
`temper/crates/temper-client-tools/src/tests/jig/replays.rs` builds an ended
person document from `f::person_document()`. No fixture has a `c::ResultDetails`,
so build one in the test file:

- `an_answered_person_task_names_who_answered`: ended with `chosen_by`
  `Some(f::PARTY)` prints `answered by: you` and no `taken by:`; with another
  party, `answered by:` and that party's words.
- `a_closing_person_task_with_no_holder_says_nothing_of_who_took_it`: phase
  `Closing`, `taken_by` `None`, prints neither line.
- `a_person_task_nobody_answered_keeps_its_holder_line`: live with a holder, live
  with none, and ended with no `chosen_by`.
- `an_answered_person_tasks_json_keeps_taken_by`: `f::TASK_JSON_CALL` still prints
  `"taken_by":null`.

In `person_task_tools.rs`, at the end of `answer_choice`, after `completed(ctl,
task)`, which proves the task ended:

```rust
let shown = ctl.text(&[b"task", b"show", number.as_bytes()]);
assert!(shown.contains("answered by: you"), "answered task: {shown}");
assert!(!shown.contains("taken by:"), "answered task: {shown}");
```

Run:

```
heavy cargo nextest run -p temper-client-tools --lib -E 'test(answered) | test(closing_person) | test(nobody_answered)'
heavy cargo nextest run -p temper-client-world --test suite -E 'test(a_person_answers_a_delegated_choice_and_wakes_its_goal)'
```

Then the checks your run's text gives, with `cargo fmt` and clippy on both
packages. Commit and push.

## Report

As your run's text says. Name what `person_page` received in the real story.
