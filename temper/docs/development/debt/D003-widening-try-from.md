# D003: Lengths and counts are widened with `try_from` beside skein-lib's helpers

- **Kind:** primitives
- **Where:** `temper-engine-domain`, `temper-engine-domain-forge`, `temper-service`, and 16 more temper crates and worlds
- **Seen:** 2026-10-31 at `4a6683c72`, in the skein review of that day (`skein/docs/development/debt/reviews/2026-10-31-skein.md`), theme T1's table ("`u64::try_from(len)` vs `len_u64`: temper 222 vs 0")
- **Checked:** 2026-11-03 at `a6a8155d35`
- **Value:** low (shorter lines, one spelling of an infallible widening, no impossible refusal paths)
- **Size:** M (mechanical, but across about 20 crates)
- **Status:** open
- **Paid when:** temper's hand-written code widens a length with `skein_lib::length::len_u64` and a `u32` count with `skein_lib::length::usize_of`: the measure is 0, and no hand-written `usize::try_from(..).expect("u32 fits usize")` is left.
- **Measure:** `rg -c --type rust -g '!**/generated/**' 'u64::try_from\([^)]*\.len\(\)\)' temper | awk -F: '{s+=$2} END {print s}'`: 238

## What

skein-lib has both widenings, "so no step writes `try_from` for either"
(`skein/docs/design/lib.md`, 6): `length::len_u64` for a native length
and `length::usize_of` for a `u32` count. temper calls `usize_of`
nowhere and `len_u64` 5 times, all in code added since the first count
(`temper-engine-domain/src/room.rs`, `temper-engine-smith/src/room.rs`).
Its hand-written code has 238 `u64::try_from(x.len())` (107 in
`temper-engine-domain`, 54 in `temper-engine-domain-forge`, 23 in
`temper-service`), and 113 `usize::try_from(..).expect("u32 fits usize")`
among its 981 hand-written `usize::try_from`. jig adopted `usize_of`
(1,007 calls) and smith `len_u64` (119); temper has not. The generated
codecs' own conversions are skein's codegen, not this item.

## Why it matters

Each site spells an infallible widening as a fallible one, with its own
`expect` text or, worse, a refusal path that can never be taken. The
form is copied into every new step.

## What pays it back

A mechanical sweep, crate by crate, replacing the two patterns by the
helpers; one task per few crates, each with its crate's tests. It goes
well with D004 (the same crates, the same kind of change). skein's and
jig's sites are their trees' items
(`skein/docs/development/debt/D002-index-and-length-widening.md`,
`jig/docs/development/debt/D043-length-widening.md`); the helpers exist,
so this item waits on neither.
