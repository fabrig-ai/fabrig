+++
id = "chat-stories"
part = "chat-stories"
kind = "feature"
needs = ["tool-lines"]
files = [
  "temper/tests/client/tests/suite/chat.rs",
  "temper/tests/client/tests/suite/main.rs",
  "temper/tests/client-browser/tests/browser_all/browser_pages.rs",
  "temper/tests/client-browser/src/lib.rs",
]
effort = "high"
minutes = 60
+++

# The story in both worlds

## Goal

The request's test world: a chat started from `NewChat`, a change delegated, landed, and the chat closed, in the browser's world as in the
terminal's.

Acceptance:

- **The terminal's world,** `temper/tests/client/tests/suite/chat.rs`: one story, through the shipped view and its printed text: "+ New chat", the
  words and Enter, `Chat { task }` replacing the address, the delegated change's card ("Running", then "Landed" with its result's words), its link
  named "Task N", Close confirmed, "Closed" and no composer, the row "closed" under Closed on `Chats`.
- **The browser's world:** the same story in real Chromium, each step by role and name: its address `/chats/new`, then `/chats/N`, back not
  returning to `/chats/new`; the change's one line; "Open" named "Task N"; Close; the row.
- **The tier's cap:** the browser tier is at its cap of 12 tests (`temper/docs/design/client/browser.md`, 4.3). The story becomes steps of an existing
  test, `browser_pages.rs`' `history_keeps_the_chat_focus_and_filter_across_reload_and_back` or the one it best shares a setup with, never a 13th.
  A budget is the user's. Over the tier's 120 seconds is red: apply the implementer's text.

## Inputs

- `temper/docs/plan/ui-chats/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-chats/inputs/request.md`, *The proof*.
- `temper/docs/design/client/browser.md`, 4.3; `temper/docs/design/domain/testing.md`.

## Owns and may change

Owns, as `files` lists them.

May also change: `temper/tests/client/src/**`, `temper/tests/client-browser/**`, `temper/crates/temper-fake/**`: a helper the story reads, the
scripted model's change being what the first run already drives, with no new fake behaviour if that holds.

Options, each pre-approved:

- The change comes from the playground's scripted model (`start_playground_chat`) if that's the shortest world that lands one, or the fixture
  `temper/tests/client/src/fixtures.rs` holds for a landed delegation.
- Either world's closing step reuses the existing `a_closed_chat_shows_in_your_chats`, extended rather than repeated.

## Steps

1. The terminal's story, then the browser's.
2. Each rule bites: no step passes if `NewChat` goes back to `Chats` or the Chat isn't replaced in history. Name what you broke.
3. **Check:** `heavy cargo nextest run -p temper-client-world`, `heavy cargo nextest run --profile browser-temper -p temper-client-browser-world`
   whole, with its measured seconds in the report, clippy, push.

## Report

As `notes/common.md`, *Your report*, with the tier's seconds.
