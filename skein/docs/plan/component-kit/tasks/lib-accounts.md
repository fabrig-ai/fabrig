+++
id = "lib-accounts"
part = "lib-sites"
kind = "migration"
repo = "fabrig"
needs = ["ratchets", "kit-lib"]
files = ["skein/crates/skein-oauth-accounts/src/**", "skein/scripts/old-forms.txt"]
effort = "high"
minutes = 40
+++
# lib's primitives in skein-oauth-accounts

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/lib-common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, 3.4. The slab walks are lib-accounts-slab's, after this.

## Sites

- `src/component.rs:122` `enum Lifecycle` → `skein_lib::Lifecycle`.
- `src/component.rs:1239` `earlier(&mut Option<Time>, Time)` →
  `skein_lib::earliest`.
- `src/redirect.rs:109` `hex` → `hex::value`.
- The discarding pop loops: four in `src/component.rs`, five in
  `src/listener.rs` → `clear()`.

## Acceptance

- `lifecycle`, `earliest` and `hex_decimal` fall by one each, `pop_loop`
  by nine.
- `heavy cargo nextest run -p skein-oauth-accounts -p
  <tests/oauth-accounts' package>`, default then fuzzy, with counts.
