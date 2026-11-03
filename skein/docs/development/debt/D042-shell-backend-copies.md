# D042: skein-shell's Linux and Darwin backends copy their POSIX parts

- **Kind:** duplication
- **Where:** `skein-shell/src/linux/terminal.rs`, `skein-shell/src/darwin/terminal.rs`, `skein-shell/src/linux/group.rs`, `skein-shell/src/darwin/group.rs`
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (the spot checks; its runtime area report)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low
- **Size:** S
- **Status:** open
- **Paid when:** POSIX code that is the same on both systems lives once (a shared module both backends use); `linux/` and `darwin/` hold only what differs.

## What

`linux/terminal.rs` and `darwin/terminal.rs` (90 lines each) differ in
their first comment line only. `linux/group.rs` (135 lines) and
`darwin/group.rs` (123) differ in 46 lines.

## Why it matters

A fix to the terminal or process-group code has to be made twice, and
Darwin's copy is the one no runner checks
(`skein/docs/design/shell.status.md`).

## What pays it back

A `posix/` module for the shared parts, with `cfg` on what differs.
