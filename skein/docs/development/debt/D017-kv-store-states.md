# D017: skein-kv's store keeps its lifecycle in flags, with the subset switched off

- **Kind:** complexity
- **Where:** `skein-kv/src/store.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T4, the skein-kv row; T9 item 5) and `reviews/2026-10-31-skein-areas.md` (A12)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 2)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** high
- **Size:** M
- **Status:** open
- **Paid when:** `Store`'s phases are enum variants that hold their data (`skein/docs/foundation/programming-model.md`, 5.4); its matches are per state and event with no catch-all arm; `store.rs` has no module-wide `#![expect]`.
- **Measure:** `grep -c '^#!\[expect' skein/crates/skein-kv/src/store.rs`: 5 at base

## What

`store.rs` (1,093 lines) holds 6 `bool` and 8 `Option` fields beside
its `phase`, matches `(phase, event)` tuples with a catch-all
(`(_, _) => unreachable!("snapshot event matches its request")`), uses
`loop`/`while`, and opts out of five subset lints for the whole module.
No status file records the departure (the foundation has none).

## Why it matters

This is the durable store's recovery logic, the code where a missed
state costs data, and the one step crate with its rules switched off
wholesale.

## What pays it back

Recast the store as a 5.4 machine, one state at a time, under the
existing crash and golden tests (`skein/tests/kv`). Alternatively the
designer records skein-kv as ordinary Rust; that choice is the
user's. The O(n²) copy of queued operations on each commit goes with it.
