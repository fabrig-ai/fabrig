+++
id = "oauth-one-driver"
part = "oauth-owner"
kind = "feature"
needs = ["oauth-one-owner"]
files = ["skein/tests/oauth-accounts/src/streams.rs", "skein/tests/oauth-accounts/src/lib.rs", "skein/tests/oauth-accounts/src/world.rs", "skein/tests/oauth-accounts/src/sign_in/protocol.rs", "skein/tests/oauth-accounts/src/sign_in.rs"]
effort = "high"
minutes = 120
+++

# One protocol driver

## Goal

Step 2 of skein's D048. `world.rs`'s `protocol_connection` and `sign_in/protocol.rs`'s `protocol` each drive a client against an issuer over `stream::Wire`, and 116 of their long lines are the same, nearly all in the server half. One driver takes a rig saying what differs. This also removes the duplicated limits and plan builders, and with them about 10 story branches.

Acceptance:

- `src/streams.rs` holds `pub(crate) fn drive`, and `rg -n 'fn protocol_connection' skein/tests/oauth-accounts/src` finds nothing.
- `world::protocol`, `world::tls_close`, `sign_in::protocol` and their `protocol_client` callers keep their signatures.
- The baseline passes, unre-recorded but for what a trunk merge moved before your first edit.
- `heavy cargo nextest run -p skein-oauth-accounts-world` passes, with no assertion changed.
- The net is at least 150 lines fewer, outside the baseline.

**A smaller landing is pre-approved.** If the rig can't hold the differences below within this task's minutes without moving a digest, extract the server half alone, `world.rs`'s and `protocol.rs`'s request-serving lines, for about 95 lines fewer. Name that in your report's first line: `oauth-close` writes what's left into the item.

## Inputs

- `temper/docs/plan/debt-29/notes/oauth-world.md`, whole: the table of what must survive.
- `skein/docs/development/debt/D048-oauth-accounts-world-copies.md`.
- `skein/testing/skein-world/src/stream.rs`: `Wire`, and the protocol trace.
- `temper/docs/plan/debt-29/notes/common.md`.

## Owns and may change

Owns the files in the header. `oauth-one-restart` runs beside this, on `private_cuts.rs`, `sign_in/cuts.rs` and its own new file, so leave those alone. Both of you add one `pub mod` line to `src/lib.rs`.

- **The story branches that collapse** when two copies of limits or plans become one are fine, and the cut counts about 10: `world.rs`'s at about 1074, 1128, 1136, 1138, 1141, and `protocol.rs`'s at about 34, 38, 43, 70, 74. Touch no other. If two copies disagree on a value, keep both as rig fields; that's a difference, not a slip.

## Steps

```rust
// src/streams.rs
//! One protocol driver: a client and an issuer over scripted streams.

pub(crate) struct Rig {
    pub(crate) issuer: /* .. */,
    pub(crate) endpoint: /* .. */,
    /// Reseed and rebuild the server's wire on each connect, or keep one for life.
    pub(crate) rewires: bool,
    pub(crate) callbacks: Option<Callbacks>,
    pub(crate) room: /* .. */,
    pub(crate) expected_posts: /* .. */,
}

pub(crate) fn drive(seed: u64, client: world::Client, rig: Rig) -> (world::Client, Vec<String>, TlsClose);
```

The note's table lists what the rig must carry: wire seeds, trace order, the requests served, the per-loop work and the exit. The fields bind unless a site reads better otherwise; a hook where a field can't say it is your call, named in your report. `TlsClose` and the trace stay as their readers see them.

Run the baseline first; then after each move. The protocol rows hold each fragment's bytes in their digest, so they're the rows that tell you. Then the note's **Common to the tasks**. Commit, push.

## Report

As your text says, with the search's reading, the baseline's last result, whether a trunk merge moved it, which story branches collapsed, and the net lines. If you took the smaller landing, the first line says so.
