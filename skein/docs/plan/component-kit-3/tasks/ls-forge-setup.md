+++
id = "ls-forge-setup"
part = "lib-sites"
kind = "migration"
needs = ["lib-contract", "abort-setup"]
files = ["skein/crates/skein-forge-setup/**"]
effort = "medium"
minutes = 35
+++

## Goal

skein-forge-setup's private `earlier` and its two hex-digit writers move onto lib. (`skein/docs/design/draft/component-kit.md`, 3.4.) No behaviour
changes; the crate's tests pass unchanged, but for tests of a removed
private helper that lib's tests cover.

The copies:

- `src/host.rs`: `fn earlier(earliest: &mut Option<Time>, time: Time)` → `skein_lib::earliest`.
- `src/github/manifest.rs` and `src/host/states.rs`: `fn hex(value: u8) -> u8` → `skein_lib::hex::digit` (check each copy's case; keep it).
- `src/reconcile.rs`: `pub(crate) fn hex(value: &Secret) -> Secret` stays (it writes a secret), but its body uses `hex::pair` or `hex::digit` in place of its digit table.

Acceptance: each copy above is gone or reads through lib as stated; the
counts of `notes/counts.md` it touches fall by what it removed; clippy and
the crate's tests pass.

## Inputs

- `skein/docs/plan/component-kit-3/notes/common.md`,
  `skein/docs/plan/component-kit-3/notes/lib-sites.md` (the table of lib's
  items, how to move, and the checks), `skein/docs/plan/component-kit-3/notes/counts.md`.
- lib's items as merged: `skein/crates/skein-lib/src/{time,queue,stack,lifecycle,slab,hex,decimal}.rs`.

## Owns and may change

- Owns `skein/crates/skein-forge-setup/**`.
- Nothing outside them: no public item of the crate changes.
- Pre-approved:
  - Only the copies named here; a copy of the same helper found elsewhere in the owned crates moves too, and the report names it.
  - `src/github/app.rs`'s `decimal(id) -> Box<[u8]>` already calls `Decimal::of`: it stays.

## Steps

1. Move each copy as `notes/lib-sites.md` says; remove what is left dead.
2. `python3 -I -B scripts/old_forms.py --write` from `skein/`.
3. `cargo fmt`; `heavy cargo clippy -p skein-forge-setup --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-forge-setup`. Commit and push.

## Report

As `notes/lib-sites.md` says.
