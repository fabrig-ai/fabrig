+++
id = "lib-forge"
part = "lib-sites"
kind = "migration"
needs = ["lib-kit", "counts"]
files = ["skein/crates/skein-forge/src/**", "skein/tests/forge/client/src/translate.rs"]
effort = "high"
minutes = 35
+++

## Goal

skein-forge's three hex digit copies and the forge-client world's hex value move onto lib's primitives
(`skein/docs/design/draft/component-kit.md, 3.4`): each private copy
below goes, its callers calling lib, with no change of behaviour.

Acceptance:

- The copies below are gone, or kept for a reason `notes/migrations.md`
  allows (one that does more than the lib item), named in the report.
- Counts fall: `hex_decimal` by 4; none rises.
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
  - `crates/skein-forge/src/forgejo/id.rs`: `pub(crate) fn hex(value) -> u8`;
  - `crates/skein-forge/src/github/id.rs`: `fn hex(byte) -> u8`;
  - `crates/skein-forge/src/marker.rs`: `pub(crate) fn hex(byte) -> u8`;
  - `tests/forge/client/src/translate.rs`: `fn hex(byte) -> Option<u8>`.

## Owns and may change

- Owns `skein/crates/skein-forge/src/` and, in `skein/tests/forge/client/`, `src/translate.rs` only.
- skein-forge's other crate-private helpers that the GitHub dialect imports from Forgejo's modules (the review, theme 3) are not this task's.

## Steps

1. Move each copy, the narrowest test of its package after each: read the
   copy's body against the lib item's doc first; where they differ on an
   edge (case, empty input, overflow), keep the site's edge (common.md,
   "Behaviour stays").
2. `cargo fmt`; `heavy cargo clippy -p skein-forge -p skein-forge-client-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-forge -p skein-forge-client-world`; the dependents' checks.
3. `old_forms.py --write`; commit and push.

## Report

As common.md says, with each copy: removed, or kept and why.
