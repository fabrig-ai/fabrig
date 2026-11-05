+++
id = "note-scope"
part = "note-scope"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-domain-core/src/routing/mod.rs",
  "jig/crates/jig-domain-core/src/routing/run/preparation.rs",
  "jig/crates/jig-domain-core/src/admission/goal_of.rs",
  "jig/crates/jig-domain-core/src/tests/admission/goal_of.rs",
  "jig/crates/jig-domain-core/src/runs/brief_text.rs",
  "jig/crates/jig-domain-core/src/tests/runs/brief_revisions.rs",
  "jig/crates/jig-client/src/shapes.rs",
  "jig/crates/jig-client/src/tests/shapes.rs",
  "jig/tests/core/tests/suite/stories.rs",
  "jig/docs/design/domain/authority.status.md",
]
effort = "high"
minutes = 60
+++

# A chat's own note scope

## Goal

A root conversation's run writes and reads the goal scope named by its own task number, and its brief lists that scope
first. No other run can, whatever its authority.

Acceptance:

- `Core::note_goal(&self, row: &tasks::TaskRecord) -> Option<u64>`, beside `goal_of` in `admission/goal_of.rs`: `goal_of(row)`
  if `Some`; else `Some(row.number)` when `row.flags.conversation` and the requester is no task (a root conversation); else
  `None`.
- `routing/mod.rs`, `ToolKind::Note`: a `notes::Scope::Goal { project, goal }` is `authority::NoteScope::Goal` when `project
  == task.project` and `Some(goal) == note_goal(task)`, and refused as `Finding::Scope` as today otherwise. The authority
  check that follows is unchanged: its three holders must each hold `Scopes::GOAL`.
- `note_readable` reads the same way.
- `routing/run/preparation.rs`' brief scopes push `notes::Scope::Goal` from `note_goal`, first, as today.
- The stored format is unchanged: the client's `NoteScope::Task { project, task }` already maps here.
- **A run knows its number:** `runs/brief_text.rs`, `task_read`: after `[Requested by …]` (the words before it stay first, since a fake model's cue reads `## Task\n\nWORDS`), a part
  `[You are task T in project P]\n`, cut as its neighbours are. `RunContext` holds the project; if it lacks the number, carry it. A change of meaning: run the readers of the Task section,
  `jig-domain-core`, `temper-engine-smith`, and `rg -n 'Requested by'` for those a line after it moves; regenerate what pins it: `temper-service`'s `stock_brief.rs` holds its figures by
  arithmetic with a margin of 567, which this takes from.
- **A person's goal has no note scopes:** `jig-client/src/shapes.rs`, `fill`: a template not flagged conversation is filled with `authority.notes` zero and no `note_resources`, whatever
  `creating.within.authority` holds; a conversation's keeps them.
- A party's read of that scope needs the party to see the project: confirm it in `jig-domain-core/src/reads.rs` and pin it.
- A story in `jig/tests/core/tests/suite/stories.rs`, on the shape of
  `a_note_written_corrected_by_a_person_and_recalled_keeps_the_corrected_body`, with `Scopes::GOAL` in the rules, ceiling and
  role: `a_chat_writes_its_own_scope_and_a_party_reads_it`: a chat writes its own scope; a party of the project reads it with
  `Read::ScopedNotes`; a party with no role there doesn't (unknown); the brief of its next run lists it first.
- Tests, in the same story file or `tests/admission/goal_of.rs`: `a_goals_task_cannot_write_a_chats_scope`, a goal's task with
  the chat's number; `a_goal_writes_only_its_own`; `a_task_of_another_project_cannot_write_it`; `a_delegate_of_a_chat_has_no_scope_of_its_own`,
  a conversation-flagged task with a task as its requester, naming its own number; `a_task_not_flagged_conversation_has_none`. `jig-client`'s
  `tests/shapes.rs`: `a_goal_is_filled_with_no_note_scopes`, `a_chat_keeps_its_note_scopes`. `brief_text`'s: `a_brief_names_its_task_and_project`.
- `jig/docs/design/domain/authority.status.md`'s entry for 3 goes, and `engine.status.md`'s line "10: a chat's own scope.".

## Inputs

- `temper/docs/plan/ui-chat-draft/notes/common.md`.
- `temper/docs/plan/ui-chat-draft/inputs/request.md`: *The engine*, *jig*, *A chat's own scope*.
- `jig/docs/design/domain/engine.md`, section 10; `jig/docs/design/domain/authority.md`, section 3.

## Owns and may change

Owns, as `files` lists them. May also change `jig/crates/jig-domain-core/src/tests/**` where a fixture needs the flag, the tests a brief's new line moves (`temper/tests/**`,
`temper/crates/temper-fake/src/tests/**`, `temper/crates/temper-playground/src/tests/**`, `temper/crates/temper-service/src/tests/stock_brief.rs`,
`temper/tests/playground`'s kept log by its `SKEIN_CODEGEN_WRITE`), and
`jig/docs/design/domain/engine.status.md`, that line alone; `jig/docs/design/client/domain/client.status.md`'s 19 entry, its first half alone.

Options, each pre-approved:

- **`jig-domain-core`** is shared with `note-answer`, which runs beside you in `routing/effect/note.rs`, `stored.rs`,
  `vocabulary.rs` and `runs/calls.rs`: stay out of those files.
- **Worlds' budgets:** if the story belongs better in `jig/tests/notes` or another core suite file by its fixture, put it there,
  and say so.
- **Search recalls:** `ToolKind::Recall`'s search reads `note_readable`, so it follows. If a named recall checks elsewhere, make
  that follow too.

## Steps

1. `note_goal` and its three readers.
2. The tests, each having bitten: remove the `conversation` check, and see `a_task_not_flagged_conversation_has_none` fail; remove
   the requester check, and see `a_delegate_of_a_chat_has_no_scope_of_its_own` fail.
3. `heavy cargo nextest run -p jig-domain-core -p jig-client`, the core world's package (`jig/tests/core/Cargo.toml`'s name), `--test suite -E
   'test(a_chat_writes_its_own_scope_and_a_party_reads_it)'`, then whole; `-p temper-service --lib -E 'test(stock_brief)'`; the readers the brief's line moves.
4. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report.
