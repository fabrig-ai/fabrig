+++
id = "chat-document"
part = "notes"
kind = "feature"
needs = ["note-parts"]
files = ["temper/tests/real-loop/tests/suite/notes.rs"]
effort = "medium"
minutes = 60
+++

# A chat's document revised whole after the person's follow-up, in the real loop

## Goal

The process walk `chat_document::a_chat_document_is_written_and_revised_on_the_fake_box`
(`temper/crates/temper/tests/fake_backends/walk/chat_document.rs`) is not
covered by `note-parts`' story: its revision comes from the person's
follow-up `task message`, and rewrites the note whole, paths
`note_parts` never takes. It goes beside it, short, as
`a_chats_document_is_revised_whole_after_a_follow_up` in
`temper/tests/real-loop/tests/suite/notes.rs`. The walk itself is
`walks-go`'s; you never touch it.

Acceptance: **`a_chats_document_is_revised_whole_after_a_follow_up`**,
new, beside `a_long_document_is_built_in_parts_and_revised_by_a_fresh_run`,
reusing its helpers, keeping the walk's assertions in order:

1. A chat started with the walk's words ("Please write this up. Write
   this up as a draft beside us.") calls `note` in its goal scope
   (project 1, task 1), description `Away mode`, with the walk's whole
   Markdown draft; the committed turn's result reads `Note 1 revision 1
   written · 29 lines`; the activity holds "I wrote an Away mode draft
   with its purpose, return flow and open questions.".
2. The person sends `task message 1 "Add what happens to a running
   goal."`; the run's next `note` sends the whole replacement body (`name:
   1`, `recalled: 1`, no line range) adding the seven lines headed `##
   A running goal`; its result reads `Note 1 revision 2 written · +7 −0 ·
   line 14 · 36 lines`; the activity holds "I added how a running goal
   continues and waits for your decisions.".

- The two scripts are the walk's, taken from
  `temper/crates/temper-init/template/temper.json` (its cued `Away mode`
  scripts, about 504 to 545): copy their arguments into the story's
  script, or read them from the template if the real loop's fixtures
  already do so; say which.
- **Bite:** drop the follow-up message by hand and see step 2 fail for
  it; restore.
- Clippy clean, fmt clean, `temper-real-loop-world`'s focused suite green.

## Inputs

- `temper/docs/plan/walks-lower/notes/common.md` (read first).
- `temper/docs/plan/walks-lower/inputs/request.md`, the
  `chat_document::` ruling.
- `temper/docs/design/domain/testing.md`, 2.6, the bullet **The fake box
  keeps two walks**, its **A long document built in parts** item (the
  sibling's sentence).
- `temper/docs/plan/walks-lower/tasks/note-parts.md` (its helpers and
  configuration, now on the trunk).
- `temper/tests/real-loop/tests/suite/plan_chat.rs`: a `task message`
  during a chat, and its scripted next turn.

## Owns and may change

Owns:

- `temper/tests/real-loop/tests/suite/notes.rs`: the new story; its
  sibling's helpers may be shared, not changed in what they check.

Pre-approved options:

- **Bounds:** this story's calls are small; it may run on the fixture's
  defaults rather than its sibling's raised ones.

## Steps

1. Read the inputs and `notes.rs` as `note-parts` left it.
2. Write the story; run it alone:
   `heavy cargo nextest run -p temper-real-loop-world --test suite -E
   'test(=notes::a_chats_document_is_revised_whole_after_a_follow_up)'`.
3. Bite it; restore.
4. Once, at the end: `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-real-loop-world --all-targets -- -D warnings`; `heavy cargo
   nextest run -p temper-real-loop-world` (check the count). Commit (`walks-lower: a chat's document revised whole after
   a follow-up, in the real loop`, the body saying why `note_parts` did
   not cover it) and push.

## Report

As the implementer's text says: the first line; the commit pushed; each
run with its count; the bite and its failure; the story's CPU seconds;
the minutes.
