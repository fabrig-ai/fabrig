+++
id = "accounts-entries"
part = "oauth-states"
kind = "feature"
needs = ["accounts-record-states"]
files = ["skein/crates/skein-oauth-accounts/src/component.rs", "skein/crates/skein-oauth-accounts/src/component/**", "skein/crates/skein-oauth-accounts/src/accounts_tests.rs"]
effort = "medium"
minutes = 75
+++

# skein-oauth-accounts: one `AccountEntry` per account

## Goal

The entries' clause of `skein/docs/development/debt/D022-oauth-states.md`: five parallel lists become one list of entries. No public interface changes, and behaviour stays. Production falls by about 30 lines.

Acceptance:

- `rg -o '(self|component)\.(accounts|names|sources|bindings|keepers)\b' skein/crates/skein-oauth-accounts/src/component.rs skein/crates/skein-oauth-accounts/src/component` reads nothing. It reads 209 at this plan's base.
- ```rust
  struct AccountEntry { name: Name, state: State, source: Source, binding: Option<Id<Exchange>>, store: Store }
  pub struct Component { entries: List<AccountEntry>, exchanges, admissions, lifecycle, crypto, file_owner }
  ```
- `AccountEntry` has no `Debug`. `Component`'s hand-written `Debug` still prints `lifecycle` only.
- **`entries` isn't prefilled.** `List::with_capacity` allocates nothing, and prefilling would raise `Component::new`'s heap by `limits.accounts` entries. `names`' prefill of `Name::Free` goes.
- `add` takes the first `Name::Free` in `entries`; otherwise pushes if `entries.room() > 0`; otherwise `Full`. The refusal order stays `Closed`, `Taken`, `Full`, then `admit`, and the pin below holds it.
- `settled` clears `entries`, wherever it cleared the four lists.
- A full component given an unusable configuration answers `Full`, not `Unusable`: this pin passes on the base's code first, in its own commit, and unchanged after. Moving `admit` above the free-place search goes red.
- `heavy cargo nextest run -p skein-oauth-accounts -p skein-oauth-accounts-world` passes. `SHARED_RECORDS_CEILING` and `PRIVATE_KEEPERS_CEILING` measure this crate's heap, and the merged list grows by doubling: **over budget, stop and ask with the measurement.** Don't raise a ceiling or reshape the list to duck one.
- `heavy cargo check -p jig-protocol-accounts --tests` and `heavy cargo check -p smith-local-service --tests` pass: nothing public moved.

## Inputs

- `temper/docs/plan/debt-22/notes/common.md`
- `skein/docs/development/debt/D022-oauth-states.md`
- `skein/crates/skein-lib/src/list.rs`, `room`, `with_capacity`

## Owns and may change

- **Owns:** the files in the header. Tests read these lists at `component.rs`'s two `#[cfg(test)]` helpers, `component/names.rs`'s test, and `component/nonce_tests.rs`: those lines change too.
- **The index stays:** `place: u32` is one account's position in all five lists today, and in `entries` after. `Exchange.account` stays a place.
- **Left out:** a `Free` enum variant for an entry, accounts' `Exchange`, the unreachable arms `refused` and `sign_in.rs`'s.

## Steps

1. The `add` pin, on the base's code; commit.
2. The struct, then let the compiler lead. **Borrow traps:**
   - Don't write an `entry_mut(&mut self, ..)` helper: it borrows all of `self`, and breaks every site that also uses `self.exchanges`, `self.lifecycle` or `self.file_owner`. Use `self.entries.get_mut(place)`.
   - `number(&self.entries, place)` conflicts with a live `&mut` entry: add `Name::number(&self)`, and use `entry.name.number()` at those sites.
   - Where a site holds a `&mut` state and reads another list (`component.rs`'s `Rejected` arm, its grant arm and `fire`; `component/accounts.rs`, `lock.rs`, `private.rs`, `sign_out.rs`), take one `entry` and use its fields.
   - `Result::expect` needs `Debug`, which `AccountEntry` lacks: write `Err(AccountEntry { .. }) => unreachable!(..)`.
3. **Lengths:** `names` is longer than the others today, but every extra place is `Free`, and `bindings` isn't cleared by `settled`, but no place survives the clear. Neither is observable, so nothing keeps them.
4. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-oauth-accounts --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; the search's count; the rule broken and what failed; both ceilings' measurements before and after.
