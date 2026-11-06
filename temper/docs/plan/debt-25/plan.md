# debt-25: the twenty-fifth debt round

Two independent parts (`inputs/request.md`): the spot review of debt-23, and jig's test items D030, D034, D035. No temper client view, no `temper.css`, no `jig-client-view`. This file grows with each push.

## Parts and tasks

| Source | Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---|---:|---|
| review 1, 7 | `spot-review` | `connect-cells` | | high | 30 | `connect` writes out its impossible cells; the close pin carries a failure |
| review 7 | `spot-review` | `store-pin` | | medium | 15 | a failed entry keeps a store's standing `Full` |
| review 7 | `spot-review` | `debug-rows` | | high | 35 | the table keeps one `Secret` row and the hand-written views; `Component`, `Issuer` |
| review 2 | `spot-review` | `store-flight` | `store-pin` | medium | 10 | five no-op `Flight::Idle` writes go |
| review 2, 3 | `spot-review` | `builder-shape` | | high | 55 | `Output`, `printing`, the `Caching` arm; accessors; `Holding`, `Resource`; `resume`'s exception |
| review 2 | `spot-review` | `builder-stage` | `builder-shape` | high | 40 | `Intake` moves into `Live`, drain arm guarded first; or hearth D014 |
| review 3 | `spot-review` | `llm-reusable` | | medium | 20 | `Connection::reusable`; `serve_waiting`'s exception |
| review 6 | `spot-review` | `llm-outcome` | | high | 50 | `fail_closing`; `Outcome` becomes state, with `ErrorEnded`; or only the folding, measured |
| review 4 | `spot-review` | `forge-until` | `connect-cells` | high | 40 | `rest` is `until`; three holders pass theirs into `exchange.start` |
| review 5 | `spot-review` | `tasks-tuples` | | medium | 20 | four same-typed tuples named |
| review 5 | `spot-review` | `ops-tuples` | | medium | 15 | two `(Token, Token)`s named |
| review 5 | `spot-review` | `fleet-claimed` | | medium | 25 | `State::Claimed { claiming, at }` |
| review 8 | `spot-review` | none | | | | written by this cut: `jig/docs/design/protocol/llm.md`, 5.2 and `skein/docs/design/lib.status.md`, 6 |

The rest of the plan is still being cut: D030, D034 and D035.

## What was decided

- **Finding 1 covers `Start::Handshaking` too.** The review names `state.idle()`, but `Handshaking { until }` overwrites the same cells on a TLS connection. `connect` matches the state first, so both are one written-out set.
- **`Closed` keeps its rows.** The review leaves it out, and so does the task, so no reachable cell changes.
- **Finding 7's first pin travels with finding 1:** same crate, same function, one writer.
- **`debug.rs` starts from 25 rows, of which three reach a hand-written view directly.**
- **Finding 2's `Stage` lands in `hearth-protocol-builder/src/lib.rs`,** which holds the drain arm. The request names `hearth-service-builder/src/lib.rs` as held by fabrig/hearth, and that file names none of this, since `Child` is `pub(crate)`. The task stays off it. If the guard or the move changes a test's events, the task records hearth D014 instead.
- **Finding 6's "about -15" holds for `fail_closing`.** `Outcome` itself can't go on `Closing { cancelled }` alone: `ErrorBody` outlives its terminal (`Up::End`) until `Done`, and `skein/docs/design/llm.status.md`, 3 records that as the flag's reason. So the task adds a state, measures it, and keeps `Outcome` whole if the move grows production: no partial flag.
- **Finding 4's fourth `start` caller:** `send_start` sets no deadline today, so it passes the one the connection holds. A refused start keeps today's deadline, because `until` is `start`'s first line.
- **Finding 5 is the six tuples and `Claimed`;** the parcels and the other ten tuples stay, by the request.
- **Finding 8 is two sentences,** written here, so it has no task.

## Critical path

`builder-shape` then `builder-stage` is the longest chain so far, at 95 minutes.
