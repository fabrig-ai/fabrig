+++
id = "oauth-one-restart"
part = "oauth-owner"
kind = "feature"
needs = ["oauth-one-owner"]
files = ["skein/tests/oauth-accounts/src/restart.rs", "skein/tests/oauth-accounts/src/lib.rs", "skein/tests/oauth-accounts/src/private_cuts.rs", "skein/tests/oauth-accounts/src/sign_in/cuts.rs"]
effort = "high"
minutes = 40
+++

# One restart wrapper

## Goal

Step 3 of skein's D048. `private_cuts.rs`'s `Process` and `sign_in/cuts.rs`'s `Process` and `Inner` each wrap a host, to cut it at a signal and start it again. The signal token, the completion routing, the forwarding, `is_empty` and `operations + 1` are the same. One generic wrapper holds them.

Acceptance:

- `src/restart.rs` holds `pub struct Restartable<P: Host>`. It's a public module, as `private_cuts` is, so not under `#[cfg(test)]`.
- `sign_in/cuts.rs`'s `Inner` is gone, and `private_cuts.rs`'s and `sign_in/cuts.rs`'s `Process` are this wrapper, by alias or a thin enum where a test needs one.
- The baseline passes, unre-recorded but for what a trunk merge moved before your first edit, with `private_cuts::world`'s three rows and `sign_in/cuts.rs`'s two among them.
- `heavy cargo nextest run -p skein-oauth-accounts-world` passes, with no assertion changed.
- The net is at least 50 lines fewer, outside the baseline.

## Inputs

- `temper/docs/plan/debt-29/notes/oauth-world.md`, whole: the table of what must survive.
- `skein/docs/development/debt/D048-oauth-accounts-world-copies.md`.
- `skein/testing/skein-world`'s `Host`.
- `temper/docs/plan/debt-29/notes/common.md`.

## Owns and may change

Owns the files in the header. `oauth-one-driver` runs beside this, on `streams.rs`, `world.rs`, `sign_in.rs` and `sign_in/protocol.rs`, so leave those alone. Both of you add one `pub mod` line to `src/lib.rs`.

- **Closing the signal differs:** `sign_in/cuts.rs` also requires `!work_pending`, and `private_cuts.rs` doesn't. `Client::is_empty` may not imply it, so it's a field, `wait_idle`. Merging them can move the close operation in the trace.
- **`start_sign_in`, `issued`, `opened_at`, `stored_at` and `ending`** are `sign_in/cuts.rs`'s own: a mark hook takes `opened_at` and `stored_at`, and the rest stays in its test's own code. The wrapper holds nothing about sign-in.
- **Don't move it to `skein/testing`.** `tests/io` has similar forwarders, but they're another item.

## Steps

```rust
// src/restart.rs
//! A host cut at a signal and started again, for the restart cuts of the
//! private-file and sign-in stories.

pub struct Restartable<P: Host> {
    inner: P,
    signal: /* .. */,
    state: /* .. */,
    /// Keep the signal open until the host has no work pending, as well as being empty.
    wait_idle: bool,
    issued: /* .. */,
    completions: /* .. */,
    submissions: /* .. */,
}
```

The fields bind unless a site reads better otherwise.

Run the baseline first; then after each move. Then the note's **Common to the tasks**. Commit, push.

## Report

As your text says, with the baseline's last result, whether a trunk merge moved it, and the net lines.
