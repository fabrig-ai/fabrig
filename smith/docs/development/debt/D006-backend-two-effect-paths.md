# D006: The program backend's effects leave by two paths

- **Kind:** primitives
- **Where:** `smith-protocol-program/src/backend.rs`, `smith-protocol-program/src/backend/start.rs`, `smith-protocol-program/src/mcp.rs`
- **Seen:** 2026-10-31 at `aa95945cc`, in the smith code-quality review (`reviews/2026-10-31-quality-areas.md`: report 06, finding 12)
- **Seen:** 2026-10-31 at `679eacb3c`, in the quality run's task reports (outside the repository): left; it did not fall out of the lifecycle conversion
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (one way out for effects; removes a capacity coupling)
- **Size:** S
- **Status:** open
- **Paid when:** `Backend` holds no queue of its own requests, and `mcp::Server` takes its output queue as an argument instead of being polled.

## What

`Backend` receives `below: &mut Queue<Below>`, yet keeps its own io queue
in `self.io` and copies it out in `flush` (`backend.rs:635`). The MCP server
keeps an `above: Queue<Up>` of capacity 2 (`mcp.rs:169`, `:240`), which the
backend drains with `for _ in 0_u32..2` (`backend/start.rs:182`).

## Why it matters

The step shape passes the output queues in. A queue held inside the owner
needs its capacity kept in step with the caller's by hand, and the poll's
bound of 2 is that coupling written as a literal.

## What pays it back

Pass the queues through the conversation, file and native helpers, and
change the MCP server's output face to push into the caller's queue. No
behaviour changes.
