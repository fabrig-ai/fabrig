+++
id = "note-parts"
part = "notes"
kind = "feature"
needs = []
files = ["temper/tests/real-loop/tests/suite/notes.rs", "temper/tests/real-loop/tests/suite/main.rs"]
effort = "high"
minutes = 150
+++

# A long document built in parts and revised by a fresh run, in the real loop

## Goal

The process walk `note_parts::a_long_document_is_built_in_parts_on_the_fake_box`
(`temper/crates/temper/tests/fake_backends/walk/note_parts.rs`) moves to
the real loop (`temper/tests/real-loop`, package `temper-real-loop-world`),
whose hosted smith agents have the real tools, as
`a_long_document_is_built_in_parts_and_revised_by_a_fresh_run` in a new
`temper/tests/real-loop/tests/suite/notes.rs`. Same script, same
assertions, on the real loop's service, agent and fake LLM peer in place
of the box. `chat-document` adds a sibling story to the same file after
you. The walk itself is `walks-go`'s; you never touch it.

Acceptance: **`a_long_document_is_built_in_parts_and_revised_by_a_fresh_run`**,
new, keeping every assertion of the walk, in its order:

1. **The bounded calls.** The chat's model script is the walk's
   (`note_parts::script`): seven `note` calls in the goal scope of
   project 1, goal 1, description `Parts`; every call's JSON arguments
   strictly fewer than 10,000 bytes; one 7,500-byte line, joined to a
   second 7,500-byte part into one 15,000-byte line (`join: true`), then
   76 and 76 100-byte lines, then a three-line splice at line 40
   removing two; each answered with the walk's exact result words
   (`Note 1 revision 1 written · 1 lines` … `Note 1 revision 5 written ·
   +3 −2 · line 40 · 154 lines`), each read from its committed turn
   (`task turn 1 1.N`). The stored line is longer than any one call, and
   the body at least 30,000 bytes.
2. **Both refusals.** A stale edit (`recalled: 1`) is answered `Note
   refused: revision moved, now 5`; a past-the-end one (`first: 156`)
   `Note refused: range past the end, 154 lines`; neither is saved: the
   pane read (`task notes 1 --project 1 --json`) shows one note, number
   1, revision 5, title `Parts`, not cut, its body byte for byte the
   expected one.
3. **A fresh run's brief finds the document.** With charter 1's
   `resume: false`, the chat is stopped, held, its run ended, its
   specification amended ("Revise line 40 of the existing document.",
   title "Continue this chat from its document.") and released; the new
   run's opening turn (`task turn 1 2.1`) holds the brief's note index
   `1 r5: Parts`, which cues the fresh script.
4. **Revising it by lines.** The fresh run's `note` (`recalled: 5`,
   `first: 40`, `remove: 1`) replaces line 40 and is answered `Note 1
   revision 6 written · +1 −1 · line 40 · 154 lines`; the pane shows
   revision 6 with the expected body; the reply "I found the document in
   my brief and revised it by lines." is in the activity.

- **Bite:** change one expected result's line count, or let the stale
  edit's `recalled` match the revision by hand, and see the story fail
  for it; restore.
- The story's bounds are set by its own configuration: the agent's
  `tool_payload` and the model's `output` raised to admit the walk's
  7.5 KiB arguments (the shipped agent template declares 10,662), and
  `core.load_bytes` raised for the whole note record; the small profile's
  note body bound (32,768) already holds it. Real admission stays: no
  bound lifted beyond what the walk's box configuration had.
- Clippy clean, fmt clean, `temper-real-loop-world`'s focused suite green;
  the story's CPU seconds in the report.

## Inputs

- `temper/docs/plan/walks-lower/notes/common.md` (read first).
- `temper/docs/plan/walks-lower/inputs/request.md`, the `note_parts::`
  ruling.
- `temper/docs/design/domain/testing.md`, 2.4 (the real loop: *What
  runs*, *Agents share the loop*) and 2.6, the bullet **The fake box
  keeps two walks**, its **A long document built in parts** item.
