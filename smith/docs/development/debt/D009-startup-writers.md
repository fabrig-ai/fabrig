# D009: Startup files are written by two hand-rolled hosts

- **Kind:** duplication
- **Where:** `smith-local-shell/src/credentials/private_file.rs`, `smith-local-shell/src/compose/files.rs`
- **Seen:** 2026-10-31 at `aa95945cc`, in the smith code-quality review (`reviews/2026-10-31-quality-areas.md`: report 11, finding 5)
- **Seen:** 2026-10-31 at `679eacb3c`, in the quality run's closing report (outside the repository): left, as it needs a skein-shell writer
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (one startup write path; `ResultFile` keeps one job)
- **Size:** S
- **Status:** open
- **Paid when:** smith writes its private startup files through one skein-shell call, as it reads them through `read_private_file`, and `ResultFile` writes results only.

## What

Two hosts drive open, store and close on a private kernel by hand at
startup: `credentials/private_file.rs` (156 lines, a `Writing` host) and
`ResultFile::write_startup` (`compose/files.rs:192`). skein-shell offers
`read_private_file` (`skein/crates/skein-shell/src/startup.rs`) and no
write. `ResultFile` also writes the generated agent configuration and
removes the ephemeral directory.

## Why it matters

Durable private writes are easy to get subtly wrong (sync, rename,
directory sync). Two copies in the shell can diverge, and skein's other
consumers have none to share.

## What pays it back

A startup store in skein-shell beside `read_private_file` (skein first, as
`smith/AGENTS.md` requires), both smith sites on it, and the ephemeral
cleanup in its own small host.
