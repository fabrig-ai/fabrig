# D050: Two harnesses over forge-connection sit in one package, with near-twin tests

- **Kind:** tests
- **Where:** `skein/tests/forge/machine/src/hand.rs`, `skein/tests/forge/machine/tests/suite`
- **Seen:** 2026-11-06 at `683f4e8b92`, in the Spot review of debt-24 (finding 1)
- **Checked:** 2026-11-06
- **Value:** low (remove duplicate fixtures and repeated rules)
- **Size:** M
- **Status:** open
- **Paid when:** no fixture is written twice across the crate's test fixtures and `hand`, and no `hand` test repeats a seeded test's rule, comparing without harness bookkeeping. Two harnesses stay.

## What

The crate's step fixtures and the machine world's hand repeat fixture items.
The world's hand and seeded tests also repeat rules over the same component.

## Why it matters

A fixture or rule has two places to change and to review.

## What pays it back

Share fixtures through the crate's module, built for tests or the seam.
Cut twins after comparing their assertions without harness bookkeeping,
keeping seeded coverage unless a hand controls a necessary interleaving.
