+++
id = "ls-oauth-accounts"
part = "lib-sites"
kind = "migration"
needs = ["lib-contract"]
files = ["skein/crates/skein-oauth-accounts/**"]
effort = "high"
minutes = 60
+++

## Goal

skein-oauth-accounts' private copies move onto lib: its `earlier`, its `enum Lifecycle`, its nine pop loops, its `hex`, and the walks over its exchange slab. (`skein/docs/design/draft/component-kit.md`, 3.4.) No behaviour
changes; the crate's tests pass unchanged, but for tests of a removed
private helper that lib's tests cover.

The copies:

- `src/component.rs`: `fn earlier(earliest: &mut Option<Time>, next: Time)` → `skein_lib::earliest`.
- `src/component.rs`: `enum Lifecycle` → `skein_lib::Lifecycle` (`close()`, `abort()` where the transitions are hand-written).
- `src/component.rs` (four) and `src/listener.rs` (five): `for _ in 0..q.len() { drop(q.pop()); }` → `q.clear()`.
- `src/redirect.rs`: `fn hex(byte: u8) -> Option<u8>` (either case) → `skein_lib::hex::value`.
- `src/component.rs`: the `while let Some(id) = next` walks over `exchanges` through `first_exchange` and each exchange's `next` (lines near 544, 1002, 1020, 1034, 1136, 1171) → `Slab::iter`/`iter_mut`, and the `next`/`first_exchange` links go when no walk needs them.

Acceptance: each copy above is gone or reads through lib as stated; the
counts of `notes/counts.md` it touches fall by what it removed; clippy and
the crate's tests pass.

## Inputs

- `skein/docs/plan/component-kit-3/notes/common.md`,
  `skein/docs/plan/component-kit-3/notes/lib-sites.md` (the table of lib's
  items, how to move, and the checks), `skein/docs/plan/component-kit-3/notes/counts.md`.
- lib's items as merged: `skein/crates/skein-lib/src/{time,queue,stack,lifecycle,slab,hex,decimal}.rs`.
- `skein/docs/design/oauth.md` for the accounts component's exchanges.

## Owns and may change

- Owns `skein/crates/skein-oauth-accounts/**`.
- Nothing outside them: no public item of the crate changes.
- Pre-approved:
  - Only the copies named here; a copy of the same helper found elsewhere in the owned crates moves too, and the report names it.
  - The exchange list keeps insertion order. Read each walk: one that stops at the first match (an abort that fails one exchange and returns, a collect) observes the order. If a test or `skein/docs/design/oauth.md` depends on the order, keep the links for those walks and move the rest; if none does, move all and drop the links. Say which in the report.
  - If dropping the links changes `state_digest` (the slab's hash already covers entries), only the digest changes, not behaviour: fine.

## Steps

1. Move each copy as `notes/lib-sites.md` says; remove what is left dead.
2. `python3 -I -B scripts/old_forms.py --write` from `skein/`.
3. `cargo fmt`; `heavy cargo clippy -p skein-oauth-accounts --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-oauth-accounts`. Commit and push.

## Report

As `notes/lib-sites.md` says.
