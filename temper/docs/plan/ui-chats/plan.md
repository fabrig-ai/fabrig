# ui-chats: the plan

The Chats page and every chat as the greenfield mock draws them. The request:
`temper/docs/plan/ui-chats/inputs/request.md`. The cut wrote the design before these tasks, in the request's
words: `temper/docs/design/client/README.md`, 3.5, 4.8, 5.1, 5.3, 5.5, 5.6, 5.7, 5.9, 5.11 and 11;
`client/browser.md`, 2.3, 2.5, 7.2; `client/journeys.md`, 9; `skein/docs/design/ui.md`, 3.2, and
`ui-web.md`, 7; their status files; the status of `first-run-projects`, `to-begin` and `template-places`.

## The parts

`folds-away` goes beside the chain, in skein. The rest is one chain of `needs`, since all of it conflicts in
`temper.css`, `temper/tools/gallery/src/tests.rs`, `names.txt` and the terminal's goldens:

1. `new-chat-address`: the address and its readers. (released)

The rest of the chain follows in the next push.

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

## Overlaps with the drafts beside it

`look-round-5` lands `configure-temper-name` in the same rules of `temper.css` and in `pages/configure.rs`: take
its rules on catch-up. `ui-needs` removes the Inbox's "+ New chat", which `new-chat-address` retargets.
`ui-goals` also draws `.goal-row`: this plan's tokens are `CHATS_*` so that neither names the other's.
`ui-changes` and `ui-welcome` touch the rail, `frame.rs` and `names.txt`.
