# chat-reading-a: the rules every task names

The request is `temper/docs/plan/chat-reading-a/inputs/request.md`: read it, it is short. This plan is sections 3 and 4 of `temper/docs/design/draft/chat-reading.md`, and the status beside the draft says where the design departs from it. `notes/contracts.md` holds the shapes that cross tasks: they bind.

## Departures from the implementer's text

- **Status lines:** the design for this plan is written, in the sections your spec cites. You change no design text, but your task removes the **Not yet** line your spec names from its status file, once its code holds.
- **Failing test first,** in the lowest tier that shows the rule: run it red, and your report says what it showed.

## Not this plan

Leave these exactly as they are: they are later plans, or another's.

- **Section 2, the line:** a chat's calls stay folded as they are ("N tool calls"), and no verb table starts here. Section 7's verbs and a turn's time belong to it.
- **Section 5, reading:** "Older", `pages::chat::LINES` (16), `Line::Left` and "Earlier lines omitted N" stay. "Earlier text omitted" is a different notice, and it goes.
- **Section 6, numbered lists:** `read` and `read_growing` read an ordered item as `read` reads it today.
- **The look request `chat-work`:** it draws the line and the page's look. A sheet edit here is only the hunk your spec names.
- **`docs/help/`:** nothing here changes a limit it names.
- **The user's recordings:** nothing here reads or adds one.

## Beside this plan

`fabrig/ui` has priority, on `temper-client-*` above all: keep hunks small, don't reformat, and don't move code you don't need to change. If your merge meets `ui-rename-focus`, `ui-walk-large` or `ui-final-walk` in a file your spec names, keep their change, and say so in your report.

## Limits

No budget or timeout is raised. A limit this plan raises is one your spec names. A heap ceiling that moves is named in your report, with its measurement before and after.
