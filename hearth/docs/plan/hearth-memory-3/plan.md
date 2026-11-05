# hearth-memory-3: the plan

Round 3 of hearth's move to the dynamic memory model: the mmdebstrap machine, as round 2 did git. D001 stays open. The design this plan builds on is
`hearth/docs/design/domain/render.md`, 8.5, which this cut extended. The list of checks is in
`hearth/docs/development/debt/D001-room-checks-and-max-out.md`, "What each room check keeps", which this cut extended too.

## Parts and tasks

| Part | Tasks | Needs |
|---|---|---|
| `orders` | `mmdebstrap-orders` | nothing |
| `machine` | `machine` | `orders` |
| `close` | `close` | `machine` |

The critical path is all three in turn: about 70 implementer-minutes.

## What was decided

- **The capped queues beside the machines count as checks,** as in round 2. D001's table lists none of them, so each gets its row and its test first: `requests` and `seen`
  in `resolve.rs`, `demands` and `found` in `indices.rs`, each of which keeps an order, and `pending` in `resolve.rs`, which keeps nothing.
- **`stream`'s `if let` stays.** `resolve.rs` takes one `Seen` and one demand from a delivery, and `indices.rs`'s `demand` takes one. On an uncapped queue a second would be
  lost where capacity 1 panicked. Draining instead would reshape `stream`, whose `pipe` can't stay borrowed across `self.too_many` in a loop. The machines give one, as their
  states show, so the request's rule fits: the order is stated in 8.5, and `suite/orders.rs` holds it.
- **The new tests own uncapped queues from the start,** so they bite the same before and after `machine`, which doesn't break them again.
- **`tail.rs` doesn't change,** as the request says: `admit`'s check, `Tail`'s cap and the `decoder_bound`s. D001 names a test for its order, and nothing had shown that test
  bites: 32 seeds of up to 32 lines, some not admitted, may not pass 20. `mmdebstrap-orders` breaks it to see, and writes a small test if it doesn't bite.
- **There's no `DOWN_MAX_OUT` to remove.** The request lists it, and this crate has only `UP_MAX_OUT`, in `output.rs` and `packages.rs`.
- **The order of rounds holds.** The one reader of those constants outside the crate and its world is `indices.rs`'s loop, which becomes `0..found.len()`. The service needs no
  change, so nothing new is named for the service round, unless `machine` finds otherwise. `MaxOut` isn't counted by the ratchet's pattern, and `max_out` falls by 13, from 70 to 57.
- **Uncapping costs a little heap.** `seen`, `requests`, `demands` and `found` live for one delivery, and their first push takes four items' room where 1 or 2 took less.
  `pending` is unchanged: `with_capacity(4)` takes the same four. `machine` pre-approves moving a ceiling, with the peaks and why.
- **D001's rows come and go inside this plan.** The cut adds five, `close` takes them out, and the paragraph under the table says where the orders and tests stand.

## Overlaps

None: no draft beside this one has a task on a file under `hearth/`.

## Figures that don't agree

The request says the focused suite stays under what `tools/tiers/budgets.toml` holds, 30 CPU s. `hearth/docs/design/domain/testing.status.md`, 9.1 records it over that, at
44.886618 CPU s on 2026-11-05. This round can't bring it under: it adds about six small tests, under 0.05 CPU s each, and leaves the excess where that status file holds it.
