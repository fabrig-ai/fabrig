# D010: Bounded queues that drop are still written by hand

- **Kind:** primitives
- **Where:** `jig-domain-host/src/refusals.rs`, `jig-domain-tasks/src/failures.rs`, `jig-domain-tasks/src/domain.rs`, `jig-domain-people/src/domain.rs`, `jig-domain-accounts/src/domain.rs`, `jig-domain-fleet`, `jig-protocol-worker-link`, `jig-protocol-engine-workers`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T5, the skein-lib table: "Lossy bounded queues, 33 drop counters")
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (item 12)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (one drop rule, one loss count, tested once in skein)
- **Size:** M
- **Status:** open
- **Paid when:** every jig queue that drops its oldest entry and counts the loss is a `skein_lib::Lossy`, or says in a comment why it cannot be.

## What

`skein_lib::Lossy<T>` exists (`skein/crates/skein-lib/src/lossy.rs`) and covers views,
fleet facts and host activity. The rest keep their own `lost` counters beside their own
queues: host's `Refusals` (`refusals.rs`, `lost: u64`), tasks (`failures.rs`, `domain.rs`),
people (`domain.rs`), accounts (`domain.rs`), fleet's hello history, and the worker-link
and engine-workers counters. The quality run left them because `Lossy` lacks counted
eviction, borrowed iteration, a capacity accessor and a resettable loss counter.

## Why it matters

Each copy decides again what to drop and how to count it; a fault in one shows only in
its own world.

## What pays it back

skein first: the four missing operations on `Lossy`. Then each queue converts.
