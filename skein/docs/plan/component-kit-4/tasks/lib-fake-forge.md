+++
id = "lib-fake-forge"
part = "lib-sites"
kind = "migration"
needs = ["lib-kit", "counts"]
files = ["skein/crates/skein-fake-forge-domain/src/**", "skein/crates/skein-fake-forge-github/src/**", "skein/crates/skein-fake-forge-protocol/src/**"]
effort = "high"
minutes = 45
+++

## Goal

The three fake-forge crates' digit writers and their drain loops (three crates in one task: each holds a few lines of change) move onto lib's primitives
(`skein/docs/design/draft/component-kit.md, 3.4`): each private copy
below goes, its callers calling lib, with no change of behaviour.

Acceptance:

- The copies below are gone, or kept for a reason `notes/migrations.md`
  allows, named in the report.
- Counts fall: `hex_decimal` by 3 or 4, `pop_loop` by 8; none rises.
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
  - `crates/skein-fake-forge-domain/src/actions.rs`: `fn decimal(out: &mut List<u8>, number: u64)` (`Decimal::of`);
  - `crates/skein-fake-forge-domain/src/promises.rs`: `fn hex(nibble) -> u8` (`hex::digit`);
  - `crates/skein-fake-forge-github/src/rates.rs`: `fn decimal(value: u64) -> Box<[u8]>` (`Decimal::of`);
  - `crates/skein-fake-forge-github/src/pages.rs`: `fn hex(rng, length) -> Option<Secret>` draws random digits: it may stay, its digits from `hex::digit`;
  - `crates/skein-fake-forge-github/src/server.rs`: two `for _ in 0..len { pop }` loops (`clear`);
  - `crates/skein-fake-forge-protocol/src/server.rs`: six such loops (`clear`).

## Owns and may change

- Owns the `src/` of `skein-fake-forge-domain`, `skein-fake-forge-github` and `skein-fake-forge-protocol`.
- The fakes' sharing of the real crates' codecs (the review, theme 5 and D1) is not this task's.

## Steps

1. Move each copy, the narrowest test of its package after each: read the
   copy's body against the lib item's doc first; where they differ on an
   edge (case, empty input, overflow), keep the site's edge (common.md,
   "Behaviour stays").
   Run the fake forge world too: `heavy cargo nextest run -p skein-fake-forge-tests`.
2. `cargo fmt`; `heavy cargo clippy -p skein-fake-forge-domain -p skein-fake-forge-github -p skein-fake-forge-protocol --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-fake-forge-domain -p skein-fake-forge-github -p skein-fake-forge-protocol`; the dependents' checks.
3. `old_forms.py --write`; commit and push.

## Report

As common.md says, with each copy: removed, or kept and why.
