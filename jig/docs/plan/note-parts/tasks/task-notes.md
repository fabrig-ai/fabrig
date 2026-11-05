+++
id = "task-notes"
part = "task-notes"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-tools/src/jig/rows.rs",
  "temper/crates/temper-client-tools/src/jig/runner/reads.rs",
  "temper/crates/temper-client-tools/src/jig/render.rs",
  "temper/crates/temper-client-tools/src/jig/render/**",
  "temper/crates/temper-client-tools/src/tests/jig/**",
  "temper/docs/help/reference/ctl-task.md",
]
effort = "high"
minutes = 60
+++

# `temper ctl task notes` reads a task's notes whole

## Goal

ctl can't read a note today. `task notes` makes the read a chat's document pane makes, and prints each note whole, so that a walk through
ctl, and a person at a terminal, see the document a run built.

Acceptance:

- `task notes TASK --project PROJECT [--after CURSOR] [--json]`, as `notes/contracts.md`, *`task notes`*, and
  `temper/docs/design/client/tools.md`, 5.2.
- A body at `LONG_TEXT`'s bound prints whole, in text and in JSON: nothing cuts it at `PART` or `TEXT`.
- `temper/docs/help/reference/ctl-task.md` regenerated: the test `the_task_chat_and_inbox_references_are_what_the_table_renders` holds it.
- `temper/docs/design/client/tools.status.md`: the entry `5.2: task notes` goes.

## Inputs

- `jig/docs/plan/note-parts/notes/common.md`, `jig/docs/plan/note-parts/notes/contracts.md`.
- `temper/docs/design/client/tools.md`, 5.1 and 5.2; jig's `jig/docs/design/client/domain/contract.md`, section 6, *Notes are read by scope*.
- As patterns: `rows.rs`' `TURN` and `TASK_TEXT_AT`; `runner/reads.rs`' arm for `rows::TASK_TEXT_AT`; `temper-client-domain/src/pages/report.rs`, `read`, for the read's form.

## Owns and may change

Owns, as `files` lists them.

May also change: `temper/crates/temper-client-tools/src/jig/json.rs`, a name of `Kind` or a record's; `temper/crates/temper-client-tools/src/jig/settings_adapter.rs` and `render/document/proposal.rs`, only if a `ReadValue::Notes` arm must move; `temper/crates/temper-client-tools/src/tests/help.rs`, a listed tool's name.

Options, each pre-approved:

- **The table:** arms in `reads.rs` go by a row's index. Put the new row where its neighbours' indexes stay, or name it `rows::TASK_NOTES_AT` as `TASK_TEXT_AT` is named; if `help` prints a group by its table order and the row reads wrong at the end, say so in your report and leave it.
- **`TASK`'s project:** `--project` is required, as `task list`'s, by `Kind::Project { empty: false }` and `reads.rs`' `project`.
- **A refused read** prints as `task text`'s does. An unknown task's or an empty scope's answer is an empty page: `no notes`.

## Steps

1. The row, its read, its text and its `notes` record.
2. Tests in `temper-client-tools/src/tests/jig/`: `task_notes_asks_the_panes_read`, `task_notes_prints_each_note_whole`,
   `task_notes_prints_a_body_past_a_part_whole`, `task_notes_says_a_cut_body`, `task_notes_says_how_to_read_on`, `task_notes_says_no_notes`,
   and the JSON record of one note, each having bitten. Use the module's existing front or fixtures for a read's answer.
3. `heavy cargo nextest run -p temper-client-tools`; `heavy cargo check --tests -p temper`.
4. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report.
