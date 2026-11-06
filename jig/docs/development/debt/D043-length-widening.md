# D043: Native lengths are widened with `try_from` beside skein-lib's `len_u64`

- **Kind:** primitives
- **Where:** `jig/crates/jig-client-view/src/state/fields.rs`, `jig/crates/jig-client-view/src/state/store.rs`, `jig/crates/jig-client-view/src/tests/tree/fragments/activity.rs` and `jig/crates/jig-client-view/src/tests/tree/fragments/history.rs`
- **Seen:** 2026-11-03 at `a6a8155d35`, while reconciling the four trees' backfills: skein's item on widening said jig calls `len_u64` nowhere (new here)
- **Checked:** 2026-11-06
- **Value:** low (one spelling of an infallible widening, no refusal paths that cannot be taken)
- **Size:** S
- **Status:** open
- **Paid when:** jig's hand-written code widens a native length with `skein_lib::length::len_u64`; the measure is 0.
- **Measure:** `rg -c --type rust -g '!**/generated/**' 'u64::try_from\(.*\.len\(\)' jig | awk -F: '{s+=$2} END {print s}'`: 4

## What

skein-lib widens a native length with `length::len_u64`
(`skein/docs/design/lib.md`, 6), so that no step writes `try_from` for it. jig's
hand-written code retains 4 such calls, all in `jig-client-view`: two in production,
`u64::try_from(text.bytes().len())` in `state/fields.rs` and
`u64::try_from(words.bytes().len())` in `state/store.rs`, and two in the view's fragment
tests named above. The measure's `[^)]*` used to hide the two in production, whose
argument holds a call of its own. What blocks them: fabrig/ui holds `jig-client-view`, so
no debt round edits it until it lets go.

## Why it matters

Each site spells an infallible widening as a fallible one, and new code copies the form.
skein's sites are its own item (skein's D002).

## What pays it back

A mechanical sweep, crate by crate, with each crate's tests. It does not wait on skein's
D002: the helper exists.
