# ui-chat-draft: the plan

A chat's document beside its conversation, Start goal and the rail's menu, as the request says:
`temper/docs/plan/ui-chat-draft/inputs/request.md`. The cut writes the design ahead of each batch.

## The parts so far

The engine's jig tasks and skein's, side by side, in files of their own: `note-scope`, `note-answer`, `note-room`, `skein-keys`. The rest of the plan follows
in later batches: the project's documents read; temper's stock, words, instructions, facts and fake model; the client's state and
live reading; the pane on a configuring chat; a chat's bar and tool line; `+ Draft`; Start goal; the Chats page; the rail's menu; the
recording and the proof.

## The critical path, as it stands

`note-answer` (75) → the rest, once its batch is out.

## Calls the cut made

- **No compatibility:** the store and the client's wire change in place (the request).
- **`core::Held::NotesWritten` stays as it is:** nothing makes it, and the note's answer goes by `CallPart::NoteWritten`.
- **A call's answer before its input:** a turn's text budget is greedy, so a whole body in a call's input cut the answer the tool line reads. `note-answer` takes the
  answer first.
- **A box's room:** its `hub.event_bytes` and `tool_payload` stay, since a call's payload is bounded by its model's output (`configuration.md`, 13.8). So the body's bound is the
  request's, and a box's own model writes about 10 KB in one call until the box has a larger one.
