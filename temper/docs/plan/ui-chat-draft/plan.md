# ui-chat-draft: the plan

A chat's document beside its conversation, Start goal and the rail's menu, as the request says:
`temper/docs/plan/ui-chat-draft/inputs/request.md`. The cut writes the design ahead of each batch.

## The parts

Design first, in the cut itself: jig's `domain/engine.md`, 10, `domain/authority.md`, 3, `client/domain/contract.md`, 6, `client/domain/client.md`, 14 and 19, `protocol/configuration.md`, 6 and 9,
`draft/notes-memory.md`; skein's `ui.md`, 3.4 and 6.1, `ui-web.md`, 6; temper's `client/README.md`, 3.5, 4.6, 5.3, 5.4, 5.5, `client/browser.md`, 2.3, 2.5, 7.2, `client/journeys.md`, 9, `client/forge.md`,
`domain/agent.md`, 8.1 and 8.3, `domain/forge.md`, 5.5, `protocol/configuration.md`, 7 and 13.4, `protocol/store.md`, 2, and `draft/template-places.status.md`. Their status files. `tools/citations --all` is at zero.

| # | Task | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| 1 | `note-scope` | | high, 60 | a chat's own scope; a brief's `[You are task T in project P]`; a person's goal with no note scopes |
| 2 | `note-answer` | | high, 75 | `line_change`, the answer's four fields; a call's answer before its input |
| 3 | `note-room` | | high, 80 | a body's 65,536 and what carries it on the engine's side |
| 4 | `skein-keys` | | medium, 45 | `Ctrl+\`, a keyed slot, a menu item's description |
| 5 | `person-facts` | | high, 85 | `PersonFact`, `GoalFrom`, the bound |
| 6 | `person-words` | 3 | high, 85 | a client's `long_text`; a person's words to `tasks.spec_bytes`; `set_agreed_goal_from` |
| 7 | `notes-list` | 6 | high, 75 | `NoteScope::Tasks` |
| 8 | `chat-stock` | 1, 2 | high, 80 | a chat's scope and families, its instructions, its answer's words |
| 9 | `fake-document` | 8, 3 | high, 60 | the fake model's document, proved through ctl: the engine's half ends |
| 10 | `chat-state` | 8, 6 | high, 85 | the domain's state and live reading |
| 11 | `chats-base` | 10 | medium, 60 | `ui-chats/integration` merged in |
| 12 | `document-pane` | 11, 4 | high, 90 | the pane, group and strip, on a configuring chat (`configure-chat-draft`) |
| 13 | `start-over` | 12 | high, 60 | Start over; a configuring chat's line (`configure-chat-draft`) |
| 14 | `chat-bar` | 13 | high, 90 | every chat's group, tool line, Show (`configure-chat-draft`) |
| 15 | `add-draft` | 14 | high, 90 | `+ Draft`, `NewChat { draft }` |
| 16 | `goal-sheet-state` | 15, 5, 6 | high, 85 | Start goal in the domain; the lines' constant; `GoalFrom.at` |
| 17 | `goal-sheet` | 16 | high, 85 | the sheet, "Start goal…", "Goal TITLE →", its line |
| 18 | `chats-drafts` | 17, 7 | high, 75 | Drafts, "writes NAME", "started goal TITLE", the lede |
| 19 | `rail-menu` | 18 | high, 70 | `new-menu`, landed with its account |
| 20 | `chat-draft-journey` | 19, 9 | high, 80 | the journey, or a `Question:` |
| 21 | `draft-proof` | 20 | high, 85 | the renders, `configure-chat-draft`'s account, the status |

The request's order is kept; the tasks it can run side by side do, since they touch none of each other's files: the first five, then two chains in the engine. From `chats-base` on it's one chain, since they all meet in `temper.css`,
the gallery, `names.txt`, the goldens and the prints.

## The critical path

`note-room` (80) → `person-words` (85) → `chat-state` (85) → `chats-base` (60) → `document-pane` (90) → `start-over` (60) → `chat-bar` (90) → `add-draft` (90) → `goal-sheet-state` (85) → `goal-sheet` (85) → `chats-drafts` (75) → `rail-menu` (70) →
`chat-draft-journey` (80) → `draft-proof` (85): 1,120 implementer-minutes, then the trunk's end check, which waits on `ui-chats`' landing. Every task is at most 90 minutes.

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
- **The strip's button is a disclosure,** `State::FoldsAway`, not the hand-made page's plain `button`: a press outside and Escape fold it with no state in the domain. The account says so.
- **`markdown::read`'s base is 1,** where the request writes 2: the level is base plus hashes, and the request's "level 2 ... 3" needs 1.
- **`Chat.document` stays the task's handle,** so the request's `document` is `Chat.draft`.
- **`GoalFrom.at`,** a revision by `goal-sheet-state`: no document on this base says when a task was made, and the line stands by its time.
- **`Ctrl+\` is skein's primary key,** so ⌘ on a Mac, on the next mode's button.
- **A box's room:** its `hub.event_bytes` and `tool_payload` stay, since a call's payload is bounded by its model's output (`configuration.md`, 13.8). So the body's bound is the
  request's, and a box's own model writes about 10 KB in one call until the box has a larger one.

## Overlaps with the drafts beside it

- `ui-chats`: see above. `temper.css`, `pages/chat.rs`, `session.rs`, `new_chat.rs`, `choice.rs`, the gallery and the goldens, once the drawing tasks start.
- `ui-goal-page`: its `start-domain` and `start-sheet` draw a Start goal sheet from Goals, in `pages/start_goal.rs`, with no Budget. This plan's sheet is the chat's, in `pages/chat/`: the two aren't merged, and whichever lands second
  keeps both, named in the report. `plan-filter-out` removes a field of `Address::Task`: a compile red on catch-up, one line each.
- `ui-settings`: `pages/settings.rs`, `frame.rs`, `samples.rs`; `HowWorkRuns` if it has an address when `rail-menu` runs.
- `ui-welcome`, `ui-needs`: `frame.rs`, `temper.css`, `names.txt`, `samples.rs`.
- `person-facts` and `person-words` touch two schemas in place: `ui-goal-page/made` and `ui-settings/ways-kind` have too, already merged upstream; a catch-up regenerates.
