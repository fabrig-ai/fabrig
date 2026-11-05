+++
id = "oauth-noise"
part = "spot-review"
kind = "feature"
needs = ["oauth-secret-fields"]
files = ["skein/crates/skein-oauth-accounts/src/component/sign_in.rs", "skein/crates/skein-fake-oauth/src/lib.rs", "skein/crates/skein-oauth/src/lib.rs", "temper/crates/temper-service/src/clients/google.rs"]
effort = "medium"
minutes = 15
+++

# oauth: hand-written `Option::clone`s and a duplicate helper go

## Goal

Finding 7 of `temper/docs/plan/debt-23/inputs/debt21-review.md`, what's left of it, about -15 lines. Checked against the code, nothing here changes behaviour.

Acceptance:

- `skein-oauth-accounts/src/component/sign_in.rs`: `*id_token = match &record.id_token { .. }` is `id_token.clone_from(&record.id_token)`, as its neighbours are.
- `skein-fake-oauth/src/lib.rs`'s `post`: `next` clones `value.refresh_token` with `.clone()` in its condition's arm, and `post`'s `#[expect(clippy::manual_map, ..)]` goes if nothing else in `post` still needs it; the compiler says.
- `expose_optional` goes: `skein_oauth::secret_ref` is exported beside `record`'s other items in `skein-oauth/src/lib.rs`, and `skein-fake-oauth` calls that.
- `temper-service/src/clients/google.rs`'s `Registration` reads `client_secret: Some(self.settings.secret.clone())`.
- `heavy cargo nextest run -p skein-oauth --lib`, `-p skein-oauth-accounts`, `-p skein-fake-oauth`, `-p skein-oauth-world`, `-p skein-oauth-accounts-world` and `-p temper-service --lib` pass.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`
- `temper/docs/plan/debt-23/inputs/debt21-review.md`, finding 7

## Owns and may change

- **Owns:** the files in the header.
- **Skipped, with the reason:** "`record.rs`'s `secret` helper was re-inlined": no copy is left there to share. Its fields are `Secret` now, and the one `Some(value) => Some(Secret::new(value))` is `skein-oauth/src/documents.rs`'s. `component/github.rs` is `connection-deadline`'s.
- **Stays off** the other `Some(value) => Some(value.expose())` matches in `jig-protocol-engine-clients` and `temper-client-domain`: they aren't this finding's.

## Steps

1. The four changes.
2. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-oauth -p skein-oauth-accounts -p skein-fake-oauth -p temper-service --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items. No new rule, so no test to bite.
