# hearth-memory-3: the plan

Round 3 of hearth's move to the dynamic memory model: the mmdebstrap machine, as round 2 did git. D001 stays open. The design this plan builds on is
`hearth/docs/design/domain/render.md`, 8.5, which this cut extended. The list of checks is in
`hearth/docs/development/debt/D001-room-checks-and-max-out.md`, "What each room check keeps", which this cut extended too.

## Parts and tasks

| Part | Tasks | Needs |
|---|---|---|
| `orders` | `mmdebstrap-orders` | nothing |

The rest of the plan, `machine` and `close`, follows in the next push. The critical path as it stands is `mmdebstrap-orders`, about 30 implementer-minutes.

## What was decided

- **The capped queues beside the machines count as checks,** as in round 2. D001's table lists none of them, so each gets its row and its test first.

## Overlaps

None: no draft beside this one has a task on a file under `hearth/`.
