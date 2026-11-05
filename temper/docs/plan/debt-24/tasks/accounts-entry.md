+++
id = "accounts-entry"
part = "spot-review"
kind = "feature"
needs = ["accounts-lending"]
files = ["skein/crates/skein-oauth-accounts/src/component.rs", "skein/crates/skein-oauth-accounts/src/component/**"]
effort = "high"
minutes = 75
+++

# skein-oauth-accounts: `AccountEntry` answers for itself

## Goal

Finding 4 of `temper/docs/plan/debt-24/inputs/debt22-review.md`, checked against the code. The record went into one `AccountEntry`, but the code still asks for it by position: 92 `expect("account entry")` under `component.rs` and `component/`, and `number(&self.entries, place)` 31 times beside `entry.name.number()` 22.

Acceptance:

- `live`, `wanted`, `refreshable` and `refresh_ready` are methods on `AccountEntry`; the `Component` methods of those names go. `refresh_ready` keeps its `now` parameter. `is_removing` follows, if it reads the same.
- `names::number` goes: `rg -n 'number\(&self\.entries' skein/crates/skein-oauth-accounts/src` reads 0.
- **One `let entry` per function:** a function takes its entry once and reads through it. `rg -c 'expect\("account entry"\)' skein/crates/skein-oauth-accounts/src/component.rs skein/crates/skein-oauth-accounts/src/component/` falls by at least a third, and a function holding two has a `&mut self` call or a loop between them. Nothing else is left as a second lookup.
- **Out of range stays as it is.** `refreshable` answers `false` on an account past the list, the others panic. Check `refreshable`'s callers: if every one has its account checked, the method takes the entry. If one doesn't, that caller answers `false` on a `None` itself.
- No test's assertion changes. `heavy cargo nextest run -p skein-oauth-accounts --lib`, `-p skein-oauth-accounts-world` and `-p jig-protocol-accounts --lib` pass.
- Production's net lines fall by about 50.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/inputs/debt22-review.md`, finding 4
- `skein/docs/design/oauth.md`, 6.2, 6.4

## Owns and may change

- **Owns:** the files in the header. `skein/crates/skein-oauth-accounts/src/*_tests.rs` and `component/nonce_tests.rs` only if a test calls a moved method, in its spelling.
- **A borrow the compiler refuses** across a `&mut self` call stays a second lookup: don't copy a field out or restructure a function to save a line. Say how many in the report.
- **No lifecycle changes:** `State`, `Lending`, `Refresh` and what `accounts-lending` put in their place stay as they are.

## Steps

1. The four methods and `names::number`, then `component.rs`, then one `component/` file at a time. Check with `heavy cargo check -p skein-oauth-accounts --tests` as you go.
2. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-oauth-accounts --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; the counts before and after. This is a move: no rule to break by hand.
