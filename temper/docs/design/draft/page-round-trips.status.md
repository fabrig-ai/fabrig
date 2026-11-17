# page-round-trips: status

2026-11-17. Where the design lags or departs from [page-round-trips.md](page-round-trips.md).

- **Design lags:**
  - 2: no design document holds a page to a number of round trips.
  - 3: jig's `protocol/client/default-http.md`, section 1, carries one message per `POST`, with no batch route, and section 4 admits each as its own call; jig's `protocol/client/wire.md`, section 2, has a reload send its predecessor's stream gone and wait for its answer before opening its own.
  - 5: jig's `protocol/client/README.md`, section 1, opens the stream with a hello of the engine's versions and its heartbeat alone, so the client opens a watch of `me` and waits for its value before the frame's and the page's reads; and a setting's read answers with no revision.
  - 6: `../client/pages/work.md`, section 1, draws no heading until the four reads have answered, and its sections with it.
- **Open:**
  - 10: Home's budget and the inbox's cards, a draft of their own.
