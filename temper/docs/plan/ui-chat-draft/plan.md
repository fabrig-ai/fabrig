# ui-chat-draft: the plan

A chat's document beside its conversation, Start goal and the rail's menu, as the request says:
`temper/docs/plan/ui-chat-draft/inputs/request.md`. The cut writes the design ahead of each batch.

## The parts so far

The engine's jig tasks and skein's, side by side, in files of their own: `note-scope`, `note-answer`, `note-room`, `skein-keys`. Then, chained where they share files:
`person-words` and `notes-list` on the client's wire; `chat-stock`, `person-facts` and `fake-document` in temper's engine. The rest of the plan follows in later batches: the client's
state and live reading; the pane on a configuring chat; a chat's bar and tool line; `+ Draft`; Start goal; the Chats page; the rail's menu; the recording and the proof.

## The critical path, as it stands

`note-answer` (75) → `chat-stock` (80) → `fake-document` (60) → the rest, once its batch is out.

## How it sits on ui-chats

The request starts this plan after `ui-chats` has landed, and fab made the trunk at `ui-chats/integration`, which holds every chat in the session layout and `NewChat`, but not yet `chat-byline`,
`conversation-turns` or `tool-lines`. So `after = ["ui-chats"]`, as `ui-goal-page` has for its plans, holds this plan's end check and landing until `ui-chats` has landed; the engine's tasks and the
client's domain need nothing of those tasks; and `chats-base`, ahead of every task that draws, brings what `ui-chats/integration` has by then. fab refuses a `needs` on another plan's task, so that can't be said
by `needs`.

## Calls the cut made

- **No compatibility:** the store and the client's wire change in place (the request).
- **`core::Held::NotesWritten` stays as it is:** nothing makes it, and the note's answer goes by `CallPart::NoteWritten`.
- **A call's answer before its input:** a turn's text budget is greedy, so a whole body in a call's input cut the answer the tool line reads. `note-answer` takes the
  answer first.
- **A run knows its number:** a brief didn't say it, so `note-scope` adds `[You are task T in project P]` after `[Requested by …]`.
- **A person's goal has no note scopes:** a role's `Scopes(1)` reaches a filled task, so `shapes` fills them for a conversation alone; `goal.rs` and `forge_route.rs` zero theirs today.
- **A goal's words past `people.words`:** the request has `tasks.spec_bytes` holding them, but a person's spec stopped at `people.words`, 8,192, and a web client's field at 512. `person-words` changes
  people-v14's ceiling and the wire's in place, and gives a client its `long_text`.
- **`Note.cut` keeps its meaning:** a reference list cut. `NoteScope::Tasks` marks its body's own `cut`.
- **"Plan, then wait for me" has its rail:** `shapes::set_agreed_goal_from`, whose `asks` the root's refusal reads; no field on `GoalDraft`, so `ui-goal-page`'s literal is untouched.
- **The Budget's default** is `me`'s `within.budget`: `temper ctl goal start` has none to copy.
- **A party's own bound on facts:** `GoalFrom` has no bound per project, so `Dismiss` refuses at `DISMISSED` per party too, which keeps the document whole.
- **A box's room:** its `hub.event_bytes` and `tool_payload` stay, since a call's payload is bounded by its model's output (`configuration.md`, 13.8). So the body's bound is the
  request's, and a box's own model writes about 10 KB in one call until the box has a larger one.

## Overlaps with the drafts beside it

- `ui-chats`: see above. `temper.css`, `pages/chat.rs`, `session.rs`, `new_chat.rs`, `choice.rs`, the gallery and the goldens, once the drawing tasks start.
- `ui-goal-page`: its `start-domain` and `start-sheet` draw a Start goal sheet from Goals, in `pages/start_goal.rs`, with no Budget. This plan's sheet is the chat's, in `pages/chat/`: the two aren't merged, and whichever lands second
  keeps both, named in the report. `plan-filter-out` removes a field of `Address::Task`: a compile red on catch-up, one line each.
- `ui-settings`: `pages/settings.rs`, `frame.rs`, `samples.rs`; `HowWorkRuns` if it has an address when `rail-menu` runs.
- `ui-welcome`, `ui-needs`: `frame.rs`, `temper.css`, `names.txt`, `samples.rs`.
- `person-facts` and `person-words` touch two schemas in place: `ui-goal-page/made` and `ui-settings/ways-kind` have too, already merged upstream; a catch-up regenerates.
