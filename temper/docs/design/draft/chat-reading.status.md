# chat-reading: status

2026-11-06. Where the design lags or departs from [chat-reading.md](chat-reading.md).

- **Design lags:**
  - 2: `../client/README.md`, 3.5: a chat's calls are folded to "N tool calls" a turn, and its work lines are drawn from those. jig's `client/domain/view.md`, section 9: each call a folded line naming its tool.
  - 5: `../client/README.md`, 3.5: "Older" at a conversation's head. `../client/README.md`, 5.4: a window of 16 lines, and "Earlier lines omitted N". jig's `client/domain/client.md`, section 10: a window says how many lines it left out.
  - 6: `skein/docs/design/ui.md`, 3.1 and section 10: a list role with no order or start.
- **Design departs:**
  - 3: *Drawn once:* `skein/docs/design/ui.md`, section 10, and `../client/README.md`, 3.5: the page reads the whole text again at each delivery, and a closed block's nodes come out the same, so it stays on the page unchanged. Nothing keeps a block read. The whole tree is rebuilt at each delivery as it is for any change, and a block kept apart would be a new mechanism for no sight of difference. The plan `chat-reading-a` decided, under its request's "take the nearest thing".
  - 3: *Whole:* an answer past the engine's retained bytes (`views.text_bytes`, 65,536) arrives whole only to a page following it from its start; a page opened later shows the retained end until the turn commits. One delivery's wire field stops at the same 65,536, so holding more needs a chunked delivery. The plan `chat-reading-a` decided, likewise.
  - 4: *The order changes:* jig's `client/domain/client.md`, section 14: an outcome's line comes before inputs, not after them. A note's input can fill the budget, and the chat's document line and "Show" read that call's outcome. An outcome's line is bounded by `text`, so it costs the input little. The plan `chat-reading-a` decided, under its request's "an outcome's first line breaks a card".
  - 4: *A delegate's card:* jig's `client/domain/client.md`, section 10: every result's summary stays whole, priced last, not only those a card reads: the transcript knows no tool's words, and a whole summary is what the agent's page, the tools and a brief read. The plan `chat-reading-a` decided, likewise.
  - 4: *Whatever their results weigh:* jig's `client/domain/client.md`, section 10: a turn past `document_bytes` on the wire (1,048,576 bytes) reads as too large, with no calls' names, as it does today. Reading it needs a page past the client's document bound. The plan `chat-reading-a` decided, under its request's "take the nearest thing".
- **Open:**
  - 7: the line's verbs; a turn's time.
