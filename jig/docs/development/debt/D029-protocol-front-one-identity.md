# D029: The protocol front's fixture authenticates one party, so six stories are ignored

- **Kind:** tests
- **Where:** `tests/client-protocol/tests/suite/shared/a.rs`, `tests/client-protocol/src`, `tests/client/src/harness/seed.rs`
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (items 15 and 17: six role stories ignored on the Protocol front; `Seed` supports one project)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium (the shared role stories run over real HTTP)
- **Size:** M
- **Status:** open
- **Paid when:** the six stories in `tests/client-protocol/tests/suite/shared/a.rs` run unignored on the protocol front, and `Seed` can place a story on more than one project.
- **Measure:** `rg -o '#\[ignore' jig/tests/client-protocol | wc -l` (6 at base)

## What

Six shared role stories are `#[ignore = "printed_link_has_one_identity: this protocol
fixture authenticates only its first owner"]` (`tests/client-protocol/tests/suite/shared/a.rs`, lines 80 to 115). `Seed`
(`tests/client/src/harness/seed.rs`, line 7) offers `ChatProject`, `Profile` and a few
more for one project, so stories that need several parties or projects cannot be shared.

## Why it matters

The protocol front is the one closest to shipping code; the role stories, which check who
may see and do what, skip it.

## What pays it back

Multi-identity authentication in the protocol fixture (several printed links), and
`Seed` variants for several projects and parties.
