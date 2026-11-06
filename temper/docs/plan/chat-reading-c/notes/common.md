# chat-reading-c: the rules every task names

The request is `temper/docs/plan/chat-reading-c/inputs/request.md`: read it, it is short. This plan is section 5 of `temper/docs/design/draft/chat-reading.md`. `temper/docs/plan/chat-reading-c/notes/contracts.md` holds the shapes that cross tasks: they bind.

## Departures from the implementer's text

- **Status lines:** the design for this plan is written, in the sections your spec cites. You change no design text, but your task removes the **Not yet** line or clause your spec names from its status file, once its code holds.
- **Failing test first,** in the lowest tier that shows the rule: run it red, and your report says what it showed.

## Not this plan

Leave these exactly as they are: they are other plans', or another's.

- **Section 2, the line:** its verbs, a run's time and an agent's page's running state are `chat-reading` (b)'s. An agent's page keeps having no "Latest" pill.
- **Section 6, numbered lists:** `chat-reading-d`'s.
- **The look request `chat-work`:** `look-round-6` draws the states. This plan changes what a state does, and draws nothing new but what its spec names.
- **`docs/help/`:** nothing here changes a limit it names.
- **The user's recordings:** nothing here reads or adds one. A journey that moves is named in your report, for fabrig/ui.

## Beside this plan

`fabrig/ui` has priority, on `temper-client-*` and `skein-ui*` above all, and its `look-round-6` lands the log's pill, `State::ToLogEnd` and `data-log-away`, the conversation's run line and its "Older". Keep hunks small, don't reformat, and don't move code you don't need to change. If your merge meets another draft in a file your spec names, keep their change, and say so in your report.

`ui-rename-focus` set `#app > .temper-session .temper-thread > [role=log] { position: relative; ... }` in `temper.css`: keep it.

## Limits

No budget or timeout is raised. A limit this plan changes is one your spec names. A heap ceiling that moves is named in your report, with its measurement before and after. The browser tier's budget is the user's (`skein/docs/foundation/testing-strategy.md`, 8.1): a story takes the fewest page loads that prove it.
