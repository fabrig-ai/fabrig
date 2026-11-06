# chat-reading-b: the rules every task names

The request is `temper/docs/plan/chat-reading-b/inputs/request.md`: read it, it is short. This plan is section 2 of `temper/docs/design/draft/chat-reading.md`, and what `look-round-6` left of it. `look-round-6` drew the line; this plan gives it the table's further tools, a time for a run of one turn, a running state and "Latest" on an agent's page, and the stories that run it through a shell.

## Departures from the implementer's text

- **Status lines:** the design for this plan is written, in the sections your spec cites. You change no design text, but your task removes the **Not yet** line your spec names from `temper/docs/design/client/README.status.md`, once its code holds.
- **Failing test first,** in the lowest tier that shows the rule: run it red, and your report says what it showed.

## What holds everywhere

- **Never a result's body on the line or in its rows:** its first line, cut to the row, and no more.
- **Keep hunks small.** `fabrig/ui` has priority on `temper-client-*`: don't reformat, and don't move code you don't need to change. If your merge meets another plan's change in a file your spec names, keep theirs, and say so in your report.
- **`chat-reading-c` is writing the same conversation:** `pages/chat.rs`, `chat_line.rs`, `agent.rs`, `words.rs`, `tests/chat.rs`, `tests/task.rs`, the domain's `pages/task.rs`, the terminal's prints. Its `Line::Left`, `LINES` and "Older" are going: don't build on them, and don't remove them.
- **A word's new id** is in 3800..=3839, this plan's block, above the occupied range. `tests::accounts::english_phrase_declarations_match_the_vocabulary` regenerates with `SKEIN_CODEGEN_WRITE=1` through heavy.
- **No budget or timeout is raised.** Formats change in place before v0.1.0, and nothing here changes one.

## Not this plan

Section 3 (the answer), 4 (what a turn keeps), 5 (reading, but for "Latest" on an agent's page) and 6 (lists); `docs/help/`; the user's recordings; jig's reference UI, which keeps a folded line a call (`temper/docs/design/draft/chat-reading.status.md`, 2).
