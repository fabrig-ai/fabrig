+++
id = "issuer-authorization"
part = "issuer-authorization"
kind = "feature"
needs = []
files = ["skein/crates/skein-oauth-issuer/src/lib.rs", "skein/crates/skein-oauth-issuer/src/tests/authorization.rs", "skein/docs/design/oauth-issuer.status.md"]
effort = "medium"
minutes = 15
+++

# skein-oauth-issuer: an authorization prints its state's and nonce's lengths

## Goal

`Authorization` derives `Debug` over the client's state and nonce, which `skein/docs/design/oauth.md`, 4 keeps out of `Debug`. It prints `state_len` and `nonce_len` instead.

Acceptance:

- `Authorization` prints `Authorization { client: [..], redirect_uri: [..], state_len: Some(N), nonce_len: None, challenge: [..] }`: each length an `Option<usize>`, `None` where the query had none.
- The tests below pass and fail with the derive put back.
- `heavy cargo nextest run -p skein-oauth-issuer -p skein-oauth-issuer-world` passes.

## Inputs

- `temper/docs/plan/debug-secrets/notes/common.md`
- `skein/docs/design/oauth-issuer.md`, 4
- `skein/docs/design/oauth.md`, 4

## Owns and may change

- **Owns:** the files in `files`.
- **Status entry:** in `skein/docs/design/oauth-issuer.status.md`, remove the `4:` entry; the file then says `No gap.`
- **Stays as it is:** `Refusal::Back`'s `location` (`skein/docs/design/oauth-issuer.md`, 10) and `Authorization`'s `challenge`, which `oauth.md`, 4 doesn't name.

## Steps

1. In `src/lib.rs`, drop `Debug` from `Authorization`'s derive and write its `Debug` beside it, as `TokenRequest`'s is written.
2. In `src/tests/authorization.rs`, reaching the value through `issuer.authorize(&query)`:
   - `debug_of_an_authorization_shows_the_state_and_nonce_lengths_never_their_bytes`: a canary as the state and another as the nonce; neither appears in either rendering; the exact form with `Some` of each length.
   - `debug_of_an_authorization_without_a_state_or_nonce_shows_none`: the exact form with `state_len: None, nonce_len: None`.
3. Narrow tests, then `cargo fmt`, `old_forms.py` and clippy on `skein-oauth-issuer`, as the implementer's text has them. Commit and push.

## Report

As the implementer's text has it, and name each rule you broke by hand with the failure you saw.
