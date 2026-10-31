+++
id = "ls-fake-forge"
part = "lib-sites"
kind = "migration"
needs = ["lib-contract"]
files = ["skein/crates/skein-fake-forge-github/**", "skein/crates/skein-fake-forge-protocol/**", "skein/crates/skein-fake-forge-domain/**"]
effort = "medium"
minutes = 35
+++

## Goal

The fake forge's pop loops and its hex-digit writer move onto lib. (`skein/docs/design/draft/component-kit.md`, 3.4.) No behaviour
changes; the crate's tests pass unchanged, but for tests of a removed
private helper that lib's tests cover.

The copies:

- `skein-fake-forge-github/src/server.rs` (two) and `skein-fake-forge-protocol/src/server.rs` (six): pop loops → `clear()`.
- `skein-fake-forge-domain/src/promises.rs`: `fn hex(nibble: u8) -> u8` writes UPPERCASE: → `skein_lib::hex::digit(nibble).to_ascii_uppercase()` at its callers, or keep the case another way lib allows.

Acceptance: each copy above is gone or reads through lib as stated; the
counts of `notes/counts.md` it touches fall by what it removed; clippy and
the crate's tests pass.

## Inputs

- `skein/docs/plan/component-kit-3/notes/common.md`,
  `skein/docs/plan/component-kit-3/notes/lib-sites.md` (the table of lib's
  items, how to move, and the checks), `skein/docs/plan/component-kit-3/notes/counts.md`.
- lib's items as merged: `skein/crates/skein-lib/src/{time,queue,stack,lifecycle,slab,hex,decimal}.rs`.

## Owns and may change

- Owns `skein/crates/skein-fake-forge-github/**`, `skein/crates/skein-fake-forge-protocol/**`, `skein/crates/skein-fake-forge-domain/**`.
- Nothing outside them: no public item of the crate changes.
- Pre-approved:
  - Only the copies named here; a copy of the same helper found elsewhere in the owned crates moves too, and the report names it.
  - `skein-fake-forge-domain/src/actions.rs`'s and `skein-fake-forge-github/src/rates.rs`'s `decimal` already call `Decimal::of`: they stay.
  - `skein-fake-forge-github/src/pages.rs`'s `hex(rng, length)` (a random secret) stays; its digits may come from `hex::digit`.

## Steps

1. Move each copy as `notes/lib-sites.md` says; remove what is left dead.
2. `python3 -I -B scripts/old_forms.py --write` from `skein/`.
3. `cargo fmt`; `heavy cargo clippy -p skein-fake-forge-github -p skein-fake-forge-protocol -p skein-fake-forge-domain --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-fake-forge-github -p skein-fake-forge-protocol -p skein-fake-forge-domain`. Commit and push.

## Report

As `notes/lib-sites.md` says.
