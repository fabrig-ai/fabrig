+++
id = "ls-forge-connection"
part = "lib-sites"
kind = "migration"
needs = ["lib-contract", "abort-connection"]
files = ["skein/crates/skein-forge-connection/**"]
effort = "medium"
minutes = 40
+++

## Goal

skein-forge-connection's inline earliest-deadline merges and its four pop loops move onto lib. (`skein/docs/design/draft/component-kit.md`, 3.4.) No behaviour
changes; the crate's tests pass unchanged, but for tests of a removed
private helper that lib's tests cover.

The copies:

- `src/component.rs` (near 547-585): `if earliest.is_none() || Some(time) < earliest { earliest = Some(time); }` and its `&&` variants → `earliest = skein_lib::earliest(earliest, Some(time))` (keep each condition that guards the merge).
- `src/connection.rs` (two), `src/installations.rs` (one), `src/tests.rs` (one): pop loops → `clear()`.
- Any other inline merge of two optional deadlines in the crate (`rg -n 'is_none\(\) \|\|' src`).

Acceptance: each copy above is gone or reads through lib as stated; the
counts of `notes/counts.md` it touches fall by what it removed; clippy and
the crate's tests pass.

## Inputs

- `skein/docs/plan/component-kit-3/notes/common.md`,
  `skein/docs/plan/component-kit-3/notes/lib-sites.md` (the table of lib's
  items, how to move, and the checks), `skein/docs/plan/component-kit-3/notes/counts.md`.
- lib's items as merged: `skein/crates/skein-lib/src/{time,queue,stack,lifecycle,slab,hex,decimal}.rs`.

## Owns and may change

- Owns `skein/crates/skein-forge-connection/**`.
- Nothing outside them: no public item of the crate changes.
- Pre-approved:
  - Only the copies named here; a copy of the same helper found elsewhere in the owned crates moves too, and the report names it.
  - The local variable named `earliest` shadows lib's function: rename the variable (`next`), not lib's item.

## Steps

1. Move each copy as `notes/lib-sites.md` says; remove what is left dead.
2. `python3 -I -B scripts/old_forms.py --write` from `skein/`.
3. `cargo fmt`; `heavy cargo clippy -p skein-forge-connection --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-forge-connection`. Commit and push.

## Report

As `notes/lib-sites.md` says.
