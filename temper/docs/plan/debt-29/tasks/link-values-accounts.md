+++
id = "link-values-accounts"
part = "link-values"
kind = "migration"
needs = ["link-readers-temper"]
files = ["jig/crates/jig-domain-accounts/**", "jig/crates/jig-domain-hub-link/**", "jig/crates/jig-domain-core/**", "jig/crates/jig-protocol-accounts/**", "jig/tests/core/**", "jig/tests/accounts/**", "jig/tests/ops/**", "jig/tests/protocol-accounts/**", "jig/testing/**", "jig/examples/ops/domain/**", "jig/tests/boundary/**", "jig/docs/development/debt/D006-link-values-two-paths.md", "Cargo.lock"]
effort = "medium"
minutes = 30
+++

# `jig-domain-accounts` re-exports no link value

## Goal

The second part of jig's D006. `jig-domain-accounts/src/lib.rs`'s two `pub use jig_domain_link` lines go, and jig's readers of `SourceName`, `Worker`, `Form`, `Grant`, `GrantTable` and `Scheme` through it name `jig_domain_link`: 109 uses at the cut. Temper's already do (`link-readers-temper`).

Acceptance:

- `rg -n 'pub use jig_domain_link' jig/crates/jig-domain-accounts` finds nothing.
- `rg -n 'pub use jig_domain_link' jig/crates | wc -l` reads 4.
- The item's **Measure** says 4, and its **What** agrees.
- The net is at most +50 lines.

## Inputs

- `temper/docs/plan/debt-29/notes/link-values.md`, whole.
- `jig/docs/development/debt/D006-link-values-two-paths.md`.
- `jig/docs/design/domain/root.md`, 2.
- `temper/docs/plan/debt-29/notes/common.md`.

## Owns and may change

Owns the `use` lines and paths of these six values in jig, each manifest's dependency line, and the item. Nothing else of those files. `Cargo.lock` follows by cargo.

Start by deleting the two lines: the compiler then names every reader, and the packages the cut saw are `jig-domain-core`, `jig-domain-hub-link`, `jig-protocol-accounts`, `jig-accounts-world`, `jig-core-world`, `jig-test-domain`, `jig-test-service`, `jig-core-kit`, `jig-ops-domain`, `jig-ops-world`, `jig-protocol-accounts-world`, `jig-conformance` and `jig-client-protocol-kit`.

- **The boundary's lists:** `jig/tests/boundary/src/tests/graph.rs` and `tests.rs` hold some crates' exact dependencies. A list a new `jig-domain-link` line breaks gains that one name, as `jig/docs/design/domain/root.md`, 2's "a shared leaf value crate, like skein-lib" allows. A test whose name says what a crate links keeps its name. Report each list you change.
- **A reader in `jig-client-view`** is a question: the cut found none.

## Steps

Follow the note. Keep a private `use jig_domain_link::{..};` at `jig-domain-accounts`'s root for its own `crate::` readers. Then what the note's **What runs** lists. Commit, push.

## Report

As your text says, with the searches' readings, the lists you changed, and the net lines.
