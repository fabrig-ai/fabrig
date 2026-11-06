+++
id = "link-values-fleet"
part = "link-values"
kind = "migration"
needs = ["link-readers-temper", "link-values-accounts"]
files = ["jig/crates/jig-domain-fleet/**", "jig/crates/jig-domain-core/**", "jig/crates/jig-domain-hub-link/**", "jig/crates/jig-protocol-engine-clients/src/**", "jig/crates/jig-protocol-accounts/**", "jig/tests/core/**", "jig/tests/fleet/**", "jig/tests/channel/**", "jig/testing/**", "jig/examples/ops/domain/**", "jig/tests/boundary/**", "jig/docs/development/debt/D006-link-values-two-paths.md", "Cargo.lock"]
effort = "medium"
minutes = 45
+++

# `jig-domain-fleet` re-exports no link value

## Goal

The third part of jig's D006. `jig-domain-fleet/src/lib.rs`'s two `pub use jig_domain_link` lines go, and jig's readers of its 22 names through it name `jig_domain_link`: 216 uses at the cut. `fleet::WorkerProblem` becomes `jig_domain_link::Problem`.

`jig-domain-core/src/limits.rs`'s `pub use fleet::WorkerProblem as Problem;`, with its name in `jig-domain-core/src/lib.rs`'s `pub use limits::{..}`, is the same path one crate further. It becomes a private `use jig_domain_link::Problem;`, and `Core::new` keeps its error type. A reader of `core::Problem` outside `jig-domain-core`, such as `jig/examples/ops/domain/src/domain.rs`, names `jig_domain_link::Problem`.

Acceptance:

- `rg -n 'pub use jig_domain_link' jig/crates/jig-domain-fleet` finds nothing.
- `rg -n 'pub use jig_domain_link' jig/crates | wc -l` reads 2.
- `rg -n 'WorkerProblem' jig temper -g '*.rs'` finds nothing, unless a file keeps it as its own `use .. as`, which you name.
- `rg -n 'pub use fleet::' jig/crates/jig-domain-core/src/limits.rs` finds nothing.
- The item's **Measure** says 2, and its **What** agrees.
- The net is at most +80 lines.

## Inputs

- `temper/docs/plan/debt-29/notes/link-values.md`, whole.
- `jig/docs/development/debt/D006-link-values-two-paths.md`.
- `jig/docs/design/domain/root.md`, 2.
- `temper/docs/plan/debt-29/notes/common.md`.

## Owns and may change

Owns the `use` lines and paths of these values in jig, each manifest's dependency line, and the item. Nothing else of those files. `Cargo.lock` follows by cargo.

Start by deleting the two lines: the compiler then names every reader, and the packages the cut saw are `jig-domain-core`, `jig-domain-hub-link`, `jig-protocol-engine-clients`, `jig-protocol-accounts`, `jig-fleet-world`, `jig-core-world`, `jig-test-domain`, `jig-channel-kit`, `jig-core-kit`, `jig-test-service`, `jig-test-profile`, `jig-client-kit`, `jig-ops-domain` and `jig-channel-world`.

- **The boundary's lists:** as `link-values-accounts` did, a list a new `jig-domain-link` line breaks gains that one name, and the core family's check already allows it. Report each list you change.
- **`fleet::Operation`** is link's. `fleet::Attempt..` and the rest of `boundary.rs`'s are fleet's own, and stay.

## Steps

Follow the note. Keep a private `use jig_domain_link::{..};` at `jig-domain-fleet`'s root for its own `crate::` readers, renaming `crate::WorkerProblem` there only if the private `use` can't keep it: `use jig_domain_link::Problem as WorkerProblem;` inside the crate is fine. Then what the note's **What runs** lists. Commit, push.

## Report

As your text says, with the searches' readings, the lists you changed, any temper line the note's temper check made, and the net lines.
