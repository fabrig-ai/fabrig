+++
id = "oauth-tokens"
part = "oauth-secrets"
kind = "interface"
needs = []
files = ["skein/crates/skein-oauth/src/record.rs", "skein/crates/skein-oauth/src/documents.rs", "skein/crates/skein-oauth/src/jwt.rs", "skein/crates/skein-oauth/src/machine.rs", "skein/crates/skein-oauth/src/machine/**", "skein/crates/skein-oauth/src/machine_tests.rs", "skein/crates/skein-oauth/src/machine_tests/**", "skein/crates/skein-oauth/src/tests/**"]
effort = "high"
minutes = 90
+++

# skein-oauth: a record's and a token answer's credentials are `Secret`

## Goal

The first part of `skein/docs/development/debt/D009-oauth-credentials-raw.md`. `SavedToken`, `RefreshState`, `TokenResponse`, `RefreshRequest` and `Registration.client_secret` hold `Secret`, and each derives `Debug`. `Claims`, which holds no credential, derives it too.

Acceptance:

- `SavedToken.{access_token,refresh_token,id_token}`, `RefreshState.{refresh_token,id_token}`, `TokenResponse.{access_token,refresh_token,id_token}`, `RefreshRequest.refresh_token` and `Registration.client_secret` are `Secret` (or `Option<Secret>` where they were optional).
- `rg -c 'missing_debug_implementations' skein/crates/skein-oauth/src/record.rs skein/crates/skein-oauth/src/documents.rs skein/crates/skein-oauth/src/jwt.rs` reads nothing, and `machine.rs`'s count falls from 7 to 6. The whole **Measure** reads 19.
- Nothing under `skein-oauth`'s `schema/`, `src/generated/` or `golden/` changes. `record.rs`'s `encode_record` and `decode_record` keep their signatures and lose their wrapping.
- `heavy cargo nextest run -p skein-oauth -p skein-oauth-world -p skein-oauth-accounts -p skein-oauth-accounts-world -p skein-fake-oauth -p skein-fake-peers-tests -p skein-oauth-issuer-world` passes, with no assertion's meaning changed.
- Every reader in `temper/docs/plan/debt-21/notes/oauth-secrets.md`, Readers, passes `heavy cargo check -p <package> --tests`.

## Inputs

- `temper/docs/plan/debt-21/notes/common.md`
- `temper/docs/plan/debt-21/notes/oauth-secrets.md`, binding
- `skein/docs/development/debt/D009-oauth-credentials-raw.md`
- `skein/docs/design/oauth.md`, 4; `skein/docs/design/lib.md`, 6
- `skein/crates/skein-lib/src/secret.rs`; the model, `skein/crates/skein-oauth-issuer/src/lib.rs` and `src/requests.rs`

## Owns and may change

- **Owns:** the files in `files`.
- **These public fields change type:** the request asks for it. No other public interface changes. `Event`, `Request`, `Redirect`, `HttpRequest`, `HttpResponse` and `Client` keep their expects: `oauth-exchange` takes them.
- **Readers:** the note's rule. The production lines the cut expects: `jig/crates/jig-protocol-accounts/src/translate.rs`, `temper/crates/temper-service/src/clients/google.rs`, `temper/crates/temper-fake/src/oauth.rs`, `skein/crates/skein-fake-peers/src/oauth.rs`, `skein/crates/skein-fake-oauth/src/lib.rs`, `skein/crates/skein-oauth-accounts/src/**`. A `Box<[u8]>` field of the accounts or fake crates stays a box here, reading `expose()` or `into_exposed()` at the line; their tasks move those fields.
- **If `oauth::Event` or a field outside this list won't compile without changing type:** keep it a box at that line with `into_exposed()`, and say so in the report.

## Steps

1. `record.rs`, `documents.rs`, `jwt.rs`, `Registration`; `skein-oauth`'s tests.
2. The readers, by tree: skein, then jig, smith, temper.
3. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`, and each other tree you changed; `heavy cargo clippy` over the skein packages you changed and each other package you changed, `--all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; the **Measure**; the readers changed per tree. No new test here: `oauth-canary` holds it.
