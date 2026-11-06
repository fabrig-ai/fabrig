+++
id = "concat-name"
part = "concat-name"
kind = "feature"
needs = []
files = ["skein/crates/skein-fake-forge-forgejo/src/setup.rs", "skein/crates/skein-fake-forge-forgejo/src/setup/**", "skein/crates/skein-oauth-issuer/src/documents.rs"]
effort = "medium"
minutes = 10
+++

# `bytes::concat` goes by one name

## Goal

Finding 4 of `temper/docs/plan/debt-29/inputs/debt27-review.md`. skein's D007 keeps the name it paid off by alias: `skein-fake-forge-forgejo/src/setup.rs` has `use skein_lib::bytes::{concat, concat as joined};`. And `skein-oauth-issuer/src/documents.rs` keeps a `#[expect(unused_qualifications)]` only to spell `skein_lib::bytes::concat` in full.

The finding's third part, recording `forgejo/server/files.rs`'s exemption, the cut has written in `skein/docs/design/lib.status.md`. That `joined` stays.

Acceptance:

- `rg -n 'concat as|\bjoined\(' skein/crates/skein-fake-forge-forgejo -g '*.rs'` finds nothing.
- `rg -n 'unused_qualifications|skein_lib::bytes::concat' skein/crates/skein-oauth-issuer/src/documents.rs` finds nothing.
- No other change.

## Inputs

- `temper/docs/plan/debt-29/inputs/debt27-review.md`, finding 4.
- `skein/docs/design/lib.md`, 6.
- `temper/docs/plan/debt-29/notes/common.md`.

## Owns and may change

Owns the files in the header, for this edit alone. Skein's other files stay: `skein-forge/src/forgejo/server/files.rs` keeps its `joined`, which writes a slash between segments.

The draft `ui-repo-link` changed `skein-oauth-issuer/src/**` on its own trunk. If fab's merge brings more `skein_lib::bytes::concat` into `documents.rs`, they get the same edit.

## Steps

In `setup.rs`, `use skein_lib::bytes::concat;`. `setup/migrate.rs`, `setup/creation.rs` and `setup/accounts.rs` call `concat(` or `super::concat(`; add the `use` a file needs, as its neighbours do.

In `documents.rs`, the two calls become `bytes::concat`, and the `#[expect(unused_qualifications, ..)]` on `tokens` goes.

No new test. `heavy cargo nextest run -p skein-fake-forge-forgejo -p skein-oauth-issuer`. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-fake-forge-forgejo -p skein-oauth-issuer --all-targets -- -D warnings`. Commit, push.

## Report

As your text says, with the two searches' readings.
