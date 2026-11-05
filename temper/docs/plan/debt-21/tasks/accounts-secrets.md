+++
id = "accounts-secrets"
part = "oauth-secrets"
kind = "interface"
needs = ["oauth-exchange"]
files = ["skein/crates/skein-oauth-accounts/src/**"]
effort = "high"
minutes = 60
+++

# skein-oauth-accounts: credentials are `Secret`, and the crate's expects go

## Goal

The accounts part of `skein/docs/development/debt/D009-oauth-credentials-raw.md`. Every credential `skein-oauth-accounts` holds is `Secret`, and it keeps no `missing_debug_implementations` expect.

Acceptance:

- `Event::Granted.token` and every owned record, kept or held, are `Secret` (`Request::HandIn.record`, `State.record`, `Stage::Keeping`, `Locked::Held.record`, `Keeper::Owner.kept`, `redirect.rs`'s `Fields`), where they aren't already a `SavedToken`. A record that is a `SavedToken` stays one.
- `Locked`, `Account`, `Keeper` and `FileLower` derive `Debug`. `Request` and `Event` have one printing their carriers' lengths, or derive it if every carrier's type already does that. `Component`'s stops at its edge: never through `Web`, as the note says.
- `accounts.rs`'s `large_enum_variant` expect stays. `rg -c 'missing_debug_implementations' skein/crates/skein-oauth-accounts` reads nothing.
- `heavy cargo nextest run -p skein-oauth-accounts -p skein-oauth-accounts-world` passes, with no assertion's meaning changed: `suite/memory.rs` and `suite/private.rs` among them.
- `jig-protocol-accounts` and `jig-protocol-accounts-world` tests pass: `jig/crates/jig-protocol-accounts/src/moved.rs` writes these records through `encode_record`, and its bytes don't change.
- Every reader of `skein-oauth-accounts` in the note's Readers passes `heavy cargo check -p <package> --tests`.

## Inputs

- `temper/docs/plan/debt-21/notes/common.md`
- `temper/docs/plan/debt-21/notes/oauth-secrets.md`, binding
- `skein/docs/development/debt/D009-oauth-credentials-raw.md`
- `skein/docs/design/oauth.md`, 4; `skein/docs/design/lib.md`, 6
- `skein/crates/skein-lib/src/secret.rs`; the model, `skein/crates/skein-oauth-issuer/src/lib.rs` and `src/requests.rs`

## Owns and may change

- **Owns:** `skein/crates/skein-oauth-accounts/src/**`.
- **These public fields change type:** the request asks for it. No other public interface changes.
- **Readers:** the note's rule. The production lines the cut expects: `jig/crates/jig-protocol-accounts/src/translate.rs`, `smith/crates/smith-local-service/src/credentials/terminals.rs` (`Ready.value` stays a box), `smith/crates/smith-local-shell/src/credentials.rs`, `temper/crates/temper-service/src/**`.
- **Stored bytes:** record bytes written through `component.rs`, `component/accounts.rs` and `component/private.rs` go out with `expose()`, the same bytes. If a test of a stored form moves, stop and ask.
- **`Phase` and `AccountEntry` keep their shape** (`skein/docs/development/debt/D022-oauth-states.md`).

## Steps

1. The fields; the `Debug`s; the crate's tests.
2. The readers, by tree.
3. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`, and each other tree you changed; `heavy cargo clippy` over the packages you changed, `--all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; the **Measure**. No new test here: `oauth-canary` holds it.
