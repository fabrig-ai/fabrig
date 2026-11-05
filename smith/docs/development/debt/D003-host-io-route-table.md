# D003: The local host routes io completions through its own table

- **Kind:** primitives
- **Where:** `smith-local-service/src/host_io.rs`, `smith-local-service/src/credentials/transport.rs`, `smith-agent-service/src/service/up.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in skein's review (`skein/docs/development/debt/reviews/2026-10-31-skein-areas.md`: the io area, findings F1 and F6)
- **Seen:** 2026-10-31 at `aa95945cc`, in the smith code-quality review (`reviews/2026-10-31-quality-areas.md`: report 13, Q5, `stage`)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (one routing rule instead of a table per service; services drive io through `stage` like the rest)
- **Size:** M (most of it in skein)
- **Status:** open
- **Paid when:** `HostIo` has no `Map<Token, Operation>` route table and no `native_io_valid` of its own, and the multiplexing services call `stage` instead of `io::resume`.
- **Measure:** `grep '^io_resume ' smith/scripts/old-forms.txt` is 15

## What

skein's `Io` and `FileIo` mint operation tokens that can collide on one
kernel, and nothing says which machine a completion belongs to. Each
consumer works around it. smith's is `HostIo.operations: Map<Token,
Operation>` (`host_io.rs:68`), which records every submission as `Io`,
`File` or `ProgramRoot` and routes each completion back by lookup.

Beside it, `native_io_valid` (`host_io.rs:100`) rebuilds a validity check by
hand: `is_usable`, the operation count doubled and the socket count
doubled. skein's `skein_io::valid` (`skein/crates/skein-io/src/limits.rs`) checks
operations, flights and timers instead. smith calls its own at startup
(`host_io.rs:85`, `credentials.rs:90`, the headless front).

Because these services multiplex several machines on one kernel, they cannot
use `skein_io::stage` and drive io by hand: 15 `io::resume` calls remain, in
`host_io.rs`, `credentials/transport.rs`, the agent service's `up.rs` and
their tests.

## Why it matters

The table is a second source of truth for routing, sized and checked by
hand, and the hand-written validity check can drift from skein's.

## What pays it back

skein gives the lane: a token's top bits name its namespace (`Token::tagged`,
`skein/docs/design/lib.md`, 4; skein's review's io F1), as skein's terminal
does. It still lacks one public validity check (its F6). Then
`HostIo` routes by lane, drops the table, calls `skein_io::valid` plus its
own doubling rule, and the services move onto `stage`. skein's
`skein/docs/design/io.status.md` does not record the collision yet.
