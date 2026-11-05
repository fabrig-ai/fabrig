+++
id = "note-walk"
part = "note-walk"
kind = "feature"
needs = ["note-tool", "task-notes"]
files = [
  "temper/crates/temper/tests/fake_backends/walk/note_parts.rs",
  "temper/crates/temper/tests/fake_backends/walk.rs",
  "temper/crates/temper/tests/fake_backends/walk/chat_document.rs",
]
effort = "high"
minutes = 60
+++

# A scripted model builds a 30 KB note in parts, through ctl on a fake box

## Goal

The plan's proof: on `temper box --fake-backends`, a chat's scripted model builds a note of at least 30 KB in several calls of under 10 KB each,
through the real engine and tools. Its revisions come in order, and the pane's read answers it whole.

Acceptance:

- `temper/crates/temper/tests/fake_backends/walk/note_parts.rs`, on `walk.rs`' harness, `a_long_document_is_built_in_parts_on_the_fake_box`, with
  `temper ctl` and `</dev/null` as `chat_document.rs` has them.
- **The script is the walk's own:** nothing in `temper-init/template/temper.json`, which every box carries. The walk generates it, as JSON, from
  a line generator of its own, and asserts of it: each call's `arguments` under 10,000 bytes, the final body at least 30,000.
- **Its turns,** on literal numbers of a fresh box, as `chat_document.rs` establishes them, project 1, chat 1, note 1:
  1. `note`, whole, a first part of about 7.5 KB. Answer `Note 1 revision 1 written · L lines`.
  2. to 4. `note` with `name`, `recalled`, `first` = lines + 1, `remove` 0, each about 7.5 KB, appending. Answers `+A −0 · line F · T lines`, by the
     contract's words.
  5. `note` with a range in the middle, `remove` 2 and three lines of text. Answer `+3 −2 · line F · T lines`.
  6. `note` with `recalled` 1, stale. Answer `Note refused: revision moved, now 5`.
  7. `note` with `first` past the end. Answer `Note refused: range past the end, T lines`.
  8. a sentence, `stop`.
- **The proof,** through ctl: `chat start`, `task show --until replied`, `task activity`; each `Note 1 revision R written` in the activity in order,
  R 1 to 5, each turn's complete result read as `chat_document.rs`' `written_turn` does; and `task notes 1 --project 1 --json`, the pane's read: one
  note, revision 5, `cut` false, its body equal, byte for byte, to what the walk computes, at least 30,000 bytes.
- `temper/docs/design/domain/agent.status.md`, 8.3's entry becomes: `8.3: a document built in parts is proved on a script alone: no real model has
  built one, so whether it counts lines right, appends rather than rewrites, and recovers from a refusal is unobserved.`

## Inputs

- `jig/docs/plan/note-parts/notes/common.md`, `jig/docs/plan/note-parts/notes/contracts.md`: *The words*, *`task notes`*.
- `temper/docs/design/domain/agent.md`, 8.3, *Its stories*; `temper/docs/design/protocol/setup.md`, 8.4.
- As patterns: `walk/chat_document.rs`; `temper-init/src/tests/documents.rs`, `scripted_turn`, for a turn's JSON and its tokens, `bytes ÷ 4` rounded up.

## Owns and may change

Owns, as `files` lists them: `walk.rs` for the `mod` line alone, `chat_document.rs` to make `written_turn` `pub(super)`.

Options, each pre-approved, tried in this order:

- **How the script reaches the box.** Default: two runs of one `Walk`, as `walk/sign_in.rs`' tests run it. `walk.run(vec![])` makes the retained
  sandbox; the walk adds its `{"cue": ..., "turns": ...}` to `application.demo.llm.cued` in `walk.sandbox().join("config/temper.json")`, keeping
  the file's 0600; a second `walk.run_then` serves it. The chat's words hold the cue.
- If a second run refuses the changed document, or doesn't serve the new cue: write the walk's document into its source `--config` before the
  first run, for `fake_backends.rs`' `Source::rebase` to import. Say which you used.
- If neither serves it, last, a `cued` entry in the template, mirrored in `documents.rs`' `demo()`, and say why in your report.

Other options:

- **The numbers:** if the first run leaves the second's numbers other than project 1, chat 1, note 1, read them from `project show` and generate
  the script after it, with the first run's acts; report it.
- **The chat's spend:** if the script's tokens end the chat on its spend, the walk's scripted `tokens` may be lower than `bytes ÷ 4`, as low as
  the script runs; say by how much. No limit or budget moves.
- **Over budget is red:** report the walk's CPU seconds. If the tier goes over, the implementer's text says what to ask, in order.

## Steps

1. The generator and the script, with the expected body worked out in plain Rust on a list of lines, not by `splice`.
2. The walk, reading its answers. Read a failure's log before anything reruns.
3. `heavy cargo nextest run -p temper --test process -E 'test(a_long_document_is_built_in_parts_on_the_fake_box)'`, then the same for
   `a_chat_document_is_written_and_revised_on_the_fake_box`. Check the count.
4. **It bites:** break the rule once by hand, `splice`'s added newline, and see the body's comparison fail; restore it.
5. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report, and: the sizes of each call and of the body; the walk's CPU and wall seconds; and, in a section of its own, what
this doesn't prove on a real model: whether it counts lines right, appends rather than rewrites, and recovers from a refusal; and the pane's own
drawing, which is `ui-chat-draft`'s, since this reads what the pane reads and draws nothing.

## Answers

2026-11-05 (fabrig/ui). Additions to the proof (inputs/request.md): the 30 KB note has one line longer than a call, built with join: true; and a later fresh run on the same chat finds the note from its brief and changes it by lines (finding-314 saw a goal's notes not reach its later runs). If a fresh run can't find it, that's a finding to fix here, not to work round.
