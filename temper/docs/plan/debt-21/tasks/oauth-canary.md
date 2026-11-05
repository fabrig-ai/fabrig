+++
id = "oauth-canary"
part = "oauth-secrets"
kind = "feature"
needs = ["accounts-secrets", "fake-oauth-secrets"]
files = ["skein/tests/oauth-accounts/tests/suite/debug.rs", "skein/tests/oauth-accounts/tests/suite/main.rs", "skein/docs/development/debt/D009-oauth-credentials-raw.md", "skein/docs/development/debt/D022-oauth-states.md"]
effort = "medium"
minutes = 30
+++

# The oauth crates' `Debug` holds no credential, and D009 goes

## Goal

The request's canary: no credential in `skein-oauth`, `skein-oauth-accounts` or `skein-fake-oauth` reaches `Debug`. One table in the one world that links all three. Then the item is paid and goes.

Acceptance:

- `skein/tests/oauth-accounts/tests/suite/debug.rs`, with `mod debug;` in `main.rs`, holds one test over one table: each row is a value built with a canary in every credential and carrier, and the name of its type. A small local `assert_hidden` checks `{:?}` and `{:#?}` for the canary as text **and** as a byte list, as `skein/crates/skein-http/src/tests/debug.rs` does: a derived `Debug` on `Box<[u8]>` prints `[99, 97, ...]`, so a text-only check can't fail.
- The table has every public type that holds a credential or carrier:
  - `oauth`: `SavedToken`, `RefreshState`, `TokenResponse`, `RefreshRequest`, `Registration`, `Redirect`, `HttpRequest`, `HttpResponse`, `Event::{SignIn,Redirected}`, `Request::Visit`.
  - `accounts`: `Request::{HandIn,Redirected}`, `Event::{Granted,Keep,Visit}`, `Locked::Held`.
  - `fake`: `Config`, `Body::{Token,Raw}`, `Plan`, `Event::{Authorize,Post}`, `Request::Redirect`.
  - One row each for a `Client`, a `Component` and an `Issuer` that has been through a sign-in, if the world's existing helpers reach one in a line or two; if not, say so in the report.
- **It bites:** break each by hand, see it go red for its row, restore. (a) `Redirect.code` back to `Box<[u8]>` with the derive; (b) `HttpRequest`'s hand `Debug` printing `body`; (c) `Body`'s printing `Raw`. The report names each failure.
- `rg -c 'missing_debug_implementations' skein/crates/skein-oauth skein/crates/skein-oauth-accounts skein/crates/skein-fake-oauth` reads nothing: the **Measure** reads 0.
- This search lists only carriers the note names, and any other it finds is yours to fix or to name in the report: `rg -n '\b\w*(token|code|verifier|secret|record|state|nonce|kept)\w*: (Option<)?Box<\[u8\]>' skein/crates/skein-oauth/src skein/crates/skein-oauth-accounts/src skein/crates/skein-fake-oauth/src -g '!**/tests/**' -g '!*_tests*' -g '!**/generated/**'`.
- `skein/docs/development/debt/D009-oauth-credentials-raw.md` is deleted. `skein/docs/development/debt/D022-oauth-states.md`'s "What pays it back" loses its sentence on D009. `python3 -I -B tools/citations/citations.py --all` reads 0.
- `heavy cargo nextest run -p skein-oauth-accounts-world --test suite` passes, and the new test counts in it. The test must not be slower than the world's others: it builds values and formats them.

## Inputs

- `temper/docs/plan/debt-21/notes/common.md`
- `temper/docs/plan/debt-21/notes/oauth-secrets.md`, binding
- `skein/docs/development/debt/D009-oauth-credentials-raw.md`
- `skein/docs/design/oauth.md`, 4; `skein/docs/design/lib.md`, 6
- `skein/crates/skein-lib/src/secret.rs`; the model, `skein/crates/skein-oauth-issuer/src/lib.rs` and `src/requests.rs`
- `skein/crates/skein-http/src/tests/debug.rs`
- `skein/crates/skein-oauth-issuer/src/tests/authorization.rs`, the issuer's canary

## Owns and may change

- **Owns:** the files in `files`.
- **May fix:** a field or a `Debug` in the three crates that the table or the search shows leaking; it's the same item. Readers as in the note. Say so in the report.
- **No helper is shared:** `assert_hidden` is private to `skein-http`, and the copy here is about ten lines. Moving it isn't this task's.

## Steps

1. The table; the search.
2. The three breaks, each restored.
3. The item's deletion; D022's sentence.
4. `cargo fmt`; `heavy cargo clippy -p skein-oauth-accounts-world --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; what you broke and what failed; the search's result; the table's rows.
