+++
id = "lib-forge-setup"
part = "lib-sites"
kind = "migration"
needs = ["lib-kit", "counts", "abort-setup"]
files = ["skein/crates/skein-forge-setup/src/**"]
effort = "high"
minutes = 40
+++

## Goal

skein-forge-setup's deadline merge, its hex and decimal copies and its host's ending move onto lib's primitives
(`skein/docs/design/draft/component-kit.md, 3.4`): each private copy
below goes, its callers calling lib, with no change of behaviour.

Acceptance:

- The copies below are gone, or kept for a reason `notes/migrations.md`
  allows, named in the report.
- Counts fall: `earliest` by 1, `hex_decimal` by 3 or 4; none rises.
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
  - `src/host.rs`: `fn earlier(&mut Option<Time>, Time)` (`skein_lib::earliest`);
  - `src/github/app.rs`: `pub(crate) fn decimal(id: u64) -> Box<[u8]>` (`Decimal::of`);
  - `src/github/manifest.rs` and `src/host/states.rs`: `fn hex(value) -> u8` (`hex::digit`);
  - `src/reconcile.rs`: `pub(crate) fn hex(&Secret) -> Secret` hex-encodes a whole secret: it may stay, its digits from `hex::digit`;
  - the host's private ending enum task abort-setup added (`skein_lib::Lifecycle`, if its variants are live, closing, aborting, closed; otherwise keep it and say why).

## Owns and may change

- Owns `skein/crates/skein-forge-setup/src/`.
- forge-setup's `Reply` and `Plan` shapes (the review, theme 3) are not this task's.

## Steps

1. Move each copy, the narrowest test of its package after each: read the
   copy's body against the lib item's doc first; where they differ on an
   edge (case, empty input, overflow), keep the site's edge (common.md,
   "Behaviour stays").

2. `cargo fmt`; `heavy cargo clippy -p skein-forge-setup -p skein-forge-setup-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-forge-setup -p skein-forge-setup-world`; the dependents' checks.
3. `old_forms.py --write`; commit and push.

## Report

As common.md says, with each copy: removed, or kept and why.
