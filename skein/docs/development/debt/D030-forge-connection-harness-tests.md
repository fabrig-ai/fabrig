# D030: forge-connection's step tests are mostly a machine world of their own

- **Kind:** tests
- **Where:** `skein-forge-connection/src/tests`, `skein-forge-connection/src/tests/fixtures.rs`, `skein/tests/forge/machine`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T5, step tests that are worlds in disguise) and `reviews/2026-10-31-skein-areas.md` (A3, F7)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 4)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** M
- **Status:** open
- **Paid when:** forge-connection's crate tests feed its step functions events (`skein/docs/foundation/testing-strategy.md`, 2.1), and what drives the whole component over scripted HTTP lives in the machine world.
- **Measure:** `rg -l 'Harness' skein/crates/skein-forge-connection/src/tests | wc -l`: 19 at base

## What

- 19 test files use `tests/fixtures.rs`'s `Harness`, which plays io over
  a scripted wire and answers raw HTTP: 449 of the crate's 568 tests.
- `skein/tests/forge/machine` is the machine world for the same
  component, with its own fixtures; the first review found the two
  fixture files near-copies that had drifted.
- The review counted seven io-playing harnesses across the forge
  machine and protocol worlds.

## Why it matters

Two harnesses at one tier test the same behaviours twice and drift
apart; the crate's tests are three times its code and slow its
focused suite.

## What pays it back

Move the `Harness` tests into the machine world as scenarios, keeping
true step tests in the crate.
The moved tests use the fake's own server.
