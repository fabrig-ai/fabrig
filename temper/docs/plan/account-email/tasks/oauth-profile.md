+++
id = "oauth-profile"
part = "claims"
kind = "feature"
needs = []
files = ["skein/crates/skein-oauth/src/redirect.rs", "skein/crates/skein-oauth/src/jwt.rs", "skein/crates/skein-oauth/src/machine.rs", "skein/crates/skein-oauth/src/lib.rs", "skein/crates/skein-oauth/src/tests.rs", "skein/crates/skein-oauth/src/machine_tests/**", "skein/crates/skein-oauth-accounts/src/boundary.rs", "skein/crates/skein-oauth-accounts/src/component.rs", "skein/crates/skein-oauth-accounts/src/exchange.rs", "skein/crates/skein-oauth-accounts/src/openid_tests.rs", "skein/crates/skein-fake-oauth/src/lib.rs", "skein/tests/oauth/tests/suite/openid.rs", "skein/tests/oauth-accounts/**"]
effort = "high"
minutes = 120
+++

# skein's OAuth reads the profile claims its caller asks for

## Goal

A registration asks for `email` and `name`. A sign-in's checked ID token gives them on `SignedIn`, and a refresh's gives them on a new `Refreshed` event. They are words to show: nothing reads them to choose an account, and no claim that doesn't read fails a check.

Acceptance:

- `section 1` of the contracts note, exactly.
- A table test in `skein-oauth`, over an ID token's claims and what the registration asked:
  - `email` with `email_verified` `true`, and with `"true"`: read. With `false`, with `"false"`, or with none: absent.
  - `name`: read. A registration asking neither: both absent, whatever the token says. Asking `email` alone: no `name`.
  - An `email` or `name` that is a number, repeated, empty, of 256 bytes or holding a control byte: absent, and the check passes with its subject.
  - 255 bytes: read.
- A refresh, in the same or a sibling table, over the answer's ID token: all of `iss`, `aud`, `exp` and `sub` matching gives its claims with no signature check, and each one wrong, a nonce-less token, or no ID token, gives none. Each case's record is the same: the refresh succeeds, and it keeps the sign-in's ID token.
- `skein/tests/oauth-accounts`, the world's sign-in and refresh stories: `SignedIn` carries `profile`; `Refreshed` comes after the record is kept and before that generation's `Granted`; a refresh whose answer gave nothing emits no `Refreshed`, and today's stories keep their event orders.
- A record's bytes don't change: `skein_oauth::FINGERPRINT` and the record's goldens stay.
- The focused and fuzzy tests of `skein-oauth`, `skein-oauth-accounts`, `skein-fake-oauth`, `skein/tests/oauth` and `skein/tests/oauth-accounts` pass, and `heavy cargo check --tests` on `jig-protocol-accounts`, `smith-local-service`, `smith-local-shell` and `temper-service`.

## Inputs

- `skein/docs/design/oauth.md`, 2.2 and 6.3.
- `temper/docs/plan/account-email/notes/contracts.md`, 1.
- `temper/docs/plan/account-email/notes/common.md`.

## Owns and may change

Owns the header's files. Readers, each edited only for this:

- `OpenId` literals gain `profile: oauth::Profile::default()`: `jig/crates/jig-protocol-accounts/src/party.rs`, `smith/crates/smith-local-shell/src/credentials/registration.rs`, `temper/crates/temper-service/src/clients/google.rs`, and the tests the compiler names. `keep-facts` turns jig's on. `fake::OpenId` is another type and doesn't change.
- `Event::Refreshed` arms: `smith/crates/smith-local-service/src/credentials/terminals.rs` drops it, `{}`. `jig/crates/jig-protocol-accounts/src/translate.rs` gains `Up::Refreshed { source, profile }`, and `component.rs` an arm that drops it; `keep-facts` fills that arm. Its `SignedIn` match binds `profile: _`.
- `oauth::Request::Tokens` patterns outside skein, should the compiler name any.

Options, each pre-approved:

- `temper-service/src/clients/google.rs` reads the same claims on its own, off the kept record. Leave it as it is, and say so in the report: replacing it is another task.
- The fake's `Account` already says `email`, `email_verified` and `name` through `sign_in_as`. A refresh's answer carrying an ID token: `Plan`'s `Body::Token` can give one by hand; if `add_id_token` turns out simplest, a `pub fn refresh_says(&mut self, on: bool)` on the fake, and say so.
- A `Check` variant is not needed: a claim that doesn't read is absent. Don't add one.

## Steps

```rust
// jwt.rs
pub(crate) struct Verified { pub(crate) subject: Box<[u8]>, pub(crate) profile: ProfileClaims }
pub(crate) fn verify_id_token(.., wanted: Profile, ..) -> Result<Verified, Check>;
/// A refresh's ID token: no signature; empty when any check fails (oauth.md, 2.2).
pub(crate) fn refreshed_profile(token: &[u8], expected: &RefreshExpected<'_>, wanted: Profile, wall: Wall, skew: Duration, limits: &Limits) -> ProfileClaims;

// component.rs, Stage::Keeping { candidate, missing, profile }: at `Keeping::Kept`
if exchange.signing_in() { up.push(Event::SignedIn { .., profile }); }
else if !profile.is_empty() { up.push(Event::Refreshed { account, generation, profile }); }
// then `grant(..)` as today
```

`check_claims` shares `iss`, `aud` and `exp` with the refresh's reader; the nonce and `sub` rules differ. Read the claims with `json::field`, whose repeated-field error means absent here. A `ProfileClaims` is no secret, and takes a derived `Debug`.

Each new test breaks its rule once by hand. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-oauth -p skein-oauth-accounts -p skein-fake-oauth --all-targets -- -D warnings`; `heavy cargo nextest run` on them, their worlds and direct dependents, one command at a time. Commit and push.

## Report

As the implementer's text says.
