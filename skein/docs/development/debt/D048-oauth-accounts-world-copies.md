# D048: The oauth-accounts world doubles its owner harness and switches on stories

- **Kind:** tests
- **Where:** `skein/tests/oauth-accounts/src/world.rs`, `skein/tests/oauth-accounts/src/sign_in`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T5, copies between worlds) and `reviews/2026-10-31-skein-areas.md` (A10)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 4)
- **Checked:** 2026-11-06
- **Value:** low
- **Size:** L
- **Status:** open
- **Paid when:** the oauth-accounts world has one owner harness, and its scenarios are data the referee reads, not branches on a story name inside the harness (`skein/docs/foundation/testing-strategy.md`, 10).
- **Measure:** `rg -c 'story ==' skein/crates/skein-oauth-accounts skein/tests/oauth-accounts | awk -F: '{s+=$2} END{print s}'`: 80 at base

## What

The harness branches on `story ==` 80 times. The review found two owner
adapters, two `Process` enums, two restart wrappers and two protocol
drivers (about 211 shared lines).

Measured for the plan debt-28, which left it open as bigger than its old
Size M. The world is about 8,200 lines, with 6 `Story` enums and about 100
tests. The measure undercounts: `story !=` adds 16, and `matches!` on a story
41, across lines. Most sign-in tests are `drop(run(..))`, so the `Judge` is
their only check, and a trace is compared only between two runs of the same
code: a refactor needs a baseline of trace and facts per seed and story first.
Neither pair of owners is a superset of the other. The sign-in owner can become
a fourth sub-owner, as `accounts`, `shared` and `sign_out` are in `world.rs`.

## Why it matters

A story switch hides each scenario's behaviour inside the harness, so
adding a story means editing the harness in many places, and the two
owner paths can drift.

## What pays it back

One owner harness; scenarios as rows the harness does not branch on.

The cut's estimate, 8 to 11 hours, in this order, on a baseline: the sign-in
owner as a sub-owner with one `Process`; one protocol driver; one restart
wrapper (these three about 450 lines fewer); then rows, for fixtures, for
private faults and files, for owner scripts and gates, and for expected values
(these four about level). The first three can land alone. `PasteThenListener`'s
hold, in both tiers, is the hardest branch to make data.
