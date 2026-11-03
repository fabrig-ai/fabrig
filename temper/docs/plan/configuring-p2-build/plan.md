# configuring-p2-build: plan

Phase P2 of `temper/docs/design/draft/configuring.execution.md`: settings through ctl, built
from the design `configuring-p2` released, on the reference's M3 as landed on main. Written by
the cut as it releases; each batch brings it up to date.

## Parts

| Part | Slice | Design | Tasks |
|---|---|---|---|
| `policy-item` | 1 | jig's `domain/engine.md` 17.2 and 17.6; `domain/people.md` 5.2; temper's `protocol/configuration.md` 13.1; `client/tools.md` 5.3 | `policy-surface` (released) |
| `restore` | 2, **the walk** (`walk = true`) | jig's `domain/engine.md` 17.5 and 17.10; `domain/connectors.md` 17; temper's `client/tools.md` 5.2 and 5.3 | to come |

The cross-task shapes are `notes/contracts.md` (B1 on), which takes configuring-p2's
`notes/contracts.md` (P1, P2) as binding; the standing rules, `notes/common.md`.

## The trunk

This trunk was made from `configuring-p2/integration`, which sits on
`reference-m3/integration` at 3db9a0e69a, before M3 landed. The tick's catch-up with main
waits while the cut is in flight, so the cut's first commit merges `origin/main` (M3 landed,
224965929f) into its branch, and the trunk takes it with the first batch: every task starts on
M3's `apps-*` and `kinds-*`. One conflict, `temper/docs/design/client/tools.status.md`: main
built `forge replace`, which leaves the line; P2's 5.3 entry stays.

## Batches

1. `policy-surface` (interface, 60): the setting named, refused `Shape` until its check.

## Critical path, as it stands

`policy-surface` (60), then the rest as later batches release it.

## Overlaps with other plans

- None of this batch's files is changed by a draft beside this one, as far as their trunks
  showed at the cut.
