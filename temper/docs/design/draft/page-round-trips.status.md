# page-round-trips: status

2026-11-17. Where the design lags or departs from [page-round-trips.md](page-round-trips.md).

- **Design lags:**
  - 2: no design document holds a page to a number of round trips; `../client/pages/work.md`, section 1, reads a report per goal row, one at a time, a task per row of the changes' queues and landings, the chats' documents' notes, and each drawn milestone's own reads (`../client/pages/milestone.md`, section 1).
  - 3: jig's `protocol/client/default-http.md`, section 1, carries one message per `POST`, with no batch route, and section 4 admits each as its own call; jig's `protocol/client/wire.md`, section 2, has a reload send its predecessor's stream gone and wait for its answer before opening its own.
  - 4: jig's `client/domain/contract/README.md`, section 9, leaves bundling parts open, and its 4.1's rows carry no part; jig's `domain/engine/views.md`, section 3, moves a task's last activity for words, turns and delegates' endings, not for a note; `../client/forge/records.md`, *A project's changes*, names no change's goal, chat or milestone, and its landings name their signers by party alone.
  - 5: a tasks watch carries no mark of its ended group (jig's `client/domain/contract/README.md`, 4.1).
  - 6: `../client/pages/work.md`, section 1, draws no heading until the four reads have answered, and its sections with it.
- **Design departs:**
  - 5, *a setting*: a setting is read again too where the `settings` watch names no revision while its read named one (`../client/pages/settings.md`, 1, *What the client holds*), since a setting reset to its default keeps no revision in jig's settings; R3's designer, goal 6236, 2026-11-17, so that a reset is never missed.
  - 5, *the needs' selection*: the inbox's mark is the rail's standing `inbox` watch delivering again, which the engine does only once it has marked the party's inbox changed; no number crosses the wire for it (`../client/pages/work.md`, 1). So a needs' change between the selection's read and that watch's admission waits for its next move, or for `me`'s `needs_total`; R3's designer, goal 6236, 2026-11-17, since a party's inbox keeps no revision to compare and a mark of its own would be a new count in jig's core for one filter's race.
  - 8: R3 keeps jig's HTTP mapping's paths at `v1`, the stream's route gaining `version` (jig's `protocol/client/default-http.md`, 1 and 2), since a page of an earlier version must still find the stream's route to be refused by name; R3's designer, goal 6236, 2026-11-17.
- **Open:**
  - 10: Home's budget and the inbox's cards, a draft of their own.
