# D024: The echo example, the template applications copy, keeps its stop in flags

- **Kind:** complexity
- **Where:** `skein/examples/echo/protocol/src/layer.rs`, `skein/examples/echo/protocol/src/conn.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T4, the echo row, and phase 4's order) and `reviews/2026-10-31-skein-areas.md` (A13)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** high
- **Size:** S
- **Status:** open
- **Paid when:** the echo protocol layer's shutdown is part of its state (no `stopping` or `aborting` flag beside it), and the connection's `ended`, `closed` and `call` flags are states or gone.

## What

The echo protocol layer holds `stopping: bool` and `aborting: bool`
beside its state (`layer.rs`), and its connection holds `ended`,
`closed` and `call` flags (`conn.rs`). The rest of the echo is the
tree's cleanest model of a step machine.

## Why it matters

Applications start from the echo. A flag-based stop there is copied
into every service written from it, which is how the departure spread.

## What pays it back

Move the layer's stop into its state first, before the other 5.4 items
(D017 to D023), so they have the template to follow.
