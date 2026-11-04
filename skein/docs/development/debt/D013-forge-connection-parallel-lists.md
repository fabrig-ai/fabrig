# D013: forge-connection keeps forges in parallel lists and dialects as runtime tags

- **Kind:** complexity
- **Where:** `skein-forge-connection/src/component.rs`, `skein-forge-connection/src/component`, `skein-forge-connection/src/connection.rs`, `skein-forge-connection/src/calls.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T2, the forge-connection row) and `reviews/2026-10-31-skein-areas.md` (A3, F2 and F3)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 3)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** high
- **Size:** M
- **Status:** open
- **Paid when:** the component holds one entry per forge (settings, start, dialect state, own token and plan together), with the dialect as an enum whose variants hold their plans; no `unreachable!` in non-test code names a dialect.
- **Measure:** `rg -c '\.get(_mut)?\(at\)' skein/crates/skein-forge-connection/src -g '!**/tests/**' | awk -F: '{s+=$2} END{print s}'`: 155 at base

## What

- `Component` keeps per-forge state in five lists indexed by position
  (`settings.forges`, `starts`, `github`, `own_tokens`, `own_plans`):
  155 `get(at)`/`get_mut(at)` accesses outside tests.
- Two-variant enums wrap the dialects (`CallPlan`, `OwnPlan`, and in
  `connection.rs` `Exchange`, `Heard`, `Refusal`, `Request`, `Decoder`,
  `Body`): 40 of the 54 non-test `unreachable!`s name a dialect.
- `grow::Tail` (`skein-forge-connection/src/grow.rs`) re-implements
  lib's `Tail`.

## Why it matters

Every method takes a position and re-finds the forge in five lists;
removing a forge swap-removes across all of them. The dialect, known
when a forge is added, is checked again at each use, and a mismatch is
a panic rather than a type error.

## What pays it back

`List<ForgeEntry>` with `enum Dialect { Forgejo { .. }, GitHub(Box<..>) }`,
passing `&mut ForgeEntry`; lib's `Tail` in place of `grow::Tail`. It
goes better after D010, and pairs with D020.
