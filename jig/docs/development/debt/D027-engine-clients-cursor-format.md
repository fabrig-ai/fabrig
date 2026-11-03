# D027: The owner cursors are a hand-rolled byte format with no schema

- **Kind:** primitives
- **Where:** `jig-protocol-engine-clients/src/translate/cursor.rs`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (A7 F8)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list ("Engine protocol crates": `jig-protocol-engine-clients/src/translate/cursor.rs`, no schema, golden or drift test)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium (a cursor clients hold across releases gets a checked format)
- **Size:** S
- **Status:** open
- **Paid when:** the owner cursor is a record of a skein-codec schema with goldens under the codec families' drift check (`jig/tests/codec-families.rs`).
- **Measure:** `wc -l < jig/crates/jig-protocol-engine-clients/src/translate/cursor.rs` (652 at base)

## What

`cursor.rs` (652 lines) writes and reads cursors with its own `Writer`, a leading version
byte `1` and hand-placed fields (`encode`, `put_cursor`). Clients hold these bytes and send
them back. No schema, golden or drift test covers the format; the engine-clients crate has
unit tests only.

## Why it matters

Every other byte format jig sends goes through skein-codec, its goldens and the drift
check; this one can change without anything failing.

## What pays it back

A small schema for the cursor (keys stay the store codec's), generated, with goldens.
