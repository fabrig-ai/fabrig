# jig's debt

What jig's code and tests owe, one file per item. A branch that pays an item deletes its
file and its row below (git keeps the history). An item we choose to keep stays, with
status `kept` and its reason, so a later review does not find it again. Reviews add items;
their reports are kept raw in `reviews/`, each file named by its date and scope.

## 1. What counts

Debt is what the code or tests owe: dead or duplicate code, coupling, missing or hidden
primitives, needless complexity, naming, hand-copied lints, test infrastructure copied
around, workarounds left in place, files too large to read. Not here:

- **design gaps**, where the code lags or departs from a design document: they go in that
  document's `.status.md`. An item that is also a gap covers the code's side and cites
  the status file;
- **bugs** (wrong behaviour), **features not built yet** and QA's off-path findings: they
  go to their owner or to the backlog.

## 2. An item

One file per item, named by its ID and a slug (`D007-contract-written-by-hand.md`).
IDs run from `D001` and are never reused:

```md
# D007: <title, a short claim>

- **Kind:** dead code | duplication | coupling | primitives | complexity | naming | tests | lints | other
- **Where:** <paths>
- **Seen:** <date> at `<commit>`, in <source, with its own label> (one line per source)
- **Checked:** <date> at `<commit>`
- **Value:** high | medium | low (what paying it buys)
- **Size:** S (one task) | M (one plan of a few tasks) | L (several plans)
- **Status:** open | planned (<plan>) | kept (<reason>)
- **Paid when:** <a concrete, checkable condition>
- **Measure:** <optional: a command run at the repository's root, and its value when checked>

## What

## Why it matters

## What pays it back
```

- **Paths** in **Where** are under `jig/`; one that starts with a crate's name is under
  `jig/crates/`. A **Measure** runs at the repository's root.
- **Sources** named in **Seen**: jig's tech-debt list (first look 2026-10-10, refreshed
  2026-10-18 at `86971be`; `reviews/2026-10-18-jig-tech-debt.md`); the jig code-quality
  review (2026-10-31 at `c30cf6a03`; `reviews/2026-10-31-jig.md`), cited by its own labels
  (themes T1 to T10, area findings such as A1 F3, its order-of-work items); and the
  quality run's debt list (2026-10-31 at `1b5c98834`; `reviews/2026-10-31-jig-quality-run.md`),
  which carried out the review's first phases.

## 3. Index

| ID | Title | Kind | Value | Size | Status |
|---|---|---|---|---|---|
| [D002](D002-core-untyped-continuations.md) | The core's continuations are bare numbers | complexity | high | L | open |
| [D003](D003-core-one-object.md) | The core is one object of 83 fields spread over 50 files | complexity | medium | L | open |
| [D004](D004-ops-root-jig-wiring.md) | The reference root is half jig wiring, and the test root copies it | coupling | high | L | open |
| [D006](D006-link-values-two-paths.md) | The link's values have two import paths, and a grammar is copied | duplication | low | S | open |
| [D007](D007-contract-written-by-hand.md) | The contract is written out by hand in four or five places | duplication | high | L | open |
| [D008](D008-link-translate-split-by-number.md) | The link's translation is split by number and repeats one loop | duplication | medium | M | open |
| [D009](D009-limits-mapped-by-hand.md) | Limits are copied between layers and mapped by hand | duplication | medium | M | open |
| [D010](D010-lossy-queues-by-hand.md) | Bounded queues that drop are still written by hand | primitives | low | M | open |
| [D011](D011-channel-mechanics-copied.md) | The two worker sides copy skein-channel's capacity rule | duplication | low | S | open |
| [D012](D012-tasks-operation-vocabulary.md) | tasks spells one set of operations several ways | duplication | medium | M | open |
| [D013](D013-subset-breaches.md) | Step code still uses `Deref`, a trait and `macro_rules!` | other | medium | M | open |
| [D014](D014-task-record-union.md) | `TaskRecord` unions every executor's state in 78 fields | complexity | medium | L | open |
| [D015](D015-exhaustive-lists.md) | Long or-pattern lists stand where a classifier on the type would do | complexity | medium | M | open |
| [D016](D016-positional-payloads.md) | Event payloads travel as positional arguments | complexity | medium | M | open |
| [D017](D017-unreachable-re-narrowing.md) | `unreachable!` re-narrows values the types could carry | complexity | medium | M | open |
| [D020](D020-inner-allow-guard.md) | Nothing stops an inner `#![allow]` from coming back | lints | low | S | open |
| [D021](D021-phrases-listed-five-times.md) | Each of the view's phrases is listed four or five times | duplication | medium | M | open |
| [D024](D024-view-section-context.md) | The view passes a section's context through a field set around calls | coupling | low | S | open |
| [D025](D025-store-codec-domain-rules.md) | The store codec applies domain rules | coupling | medium | M | open |
| [D026](D026-engine-clients-project-mismatch.md) | The clients' protocol layer answers a people rule itself | coupling | low | S | open |
| [D027](D027-engine-clients-cursor-format.md) | The owner cursors are a hand-rolled byte format with no schema | primitives | medium | S | open |
| [D029](D029-protocol-front-one-identity.md) | The protocol front's fixture authenticates one party, so six stories are ignored | tests | medium | M | open |
| [D030](D030-conformance-not-drop-in.md) | The conformance kit's `Application` trait is wide | tests | medium | M | open |
| [D031](D031-worlds-as-kits.md) | Worlds are used as kits by other packages | tests | medium | M | open |
| [D032](D032-coverage-by-trace-text.md) | Some tests find what happened by searching trace text | tests | low | S | open |
| [D033](D033-trace-gating-copied.md) | Each world gates and formats its own trace | tests | low | S | open |
| [D034](D034-world-replay-cost.md) | The conformance and core worlds redo whole runs to check a cut | tests | medium | M | open |
| [D035](D035-bare-numbers-in-worlds.md) | World tests speak in bare numbers, and each file names its own fixtures | tests | medium | M | open |
| [D036](D036-page-dispatch-by-string.md) | A client-world helper picks a page by its name as a string | tests | low | S | open |
| [D037](D037-copied-goldens.md) | jig keeps copies of smith's goldens that nothing compares | tests | low | S | open |
| [D038](D038-one-process-cut-by-count.md) | The one-process world places its power-loss cut by a step count | tests | low | S | open |
| [D039](D039-large-files.md) | Eight production files pass 1,500 lines | complexity | medium | L | open |
| [D040](D040-duplicate-type-names.md) | One name means several types | naming | low | M | open |
| [D041](D041-unused-parameters.md) | Production functions keep parameters they never read | dead code | low | S | open |
| [D042](D042-bundled-scenarios.md) | Shared scenarios run bundled, several to one test | tests | low | S | kept |
| [D043](D043-length-widening.md) | Native lengths are widened with `try_from` beside skein-lib's `len_u64` | primitives | low | S | open |
