+++
id = "oauth-secret-fields"
part = "spot-review"
kind = "feature"
needs = []
files = ["skein/crates/skein-oauth/src/machine.rs", "skein/crates/skein-oauth/src/machine/fetch.rs", "skein/crates/skein-oauth-accounts/src/boundary.rs", "skein/crates/skein-oauth-accounts/src/component.rs", "skein/crates/skein-oauth-accounts/src/exchange.rs", "skein/crates/skein-fake-oauth/src/lib.rs", "skein/tests/oauth-accounts/tests/suite/debug.rs"]
effort = "medium"
minutes = 60
+++

# oauth: five fields become `Secret`, and their records derive `Debug`

## Goal

Finding 6 of `temper/docs/plan/debt-23/inputs/debt21-review.md`, decided by fabrig/debt: make the five fields `Secret` and derive `Debug`. Hand-written `Debug` is what hides them today, about 135 lines. About -130 net. This is the one public change the plan asks for: these fields' types change, and nothing else of these types.

The five fields:

| Type | Field | Crate |
|---|---|---|
| `accounts::Request::Redirected` | `uri` | `skein-oauth-accounts` |
| `accounts::Event::Visit` | `url` | `skein-oauth-accounts` |
| `oauth::Request::Visit` | `url` | `skein-oauth` |
| `fake::Event::Authorize` | `url` | `skein-fake-oauth` |
| `fake::Body::Raw` | its one field | `skein-fake-oauth` |

`oauth::Event::Redirected { uri, .. }` and `oauth::Redirect { uri, .. }` are another thing, an address without its query: they stay. So do `accounts::Request::Return` and `CredentialRequest::Visit`.

Acceptance:

- `impl Debug` for `accounts::Request`, `accounts::Event`, `oauth::Request`, `fake::Event` and `fake::Body` are gone, each now `#[derive(.., Debug)]`, and `rg -n 'url_len|uri_len' skein/crates skein/tests` reads nothing. `oauth::HttpRequest`'s and `HttpResponse`'s hand `Debug`, which print `body_len`, stay.
- `exchange.visit` in `skein-oauth-accounts` holds `Option<Secret>`; a `url` passes from `oauth::Request::Visit` to `Event::Visit` without a copy.
- `skein/tests/oauth-accounts/tests/suite/debug.rs`'s rows for the five fields hold `secret()`, and the test passes. Add a row for `accounts::Component` and one for `fake::Issuer` holding a canary if each builds in about 5 lines; if not, say so. Both print through `finish_non_exhaustive`.
- `heavy cargo nextest run` passes for `-p skein-oauth`, `-p skein-oauth-accounts`, `-p skein-fake-oauth`, `-p skein-fake-peers`, `-p skein-oauth-world`, `-p skein-oauth-accounts-world`, `-p skein-oauth-issuer-world`, `-p jig-protocol-accounts`, `-p jig-protocol-accounts-world`, `-p smith-local-service`, and `-p temper-service --lib`, `-p temper-fake --lib`. `heavy cargo check --tests` for each other package the compiler names.
- `heavy cargo check -p smith-local-process-world --tests`, since its fixtures build a `fake_oauth::Body::Raw`.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`, with the departure above: this public change is decided, and these types' readers change in every tree.
- `temper/docs/plan/debt-23/inputs/debt21-review.md`, finding 6 and its caveat, which this plan's commit records in `skein/docs/design/oauth.md`, 4 and `skein/docs/design/lib.status.md`; they aren't yours.
- `skein/docs/design/oauth.md`, 4; `skein/docs/design/lib.md`, 6, "A carrier prints lengths" and "`Secret`"

## Owns and may change

- **Owns:** the files in the header.
- **Readers, each changed only in the field's type, name or value** (the required-field rule of the implementer's text, across trees):
  - Reading the bytes: `jig-protocol-accounts/src/translate.rs` (`url.len()` holds, then `address: url.into_exposed()`); `jig-protocol-accounts/src/finishes.rs` (`uri: value`, the `Secret` it already holds, dropping its copy); `smith-local-service/src/credentials/terminals.rs` (`CredentialRequest::Visit { url: url.into_exposed() }`); `temper-service/src/clients/google.rs`'s `oauth::Request::Visit` arm (`url.expose()`); `skein-oauth-accounts/src/component.rs`'s `redirected` call (`uri.expose()`); `skein-oauth/src/machine/fetch.rs`'s two (`Secret::new(url)`).
  - Tests and worlds that build or read them: `skein/tests/oauth`, `skein/tests/oauth-accounts`, `skein/tests/oauth-issuer`, `skein-oauth`'s `machine_tests`, `skein-oauth-accounts`'s `*_tests.rs` and `tests.rs`, `jig-protocol-accounts/src/tests`, `jig/tests/protocol-accounts/src/fixtures.rs`, `smith/tests/local-process/src/process/provider.rs`, `temper-fake`, `temper-service/src/tests`, `skein-fake-peers/src/oauth.rs`. A site that matches `{ .. }` doesn't change. `Secret::new(bytes::copy_of(..))` for a literal; a helper if a file builds many, not one per file.
- **Stays off:** `smith`'s and `jig`'s own `Visit` types and `temper-client-*`; none carries these fields.
- **Comments:** `boundary.rs`'s two "credential carriers print their lengths" comments go with their impls. `Secret` says it itself.
- **If a test asserts on the `Debug` text** of one of these (`url_len`, a printed trace), stop and ask with its name.

## Steps

1. `skein-oauth`, then `skein-fake-oauth`, then `skein-oauth-accounts`: each field and its readers; the compiler lists them.
2. The impls go; derive. `debug.rs`'s rows.
3. Other trees' readers.
4. `cargo fmt`; `python3 -I -B scripts/old_forms.py` in each of `skein`, `jig`, `smith`, `temper`; `heavy cargo clippy` naming every package changed, `--all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; how many reader lines changed per tree; what the two extra rows did.
