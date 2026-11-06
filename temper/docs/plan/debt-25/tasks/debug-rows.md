+++
id = "debug-rows"
part = "spot-review"
kind = "feature"
needs = []
files = ["skein/tests/oauth-accounts/tests/suite/debug.rs", "skein/docs/design/oauth.status.md"]
effort = "high"
minutes = 35
+++

# oauth debug views: rows where a `Debug` is written by hand

## Goal

Finding 7's third pin in `temper/docs/plan/debt-25/inputs/debt23-review.md`, checked against the code. `debug_hides_every_oauth_credential_and_carrier` has 25 rows, but all except three read a derived `Debug` over `Secret` fields, so they fail together or not at all. The hand-written views of `accounts::Component` and `fake::Issuer` have no row (`skein/docs/design/oauth.status.md`, 4).

Acceptance:

- The table holds one row for a derived `Debug` over `Secret` (`oauth::SavedToken`), and one for each hand-written view reachable from here: `oauth::HttpRequest`, `oauth::HttpResponse`, `oauth::Client`, `accounts::Component`, `fake::Issuer`. The other rows go, and any helper only they read.
- `accounts::Component` and `fake::Issuer` hold the canary when they print: a component told of a held `record()` (`Request::Grant`, then `Request::Locked` with `Locked::Held`, or whichever is shortest), an issuer made with `client_secret` and `refresh_token` of `secret()`. Each row can fail: printing that credential's bytes from the hand-written `Debug` turns it red.
- Rule broken by hand for each kind of row: `Secret`'s own `Debug` printing its bytes (the `SavedToken` row); `HttpRequest` printing its body; `Component` and `Issuer` each printing the credential they hold. Restore each.
- The test's name stops saying "every": it says what the table now holds. `assert_hidden` stays as it is.
- The file's net lines fall (about -80).
- `skein/docs/design/oauth.status.md`, 4's sentence that the table has no such rows goes; its `Json` and `Collector` sentence stays.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/inputs/debt23-review.md`, finding 7
- `skein/docs/design/oauth.md`, 4
- `skein/docs/design/oauth.status.md`

## Owns and may change

- **Owns:** `skein/tests/oauth-accounts/tests/suite/debug.rs`, and in `skein/docs/design/oauth.status.md` only the sentence above, which departs from the implementer's text.
- **Dependencies:** a dev line in `skein/tests/oauth-accounts/Cargo.toml` if the rows need one; the world's helpers (`skein_oauth_accounts_world::world::limits`, its account fixtures) are there to read and not to change.
- **If a row can't hold a credential in about ten lines,** build it with what it has and say so; if a hand-written view has nothing it could print, drop its row and say so in the report, and leave the status sentence naming that one.

## Steps

1. Cut the table, then add the two rows. Break each rule as above, see the row named in the failure, restore.
2. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-oauth-accounts-world --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-oauth-accounts-world --test suite -E 'test(debug)'`. Commit, run the citations tool and push.

## Report

The common note's items; rows before and after; what you broke and what failed, for each row.
