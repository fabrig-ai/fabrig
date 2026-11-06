# chat-reading: status

2026-11-06. Where the design lags or departs from [chat-reading.md](chat-reading.md).

- **Design lags:**
  - 2: `../client/README.md`, 3.5: a run's time is the committed turns' alone, so a run of one turn has none; its table holds `read_docs` and `settings` first (section 7). jig's `client/domain/view.md`, section 9: each call a folded line naming its tool.
  - 3: `../client/README.md`, 3.5 and 5.4: provisional text is bounded by `text`, 512 bytes, and marked as cut. jig's `client/domain/client.md`, 8.3: the bound on a provisional turn's bytes. `skein/docs/design/ui.md`, section 10: a delimiter never closed is text.
  - 4: jig's `client/domain/client.md`, section 14: `turn_text`'s order, an outcome taken before its input. `../client/README.md`, 5.4: 2048 bytes of a committed turn.
  - 5: `../client/README.md`, 3.5: "Older" at a conversation's head. `../client/README.md`, 5.4: a window of 16 lines, and "Earlier lines omitted N". jig's `client/domain/client.md`, section 10: a window says how many lines it left out.
  - 6: `skein/docs/design/ui.md`, 3.1 and section 10: a list role with no order or start.
- **Open:**
  - 7: the line's verbs; a turn's time.
