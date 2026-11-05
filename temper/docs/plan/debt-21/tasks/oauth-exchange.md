+++
id = "oauth-exchange"
part = "oauth-secrets"
kind = "interface"
needs = ["oauth-tokens"]
files = ["skein/crates/skein-oauth/src/machine.rs", "skein/crates/skein-oauth/src/machine/**", "skein/crates/skein-oauth/src/redirect.rs", "skein/crates/skein-oauth/src/common.rs", "skein/crates/skein-oauth/src/machine_tests.rs", "skein/crates/skein-oauth/src/machine_tests/**", "skein/crates/skein-oauth/src/tests/**"]
effort = "high"
minutes = 60
+++

# skein-oauth: a sign-in's credentials are `Secret`, and the crate's expects go

## Goal

The second part of `skein/docs/development/debt/D009-oauth-credentials-raw.md`. What a sign-in carries is `Secret`, and `skein-oauth` keeps no `missing_debug_implementations` expect.

Acceptance:

- `Redirect.{code,state}`, `Event::SignIn.{state,verifier,nonce,id_token}` and `Event::Redirected.{state,code}` are `Secret` (or `Option<Secret>` where they were optional). So are the private `SignInValues`'s credentials, `Grant::SignIn.code`, `Exchange`'s and `machine/fetch.rs`'s `Identity.token`.
- `Redirect` and `Event` derive `Debug`. `HttpRequest`, `HttpResponse` and `Request` have a hand `Debug` printing their carriers' lengths. `Client`'s stops at its edge.
- `rg -c 'missing_debug_implementations' skein/crates/skein-oauth` reads nothing. The whole **Measure** reads 13.
- Nothing under `skein-oauth`'s `schema/`, `src/generated/` or `golden/` changes.
- `heavy cargo nextest run -p skein-oauth -p skein-oauth-world -p skein-oauth-accounts -p skein-oauth-accounts-world -p skein-fake-oauth -p skein-fake-peers-tests` passes, with no assertion's meaning changed.
- Every reader in the note's Readers passes `heavy cargo check -p <package> --tests`.

## Inputs

- `temper/docs/plan/debt-21/notes/common.md`
- `temper/docs/plan/debt-21/notes/oauth-secrets.md`, binding
- `skein/docs/development/debt/D009-oauth-credentials-raw.md`
- `skein/docs/design/oauth.md`, 4; `skein/docs/design/lib.md`, 6
- `skein/crates/skein-lib/src/secret.rs`; the model, `skein/crates/skein-oauth-issuer/src/lib.rs` and `src/requests.rs`

## Owns and may change

- **Owns:** the files in `files`.
- **These public fields change type:** the request asks for it. No other public interface changes.
- **Readers:** the note's rule. The production lines the cut expects: `temper/crates/temper-service/src/clients/google.rs`, `temper/crates/temper-fake/src/oauth.rs`, `skein/crates/skein-fake-peers/src/oauth.rs`, `skein/crates/skein-fake-oauth/src/lib.rs`, `skein/crates/skein-oauth-accounts/src/redirect.rs`, `src/component/sign_in.rs`, `src/exchange.rs`. A `Box<[u8]>` field of the accounts or fake crates stays a box here; their tasks move those fields.
- **`Phase` and `Exchange` keep their shape** (`skein/docs/development/debt/D022-oauth-states.md`): their credential fields change type, nothing else.

## Steps

1. The fields; the `Debug`s; `skein-oauth`'s tests.
2. The readers, by tree.
3. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`, and each other tree you changed; `heavy cargo clippy` over the packages you changed, `--all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; the **Measure**. No new test here: `oauth-canary` holds it.
