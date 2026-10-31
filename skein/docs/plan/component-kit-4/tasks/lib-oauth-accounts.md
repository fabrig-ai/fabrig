+++
id = "lib-oauth-accounts"
part = "lib-sites"
kind = "migration"
needs = ["lib-kit", "counts"]
files = ["skein/crates/skein-oauth-accounts/src/**"]
effort = "high"
minutes = 60
+++

## Goal

skein-oauth-accounts' lifecycle, deadline merge, drain loops, hex value and walks over its exchanges move onto lib's primitives
(`skein/docs/design/draft/component-kit.md, 3.4`): each private copy
below goes, its callers calling lib, with no change of behaviour.

Acceptance:

- The copies below are gone, or kept for a reason `notes/migrations.md`
  allows, named in the report.
- Counts fall: `lifecycle` by 1, `earliest` by 1, `pop_loop` by 9, `hex_decimal` by 1; none rises.
- The tests of every package you own pass, and its direct dependents
  within skein check (`heavy cargo check --tests -p ...`; `cargo tree -i
  <package> --workspace -e normal,dev --depth 1` finds them).

## Inputs

- `skein/docs/plan/component-kit-4/notes/common.md` and
  `notes/migrations.md` (lib's primitives).
- lib's items, their docs and tests: `skein/crates/skein-lib/src/`
  (`time.rs`'s `earliest`, `lifecycle.rs`, `hex.rs`, `decimal.rs`'s
  `Decimal::parse`, `queue.rs`/`stack.rs`'s `clear`, `slab.rs`'s
  iterators).
- The copies (find each with `rg`; lines drift):
  - `src/component.rs`: `enum Lifecycle` (`skein_lib::Lifecycle`); `fn earlier(&mut Option<Time>, Time)` (`skein_lib::earliest`); four `for _ in 0..exchange.above.len() { pop }` loops (`clear`);
  - `src/listener.rs`: five such loops (`clear`);
  - `src/redirect.rs`: `fn hex(byte) -> Option<u8>` (`hex::value`);
  - `src/component.rs`: `first_exchange` and `Exchange::next`, a list threaded through the `exchanges` slab, walked by `while let Some(id) = next` about nine times.

## Owns and may change

- Owns `skein/crates/skein-oauth-accounts/src/`.
- The threaded list keeps the exchanges in admission order; `Slab::iter` visits them in slot order, which differs once slots are reused. A walk moves onto `Slab::iter`/`iter_mut` only where its order cannot be seen: a predicate (`account_has_exchanges`), a count, a fold whose result does not depend on order. A walk that tells events, submits, or stops at the first match in admission order keeps the list. If after that every walk is a list walk, the list stays whole: say so.
- A walk that calls `&mut self` methods per exchange cannot hold the slab's iterator: keep it on the list.

## Steps

1. Move each copy, the narrowest test of its package after each: read the
   copy's body against the lib item's doc first; where they differ on an
   edge (case, empty input, overflow), keep the site's edge (common.md,
   "Behaviour stays").
   Run the oauth-accounts world after each walk you move: its seeded
   stories see the order of events.
2. `cargo fmt`; `heavy cargo clippy -p skein-oauth-accounts -p skein-oauth-accounts-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-oauth-accounts -p skein-oauth-accounts-world`; the dependents' checks.
3. `old_forms.py --write`; commit and push.

## Report

As common.md says, with each copy: removed, or kept and why.
