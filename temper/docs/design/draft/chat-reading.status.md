# chat-reading: status

2026-11-06. Where the design lags or departs from [chat-reading.md](chat-reading.md).

- **Design lags:**
  - 2: `../client/README.md`, 3.5: a run's time is the committed turns' alone, so a run of one turn has none; its table holds `read_docs` and `settings` first (section 7). An agent's page has no running run or Latest pill; `chat-reading` (b) owns both with the broader table and one-turn time. jig's `client/domain/view.md`, section 9: each call a folded line naming its tool.
  - 3: `../client/README.md`, 3.5 and 5.4: provisional text is bounded by `text`, 512 bytes. jig's `client/domain/client.md`, 8.3: the bound on a provisional turn's bytes. `skein/docs/design/ui.md`, section 10: a delimiter never closed is text. The closed fenced block keeps its final newline, so the end’s mark falls on the following line; `chat-reading-a`’s `chat-live` and `read_growing` own it.
  - 4: jig's `client/domain/client.md`, section 14: `turn_text`'s order, an outcome taken before its input. `../client/README.md`, 5.4: 2048 bytes of a committed turn.
  - 5: `../client/README.md`, 3.5: "Older" at a conversation's head, kept until a log's head says it is near, the one way to earlier lines (`pages/chat.rs`, `older`), with the loading status only while that read is out, rather than when the head nears; "Latest" reads one page where the window let the end go, and isn't pressed by sending. `../client/README.md`, 5.4: a window of 16 lines, no head-near act, no PgUp/PgDn from the composer and no guarantee of position across prepending. jig's `client/domain/client.md`, section 10: a window still accounts for lines it left out; the view draws none.
  - 6: `skein/docs/design/ui.md`, 3.1 and section 10: a list role with no order or start.
- **Open:**
  - 7: the line's verbs, to be read by the user on a real chat; a turn's time.
