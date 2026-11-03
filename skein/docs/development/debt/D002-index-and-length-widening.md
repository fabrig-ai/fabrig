# D002: Index and length conversions are written by hand beside lib's helpers

- **Kind:** primitives
- **Where:** every step crate; `skein-lib/src/length.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T1, the widening and `len_u64` rows) and `reviews/2026-10-31-skein-areas.md` (A0, F11)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** M
- **Status:** open
- **Paid when:** skein's step crates widen a `u32` with `skein_lib::usize_of` and measure a native length with `skein_lib::len_u64`, never with `usize::try_from(..).expect(..)` or `u64::try_from(x.len())`, outside lib's own two helpers.
- **Measure:** `rg -o 'usize::try_from\(.*\)\.expect\(|u64::try_from\([^)]*\.len\(\)\)' skein/crates skein/examples -g '!**/tests/**' -g '!**/tests.rs' -g '!*_tests.rs' | wc -l`: 603 at base

## What

- lib has both helpers (`skein-lib/src/length.rs`): `len_u64` and,
  since 10-31, `usize_of` for a `u32` count used as an index.
- skein's step code still writes 531 single-line
  `usize::try_from(..).expect(..)` outside tests, 327 of them with a
  "u32 fits" message, against 77 calls of `usize_of`.
- It writes `u64::try_from(x.len())` 72 times, against 154 calls of
  `len_u64`.
- The count above holds a few conversions from `u64`, which `usize_of`
  does not cover; they stay, and the count's floor is above 0.

## Why it matters

The two invariants (a `u32` fits a `usize`, a native length fits a
`u64`) are stated once in lib and then restated by hand, with a dozen
messages for one fact. The copies are what jig and temper imitate: they
call `len_u64` nowhere.

## What pays it back

A mechanical migration, crate by crate in crate-graph order, as the
component kit did for its forms (`skein/docs/design/draft/component-kit.md`,
4): each task moves one crate and lowers the count. A form in
`skein/scripts/old_forms.py` would hold it at its floor. jig's and
temper's sites are their trees' items,
`jig/docs/development/debt/D043-length-widening.md` and
`temper/docs/development/debt/D003-widening-try-from.md`; the helpers
exist, so neither waits on this one.
