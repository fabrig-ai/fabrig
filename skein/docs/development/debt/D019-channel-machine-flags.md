# D019: The channel machine keeps its lifecycle in eleven bools

- **Kind:** complexity
- **Where:** `skein-channel/src/machine.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T4, the channel row) and `reviews/2026-10-31-skein-areas.md` (A6)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 2)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** S
- **Status:** open
- **Paid when:** the channel `Machine`'s lifecycle is an enum whose variants hold their data, and going closed is written once.
- **Measure:** `grep -c ': bool,' skein/crates/skein-channel/src/machine.rs`: 11 at base

## What

`Machine` holds 11 `bool` fields beside its state, and the first review
found "go closed" written about five times.

## Why it matters

`skein/docs/foundation/programming-model.md`, 5.4: no lifecycle spread
over flags beside the state. jig's worker link (`jig-protocol-worker-link`) runs on it.

## What pays it back

Recast the machine's phases as data-carrying variants with one close
path. The channel world's checks (`skein/tests/channel`) stay as they
are.
