# debt-25: the twenty-fifth debt round

Two independent parts (`inputs/request.md`): the spot review of debt-23, and jig's test items D030, D034, D035. No temper client view, no `temper.css`, no `jig-client-view`. This file grows with each push.

## Parts and tasks

| Source | Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---|---:|---|
| review 1, 7 | `spot-review` | `connect-cells` | | high | 30 | `connect` writes out its impossible cells; the close pin carries a failure |
| review 7 | `spot-review` | `store-pin` | | medium | 15 | a failed entry keeps a store's standing `Full` |
| review 7 | `spot-review` | `debug-rows` | | high | 35 | the table keeps one `Secret` row and the hand-written views; `Component`, `Issuer` |

The rest of the plan is still being cut: findings 2 to 6 and 8, then D030, D034 and D035.

## What was decided

- **Finding 1 covers `Start::Handshaking` too.** The review names `state.idle()`, but `Handshaking { until }` overwrites the same cells on a TLS connection. `connect` matches the state first, so both are one written-out set.
- **`Closed` keeps its rows.** The review leaves it out, and so does the task, so no reachable cell changes.
- **Finding 7's first pin travels with finding 1:** same crate, same function, one writer.
- **`debug.rs` starts from 25 rows, of which three reach a hand-written view directly.**

## Critical path

Nothing waits on anything yet: the three tasks are independent.
