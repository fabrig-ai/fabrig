+++
id = "forge-plan-failure"
part = "skein-forge-root"
kind = "feature"
needs = ["forge-decimal"]
files = [
  "skein/crates/skein-forge/src/plan.rs",
  "skein/crates/skein-forge/src/lib.rs",
  "skein/crates/skein-forge/src/forgejo/plans.rs",
  "skein/crates/skein-forge/src/forgejo/plans/failure.rs",
  "skein/crates/skein-forge/src/github/plans.rs",
  "skein/crates/skein-forge/src/github/plans/failure.rs",
]
effort = "medium"
minutes = 15
+++

# skein D010: one plan `Failure` at skein-forge's root

## Goal

`skein_forge::plan` holds `Failure` and `transport`, and the dialects re-export them. Then forge-connection can use one type. D010 stays open.

Acceptance:

- `rg -n 'enum Failure|fn transport' skein/crates/skein-forge/src` reads one of each, in `src/plan.rs`. Each dialect's `failure.rs` has its own classifying functions, which stay.
- `skein_forge::forgejo::plans::Failure` and `skein_forge::github::plans::Failure` name that type, and the dialects' `transport` paths still resolve.
- What `temper/docs/plan/debt-5/notes/forge-root.md` calls `What runs` passes.

## Inputs

- `temper/docs/plan/debt-5/notes/common.md`, `temper/docs/plan/debt-5/notes/forge-root.md`, `AGENTS.md`, `skein/AGENTS.md`.
- `skein/docs/design/forge.md`, 3.4 and 4.4.

## Owns and may change

- Owns: the files in the header.
- skein-forge-connection is not this task's: `connection-twins` follows, and until then its match on both dialects' `Failure` still compiles, the two being one type.

## Steps

1. `src/plan.rs`:

   ```rust
   //! What a plan's exchange can end in, in both dialects (forge.md, 4.4).
   use crate::api;
   /// Whether a broken exchange could have reached the server (forge.md, 3.4,
   /// "Unavailable is provable only before the first byte").
   #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
   pub enum Failure {
       /// (Forgejo's doc: it says what each is, for either dialect)
       Unseen,
       /// (likewise)
       Seen,
   }
   #[must_use]
   pub fn transport(failure: Failure) -> api::Error
   ```

2. `pub mod plan;` in `src/lib.rs`. Each dialect's `plans.rs` takes `pub use crate::plan::Failure;` and its `plans/failure.rs` takes `pub use crate::plan::transport;`.
3. Run `What runs`; commit (`D010: one plan Failure at skein-forge's root`, saying why) and push.

## Report

The lines added and removed.