- `temper/docs/design/domain/agent.status.md`, about 161 (the note-parts
  entry: what the walk proves and what it does not: no recall, no drawn
  pane, a scripted model). The story proves no more than the walk did.
- The walk (read only), the source of the script and helpers to carry:
  `temper/crates/temper/tests/fake_backends/walk/note_parts.rs` (`line`,
  `note`, `range`, `call`, `sentence`, `script`, `pane`, `prepare`,
  `read_results`, `start_fresh`).

What the survey found, for your bearings (verify before relying):

- `run`/`run_on` take a script or recipe and step factories; a turn is
  `api::Line::Call { name, arguments }` with `Finish::ToolCalls`, a reply
  `Line::Text` with `Finish::Stop`; `llm::played(llm::cued(…))` gives
  cued script groups (`temper/tests/real-loop/src/lib.rs`, about 140;
  `src/llm.rs`, about 78; `tests/suite/goal_delivery.rs`, about 56).
- `fixtures::signed_chat`, `calls::call`, `until`, `start`/`join`,
  `task(call_index)` and `activity_before(call_index)` sequence native
  calls; `Ran::calls` gives each call's exit and output.
- `tests/suite/goal_delivery.rs`'s `configure(document, agent, address)`
  sets engine overrides and agent fields from the story;
  `tests/suite/plan_chat.rs` uses a requester-ended chat
  (`fixtures::requester`), which a chat that is stopped and released
  needs; `fixtures::change_settings` applies a charter item.
- The caller has no step that reads a field of an earlier answer (the
  hold revision): use the fixed revision the held answer must carry and
  assert it from that answer, as the case acts do with named hold
  revisions (`src/case/acts.rs`, about 545).
- Fixture defaults: the agent's `tool_payload` 2,048 and model `output`
  2,048 (`src/fixtures.rs`, about 21 and 27); `core.load_bytes` 8,192
  inherited.

## Owns and may change

Owns:

- `temper/tests/real-loop/tests/suite/notes.rs` (new): the story and its
  helpers (the walk's script builders, carried over).
- `temper/tests/real-loop/tests/suite/main.rs`: the `mod notes;` line.

Reader and neighbour files (edits allowed as stated):

- `temper/tests/real-loop/src/fixtures.rs`, `src/lib.rs`: one small
  function appended at the end, or a `pub(crate)`→`pub` widening, only
  if the story's own configuration callback can't hold what it needs;
  named in the report. Not `src/world.rs`, `src/issuer.rs` or
  `src/calls.rs`, which `issuer-sockets` is changing.

Pre-approved options:

- **A step the caller lacks** (a field of an earlier answer as the next
  call's argument): take the fixed value and assert it from the earlier
  answer. If no fixed value holds, add the smallest `Step` variant in a
  new file `temper/tests/real-loop/src/calls/after.rs` with its `mod`
  line, and name it.
- **A bound the world can't raise without lifting real admission:** stop
  that keep, write the rest, and report it under `## Left`.

## Steps

1. Read the inputs, the walk, `goal_delivery.rs` and `plan_chat.rs`.
2. Carry the script and write the story; run it alone:
   `heavy cargo nextest run -p temper-real-loop-world --test suite -E
   'test(=notes::a_long_document_is_built_in_parts_and_revised_by_a_fresh_run)'`.
3. Bite it as above; restore.
4. Once, at the end: `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-real-loop-world --all-targets -- -D warnings`; `heavy cargo
   nextest run -p temper-real-loop-world` (check the count). Commit
   (`walks-lower: a long document built in parts in the real loop`, the
   body saying why: the real loop's hosted agents have the real tools,
   so the box adds nothing to what this checks) and push.

## Report

As the implementer's text says: the first line; the commit pushed; each
run with its count; the bite and its failure; the bounds the story set
and why; any harness line added; the story's CPU seconds; the minutes.
