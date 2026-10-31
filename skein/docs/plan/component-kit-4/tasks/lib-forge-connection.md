+++
id = "lib-forge-connection"
part = "lib-sites"
kind = "migration"
needs = ["lib-kit", "counts", "abort-connection"]
files = ["skein/crates/skein-forge-connection/src/**"]
effort = "high"
minutes = 40
+++

## Goal

skein-forge-connection's drain loops, its inline deadline merges and its ending move onto lib's primitives
(`skein/docs/design/draft/component-kit.md, 3.4`): each private copy
below goes, its callers calling lib, with no change of behaviour.

Acceptance:

- The copies below are gone, or kept for a reason `notes/migrations.md`
  allows, named in the report.
- Counts fall: `pop_loop` by 3; none rises.
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
  - `src/connection.rs`: two `for _ in 0..len { pop }` loops; `src/installations.rs`: one (`clear`);
  - inline merges of an optional deadline: `src/component.rs` (around 547-562), `src/hooks.rs` (around 116), and any other `earliest.is_none() || Some(t) < earliest` (`skein_lib::earliest`);
  - the private live, closing, aborting, closed enum task abort-connection added (`skein_lib::Lifecycle`, if its variants are those four; otherwise keep it and say why).

## Owns and may change

- Owns `skein/crates/skein-forge-connection/src/`.

## Steps

1. Move each copy, the narrowest test of its package after each: read the
   copy's body against the lib item's doc first; where they differ on an
   edge (case, empty input, overflow), keep the site's edge (common.md,
   "Behaviour stays").

2. `cargo fmt`; `heavy cargo clippy -p skein-forge-connection -p skein-forge-machine-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-forge-connection -p skein-forge-machine-world`; the dependents' checks.
3. `old_forms.py --write`; commit and push.

## Report

As common.md says, with each copy: removed, or kept and why.
