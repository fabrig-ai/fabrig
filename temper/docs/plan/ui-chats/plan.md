# ui-chats: the plan

The Chats page and every chat as the greenfield mock draws them. The request:
`temper/docs/plan/ui-chats/inputs/request.md`. The cut wrote the design before these tasks, in the request's
words: `temper/docs/design/client/README.md`, 3.5, 4.8, 5.1, 5.3, 5.5, 5.6, 5.7, 5.9, 5.11 and 11;
`client/browser.md`, 2.3, 2.5, 7.2; `client/journeys.md`, 9; `skein/docs/design/ui.md`, 3.2, and
`ui-web.md`, 7; their status files; the status of `first-run-projects`, `to-begin` and `template-places`.

## The parts

`folds-away` goes beside the chain, in skein. The rest is one chain of `needs`, since all of it conflicts in
`temper.css`, `temper/tools/gallery/src/tests.rs`, `names.txt` and the terminal's goldens:

1. `new-chat-address`: the address and its readers, `NewChat` still drawing the old page.
2. `new-chat-page`: `NewChat` on the session, `pages/session.rs`.
3. `start-walks`: the composer leaves `Chats`; every walker starts through `NewChat`.
4. `new-chat-playground`: the playground's start on `NewChat`, its banner in the frame.
5. `chats-page`: the Chats page as the mock's.
6. `session-layout`: every chat on the session; `CHAT` and `CHATBAR` renamed for the agent's page.
7. `chat-byline`: the byline and its panel (needs `folds-away` too).
8. `conversation-turns`: the column, the bubbles, the dots, the scroll.
9. `tool-lines`: the pen, a change and a goal as one line, cards as `.proposal`.
10. `chat-stories` and `playground-recording`, side by side: they share no file.
11. `proof`: the renders beside the mock, the sweep.

The request's order is kept. Its "the Chats page" comes after `NewChat` and the playground, and its last step, goldens,
gallery and recording, is split: each task regenerates what it makes stale and adds the gallery states it draws, as
*For every task of this plan* says, which leaves the recording, the stories and the proof to the end.

## The critical path

`new-chat-address` (75) → `new-chat-page` (80) → `start-walks` (70) → `new-chat-playground` (70) → `chats-page` (85) →
`session-layout` (85) → `chat-byline` (75) → `conversation-turns` (85) → `tool-lines` (85) → `chat-stories` (60) →
`proof` (75): 845 implementer-minutes. Every task is at most 90 minutes, as the request's *Size* has it.

## Calls the cut made

- **A skein mechanism:** no disclosure closes on a press outside it today, and CSS can't do it.
  `State::FoldsAway` goes into skein first (AGENTS.md, 1), as an opt-in, so that a call's fold stays open.
- **An agent's task's page keeps its look.** `pages/agent.rs` reads `CHAT` and `CHATBAR`, and the request keeps
  to chats' pages: the tokens are renamed `AGENT_PAGE` and `TASKBAR` rather than deleted.
- **The log scrolls.** skein keeps a log's end in view on the `[role=log]` element, so the log is the mock's
  `.thread`, and "Older" and the tail go inside it, to scroll together as the mock's do.
- **The playground's banner** is drawn by the frame on `Chats` and `NewChat` while it runs or is leaving, and on
  every other page while it is leaving.
- **"Start a chat"** on `ConnectModel`'s Ready goes to `NewChat`, as its words say. Welcome's whole state stays
  `Chats`.
- **"Manage usage"** leaves the byline, which is words alone, and is the chooser's panel's last control.
- **The chat's context line,** in the playground's project, stays visible: at the thread's head, in
  `.meta-line`'s look. The mock has no playground, and the terminal prints it.
- **`chats-first-run-1`** stays the first run's Chats; the playground's start is a new state,
  `chat-new-first-run-1`, beside the request's list.
- **No stored format, protocol or public interface changes,** but skein-ui's new state.
- **A migration task of its own:** `start-walks`. 24 files of tests and worlds type in "Start a chat", so moving them is
  separated from redrawing the Chats page.
- **The browser tier stays at 12 tests:** it is at its cap, so `chat-stories` adds steps to an existing test.

## Overlaps with the drafts beside it

`look-round-5` lands `configure-temper-name` in the same rules of `temper.css` and in `pages/configure.rs`: take
its rules on catch-up. `ui-needs` removes the Inbox's "+ New chat", which `new-chat-address` retargets.
`ui-goals` also draws `.goal-row`: this plan's tokens are `CHATS_*` so that neither names the other's.
`ui-changes` and `ui-welcome` touch the rail, `frame.rs` and `names.txt`.
