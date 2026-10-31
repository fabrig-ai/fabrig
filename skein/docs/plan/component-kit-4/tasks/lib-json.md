+++
id = "lib-json"
part = "lib-sites"
kind = "migration"
needs = ["lib-kit", "counts"]
files = ["skein/crates/skein-json/src/**"]
effort = "medium"
minutes = 25
+++

## Goal

skein-json's writer's hex digit and its string reader's escape digit move onto lib's primitives
(`skein/docs/design/draft/component-kit.md, 3.4`): each private copy
below goes, its callers calling lib, with no change of behaviour.

Acceptance:

- The copies below are gone, or kept for a reason `notes/migrations.md`
  allows (one that does more than the lib item), named in the report.
- Counts fall: `hex_decimal` by 1 or 2; none rises.
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
  - `crates/skein-json/src/writer.rs`: `fn hex(value) -> u8` (`hex::digit`);
  - `crates/skein-json/src/string.rs`: `fn hex(unit: u16, byte: u8) -> Result<u16, Error>` (folds one digit of a `\u` escape: its digit value is `hex::value`; it may stay as a fold, its body using lib).

## Owns and may change

- Owns `skein/crates/skein-json/src/`.
- `skein/tests/json/src/reference.rs` has its own `fn hex`: it is the independent reference decoder the world checks skein-json against, and stays independent: do not touch it.

## Steps

1. Move each copy, the narrowest test of its package after each: read the
   copy's body against the lib item's doc first; where they differ on an
   edge (case, empty input, overflow), keep the site's edge (common.md,
   "Behaviour stays").
   Run the JSON world too: `heavy cargo nextest run -p skein-json-world`.
2. `cargo fmt`; `heavy cargo clippy -p skein-json --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-json`; the dependents' checks.
3. `old_forms.py --write`; commit and push.

## Report

As common.md says, with each copy: removed, or kept and why.
