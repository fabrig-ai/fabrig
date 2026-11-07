# ui-agent-keys: the plan

PgUp and PgDn read back on an agent's page and a goal's untracked page, from the message box, as on a chat's. The request is `inputs/request.md`.

## Parts and tasks

| Part | Task | Needs | Minutes | What |
|---|---|---|---|---|
| `log` | `agent-log` | | 60 | in a browser, the log is the one scroller and holds the page's column |

Critical path: `agent-log` alone. temper/qa rewalks after it lands.

## What was decided

- **The log is the scroller, and the parts move into it.** Removing the two overflow rules alone, the first thing the request asks to try, isn't enough. The brief, run heading, held card, outcome, result and plan are the thread's `flex: none` children and rely on its scrolling; a long brief or result would run past the thread with nothing to scroll it, worst at 420 wide. So the column stands inside the log, the shape a chat's page already has (`temper/docs/design/client/README.md`, 5.9, *What scrolls*).
- **No change in skein.** The page can be the same as a chat's, so `ScrollScreen` stays as it is.
- **The terminal's tree keeps its order,** with the parts around its log: "Conversation:" would otherwise head the brief in its print. Most view tests and the client world use the terminal's words, which keeps this change's reach small.
- **The amendment's editor stays above the log.** It takes no focus, so inside a log following its end it would open out of sight. It scrolls itself where a short window can't hold it, the one exception to a single scroller, and only while it is open.
- **The page now opens at its end,** as a chat's, where it used to open at the brief. On a long conversation the brief and a held card are read back to, and a brief past a screen and a half above a cut window starts its earlier read late. Accepted, and held in section 11 for temper/qa's rewalk to judge.
- **The press at a long chat's head is left alone.** It is dropped: at `scrollTop` 0 a press has nothing above it and isn't kept. Keeping one is more than a line (per-log state in skein's `Page`, a replay after the view is put back, a limit so a held key doesn't jump many screens, a line in skein's `ui-web.md`) and wouldn't shorten the wait, since the reads chain on the log's measure, not on presses. Read in the code; the walk's trace wasn't opened.
- **Tests are lines in existing tests:** one that reads the stylesheets for which element scrolls, and the view's order tests in each shell. No browser test, no render check.
- No stored format, wire or public interface changes.

## Overlaps with drafts beside this one

- `temper-client-shell-web/temper.css`: `model-catalogue-3/models-read` also changes it. This plan's edits are confined to the agent page's rules, so a textual conflict is unlikely.
- `temper/docs/design/client/README.status.md`: also changed by `model-catalogue-3/list-close`, `model-catalogue-3/models-read` and `account-email/email-close`. This plan adds one entry beside the other 5.9 entry, and `agent-log` removes it.
- `temper-client-view/src/look.rs` takes the retired token 81 in place, so it adds nothing at the end of the list, where other plans append.
