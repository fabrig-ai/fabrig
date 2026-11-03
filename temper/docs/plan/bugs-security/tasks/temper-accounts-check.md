+++
id = "temper-accounts-check"
part = "crypto-rng"
kind = "migration"
needs = ["oauth-accounts-check"]
files = [
  "temper/crates/temper-service/src/accounts.rs",
]
effort = "medium"
minutes = 30
+++

# temper-service checks provider templates without copying its generator

## Goal

`temper/crates/temper-service/src/accounts.rs`'s `provider_templates` (~103) checks each
runtime provider's account by making a throwaway accounts component with `crypto.clone()`
(~134): a copy of the `CryptoRng` that `skein/docs/design/lib.md`, 8 forbids, and the copy
`CryptoRng` losing `Clone` will refuse to compile. Check it with `driver::Component::check`
(`skein/docs/design/oauth.md`, 6.7), which takes no generator, and drop the generator from
`provider_templates`.

Acceptance:

- `provider_templates` takes no `CryptoRng`; its check is
  `driver::Component::check(check, limits)`, mapped to the same problem as now.
- `Accounts::new` makes its one generator (stream 1) as now and hands it to the accounts
  component as now; nothing else in the file draws, clones or makes a generator.
- No test changes: the accounts tests pass as they are.
- clippy clean.

## Inputs

- `temper/docs/plan/bugs-security/notes/common.md`.
- `temper/AGENTS.md`.
- `temper/docs/plan/bugs-security/inputs/request.md`, item 3.
- `skein/docs/design/lib.md`, 8 (`CryptoRng` neither `Copy` nor `Clone`) and
  `skein/docs/design/oauth.md`, 6.7 (`Component::check`), both written by this cut;
  `Component::check` is on the trunk from `oauth-accounts-check`.

## Owns and may change

- `temper/crates/temper-service/src/accounts.rs`: `provider_templates`' parameter list and its
  check, its one call in `Accounts::new` (~173), and the `use skein_lib::{..}` line if needed.

Nothing else: `src/tests/accounts_host.rs` (included at ~439 through `#[path]`) stays as it is.

Pre-approved options:

- If `driver::Component::check`'s signature on the trunk differs from `check(accounts:
  List<Account>, limits: &Limits) -> Result<(), Unusable>`, follow the trunk's and say so.

## Steps

1. The change above; `heavy cargo check -p temper-service --tests`.
2. Narrow tests, one at a time:
   - `heavy cargo nextest run -p temper-service --lib -E 'test(/^accounts::tests::/)'`:
     `dynamic_sources_use_provider_templates_and_restore_their_kept_records`,
     `a_restored_sign_ins_metadata_leaves_its_grant_a_bearer`,
     `missing_registration_runtime_is_refused_before_start` and the sign-in world's tests among
     them (check the module path in nextest's list; the count is not zero).
   - `heavy cargo nextest run -p temper-service --lib`.
   - `heavy cargo check -p temper --tests` (the package whose `tests/support/owned.rs` builds the
     accounts runtime).
3. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `temper/`;
   `heavy cargo clippy -p temper-service --all-targets -- -D warnings`.
4. One commit saying why; push.

## Report

As the implementer's text says, and each test run's count.
