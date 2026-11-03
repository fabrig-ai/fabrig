+++
id = "issuer-debug"
part = "skein-debug"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-oauth-issuer/src/lib.rs",
  "skein/crates/skein-oauth-issuer/src/form.rs",
  "skein/crates/skein-oauth-issuer/src/tests/token.rs",
]
effort = "medium"
minutes = 40
+++

# skein-oauth-issuer's token request and form print their secrets as lengths

## Goal

`skein/crates/skein-oauth-issuer/src/lib.rs`'s `TokenRequest` (~71) derives `Debug`, and its
private `verifier: Box<[u8]>` is the PKCE code verifier, so a debug print of a token request
shows it; its `code` is a `Secret` already. The token endpoint's parsed form,
`src/form.rs`'s `Field` (~6), derives `Debug` over each parameter's `value`, the client secret,
the code and the verifier among them, and `Form` prints every `Field`. `skein/docs/design/oauth.md`,
4 already says "every type carrying a token, authorization code, verifier, state, nonce, ID
token or client secret omits its bytes from `Debug`": give both a hand-written `Debug`.

Acceptance:

- `TokenRequest` prints `TokenRequest { client: .., code: Secret { len: N }, redirect_uri: ..,
  verifier_len: N }`: `client` and `redirect_uri` as derived, the verifier as its length.
- `Field` prints `Field { name: .., value_len: N, past: .. }`; `Form` keeps its derive and so
  prints no value.
- The new tests fail on the old code and pass on the new.
- `Authorization` (~55) keeps its derive (below). clippy clean; the crate's lib tests pass.

## Inputs

- `temper/docs/plan/bugs-security/notes/common.md` and `notes/debug.md`.
- `skein/AGENTS.md`.
- `temper/docs/plan/bugs-security/inputs/request.md`, item 4.
- `skein/docs/design/oauth.md`, 4, the paragraph "Every type carrying a token, ...";
  `skein/docs/design/oauth-issuer.md`, 5 (the token request: "the redirect URI and the PKCE
  verifier, private").
- Readers, surveyed: no test asserts either type's `Debug` text; `src/tests/token.rs` ~103 calls
  `expect_err` on `token_request`'s result, which needs `TokenRequest: Debug`, so the type keeps
  a `Debug`.

## Owns and may change

- `skein/crates/skein-oauth-issuer/src/lib.rs`: `TokenRequest`'s derive and a hand-written
  `impl core::fmt::Debug for TokenRequest`; nothing else.
- `skein/crates/skein-oauth-issuer/src/form.rs`: `Field`'s derive and its hand-written impl.
- `skein/crates/skein-oauth-issuer/src/tests/token.rs`: the tests below, appended.

Nothing else. `Authorization`'s `state` and `nonce` are outside this task:
`oauth-issuer.md`, 4 shows its `#[derive(.., Debug)]`, which is this plan's question
`issuer-authorization` (`plan.toml`); leave it, and name it under `## Left`.

Pre-approved options:

- If clippy's `missing_fields_in_debug` fires, a scoped `#[expect(clippy::missing_fields_in_debug,
  reason = "oauth.md, 4: a verifier's and a form value's bytes stay out of Debug")]`.
- If the form test cannot reach `Form::read` from `tests/token.rs`, put it in the module of the
  crate's tests that can, and say where.

## Steps

1. Tests first, appended to `src/tests/token.rs` (the crate's lib tests, `tests::token`), with a
   made-up canary in both forms (`notes/debug.md`):

   ```rust
   #[test]
   fn debug_of_a_token_request_shows_the_verifiers_length_never_its_bytes() {
       // a body like tests.rs's BODY whose code_verifier is a made-up 43-character canary
       // (unreserved characters, as PKCE takes), through issuer().token_request: formatted,
       // "verifier_len: 43" and "code: Secret { len: " present, the canary absent in both forms
   }

   #[test]
   fn debug_of_a_parsed_form_shows_value_lengths_never_values() {
       // crate::form::Form::read(b"client_secret=<canary>&code=<canary>", None): formatted, the
       // canary absent in both forms, "value_len: " present, the names present
   }
   ```

   Run `heavy cargo nextest run -p skein-oauth-issuer --lib -E 'test(/debug_of_/)'` on the old
   code: both fail. Keep the output.
2. The two impls, a line above each citing oauth.md, 4.
3. Narrow tests, one at a time:
   - `heavy cargo nextest run -p skein-oauth-issuer --lib -E 'test(/debug_of_/)'`: 2 passed.
   - `heavy cargo nextest run -p skein-oauth-issuer --lib`.
4. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `skein/`;
   `heavy cargo clippy -p skein-oauth-issuer --all-targets -- -D warnings`.
5. One commit, the impls and their tests, saying why; push.

## Report

As the implementer's text says, and: the failing run on the old code, the passing runs and their
counts; `Authorization` under `## Left`.

## Answers

2026-11-03, run issuer-debug-1: the slot's checkout changed to another branch mid-run; restore
it or take a new slot? The slot is fab's, not the task's: the next run works in the slot fab
assigns, set up on this task's branch from `bugs-security/integration`. Start by checking
`git status` shows that branch and that `temper/docs/plan/bugs-security/` is present; if the
checkout changes underneath again, stop as run 1 did, changing nothing. The spec stands as
written; nothing of run 1 is to be kept.
