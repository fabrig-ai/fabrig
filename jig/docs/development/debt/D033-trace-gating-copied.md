# D033: Each world gates and formats its own trace

- **Kind:** tests
- **Where:** `tests/core/src/world/constructors.rs`, `tests/host/src/trace.rs`, `tests/tasks/src/trace.rs`, `testing/jig-conformance/src/trace.rs`, `tests/tasks/tests/suite/fuzzy/amendments.rs`, `tests/fleet/src/world.rs`, `tests/fleet/src/turns.rs`
- **Seen:** 2026-10-18 at `86971be`, in jig's tech-debt list ("Test cost": the fleet world formats every event)
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (A5 F16: the `JIG_WORLD_TRACE` gate copied in 6 files)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (cheaper runs; one trace switch)
- **Size:** S
- **Status:** open
- **Paid when:** jig's worlds use one trace from skein-world (typed, formatted only on failure or when asked), and no world reads `JIG_WORLD_TRACE` itself.
- **Measure:** `rg -l JIG_WORLD_TRACE jig -g '*.rs' | wc -l` (5 at base)

## What

Five files read `JIG_WORLD_TRACE` to decide whether to trace. The fleet world does not
gate at all: it formats every event, request and stimulus as it happens (`format!` six
times in `tests/fleet/src/world.rs`, twice in `turns.rs`), though the trace is read only
on failure.

## Why it matters

Each copy decides again what is traced and when; formatting on every step costs CPU on
every run (the worlds' CPU budgets, `skein/docs/foundation/testing-strategy.md`).

## What pays it back

A skein-world `Trace` (skein first), adopted by each world; the fleet world switched to it.
