+++
id = "lib-small"
part = "lib-sites"
kind = "migration"
repo = "fabrig"
needs = ["ratchets", "kit-lib"]
files = ["skein/crates/skein-oauth-issuer/src/form.rs", "skein/crates/skein-ui/src/phrases.rs", "skein/tests/forge/client/src/translate.rs", "skein/scripts/old-forms.txt"]
effort = "medium"
minutes = 25
+++
# lib's primitives in oauth-issuer, ui and the forge client's world

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/lib-common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, 3.4, for three single sites.

## Sites

- `skein/crates/skein-oauth-issuer/src/form.rs:111` `hex` → `hex::value`.
- `skein/crates/skein-ui/src/phrases.rs:533` `decimal(value, writer)`
  → `Decimal::of(value)` written into the writer.
- `skein/tests/forge/client/src/translate.rs:47` `hex` → `hex::value`.

skein-oauth's `documents::earliest` reads a document's tokens: not a
copy, it stays (decisions.md).

## Acceptance

- `hex_decimal` falls by three.
- `heavy cargo nextest run -p skein-oauth-issuer -p skein-ui -p
  <tests/forge/client's package>`, with counts.
