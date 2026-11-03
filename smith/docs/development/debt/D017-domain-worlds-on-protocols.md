# D017: Two domain worlds call protocol crates

- **Kind:** coupling
- **Where:** `tests/session/src/translate.rs`, `tests/session/Cargo.toml`, `tests/tools/src/translate.rs`, `tests/tools/Cargo.toml`
- **Seen:** 2026-10-31 at `aa95945cc`, in the smith code-quality review (`reviews/2026-10-31-quality-areas.md`: report 13, Q6, domain worlds on protocol crates)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (domain worlds test the domain alone; protocol changes stop breaking them)
- **Size:** S
- **Status:** open
- **Paid when:** `tests/session` and `tests/tools` depend on no `smith-protocol-*` crate, or the departure is recorded in `smith/docs/design/testing.status.md` with its reason.

## What

"Domain worlds use domains and fakes only, never protocol crates"
(`smith/docs/design/testing.md`, 2). Yet the session world prices and
decodes through the LLM component: `tests/session/src/translate.rs:223` to
`:261` calls `smith_protocol_llm::fixture::small`, `Stated::from_tools`,
`decode` and `render_outcome`, and its manifest also names
`smith-protocol-channel` and `smith-protocol-translate`. The tools world
builds listings with `smith_protocol_machine::Listing`
(`tests/tools/src/translate.rs:133`).

`smith/docs/design/testing.status.md` does not record this departure.

## Why it matters

A change in the LLM component's decoding or rendering breaks a domain
world, and the domain world proves the protocol's behaviour instead of the
domain's.

## What pays it back

Give the session world a fake of the decode it needs (the typed reply the
domain receives), and the tools world a domain-side listing fixture, then
drop the protocol dependencies. If a protocol call is the honest fixture,
record the departure in the status file instead and mark this item kept.
