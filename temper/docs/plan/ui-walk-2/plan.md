# ui-walk-2: the plan

Two findings of the second whole walk: A4, an answer that didn't send, and R1, a command that doesn't run. The request: `temper/docs/plan/ui-walk-2/inputs/request.md`. `Failed: cut short` applies until `report.md` says otherwise; this file is brought up to date with each push.

## The tasks

| # | Task | Needs | Effort, minutes | What it gives |
|---|---|---|---|---|
| 1 | `needs-press` | | high, 90 | A4: a press survives a refresh of the inbox; a bound's refusal says so |

## The critical path

`needs-press` (90), so far.

## What the cut decided

- **A4's cause, from the code, to be proved red first:** `Inbox::refresh` blanks each card's page, and empties the cards at a filtered address, for the round trips of its reads, on every change of the inbox. A4's box had a goal running and a proposal arriving. The choices leave the page and come back as new elements, and `needs::answer` returns with nothing shown. The hold's timer, as far as reading goes, is sound; the task tests it anyway.
- **The fix keeps what's drawn, rather than making the loss visible:** a lost click never reaches the domain, so no message could report it. The design's new lines are `temper/docs/design/client/README.md`, 3.5, *A press stays under the finger*. What a bound still refuses gets a toast.

## Overlaps not avoided

- `pages/inbox.rs`, `pages/needs.rs`, `tests/pages/inbox.rs`, with `ui-walk-large/needs-proposal`: the fault is there, and it can't wait.
- `temper-client-view/src/words.rs` and `temper/docs/design/client/README.status.md`, with several plans: one addition each.
