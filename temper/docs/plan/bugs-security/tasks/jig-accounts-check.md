+++
id = "jig-accounts-check"
part = "crypto-rng"
kind = "migration"
needs = ["oauth-accounts-check", "jig-resources-debug"]
files = [
  "jig/testing/jig-test-service/src/accounts.rs",
]
effort = "medium"
minutes = 30
+++

# jig-test-service checks provider templates without copying its generator

## Goal

`jig/testing/jig-test-service/src/accounts.rs`'s `provider_templates` (~107) checks each
runtime provider's account by making a throwaway accounts component with `crypto.clone()`
(~141): a copy of the `CryptoRng` that `skein/docs/design/lib.md`, 8 forbids, and the copy
`CryptoRng` losing `Clone` will refuse to compile. Check it with `driver::Component::check`
(`skein/docs/design/oauth.md`, 6.7), which takes no generator, and drop the generator from
`provider_templates`.

Acceptance:

- `provider_templates` takes no `CryptoRng`; its check is
  `driver::Component::check(check, limits)`, mapped to the same `problem(.., What::Form)` as now.
- `Accounts::new` makes its one generator (stream 1) as now and hands it to the jig accounts
  component as now; nothing else in the file draws, clones or makes a generator.
- No test changes: the accounts world's tests pass as they are.
- clippy clean.

## Inputs

- `temper/docs/plan/bugs-security/notes/common.md`.
- `jig/AGENTS.md`.
- `temper/docs/plan/bugs-security/inputs/request.md`, item 3.
- `skein/docs/design/lib.md`, 8 (`CryptoRng` neither `Copy` nor `Clone`) and
  `skein/docs/design/oauth.md`, 6.7 (`Component::check`), both written by this cut;
  `Component::check` is on the trunk from `oauth-accounts-check`.

## Owns and may change

- `jig/testing/jig-test-service/src/accounts.rs`: `provider_templates`' parameter list and its
  check, its one call in `Accounts::new` (~178), and the `use skein_lib::{..}` line if
  `CryptoRng` is still named there (it is: `Accounts::new` makes one).

Nothing else.

Pre-approved options:

- If `driver::Component::check`'s signature on the trunk differs from `check(accounts:
  List<Account>, limits: &Limits) -> Result<(), Unusable>`, follow the trunk's and say so.

## Steps

1. The change above; `heavy cargo check -p jig-test-service --tests`.
2. Narrow tests, one at a time (none of jig-test-service's own tests reach a runtime with
   providers; the one-process world's do):
   - `heavy cargo nextest run -p jig-one-process-world --test suite -E 'test(/accounts::/)'`:
     `a_person_signs_in_uses_the_plan_and_signs_out`, `a_refused_plan_grant_fails_the_run`,
     `a_restored_sign_in_is_refreshed_and_signs_out` among them.
   - `heavy cargo nextest run -p jig-test-service --lib`.
3. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `jig/`;
   `heavy cargo clippy -p jig-test-service --all-targets -- -D warnings`.
4. One commit saying why; push.

## Report

As the implementer's text says, and each test run's count.
