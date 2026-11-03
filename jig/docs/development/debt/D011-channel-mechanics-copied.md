# D011: The two worker sides copy skein-channel's capacity rule

- **Kind:** duplication
- **Where:** `jig-protocol-worker-link/src/limits.rs`, `jig-protocol-engine-workers/src/limits.rs`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (A7 F10)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list ("Engine protocol crates": `channel_capacity` copied)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (the rule changes in skein alone)
- **Size:** S
- **Status:** open
- **Paid when:** neither crate defines `channel_capacity`; both ask skein-channel whether a `Limits` is valid and what it holds.
- **Measure:** `rg -n 'fn channel_capacity' jig/crates | wc -l` (2 at base)

## What

`fn channel_capacity(limits: &skein_channel::Limits) -> Option<u32>` is written in both
`jig-protocol-worker-link/src/limits.rs` (line 83) and
`jig-protocol-engine-workers/src/limits.rs` (line 66), re-deriving skein-channel's
internals.

## Why it matters

When skein-channel's framing changes, both copies go stale silently.

## What pays it back

A validity and capacity function in skein-channel (skein first), called from both.
