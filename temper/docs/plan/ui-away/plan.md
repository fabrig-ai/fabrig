# ui-away: the plan

From `inputs/request.md`: the greenfield mock's Away works for real. The switch, a goal's host tool `decided`,
Home's away form, Overturn and Settings' section. This file grows with each push.

## How it sits on the plans before it

The request needs `ui-home-status`, and a plan of the settings area if one is queued; the rulings build on
`ui-goal-page`'s Start goal sheet. None has landed as this is cut. `ui-home-status` and `ui-settings` are in the
merge queue, `ui-goal-page` still has `goal-proof` to run.

- **`after = ["ui-home-status", "ui-settings", "ui-goal-page"]`,** beside fab's own options: this trunk may carry
  their unlanded commits, and the option holds its end check and landing until they've landed.
- **`home-base`** merges `ui-home-status` into this trunk, or main once it has landed.

## Parts and tasks

| # | Task | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| 1 | `home-base` | | high, 60 | `ui-home-status`' merged code on this trunk |
| 2 | `client-base` | 1 | high, 60 | `ui-settings`' and `ui-goal-page`'s merged code |
| 3 | `away-wire` | 1 | medium, 40 | the wire's requests, subjects and documents |
| 4 | `away-forge` | 1 | high, 90 | families 38 and 39, the switch, the judgement, Overturn |
| 5 | `client-zone` | 1 | high, 60 | the browser's zone, and the clocks read in it |
| 6 | `away-route` | 3, 4 | high, 60 | the root and the service carry them |
| 7 | `decided-tool` | 4 | high, 90 | the host tool, named once; the figures |
| 8 | `decided-charter` | 7 | medium, 40 | charter 1's words |
| 9 | `away-client` | 2, 3, 5 | high, 75 | the client's watch, its act and toasts |
| 10 | `away-switch` | 9 | high, 90 | the control, the chip, the terminal, `rail-away-on` |
| 11 | `home-decided` | 9 | high, 75 | Home watches `decided`, reads its tasks, `Facts.away` |
| 12 | `home-away` | 10, 11 | high, 90 | the intro, the section, Needs' head, `home-away`, `home-away-one` |
| 13 | `away-walk` | 6, 7, 11 | high, 60 | the meeting point: real roots, a fake clock past `until` |
| 14 | `overturn` | 12 | high, 75 | the request, the message, its toasts, `home-away-overturned` |
| 15 | `settings-away` | 14 | high, 75 | the section, "Limits" lands on it, `settings-away` |
| 16 | `goal-asks` | 8, 15 | high, 75 | "When it needs you" on the sheet, "How it runs" on the page |
| 17 | `away-proof` | 13, 16 | high, 90 | the mock beside it at three widths, the fake box's walk, the statuses |

The request's task 1, the design, is this cut's own: `domain/forge.md`, 5.8 and 5.9; `protocol/store.md`, 2 and 3; `client/README.md`, 3.5
(*Away*), 4.6, 5.12, sections 8 and 11; `client/browser.md`, 2.1 and 7.2; `domain/agent.md`, 4.3; `protocol/configuration.md`, 13.4. Its 2 is
tasks 3, 4, 6, 9 and 10; its 3 is 7 and 8; its 4 is 11 and 12; its 5 is 14; its 6 is 15. Tasks 1 and 2 are what it stands on, 5 is what "08:00" needs, 13 and 17 are its proof and 16 is the rulings'.

## Critical path

`client-base`, `away-client`, `away-switch`, `home-away`, `overturn`, `settings-away`, `goal-asks`, `away-proof`: ~630 implementer-minutes, then the trunk's end check, held by `after` until `ui-home-status`, `ui-settings` and
`ui-goal-page` have landed. `away-forge`, `decided-tool`, `decided-charter` stand beside it, ~220, and join at `goal-asks`; `away-walk` joins at the proof. `away-wire`, `away-forge` and `client-zone` start at once. The view, the stylesheet, the gallery and the goldens take one writer at a time, 10, 12, 14, 15, 16, then 17.

## What the cut settled, for the user to see

- **A change of stored format and protocol:** families 38 `Away` and 39 `Decided`, `CallPayload`'s `decided`; the wire's `SetAway`, `Overturn`,
  `Aways`, `Decisions`, `Watch::Away`, `Watch::Decided`. No compatibility is kept.
- **The clocks now read in the browser's zone, every one of them.** The request's "next 08:00 in the browser's zone" with `TimeForm::Clock`
  would otherwise read "Away until 06:00" off universal time, since every clock in the client is universal today. So the zone becomes the
  shell's input, closing `client/README.md`, section 11's open question, and 5.12's forms read at it. That moves clocks on every page in a real
  browser; at zone 0 no golden moves. The terminal stays universal. The offset is read at start, so a change of daylight saving shows at the next load.
- **`off`, a field the request doesn't name,** in place of its `on`: "23:10 to 08:03" needs the time it went off, and a row with both could
  disagree with itself. On is `off` none; the wire's request keeps `on`.
- **`party`, a field the request doesn't name, on a `Decided` row.** "While you were away" and "decided for you" would be false on a colleague's
  Home, and Overturn sends the requester's message. So `decided`'s document is the asking party's alone, and another's row is unknown to `Overturn`.
- **The requester is found through the chat.** The request says `domain.core.tasks.delegation`, but a chat's goal has the chat as its requester: the
  root takes jig's `served_chat`, then the chat's person. Any other caller is told to ask.
- **Only goals call it** is held by the root, not only the words: goals and chats share charter 1, so a chat's call answers ask. An advisor lacks the control
  family and is refused.
- **An overturned row whose task has ended says "landed…",** not "it asks you again", took still struck: a refused message leaves the row marked, and its task
  won't ask. The request's list read literally would say something false there.
- **The chip is a second button,** as the mock's is, since the rail's foot is hidden on a phone, and project pages have no banner: the rail is the phone's top bar.
- **"Terminal: the same button in its `navigation`"** is the rail's button: `navigation` isn't drawn on a project page, where the terminal prints the rail.
- **"Limits" focuses the section:** `Address` holds no anchor, so a one-shot request in `Pages` moves focus, and a browser brings it into view.
- **`Asks: always` is a line in the goal's words,** read by the root's rail, the charter's sentence and the goal's page; nothing stores it apart.
- **A refused `Overturn`** (a second device got there first) toasts "That was overturned already." and sends no message. The request names no words for it.

## Overlaps with the drafts beside this one

- **`ui-home-status`, `ui-settings`, `ui-goal-page`:** this trunk carries them through `home-base` and `client-base`. Their own catch-ups will meet this trunk's
  resolutions at the next merge of main; generated files and goldens are regenerated then, by `notes/common.md`.
- **`ui-chat-draft`** changes `frame.rs` (`rail-menu`), `Dismissed` and the store's schema (`goal-sheet-state`), `start_goal`'s neighbours and `words.rs`, `look.rs`, `temper.css`
  and the gallery. Unavoidable: `away-switch` holds the rail's foot, which `rail-menu` also changes, and a store byte may move.
- **`temper-client-view`'s `words.rs`, `look.rs`, `temper.css`, the gallery:** one client task of this plan at a time holds them, in a chain.
