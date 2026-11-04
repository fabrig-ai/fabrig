# debt-8: finish jig D001 as the user decided

The user answered `d001-scope` with (c): `Now::Activate` stays in its decision, so a release stays one commit, and
only `Now::PrepareAgent` leaves. One part, two tasks, and nothing else is left of the request.

## The part

**jig-core-steps** (jig D001; `jig/docs/design/domain/root.md`, 3), slice 1. `notes/contracts.md` holds the shapes.

- `tasks/core-prepare-agent.md`, high, ~110 minutes, needs nothing: the variants go, `Event::Activate` gains `busy`,
  temper reads its read room in the activation's step. It can't be split: the variants' removal is one atomic change
  across jig and temper.
- `tasks/d001-close.md`, docs, medium, ~20 minutes, needs the last: **Paid when** rewritten, the measure confirmed at
  39, the item and its row deleted, the fixed answers added to D004.

## What the cut decided

- **The Busy route is a field on `Event::Activate`, read when the activation runs.** The held spec's gate on `Resume`
  and its `expect` can't stand alone: with `Now::Activate` in the decision, an activation doesn't pass through `Resume`,
  so a full table would panic, and a gate there would hold jobs and agents with no window. The core's check sits in
  `route_activate`'s `Allow` arm, where `Now::PrepareAgent` was, so what waits is what waits today.
- **`Now::StartPreparation`'s `from` is no longer optional.**
- **The design moved with the first push:** `jig/docs/design/domain/root.md`, 3 is rewritten, 14's question closes, and
  `root.status.md` and `STATUS.md` name what's left to build. `core-prepare-agent` closes those entries.
- The held spec's Measure of 38 is 39 here, as the request says. The base reads 40, not the item's 39: one arm goes.
- **One order moves, and only if a test sees it:** the second work hop goes, so an admitted agent prepares one hop
  earlier in its decision than a job activated behind it. Any test whose visible events move stops and asks.

## Overlaps

Unavoidable: `core-prepare-agent` removes one arm each from `temper-service`'s and `jig-test-service`'s
`clients/translation.rs`, and `jig-test-service`'s `route.rs`, where debt-7's `jig-store-host` and `temper-store-host`
own the whole crate's `src/**`. The arms are one line and the spec says to edit what `rg` finds. `d001-close` meets
debt-7's `service-items` in `jig/docs/development/debt/README.md`, a different row.

## Critical path

`core-prepare-agent` → `d001-close` (~130 implementer-minutes).
