+++
id = "lib-oauth"
part = "lib-sites"
kind = "migration"
needs = ["kit-lib", "counts"]
files = ["skein/crates/skein-oauth-accounts/src/component.rs", "skein/crates/skein-oauth-accounts/src/listener.rs", "skein/crates/skein-oauth-accounts/src/redirect.rs", "skein/crates/skein-oauth-issuer/src/form.rs", "skein/crates/skein-fake-oauth/src/lib.rs", "skein/scripts/old-forms.txt"]
effort = "medium"
minutes = 40
+++
# oauth-accounts, the issuer and the fake on lib's primitives

## Goal

Draft, 3.4: oauth-accounts' private `earlier`, `Lifecycle` and pop
loops, and the private hex values of the accounts, the issuer and the
fake, move onto lib's, by `notes/lib.md`. Its slab walks are
`lib-accounts-slab`'s, next.

Acceptance:

- `skein-oauth-accounts`: `src/component.rs:1239` (`fn earlier`, `&mut`
  shape) goes for `skein_lib::earliest`; `src/component.rs:122` (`enum
  Lifecycle`) goes for `skein_lib::Lifecycle`; the 4 pop loops in
  `component.rs` and 5 in `listener.rs` become `clear()`;
  `src/redirect.rs:109` (`fn hex`, a value) goes for `hex::value`.
- `skein-oauth-issuer`: `src/form.rs:111` (`fn hex`, a value) goes for
  `hex::value`.
- `skein-fake-oauth`: `src/lib.rs:704` (`const fn hex`, a value) goes for
  `hex::value` (its callers are not const).
- `earliest` and `lifecycle` fall by 1, `pop_loop` by 9, `hex_decimal`
  by 3 (`--write`).
- `heavy cargo nextest run -p skein-oauth-accounts -p
  skein-oauth-accounts-world -p skein-oauth-issuer -p
  skein-oauth-issuer-world -p skein-fake-oauth -p skein-oauth-world`
  passes (counts named); clippy clean for the three crates.

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`, `notes/lib.md`.
- `skein_lib::{earliest, Lifecycle, hex, Queue::clear}` on your base.

## Owns and may change

- The files in the header, `skein/scripts/old-forms.txt`.
- The linked list over `exchanges` (`first_exchange`, `Exchange::next`
  and the `while let` walks) is `lib-accounts-slab`'s: leave it.
- `skein-oauth`'s `documents::earliest` reads a document's tokens: not a
  copy, not yours.
- The oauth-accounts world (`skein/tests/oauth-accounts`) is
  `stage-oauth`'s: run its tests, do not edit it.

## Steps

1. Each site by `lib.md`. A pop loop over a `List` or another container
   without `clear` is not lib's `Queue`/`Stack`: use the container's own
   `clear` if it has one, else leave it and name it.
2. `--write` from `skein/`; checks of `common.md`; commit; push.

## Report

Per `common.md`.
