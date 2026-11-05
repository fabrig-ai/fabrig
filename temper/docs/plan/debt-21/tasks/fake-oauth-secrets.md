+++
id = "fake-oauth-secrets"
part = "oauth-secrets"
kind = "interface"
needs = ["oauth-exchange"]
files = ["skein/crates/skein-fake-oauth/src/**"]
effort = "medium"
minutes = 45
+++

# skein-fake-oauth: credentials are `Secret`, and the crate's expects go

## Goal

The fake's part of `skein/docs/development/debt/D009-oauth-credentials-raw.md`. Every credential `skein-fake-oauth` holds is `Secret`, and it keeps no `missing_debug_implementations` expect.

Acceptance:

- `Config.{client_secret,refresh_token}`, `Request::Redirect.{code,state}`, the private `Code.{value,challenge,nonce}`, `Field`'s credentials, the spent list and `granted()`, which returns `Box<[Secret]>`, are `Secret`. `src/key.rs` is a public fixture, "never a secret", and stays.
- `Config`, `Plan` and `Request` derive `Debug`. `Body` and `Event` have one printing their carriers' lengths: `Body::Raw` stays a byte box, having 63 uses. `Issuer`'s stops at its edge.
- `rg -c 'missing_debug_implementations' skein/crates/skein-fake-oauth` reads nothing.
- `heavy cargo nextest run -p skein-fake-oauth -p skein-fake-peers-tests -p skein-oauth-world -p temper-fake` passes, with no assertion's meaning changed.
- Every reader of `skein-fake-oauth` in the note's Readers passes `heavy cargo check -p <package> --tests`.

## Inputs

- `temper/docs/plan/debt-21/notes/common.md`
- `temper/docs/plan/debt-21/notes/oauth-secrets.md`, binding
- `skein/docs/development/debt/D009-oauth-credentials-raw.md`
- `skein/docs/design/oauth.md`, 4; `skein/docs/design/lib.md`, 6
- `skein/crates/skein-lib/src/secret.rs`; the model, `skein/crates/skein-oauth-issuer/src/lib.rs` and `src/requests.rs`

## Owns and may change

- **Owns:** `skein/crates/skein-fake-oauth/src/**`.
- **These public fields change type:** the request asks for it. No other public interface changes.
- **Readers:** the note's rule. The production lines the cut expects: `skein/crates/skein-fake-peers/src/oauth.rs`, `temper/crates/temper-fake/src/oauth.rs`.

## Steps

1. The fields; the `Debug`s; the crate's tests.
2. The readers, by tree.
3. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`, and each other tree you changed; `heavy cargo clippy` over the packages you changed, `--all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; the **Measure**. No new test here: `oauth-canary` holds it.
