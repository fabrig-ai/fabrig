# D003: `Env` is rebuilt by hand where `Env::with` does it

- **Kind:** primitives
- **Where:** `skein-lib/src/env.rs`; callers across skein's crates, tests and examples
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T1, the `Env::with` row) and `reviews/2026-10-31-skein-areas.md` (A0, F11)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** S
- **Status:** open
- **Paid when:** no skein code builds an `Env` from another `Env`'s `now` and `wall` by hand; each such site calls `Env::with`.
- **Measure:** `rg -c 'Env \{ now: [a-z_.]*now' skein -g '*.rs' | awk -F: '{s+=$2} END{print s}'`: 97 at base

## What

`Env::with` landed on 10-31 (`skein-lib/src/env.rs`): the same clocks
with another layer's limits. Its only caller is its own test
(`skein-lib/src/tests/env.rs`). skein still writes
`Env { now: env.now, wall: env.wall, limits: .. }` 97 times, 73 of them
under `crates/` and `examples/`.

## Why it matters

Each rewrap is three lines where one would do, and a reader has to
check that both clocks were carried. The review counted 162 more in jig
and 72 in temper. jig has since moved onto `Env::with` (7 left by this
item's measure, not worth an item).

## What pays it back

Replace the rewraps with `env.with(limits)`, crate by crate. A
search-and-replace with a review of each site is enough; nothing
depends on it.
