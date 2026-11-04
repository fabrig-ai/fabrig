# D030: The conformance kit's `Application` trait is wide

- **Kind:** tests
- **Where:** `testing/jig-conformance/src/harness.rs`, `tests/ops/src`, `tests/core/src/conformance.rs`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T6 "the conformance world is not drop-in", A11 F1, F2; order of work, item 18)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium (an application adopts the kit without adapters of its own)
- **Size:** M
- **Status:** open
- **Paid when:** an application adopts `jig-conformance` by naming its root, its store and its fakes; the kit answers the core's own peers itself.
- **Measure:** `awk '/pub trait Application/,/^}/' jig/testing/jig-conformance/src/harness.rs | grep -cE '^\s+fn '` (22 at base)

## What

`ops` now runs the shared scenarios (`tests/ops/tests/suite/scenarios.rs`), which paid
part of this. `Application` (`harness.rs`, line 88) still has 22 methods and 10
associated types, and each adopter rebuilds the core's peers: the core world's
`store_view` (`tests/core/src/conformance.rs`, line 317) and `ops`'s joint and
conformance adapters (`tests/ops/src/conformance/mod.rs`).

## Why it matters

`jig/docs/design/README.md`, 6.6 ships the conformance world for applications. A wide
trait means each application writes the same adapter, and the adapters drift.

## What pays it back

A `jig_conformance::core` module that answers the core's peers and builds the snapshot
from core rows; the referee keyed by its own effect vocabulary; the trait narrowed to
what an application owns.
