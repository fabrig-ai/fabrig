# page-round-trips: status

2026-11-17. Where the design lags or departs from [page-round-trips.md](page-round-trips.md).

- **Design lags:**
  - 2: no design document holds a page to a number of round trips; `../client/pages/work.md`, section 1, reads a report per goal row, one at a time, a task per row of the changes' queues and landings, the chats' documents' notes, and each drawn milestone's own reads (`../client/pages/milestone.md`, section 1).
  - 3: jig's `protocol/client/default-http.md`, section 1, carries one message per `POST`, with no batch route; section 4 admits each as its own call.
  - 4: jig's `client/domain/contract/README.md`, section 9, leaves bundling parts open, and its 4.1's rows carry no part; `../client/forge/records.md`, *A project's changes*, names no change's goal, chat or milestone.
  - 3, 5: jig's `protocol/client/README.md`, section 1, opens the stream with a hello of the engine's versions and its heartbeat alone, so the client opens a watch of `me` and waits for its value before the frame's and the page's reads; a reload sends `stream-gone` and waits for its answer before the stream (jig's `protocol/client/default-http.md`, section 1).
  - 6: `../client/pages/work.md`, section 1, draws no heading until the four reads have answered, and its sections with it.
- **Open:**
  - 10: Home's budget and the inbox's cards, a draft of their own.
