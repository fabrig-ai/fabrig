+++
id = "oauth-sign-in"
part = "spot-review"
kind = "feature"
needs = []
files = ["skein/crates/skein-oauth/src/machine.rs", "skein/crates/skein-oauth/src/machine/fetch.rs", "skein/crates/skein-oauth/src/machine/revoke.rs"]
effort = "high"
minutes = 45
+++

# skein-oauth: the sign-in's fields live in `Grant::SignIn`

## Goal

Finding 3 of `temper/docs/plan/debt-24/inputs/debt22-review.md`, checked against the code. `Exchange` holds `state`, `verifier`, `nonce`, `visit`, `fetch` and `keys` for every grant, and only a sign-in reads them: `refresh` and `start_revocation` fill them with `None` and an empty-secret sentinel.

Acceptance:

- ```rust
  enum Grant {
      SignIn {
          key: u32,
          generation: u64,
          code: Option<Secret>,
          state: Secret,
          verifier: Option<Secret>,
          nonce: Option<Secret>,
          visit: Option<Box<[u8]>>,
          fetch: Option<fetch::Fetch>,
          keys: Box<[crate::keys::Key]>,
      },
      Refresh { prior: RefreshState },
      Revoke { prior: RefreshState },
  }

  struct Exchange { registration: Registration, grant: Grant, deadline: Time, attempts: u32 }
  ```

  `Secret::new(bytes::copy_of(b""))` is gone from `machine.rs` and `machine/revoke.rs`.
- **These fields stay out of `Debug`:** neither `Grant`, `Exchange` nor a struct holding their fields derives or implements it.
- `refresh` and `start_revocation` share one private check for their common lines (the token's length, `validate_refresh_additions`, `issued_id`'s `client_id`), called at the same point in each, so each failure comes in today's order. `start_revocation`'s `endpoint` check stays before it.
- `keys_request`'s `_now` goes, with its callers' argument.
- The state comparison in the redirect's admission reads `Grant::SignIn`'s `state`; `Grant::Refresh | Grant::Revoke` take the same answer as `exchange.grant`'s other sign-in-only reader beside it (`Failure::Malformed`, as `Inadmissible::Address`), which `Phase::AwaitRedirect` can't reach: only a sign-in visits. `exchange.fetch.is_some()`'s three readers ask one `Exchange::fetching(&self) -> bool`.
- `heavy cargo nextest run -p skein-oauth --lib` and `-p skein-oauth-accounts --lib` pass with no assertion changed, and `heavy cargo nextest run -p skein-oauth-accounts-world` does.
- Production's net lines fall by about 25.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/inputs/debt22-review.md`, finding 3
- `skein/docs/design/oauth.md`, 3, 4
- `skein/docs/foundation/programming-model.md`, 5.4

## Owns and may change

- **Owns:** the files in the header. `skein/crates/skein-oauth/src/machine_tests/**` only if a test reads a moved field, in its spelling.
- **`Grant::SignIn` may hold a small boxed struct instead** of nine fields, if that makes the code shorter; the same rule on `Debug` holds.
- **A memory pin** in `skein-oauth` or `skein-oauth-accounts` that shrinks is updated, and the report says so; one that grows stops and asks. By the cut's search there is none on `Exchange`.

## Steps

1. The fields; the readers; the sentinel's two sites.
2. The shared check; `_now`.
3. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-oauth --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items. This is a move of state: no rule to break by hand.
