# D048: The oauth-accounts world doubles its owner harness and switches on stories

- **Kind:** tests
- **Where:** `skein/tests/oauth-accounts/src/world.rs`, `skein/tests/oauth-accounts/src/sign_in`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T5, copies between worlds) and `reviews/2026-10-31-skein-areas.md` (A10)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 4)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low
- **Size:** M
- **Status:** open
- **Paid when:** the oauth-accounts world has one owner harness, and its scenarios are data the referee reads, not branches on a story name inside the harness (`skein/docs/foundation/testing-strategy.md`, 10).
- **Measure:** `rg -c 'story ==' skein/crates/skein-oauth-accounts skein/tests/oauth-accounts | awk -F: '{s+=$2} END{print s}'`: 80 at base

## What

The harness branches on `story ==` 80 times. The review found two owner
adapters, two `Process` enums, two restart wrappers and two protocol
drivers (about 211 shared lines).

## Why it matters

A story switch hides each scenario's behaviour inside the harness, so
adding a story means editing the harness in many places, and the two
owner paths can drift.

## What pays it back

One owner harness; scenarios as rows the harness does not branch on.
