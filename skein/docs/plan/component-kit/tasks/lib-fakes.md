+++
id = "lib-fakes"
part = "lib-sites"
kind = "migration"
repo = "fabrig"
needs = ["ratchets", "kit-lib"]
files = ["skein/crates/skein-fake-forge-domain/src/**", "skein/crates/skein-fake-forge-github/src/**", "skein/crates/skein-fake-forge-protocol/src/**", "skein/crates/skein-fake-llm-protocol/src/**", "skein/crates/skein-fake-oauth/src/**", "skein/scripts/old-forms.txt"]
effort = "medium"
minutes = 40
+++
# lib's primitives in the fakes

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/lib-common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, 3.4, for the fake peers' crates.

## Sites

- skein-fake-forge-domain: `src/actions.rs:278` `decimal(out, n)` →
  `Decimal::of`; `src/promises.rs:920` `hex` → `hex::digit`.
- skein-fake-forge-github: `src/rates.rs:206` `decimal` → `Decimal::of`;
  `src/pages.rs:542` `hex(rng, length)` generates random digits: it
  stays, built on `hex::digit`; the discarding pop loops in
  `src/server.rs` (two) → `clear()`.
- skein-fake-forge-protocol: the six discarding pop loops in
  `src/server.rs` (~385-391 and the rest) → `clear()`.
- skein-fake-llm-protocol: `src/provider/inflate.rs:359`, a test's
  `hex(text) -> Vec<u8>`: it stays, built on `hex::value`.
- skein-fake-oauth: `src/lib.rs:704` `hex` → `hex::value`.

## Acceptance

- `hex_decimal` falls by four, `pop_loop` by eight.
- `heavy cargo nextest run` of the five packages and tests/fake-forge,
  with counts.
