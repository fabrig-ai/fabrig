# D002: The core's continuations are bare numbers

- **Kind:** complexity
- **Where:** `jig-domain-core/src/routing/flight.rs`, `jig-domain-core/src/routing/run/preparation.rs`, `jig-domain-core/src/vocabulary.rs`, `jig-domain-core/src/lib.rs`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T7, A1 F1, F5; order of work, item 9)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (item 9)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** high (fewer maps and `unreachable!` arms in the core; one lookup per answer)
- **Size:** L
- **Status:** open
- **Paid when:** a child's answer finds its continuation by one typed lookup; no `Map<u64, u64>` keyed by task stands for a pending claim, and no loop over `flights` searches for one.

## What

The quality run added `flights: Slab<flight::Flight>` (`lib.rs`), so part of this is paid.
What remains:

- `claiming: Map<u64, u64>` keyed by task number, with its fallbacks
  (`jig-domain-core/src/routing/run/preparation.rs`, `jobs.rs`: insert, `get`, `remove`);
- flights still found by scanning the slab (`jig-domain-core/src/routing/flight.rs`, three `for (_, flight) in &self.flights` loops);
- `Work::Event(Event)` wraps public entrances that the core uses as its own continuations (`vocabulary.rs`, `pub(crate) enum Work`);
- 93 uses of `u64::MAX` in the core's code, some as sentinels.

## Why it matters

"Child X answered request R, so continue R" is the core's central step. Without a type
for it, every map that holds a continuation needs its own fallback and its own
`unreachable!` arms (D017), and a token can be tried against the wrong map.

## What pays it back

The flight slab's id as the only reply token, holding the continuation's data per kind;
the claim and procedure holdings moved into it; public `Event` split from the core's
internal `Work`. Goes with D001 and helps D003 and D017.
