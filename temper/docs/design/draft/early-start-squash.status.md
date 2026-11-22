# early-start-squash: status

2026-11-17. Where the design lags or departs from [early-start-squash.md](early-start-squash.md).

The design carries the draft: section 2 in `../domain/forge/change/questions.md`, 1 (*Early starts*: *A running producer catches up*); `../domain/worker/checkouts.md` (*The landing pin*, *Push commits what the run checked*, *A save pushes unchecked* and *A landed dependency's catch-up*, with its open question *An earlier head lost*); `../protocol/git-operations.md`, 1.7; and `../protocol/channel.md`, section 1.

- **Design lags:** none.
- **Design departs:**
  - 2: the catch-up runs in the run's own hold, its pin fetched afresh, rather than in a fresh hold: the producer's unsaved work lives only in that hold's checkout, which a fresh hold would start clean. The goal's designer, 2026-11-17.
  - 2: a delivery that catches up cleanly pushes nothing and is answered so that the run delivers again, its checks then running on the merged tree, rather than delivering on its own: its checks ran on the tree before the merge. A save, which runs no check, goes on and pushes the merge. The goal's designer, 2026-11-17.
  - 4: the engine's stories run through the guard's engine half, the worker's `landed` ask, and push the head the checkout's catch-up makes: the engine's world runs no worker, and the checkout's half is the worker checkout world's. The goal's designer, 2026-11-17.
- **Open:** none.
