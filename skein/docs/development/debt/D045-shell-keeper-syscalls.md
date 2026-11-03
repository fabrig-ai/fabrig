# D045: skein-shell's public API carries the harness's syscalls

- **Kind:** coupling
- **Where:** `skein-shell/src/process.rs`, `skein-shell/src/linux.rs`, `skein-shell/src/darwin.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T9 item 4; A13) and `reviews/2026-10-31-skein-areas.md` (A13)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low
- **Size:** S
- **Status:** open
- **Paid when:** the calls only the test harness's keeper needs sit in a fenced module (`skein_shell::keeper` or similar) that services do not reach by default, and shell.md designs it.

## What

`poll_child`, `close_keeper_fd` and `abandon_binary`
(`process.rs`) and `wait_readable` (`linux.rs`, `darwin.rs`) are public
functions of the crate every service links, though only the harness's
keeper calls them. `skein/docs/design/shell.md`, 7.5, lists each
system's parts and names no such module.

## Why it matters

A service author sees raw process and descriptor calls beside the
`Host` loop, with nothing saying they are not for services.

## What pays it back

Move them into one `keeper` module, and record it in `skein/docs/design/shell.md`, 7.5.
