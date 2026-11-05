+++
id = "accounts-lending"
part = "spot-review"
kind = "feature"
needs = []
files = ["skein/crates/skein-oauth-accounts/src/component.rs", "skein/crates/skein-oauth-accounts/src/component/**", "skein/crates/skein-oauth-accounts/src/accounts_tests.rs", "skein/crates/skein-oauth-accounts/src/tests.rs"]
effort = "high"
minutes = 50
+++

# skein-oauth-accounts: `Notice` is two lifecycles, and `has_work` agrees with `fire`

## Goal

Finding 5 of `temper/docs/plan/debt-24/inputs/debt22-review.md`, the last clause of finding 7, and the review's second correctness note. `Notice` is the product of two independent facts, which `skein/docs/foundation/programming-model.md`, 5.4 says is one enum per lifecycle. `has_work` answers true for a due record `fire` then skips.

Acceptance:

- **`Notice` goes.** `State::Record` holds two fields of two two-variant enums in its place, one for the expiring notice having been announced and one for the provider having rejected the token, each named for what it says. Their four methods go, and each site reads the field.
- **`Refresh` stays:** it's a real machine.
- **`held` returns `Lending` only if it drops net lines.** The two sites building a `Lending` from it (`lock.rs`'s `holding`, `component.rs`'s `lending`) would lose their `if`; the eight others would gain a comparison. Otherwise it stays, and the report says so.
- **The correctness note, pin first.** `has_work`'s last loop says a `State::Record { refresh, earliest: None, .. }` with `refresh.due()` has work, in `Lifecycle::Live` with exchange room. `fire` starts it only if the account is live, `entry.binding.is_none()` and `record.refresh_token.is_some()`. So with a sign-in running over a record past its lead (`fire` sets `Due` regardless of the binding), or a token-less record the provider has rejected (`renewal.request()`), `has_work` reads true and `fire` does nothing, and `jig-protocol-accounts`'s `work_pending` reads true with it.
  - Write a test in `accounts_tests.rs`: drive to that state, `fire`, and assert `!owner.component.has_work()`. Run it on today's code and see it fail. **If the state can't be reached**, the note is wrong: report what stops it, delete the test, and change nothing.
  - The fix: the arm asks what `fire` asks, the same three conditions. Run the test again.
  - **If any other test's visible events or outcome change, stop and ask.**
- `queued_rejection` (`accounts_tests.rs`) reads clearer: `blocker` and the bare `20` and `30` have names saying what they are. Nothing it does changes.
- `heavy cargo nextest run -p skein-oauth-accounts --lib`, `-p skein-oauth-accounts-world`, `-p jig-protocol-accounts --lib` and `-p jig-protocol-accounts-world` pass with no other assertion changed: `has_work` is read by those packages.
- Production's net lines fall by about 15 before the fix's two lines.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/inputs/debt22-review.md`, findings 5, 7 and the correctness notes
- `skein/docs/foundation/programming-model.md`, 5.4
- `skein/docs/design/oauth.md`, 6.2, 6.4

## Owns and may change

- **Owns:** the files in the header.
- **`AccountEntry`'s methods and `names::number` aren't yours:** `accounts-entry` takes them after you. Leave `self.entries.get…` as it is where you aren't changing the line.

## Steps

1. The pin; see it fail; the fix.
2. `Notice`'s two enums, then `held`.
3. `queued_rejection`'s names.
4. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-oauth-accounts --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; the state the pin reached and the failure it showed before the fix, or why the note is wrong; `held`'s call.
