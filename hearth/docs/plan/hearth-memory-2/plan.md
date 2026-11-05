# hearth-memory-2: the plan

Round 2 of hearth's move to the dynamic memory model, and the start of D001: what each room check keeps, then the git machine uncapped. D001 stays
open. The design this plan builds on is `hearth/docs/design/domain/render.md`, 2.2 and 8.5, which this cut wrote. The list of checks is in
`hearth/docs/development/debt/D001-room-checks-and-max-out.md`, "What each room check keeps".

## Parts and tasks

| Part | Tasks | Needs |
|---|---|---|
| `orders` | `git-orders`, `service-orders` | nothing |
| `machine` | `machine` | both `orders` tasks |
| `close` | `close` | `machine` |

The critical path is `service-orders`, `machine`, `close`: about 120 implementer-minutes.

## What was decided

- **The list goes to D001.** The request puts it in the status file of the document that owns it, or in D001 if none does. The checks span render,
  the format, the store and the client, and no design document says how `iterate` paces work, so none owns the list. An order goes to `render.md`,
  8.5, and the one gap to `render.status.md`.
- **No room check alone keeps an order, by reading.** A queue is first in, first out, so a check only delays: an order could only break across two
  queues, or at an entry no predicate guards, and a survey traced both. Five orders stand beside the checks, held by predicates or machine state,
  and 8.5 states them. The checks keep capacities, one (`protocol_room` before `fire`) keeps nothing, and seven hand-grown lists keep nothing.
- **A reading isn't a test, so `service-orders` runs it.** `what_a_call_prints_and_stores_does_not_depend_on_its_queues` runs four scenarios at
  `queue` 3, 32 and 4,096 and compares what they print and store. That is the request's test of an order, turned around: any order only room keeps
  shows as a difference. Five bites then show which test holds which order. A difference is a finding, and `close` stops on it and asks.
- **`file_told` before `told` isn't stated as an order.** Nothing traced depends on it. `service-orders`, row 6, swaps the loops to check; `close`
  writes down what that found.
- **The capacity-1 queues beside git's machines count as checks.** They aren't `.room()` calls, but a machine's second output panicked on them, so
  uncapping them takes that check away. `git-orders` puts tests where those panics were. The bigger of those is a piece holding the next read: with
  the queue uncapped, a machine that read ahead would pile a blob's pieces up in memory and nothing would stop it.
- **`Tail`'s queue stays capped.** It holds what a child's standard error pushes in, so it's an edge (`skein/docs/foundation/programming-model.md`,
  6.1). Its `.room()` calls go; its cap and `decoder_bound` stay. The request's "internal queues" are the ones beside the machines.
- **The order of rounds holds.** Nothing outside `hearth-protocol-git` reads its `MaxOut`, `UP_MAX_OUT` or `DOWN_MAX_OUT` (`indices.rs`'s
  `packages::UP_MAX_OUT` is mmdebstrap's), so the service doesn't need a change and no question is open. `hearth-protocol-builder` changes in six lines,
  each one queue's constructor. `MaxOut` isn't counted by the ratchet's pattern, so `max_out` falls by 6.
- **`hearth-protocol-builder`'s own `.room()` calls stay.** `revision_stdout`'s and `indices.rs`'s size edges, and they aren't the machine's.
- **The store step's `!above.is_empty()` stays as it is.** It reads the whole of `calls` where it means "this entry emitted". The request asks for a
  test here and for hearth not to change, so this round holds it by a test and `render.status.md` records it. D001 gives the fix to the service round.
- **Uncapping costs a little heap.** `Queue::uncapped` takes four items' room on its first push, where a capacity of 1 took one. git's ceiling of 1,900
  bytes may move; `machine` pre-approves it, and gives the peaks and why.

## Overlaps

- **`hearth/docs/development/debt/D001-room-checks-and-max-out.md`:** `debt-14/ratchet` edits its **Measure** line and its "Eight of the 85" line. This
  cut's sections are at the end of the file, away from both, and `close`, which must edit **Measure**, goes last.
- **`hearth/scripts/old-forms.txt`:** `debt-14` changes it three times, and `machine` lowers it. fab recounts where merges meet.
- **`hearth/crates/hearth-protocol-git/src/listing.rs`:** `debt-14/digits` changes `number`, and `machine` changes the imports and constants at the top.
- **`hearth/tests/builder`:** `debt-14` changes `suite/domain.rs`, `suite/properties.rs` and `real.rs`. `service-orders` keeps to a new file, one line of
  `main.rs` and `simulated.rs`.

## Figures that don't agree

The request says the focused suite stands at 30.2 CPU s, the gate's record. `hearth/docs/design/domain/testing.status.md`, 9.1 says 44.9, from
`focused-budget`'s measurement on 11-05. This plan works from the request's figure and keeps its tests small either way. It doesn't reconcile them.
