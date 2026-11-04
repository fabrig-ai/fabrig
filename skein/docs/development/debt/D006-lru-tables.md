# D006: Least-recently-used tables are hand-rolled in both forge dialects

- **Kind:** primitives
- **Where:** `skein-forge/src/forgejo/plans/tables.rs`, `skein-forge/src/github/plans/tables.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T1, the LRU row) and `reviews/2026-10-31-skein-areas.md` (A1, F4)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 1d)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** S
- **Status:** open
- **Paid when:** skein-lib holds one bounded least-recently-used map, and both dialects' repository and user tables use it; the user table, the same in both, lives once at skein-forge's root.
- **Measure:** `rg -c 'fn tick\(&mut self\) -> u64' skein/crates | awk -F: '{s+=$2} END{print s}'`: 2 at base

## What

Each dialect's `plans/tables.rs` keeps two tables (repositories, user
logins) evicted by least recent use, with its own `tick` and eviction
loops: four hand-rolled LRU tables. The review found them drifted:
GitHub's tick saturates where Forgejo's expects, and only GitHub guards
a capacity of 0. Each copy has its own tests.

## Why it matters

`skein/docs/foundation/programming-model.md`, 10.2: application code
does not hand-roll data structures; what is missing goes into lib. Two
drifted copies of one rule answer the same question two ways.

## What pays it back

`skein_lib::Lru` (a concrete, priced container like lib's others), then
both tables on it, and the user table moved to the root. It is one of
the neutral pieces D010 moves.
