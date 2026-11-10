# host: status

2026-11-10. Where smith's code lags or departs from [host.md](host.md).

Section 4 has no known gap in retaining a panic's closing line and
message when they fit the detail bound. The host-protocol detail table
has a report with a backtrace longer than 4 KiB, retained at 1,024
bytes, with delivery chunks of 1, 20 and 4,096 bytes. It checks the
closing location and message within the existing bound; the keeper
remains a tail.

- **Not yet:**
  - 2, 9.1: live content in one process. The inline agent's LLM deltas
    never reach its owner: `smith-protocol-llm` hands deltas over only
    for a channel's ask, and `smith-local-service`'s `pop_live()` reads
    a spawned agent's alone.
  - 4: an agent spawned in a contained tree, in its view, and gone once
    the tree is proved empty: `smith-host-protocol/src/process.rs`
    spawns a plain child in a process group.
