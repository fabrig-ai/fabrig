# D029: Four HTTP faces for the fake GitHub service sit beside its own server

- **Kind:** tests
- **Where:** `skein-forge-connection/src/tests/github_files/fake.rs`, `skein/tests/forge/machine/src/github.rs`, `skein/tests/forge/machine/src/joint/github.rs`, `skein/tests/forge/protocol/src/github.rs`, `skein-fake-forge-github/src/server.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T5, duplicate harnesses) and `reviews/2026-10-31-skein-areas.md` (A3, F1)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** high
- **Size:** M
- **Status:** open
- **Paid when:** every test that serves the fake GitHub over HTTP drives `skein_fake_forge_github::server`; the fixture App, accounts and repositories are built once, in the fake.
- **Measure:** `rg -l 'Outcome::Admit' skein/crates/skein-forge-connection/src skein/tests/forge | wc -l`: 4 at base

## What

The fake service's `Outcome::{Reply, Admit, Call}` loop is written four
times outside the fake, each with its own hand-written request parsing
and response heads: 396, 626, 375 and 561 lines. The fake's own
`server.rs` (792 lines) is that face as a step machine. Fixture
accounts and repositories are built in three places;
`skein/tests/forge/protocol/src/github_fixture.rs` says the machine
world's service and domain are private.

## Why it matters

`skein/docs/foundation/testing-strategy.md`, 4: one fake at every tier,
only its face changes. Four faces can each answer differently from the
fake's own, so a test can pass against a face the product never meets.
The review put the saving at 1.2 to 1.5k lines.

## What pays it back

Drive `server::Server` in every world and the step-test harness, and
move the fixtures into the fake. It pairs with D030.
