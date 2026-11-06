# hearth-memory-5: the plan

Round 5 of hearth's move to the dynamic memory model: the records machine, as rounds 2 to 4 did git, mmdebstrap and config. It's the last machine before the service. D001 stays open. The design this plan builds on is `hearth/docs/design/domain/render.md`, 8.5, which this cut extended. The list of checks is in `hearth/docs/development/debt/D001-room-checks-and-max-out.md`, "What each room check keeps", which this cut extended too.

## Parts and tasks

| Part | Tasks | Needs |
|---|---|---|
| `orders` | `printer-orders` | nothing |
| `machine` | `machine` | `orders` |
| `close` | `close` | `machine` |

The critical path is `printer-orders`, `machine` and `close` in turn: about 60 implementer-minutes.

## What was decided

- **The capped queues beside the machine count as checks,** as in rounds 2 to 4. D001's table listed neither, so each gets its row and its order's test first. `Client`'s `requests` keeps one demand out, with a piece and then the next demand from a `Room`. `Client`'s `events` keeps one answer a record. The printer's state keeps both, and render's 8.5 now says so, which closes section 10's question on these queues.
- **No records world.** hearth has none, and a new package costs more than this round. The machine tests sit beside `Machine`, in the crate's own tests, over chunk sizes and fault points, and own uncapped queues so they bite the same before and after `machine`. 7.2's world test under load is the protocol world: `Below::stream` now checks each client output's demands and pieces in every run, at no cost in runs, and one new test takes the chunk down to 16 bytes, the only way a run makes `requests` hold a piece and a demand at once.
- **"Its reservations" are the capacities themselves.** The crate has no `.room()`. Its `List::with_capacity(count)` and `Writer::new(total)` in `form/` size an encoding from a measured length, and the `Stack` in `parts/exports.rs` is a depth. None reserves room for an output, so they stay.
- **`takes` doesn't move.** The row "One record at a time" stands, and `what_a_call_prints_and_stores_does_not_depend_on_its_queues` keeps passing, as the request asks. No test is added for it: a second record while one is out panics in the printer's `Writing`, whatever the queues.
- **The order of rounds holds.** Nothing outside `printer.rs` and `src/tests/printer.rs` reads `MaxOut`, `UP_MAX_OUT` or `DOWN_MAX_OUT`, so the service needs no change, and nothing new is named for the service round unless `machine` finds otherwise. `max_out` falls by 14, from 53 to 39: two names in `printer.rs` and twelve in the test's `Machine`, whose per-entry assertions stay with their figures as literals. `.room()` in non-test step code doesn't move: neither `client.rs` nor the crate calls it.
- **Uncapping costs a little heap.** Two `Client`s, `events` at four items' room where it took one and `requests` at four where it took two. `machine` pre-approves moving a ceiling, with the peaks and why.
- **D001's rows come and go inside this plan.** The cut adds two and `close` takes them out, with its **Not yet** on 8.5.

## What this round leaves as it is

Seen in the survey, outside this request, and left for whoever owns them:

- **`printer::Request::Close` has no production caller:** `Client` closes its output through io. `close_withdraws_the_demand_and_answers_closed` stays.
- **`Up::Failed` doesn't reach the printer in production:** `Client::up` takes it first and calls `fail`, so the printer's `Below::Failed` and `Client::resume`'s `Event::Failed` arm run only under the machine's own tests. `a_record_is_answered_once_whatever_the_chunk` holds the machine's answer anyway.
- **`takes`' `requests.is_empty() && events.is_empty()`** repeat what `Stream` already says, as far as this survey reads it. `debt-25` is rewriting `Stream`, so this round doesn't touch them.

## Overlaps

`debt-25`, whose trunk hasn't landed, rewrites much of `hearth-protocol-builder/src/` on `client.rs`'s `Output` and `Stream`. Two lines of this plan meet it: `machine` changes `events` and `requests` in `Client::new`, and `debt-25` changes `output`, the line right after them. Whichever lands second has a one-hunk conflict, solved by taking both. This couldn't be avoided: those two lines are the round. `machine` is held to them, and `printer-orders` and `close` touch nothing `debt-25` does. `debt-25` leaves D001 as it stands on main.

No other draft beside this one lists a file under `hearth/`.

## Figures that don't agree

The request gives the focused suite as about 32 CPU s on the gate's record, over the 30 of `tools/tiers/budgets.toml`. `hearth/docs/design/domain/testing.status.md`, 9.1 records 44.886618 CPU s by `tools/tiers/cpu.py` on 2026-11-05, and says which of the two measures the suite is unknown. This round raises no budget and can't bring the suite under it: it adds three small tests, under 0.05 CPU s each, so the suite's own figure may rise by up to 0.15 CPU s, and `printer-orders` reports what they cost.
