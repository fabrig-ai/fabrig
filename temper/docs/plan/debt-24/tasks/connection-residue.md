+++
id = "connection-residue"
part = "connection-worlds"
kind = "feature"
needs = ["hand-seeded"]
files = ["skein/crates/skein-forge-connection/src/tests.rs", "skein/crates/skein-forge-connection/src/tests/**", "skein/crates/skein-forge-connection/src/component/tests.rs", "skein/crates/skein-forge-connection/src/connection/**/tests.rs", "skein/crates/skein-forge-connection/Cargo.toml", "skein/tests/forge/machine/src/hand.rs", "Cargo.lock"]
effort = "high"
minutes = 30
+++

# forge-connection's crate tests feed its step functions

## Goal

Skein D030's **Paid when**, the crate's half. `Harness` leaves the crate, and what stays feeds step functions events (`skein/docs/foundation/testing-strategy.md`, 2.1).

Acceptance:

- `rg -lw Harness skein/crates/skein-forge-connection/src` reads nothing. `ConnectionHarness` and `HookHarness` stay, as `temper/docs/plan/debt-24/notes/hand.md` says.
- `src/tests/fixtures.rs` holds `ConnectionHarness`, `HookHarness` and the fixtures the crate's tests read, and no more. `hand.rs`'s temporary copies, and the crate's of helpers nothing in the crate reads, go.
- **`component/tests.rs`' three `Harness` tests** can't move: they read `common`, `forges`, `own_closed` and `shed`, which are private to `Component`, and so aren't the seam's. They feed `Component` events by hand, as its other tests there do, with the same asserts.
- The crate's dev-dependencies lose what only `Harness` read, `skein-world`'s `stream` among them if nothing else does.
- `heavy cargo nextest run -p skein-forge-connection --lib` and `-p skein-forge-machine-world` pass. The report gives the crate's test count and lines before and after, and how many tests left it.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/hand.md`
- `skein/docs/foundation/testing-strategy.md`, 2.1
- `skein/docs/design/forge-connection.md`, 10

## Owns and may change

- **Owns:** the files in the header.
- **A third test in `component/tests.rs` that can't feed events without a socket's bytes:** it asserts over what `Harness` played, so stop and ask, naming it.

## Steps

1. The three tests; then `Harness` and the helpers go.
2. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge-connection -p skein-forge-machine-world --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; the counts above. **The hand-fed tests must bite:** break the rule each pins by hand, name the failure.
