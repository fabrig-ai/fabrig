+++
id = "oauth-accounts-check"
part = "crypto-rng"
kind = "interface"
needs = []
files = [
  "skein/crates/skein-oauth-accounts/src/component.rs",
  "skein/crates/skein-oauth-accounts/src/tests.rs",
  "skein/docs/design/oauth.status.md",
]
effort = "medium"
minutes = 40
+++

# skein-oauth-accounts checks accounts without a generator

## Goal

jig-test-service and temper-service check each provider's account template by making a
throwaway `Component` with `crypto.clone()`: the one copy of a `CryptoRng` in their code, which
`CryptoRng` losing `Clone` (`skein/docs/design/lib.md`, 8) forbids. `Component::new` never draws
from its generator; it only checks and stores it. Give the crate `Component::check`, the check
half of `new`, taking no generator (`skein/docs/design/oauth.md`, 6.7), and make `new` use it.

Acceptance:

- `pub fn check(accounts: List<Account>, limits: &Limits) -> Result<(), Unusable>`, an associated
  function of `Component`, refuses exactly what `new` refuses with the same `Unusable` (too many
  accounts, invalid limits, and each account's refusal from `accounts::admit`), in the same
  order, and accepts what `new` accepts.
- `new` refuses by the same code (one checking path, no copy of the checks), and every existing
  test of the crate passes unchanged.
- Tests below; clippy clean.
- `oauth.status.md` drops the entry this closes.

## Inputs

- `temper/docs/plan/bugs-security/notes/common.md`.
- `skein/AGENTS.md`.
- `temper/docs/plan/bugs-security/inputs/request.md`, item 3.
- `skein/docs/design/oauth.md`, 6.7 (the sentence on `Component::check`, written by this cut)
  and 6.2; `skein/docs/design/lib.md`, 8 (`CryptoRng` neither `Copy` nor `Clone`, written by
  this cut).
- `skein/crates/skein-oauth-accounts/src/component.rs`, `Component::new` (~139), and
  `component/accounts.rs`'s `admit` (~10), which `check` calls as `new` does.

## Owns and may change

- `skein/crates/skein-oauth-accounts/src/component.rs`: `Component::check`, and `new` reshaped to
  use the same checks; nothing else of the component.
- `skein/crates/skein-oauth-accounts/src/tests.rs`: the tests below, and the two refusal helpers
  named in Steps extended; every existing assertion kept.
- `skein/docs/design/oauth.status.md`: its **Not yet** line deleted, so the body reads `No gap.`
  as before; the date line today's.

Nothing else. `lib.rs` already exports `Component`. The callers in jig and temper are later
tasks (`jig-accounts-check`, `temper-accounts-check`): this task leaves them as they are.

Pre-approved options:

- If `admit` consumes what `new` then keeps, `new` may run its own loop over `admit` while
  `check` runs the same loop and drops the results, provided the refusals come from one private
  function both call (the limits and count checks, then `admit` per account). Say which shape
  you took.
- If `check` would need anything else of `new`'s (a `Name` list, a `Slab`), it allocates nothing
  beyond what `admit` allocates; say so in the report if it cannot.

## Steps

1. In `component.rs`:

   ```rust
   impl Component {
       /// Checks accounts against the limits as making the component does, making nothing
       /// and taking no generator (oauth.md, 6.7).
       pub fn check(accounts: List<Account>, limits: &Limits) -> Result<(), Unusable> { .. }
       pub fn new(accounts: List<Account>, limits: &Limits, crypto: CryptoRng) -> Result<Component, Unusable> { .. }
   }
   ```

2. Tests in `src/tests.rs` (the crate's lib tests, module `tests`):
   - `refuses_limits` (~669): also `assert_eq!(Component::check(List::with_capacity(0), bounds).err(), Some(Unusable::Limits))`.
   - `a_plaintext_endpoint_off_loopback_is_refused_at_startup` (~437): `check` of the same
     account gives `Unusable::Plaintext { account: 0 }`.
   - `check_accepts_what_new_accepts`: two `Account::HandedIn` under `limits()` (as the fixture
     at ~75 builds them): `check` is `Ok(())`.
   - `check_refuses_more_accounts_than_the_bound`: three accounts under `limits()` (`accounts:
     2`): `check` and `new` both `Err(Unusable::Accounts { bound: 2 })`.
3. Narrow tests, one at a time:
   - `heavy cargo nextest run -p skein-oauth-accounts --lib -E 'test(/check_|refuses_limits|valid_refuses|plaintext_endpoint/)'`.
   - `heavy cargo nextest run -p skein-oauth-accounts --lib`.
   - `heavy cargo nextest run -p skein-oauth-accounts-world` (`skein/tests/oauth-accounts`, the
     world that builds components through `new`).
4. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `skein/`;
   `heavy cargo clippy -p skein-oauth-accounts --all-targets -- -D warnings`;
   `python3 -I -B tools/citations/citations.py --base origin/main` from the root.
5. One commit for the code and its tests, one for `oauth.status.md`, each saying why; push.

## Report

As the implementer's text says, and: the shape `new` and `check` share, and each test's count.
