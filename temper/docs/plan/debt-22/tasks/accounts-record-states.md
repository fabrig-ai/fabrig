+++
id = "accounts-record-states"
part = "oauth-states"
kind = "feature"
needs = []
files = ["skein/crates/skein-oauth-accounts/src/component.rs", "skein/crates/skein-oauth-accounts/src/component/accounts.rs", "skein/crates/skein-oauth-accounts/src/component/lock.rs", "skein/crates/skein-oauth-accounts/src/component/sign_in.rs", "skein/crates/skein-oauth-accounts/src/component/private.rs", "skein/crates/skein-oauth-accounts/src/component/sign_out.rs", "skein/crates/skein-oauth-accounts/src/accounts_tests.rs"]
effort = "high"
minutes = 60
+++

# skein-oauth-accounts: a record's flags become one enum per lifecycle

## Goal

The record's clause of `skein/docs/development/debt/D022-oauth-states.md`: `State::Record`'s five flags and `rejection` become three lifecycles. `skein/docs/foundation/programming-model.md`, 5.4 says one enum per lifecycle and never one for their product, so these are three fields, not 32 `State` variants. No public interface changes, and behaviour stays. Production grows by about 35 lines; the cut's cap is 60.

Acceptance:

- `rg -n '\b(announced|held|rejected|attempted|due): (bool|true|false)\b|\brejection\b' skein/crates/skein-oauth-accounts/src/component.rs skein/crates/skein-oauth-accounts/src/component` reads nothing. It reads 22 at this plan's base.
- `State::Record` is `{ record, expiry, lead, earliest, lending: Lending, refresh: Refresh, notice: Notice }`, with:

  ```rust
  enum Lending { Free, Held }                     // a grant is lent or asked
  enum Refresh { Untried, Due, Spent, DueAgain }  // Spent: started, or impossible with no refresh token
  enum Notice { Quiet, Announced, Rejected, RejectedAnnounced }
  ```

- `rejection` goes: it's `Some(generation)` exactly when `rejected`, since `record` never changes inside `Record` and only `component.rs`'s `recorded` writes `rejection: None`. `repeated` is the old `rejected`. If you find a record changing generation in place, stop and ask.
- `State` has no `Debug` and gains none; `Lending`, `Refresh` and `Notice` may derive it, as they hold no credential.
- `recorded`, `just_kept` and `set_held` take `Lending`.
- The pins below pass on the base's code first, in their own commit, and unchanged after.
- `heavy cargo nextest run -p skein-oauth-accounts -p skein-oauth-accounts-world` passes, `SHARED_RECORDS_CEILING` and `PRIVATE_KEEPERS_CEILING` included: over budget, stop and ask with the measurement.

## Inputs

- `temper/docs/plan/debt-22/notes/common.md`
- `skein/docs/development/debt/D022-oauth-states.md`
- `skein/docs/foundation/programming-model.md`, 5.4

## Owns and may change

- **Owns:** the files in the header, the flags' lines only. `accounts-entries` follows you and reshapes the lists: don't merge any list here.
- **Pins go in** `accounts_tests.rs`, one table. At most 60 new lines.
- **Left out, not needed for the item:** accounts' `Exchange` and its 19 fields, `pending_grant`, `aborted`, `file_active`, splitting `held` into asked and lent, `Loaded`.

## Steps

1. **Pins first,** on the base's code, as rows. Search first: skip a row an existing test holds, and say so.
   - `Rejected` after a failed refresh (the `DueAgain` value) refreshes once more, and a grant meanwhile takes the "no refresh" path in `component.rs`'s `None =>` arm of its grant-or-refresh decision.
   - `due` survives the direct `held = false` sites and is cleared by `set_held(false)`: `has_work` shows it.
   - A rejected record announces no `Expiring`.
2. **The moves, keeping each exact:**
   - `Refresh`: a start goes to `Spent`. `Rejected`: `Untried | Due` to `Due`, `Spent | DueAgain` to `DueAgain`. Forgoing: `Due` to `Untried`, `DueAgain` to `Spent`. `fire` at lead: `Untried | Due` to `Due`, or to `Spent` with no refresh token.
   - `Notice`: `Announced` from `Quiet`; `RejectedAnnounced` when both have happened, in either order; only a new record clears them.
   - Reads are short private methods (`fn due`, `fn attempted`, `fn announced`, `fn rejected`), each an exhaustive `match`.
3. **Trap, keep it exact:** `set_held(false)` clears `due`. The sites in `component.rs` that set `*held = false` directly (in the `Rejected` arm, `fire`, `grant`) leave it. `accounts.rs`'s site clears `due` only when not wanted. Don't unify them.
4. Break a pin by hand: make `set_held(false)` keep `Due`, see its row red; restore.
5. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-oauth-accounts --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; the search's count; the rule broken and what failed; the pins an existing test already held.
