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

| D030 | `conformance` | `trait-unused` | | high | 40 | `Key`, `Write`, `Record`, `Input::Restore`, `configuration_heap` go; Measure 21 |
| D030 | `conformance` | `trait-policy` | `trait-unused` | high | 35 | the harness compares `Policy` by value; `policy_generation`, `recovery` go; Measure 19 |
| D030 | `conformance` | `core-view` | `trait-policy` | high | 35 | `jig_conformance::core::view`, read by both store views |
| D030 | `conformance` | `core-answers` | `core-view` | medium | 25 | the grant's hand copies go; one settled answer, if smith-free |
| D030 | `conformance` | `conformance-close` | `core-answers` | medium | 15 | the README; D030 rewritten, stays open |
| D034 | `replay` | `alert-table` | | medium | 30 | one table test, one baseline, 7 runs for 12; `assert_alert` once |
| D034 | `replay` | `outside-derives` | `core-view` | medium | 20 | `PartialEq, Eq` in the kits, no step crate |
| D034 | `replay` | `typed-outside` | `outside-derives`, `core-answers` | high | 40 | `Outside`, `first_difference`, one `compare` |
| D034 | `replay` | `observer-guard` | `typed-outside`, `alert-table`, `core-view` | high | 35 | no view when nothing committed, measured; D034 rewritten, stays open |
| D035 | `numbers` | `numbers-testing` | | medium | 30 | four test kits; 17 of the Measure |
| D035 | `numbers` | `numbers-client` | | medium | 30 | scripted engine, client settings |
| D035 | `numbers` | `numbers-accounts` | | medium | 30 | one `ALICE`, `BOB`, `PROVIDER`; story names; acts once |
| D035 | `numbers` | `numbers-people` | | medium | 25 | sign-ins, holds; `sign_in_owner` |
| D035 | `numbers` | `numbers-tasks` | | medium | 40 | `Party::Person(9)` named; the spends; acts once |
| D035 | `numbers` | `numbers-core` | | high | 60 | `OWNER`, `CONNECTOR`, `phase`; a sign-in builder |
| D035 | `numbers` | `numbers-close` | the six above | medium | 10 | Measure 2; the item goes |

855 implementer-minutes in all: 355 in `spot-review`, 150 in `conformance`, 125 in `replay`, 225 in `numbers`. By the surveys' reckoning, not measured, test lines fall by about 150 in D030 and 200 in D035, and D034 ends about level. No debt item closes in `spot-review`, by the request. D035 closes. D030 and D034 stay open, narrowed (below).

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
- **Every finding held on checking, so none is skipped.** Line numbers had drifted, and the tasks find code by name.
- **D030 stays open.** `Application` reaches 19 functions and stops. `jig/testing/jig-conformance` can't decode (the charter links smith), the first period and `Drained` differ between adopters in events the referee sees, and a trait must remain for `Harness<A>`. Lowering the count further, with sub-traits or an enum-argument hook, would pay nothing, so the **Paid when** doesn't hold. `jig/docs/design/domain/testing.status.md`, 5 says so, and the item is rewritten. `notes/contracts.md` holds the shape.
- **D030's referee clause was already paid:** the referee owns its effect vocabulary.
- **D034's `OnceLock` can't work:** nextest gives each test a process. One table test shares the baseline instead. If it passes the 10 s fuzzy guide, the task asks.
- **D034's row feed is left out, by judgement,** and the item stays open for it alone. `store_view`'s avoidable cost is that it runs on every inspect and tick, and a check on `applied` removes that. A row feed would still rebuild `snapshot(&store.rows)` and walk the referee's tasks per commit, unless the referee takes deltas. It would also add an equivalence test, against the request's fewer test lines and "keep the plan small". `observer-guard` measures what's left, so fabrig/debt can decide with figures.
- **`fuzzy/joint.rs`'s six acceptance and move tests keep their baselines:** each runs its own seed, and sharing would narrow them to one.
- **D035's Measure is told apart from its point:** the Measure misses `account(5, 1, ..)`, so the sweeps pay the item, and `notes/numbers.md` says so. `System::new(16, 16)` reads its limits rather than joining the exclusions. Two `turn_line` calls stay, explained, because the link crate's expected page pins their values.
- **Left out of D035, as not fixtures or verbs:** ops's `look` and `apply`, which are copies of a fake; `world()` in engine-workers and worker-link, which are constructors. Ops's `assert_alert` is `alert-table`'s.
- **No table here pins a refactor.** `alert-table`'s rows each hold a crash at a different commit, and `debug-rows` cuts the one table the review named.

## Overlaps

No overlap with `ui-final-walk`: nothing here touches its files, and the queue's other drafts list none. Inside this plan, one trait edit breaks four adopters, so the D030 tasks and `typed-outside` run in a line. `numbers-core` and `core-view` both touch `jig-core-kit`, in different files (`fixtures.rs`, `conformance.rs`). `core-answers` and `outside-derives` run side by side: one changes bodies, the other derive lines.

## Critical path

`trait-unused`, `trait-policy`, `core-view`, `core-answers`, `typed-outside`, `observer-guard`: 210 implementer-minutes. The next longest is `builder-shape`, `builder-stage`, at 95.
