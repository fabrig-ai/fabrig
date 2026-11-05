# hearth-memory-2: the plan

Round 2 of hearth's move to the dynamic memory model, and the start of D001: what each room check keeps, then the git machine uncapped.
`hearth/docs/design/domain/render.md`, 2.2 and 8.5, which this cut wrote, states the orders.

## Parts and tasks

| Part | Tasks | Needs |
|---|---|---|
| `orders` | `git-orders` | nothing |

The rest of `orders` (the service's checks) and the `machine` part follow in later batches.

## What was decided

- **The list goes to D001.** The request puts it in the status file of the document that owns it, or in D001 if none does. The checks span render,
  the format, the store and the client, and no design document says how `iterate` paces work, so none owns the list.
- **The capacity-1 queues beside git's machines count as checks.** They aren't `.room()` calls, but a machine's second output panicked on them, so
  uncapping them takes that check away. `git-orders` puts tests where those panics were.
- **The `machine` part goes after every order test,** as the request's section 1 says.

## Overlaps

See the later batches.
