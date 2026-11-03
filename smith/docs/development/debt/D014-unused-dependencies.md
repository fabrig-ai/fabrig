# D014: Six manifests name a dependency no source uses

- **Kind:** dead code
- **Where:** `smith-agent-shell/Cargo.toml`, `smith-local-protocol/Cargo.toml`, `smith-local-shell/Cargo.toml`, `tests/hosts/Cargo.toml`, `tests/protocol-llm/Cargo.toml`, `tests/session/Cargo.toml`
- **Seen:** 2026-10-31 at `aa95945cc`, in the smith code-quality review (`reviews/2026-10-31-quality-areas.md`: report 13, Q6, unused dependencies)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (smaller build graphs; manifests that say what a crate uses)
- **Size:** S
- **Status:** open
- **Paid when:** no smith manifest names a dependency that none of its package's sources uses, unless a comment beside it says why (a feature it turns on).

## What

The review counted 56 dependencies named in no source; the quality run's
sweep removed most. At base, six manifests still name
`smith-protocol-channel` and no source in their package says
`smith_protocol_channel`. Each is a leftover of moving the shared
translation into `smith-protocol-translate`. The root manifest and the
other crates are clean by the same search.

## Why it matters

Each unused edge is built and linked for nothing, and it hides the
dependency graph the design describes.

## What pays it back

Remove the six lines and update `Cargo.lock`, checking each package still
builds (a dependency kept only for a feature would show there). A check
that compares manifests with sources would keep it so.
