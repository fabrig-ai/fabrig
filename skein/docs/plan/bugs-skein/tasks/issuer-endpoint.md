+++
id = "issuer-endpoint"
part = "oauth-endpoints"
kind = "feature"
needs = ["oauth-start-refused"]
files = ["skein/crates/skein-oauth/src/redirect.rs", "skein/crates/skein-oauth/src/machine_tests.rs", "skein/crates/skein-oauth/src/machine_tests/openid.rs", "skein/docs/design/oauth.status.md"]
effort = "medium"
minutes = 20
+++

# An issuer's `https` endpoint names a host

## Goal

`skein-oauth`'s `issuer_endpoint` (`src/redirect.rs`) returns true for anything that starts
`https://`, so `https://` alone and `https://a@b` are admitted. Its `http` branch already reads the
authority. Read it for `https` too. (Bug 4, re-checked: holds.)

Acceptance:

- `https://`, `https://a@b`, `https://@b/x`, `https:///x`, `https://?q` and `https://:443/x` are
  refused; `https://issuer.example`, `https://issuer.example:8443/t?x=1` and `https://a/b@c` are
  admitted.
- The test fails on the old code.

## Inputs

- `skein/docs/plan/bugs-skein/notes/common.md`.
- `skein/docs/design/oauth.md`, 1 ("`https`, with a host and no userinfo"; "Every endpoint the
  terms name").

## Owns and may change

- Owns: `src/redirect.rs` (`issuer_endpoint` and a small private helper holding the authority loop
  its `http` branch has now); `src/machine_tests.rs`; `src/machine_tests/openid.rs` (one metadata row).
- `skein/docs/design/oauth.status.md`: remove this plan's **Not yet** entry for section 1
  (`notes/common.md`).
- No reader changes. `issuer_endpoint` is `pub(crate)`: `keys.rs` and `redirect.rs` call it. No test
  or fixture in any tree uses userinfo or an empty host in a registration (surveyed). The same
  `starts_with` on a confidential redirect (`redirect.rs`, near 64) is not this bug: `## Left`.

## Steps

1. Add `issuer_https_endpoints_name_a_host` to `src/machine_tests.rs`, beside and modelled on
   `issuer_http_endpoints_are_admitted_only_on_numeric_loopback`: the same `Event::SignIn` loop,
   expecting `Failure::Malformed` for the refused six and a start for the admitted three. Add a
   `jwks_uri` row of `https://a@b` to the metadata cases in `src/machine_tests/openid.rs`. Run them
   on the old code and quote the failure.
2. The authority is what stands between `://` and the first `/` or `?`. Refuse it empty, with `@`,
   or starting `:`.
3. `heavy cargo nextest run -p skein-oauth --lib -E 'test(issuer_)'`, then the focused tests of
   `skein-oauth`, `skein-oauth-world`, `skein-oauth-accounts`, `skein-oauth-accounts-world`.
   `heavy cargo check -p smith-local-shell -p temper-service --tests`, the callers beyond skein.
   `cargo fmt`, `python3 -I -B skein/scripts/old_forms.py`, `heavy cargo clippy -p skein-oauth
   --all-targets -- -D warnings`.
4. Commit, push.

## Report

As the implementer's text says, with what the test showed on the old code.
