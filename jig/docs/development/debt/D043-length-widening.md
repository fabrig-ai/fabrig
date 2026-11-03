# D043: Native lengths are widened with `try_from` beside skein-lib's `len_u64`

- **Kind:** primitives
- **Where:** `jig-domain-people`, `jig-domain-core`, `jig-client-protocol-link`, `jig-protocol-store`, and the other crates the measure finds
- **Seen:** 2026-11-03 at `a6a8155d35`, while reconciling the four trees' backfills: skein's `skein/docs/development/debt/D002-index-and-length-widening.md` says jig calls `len_u64` nowhere (new here)
- **Checked:** 2026-11-03 at `a6a8155d35`
- **Value:** low (one spelling of an infallible widening, no refusal paths that cannot be taken)
- **Size:** S
- **Status:** open
- **Paid when:** jig's hand-written code widens a native length with `skein_lib::length::len_u64`; the measure is 0.
- **Measure:** `rg -c --type rust -g '!**/generated/**' 'u64::try_from\([^)]*\.len\(\)\)' jig | awk -F: '{s+=$2} END {print s}'`: 242

## What

skein-lib widens a native length with `length::len_u64`
(`skein/docs/design/lib.md`, 6), so that no step writes `try_from` for it. jig adopted
`usize_of` (about 1,000 calls) but calls `len_u64` nowhere. Its hand-written code has 242
`u64::try_from(x.len())`, 179 of them outside tests; `jig-domain-people/src/amendment.rs`
alone has 16, each as `u64::try_from(..).ok()?`, a refusal path that cannot be taken.

## Why it matters

Each site spells an infallible widening as a fallible one, and new code copies the form.
skein's and temper's sites are their own items (skein's D002, temper's
`temper/docs/development/debt/D003-widening-try-from.md`).

## What pays it back

A mechanical sweep, crate by crate, with each crate's tests. It does not wait on skein's
D002: the helper exists.
