# D012: `session.rs` and `run.rs` are too large to read

- **Kind:** complexity
- **Where:** `smith-domain-session/src/session.rs`, `smith-domain-run/src/run.rs`
- **Seen:** 2026-10-31 at `aa95945cc`, in the smith code-quality review (`reviews/2026-10-31-quality.md`: theme "Layout")
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (each module readable in one sitting; moves lines, saves few)
- **Size:** M
- **Status:** open
- **Paid when:** no hand-written production file in smith's two largest domains is over 1,500 lines, and each module holds one topic.
- **Measure:** `wc -l smith/crates/smith-domain-session/src/session.rs smith/crates/smith-domain-run/src/run.rs` is 2,516 and 1,907

## What

The review counted nine production files over 1,500 lines. Five remain:
`session.rs` (2,516, from 3,788), `run.rs` (1,907), `smith-events`'
`read.rs` (1,625) and `write.rs` (1,612), and the local host's `host_io.rs`
(1,559). The last three belong to D001, D002 and D003. This item is the two
domain files. `session.rs` keeps 72 top-level functions beside its nine
submodules in `session/`; `run.rs` keeps 62 beside its five in `run/`.

## Why it matters

These are the domains' hearts, and reviewers sample them rather than read
them. A file this size hides near-copies and half-done transitions (the
review's first defect was in `run.rs`).

## What pays it back

Split by topic when the file is next opened for work, as the review asks:
one module per machine or concern, in reading order. No behaviour or
interface change.
