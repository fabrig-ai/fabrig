+++
id = "lib-llm-connection"
part = "lib-sites"
kind = "migration"
needs = ["lib-kit", "counts"]
files = ["skein/crates/skein-llm-connection/src/**"]
effort = "high"
minutes = 35
+++

## Goal

skein-llm-connection's lifecycle and deadline merge move onto lib's primitives
(`skein/docs/design/draft/component-kit.md, 3.4`): each private copy
below goes, its callers calling lib, with no change of behaviour.

Acceptance:

- The copies below are gone, or kept for a reason `notes/migrations.md`
  allows (one that does more than the lib item), named in the report.
- Counts fall: `lifecycle` by 1, `earliest` by 1; none rises.
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
  - `crates/skein-llm-connection/src/component.rs`: `enum Lifecycle` (`skein_lib::Lifecycle`);
  - `crates/skein-llm-connection/src/deadlines.rs`: `fn earlier(Option<Time>, Option<Time>) -> Option<Time>` (`skein_lib::earliest`), and any inline merge of two optional deadlines in the crate.

## Owns and may change

- Owns `skein/crates/skein-llm-connection/src/`.

## Steps

1. Move each copy, the narrowest test of its package after each: read the
   copy's body against the lib item's doc first; where they differ on an
   edge (case, empty input, overflow), keep the site's edge (common.md,
   "Behaviour stays").
2. `cargo fmt`; `heavy cargo clippy -p skein-llm-connection -p skein-llm-connection-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-llm-connection -p skein-llm-connection-world`; the dependents' checks.
3. `old_forms.py --write`; commit and push.

## Report

As common.md says, with each copy: removed, or kept and why.
