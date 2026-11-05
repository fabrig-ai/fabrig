# hearth-memory-4: the plan

Round 4 of hearth's move to the dynamic memory model: the config machine, as rounds 2 and 3 did git and mmdebstrap, and one sentence of `format.md`. D001 stays open. The design this plan builds on is
`hearth/docs/design/domain/render.md`, 8.5, which this cut extended. The list of checks is in `hearth/docs/development/debt/D001-room-checks-and-max-out.md`, "What each room check keeps", which this cut
extended too.

## Parts and tasks

| Part | Tasks | Needs |
|---|---|---|
| `sentence` | `container-sentence` | nothing |
| `orders` | `config-orders` | nothing |
| `machine` | `machine` | `orders` |
| `close` | `close` | `machine` |

The critical path is `config-orders`, `machine` and `close` in turn: about 65 implementer-minutes. `container-sentence` stands apart, in its own commit.

## What was decided

- **The capped queues beside the machine count as checks,** as in rounds 2 and 3. D001's table lists none of them, so each gets its row, and each order its test, first. `Decoder`'s `events` keeps nothing:
  the tokenizer answers one `Next` with one event, and `drive` takes it before it asks the next. `Decode`'s `below` and `above`, in `read.rs`, each keep an order.
- **`Decode::resume`'s `if let`s stay.** It takes one event and one demand a call, and an entry point gives one of each. The order is stated in 8.5, and `suite/orders.rs` holds it.
- **The new tests own uncapped queues from the start,** so they bite the same before and after `machine`, which doesn't break them again. `run`'s per-tick assertion stays, and only its message changes,
  which is one of the four `max_out` counts this round removes.
- **"Its reservations" are the two hand-grown lists:** `values` in `json.rs` and `pending` in `document.rs`, each of which D001's table says keeps nothing. What else looks like a cap stays. The depth
  stacks and the key map are the edge (`Fault::Keys`, the tokenizer's depth), and the lists sized by a count already read allocate as they fill and refuse nothing. Moving those to `uncapped` would only
  take more room.
- **The bound formulas stay, and stay true.** `decoder_bound` and `decoded_bound` count `Queue::price(1)` for `events`, and an uncapped queue's first push takes four items' room, so they price four, and
  their two value tests move by three events. `valid` reads only whether each is `Some`. `decoder_bound`'s `storage` needs no change: an open array's four slots cost at least three bytes of document,
  4/3 of a record a byte under the 3/2 it counts.
- **The order of rounds holds.** Nothing outside `json.rs` reads `MaxOut`, `UP_MAX_OUT` or `DOWN_MAX_OUT`, so the service needs no change, and nothing new is named for the service round unless `machine`
  finds otherwise. `max_out` falls by 4, from 57 to 53: three names in `json.rs` and the world's message. `MaxOut` isn't counted by the ratchet's pattern. `.room()` in non-test step code falls by 2, from 23.
- **Uncapping costs a little heap.** `events`, `Decode`'s two queues and a new array's first push each take four items' room where one took less. `machine` pre-approves moving a ceiling, with the peaks and why.
- **D001's rows come and go inside this plan.** The cut adds three, `close` takes them out with the two `.room()` rows, and the paragraph under the table says where the orders and tests stand. Render's
  section 10 now asks only about `Client`'s queues, and its status holds a **Not yet** on 8.5 until `close`.

## Overlaps

`debt-21`, whose trunk hasn't landed, changes two things beside this plan, and neither is in any task's `files`:

- It moves `hearth/tests/render/src/golden/repository.rs` and `JSON_QUEUE` into `hearth/testing/hearth-test-render`, so `machine` leaves both alone. That one test driver keeps a capped queue beside the
  decoder, where it can lose nothing: an extra output panics. It's the one queue this round leaves capped by choice.
- It takes `generate::mutate` out of `hearth/tests/config/src/generate.rs`, so `config-orders` writes its refused documents by hand and uses only `generate::document`, which stays.

No other draft beside this one has a task on a file under `hearth/`.

## Figures that don't agree

The request gives the focused suite as about 32 CPU s on the gate's record, over the 30 of `tools/tiers/budgets.toml`. `hearth/docs/design/domain/testing.status.md`, 9.1 records 44.886618 CPU s by
`tools/tiers/cpu.py` on 2026-11-05, and says which of the two measures the suite is unknown. This round raises no budget and can't bring the suite under it: it adds three small tests, under 0.05 CPU s
each, so the suite's own figure may rise by up to 0.15 CPU s, and `config-orders` reports what they cost.
