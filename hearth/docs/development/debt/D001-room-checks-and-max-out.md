# D001: The service and its machines pace work by room checks and `MAX_OUT`

- **Kind:** complexity
- **Where:** `hearth-service-builder/src/lib.rs`, `hearth-protocol-builder/src/lib.rs`, `hearth-protocol-git`, `hearth-protocol-mmdebstrap`, `hearth-protocol-config/src/json.rs`, `hearth-client-protocol-records/src/printer.rs`, `hearth-domain-builder/src/lib.rs`, `hearth-domain-render/src/lib.rs`, `hearth-domain-store/src/lib.rs`, `hearth-domain-tree/src/lib.rs`, `hearth-domain-build/src/lib.rs`, `tests/builder/src/fixtures/bounds.rs`
- **Seen:** 2026-11-04 at `66968a30ef`, in the request of plan hearth-workspace (its port table, and its rule that adopting the newer model in full is out of scope)
- **Seen:** 2026-11-04 at `66968a30ef`, in `hearth/scripts/old-forms.txt` (`max_out`)
- **Checked:** 2026-11-04 at `66968a30ef`
- **Value:** high (hearth paces as the other trees do; `iterate` shrinks; new code stops copying the old form)
- **Size:** L
- **Status:** open
- **Paid when:** no hearth step crate defines a `MaxOut` or a `MAX_OUT` constant, `iterate` asks no queue for room but the kernel's edge, the queues between a service's own layers are uncapped, and the orders the room checks kept are named predicates with world tests.
- **Measure:** `grep '^max_out ' hearth/scripts/old-forms.txt`: `max_out 85`

## What

The port took the renames and stopped there, as its request said.
Under `skein/docs/foundation/programming-model.md`, 6.3 and 7.1, nothing
caps the entries between a root's own children; only the edges refuse.
hearth still holds the old form:

- every step crate of the builder declares what one entry point may emit:
  `MAX_OUT` in five domains, `MaxOut` and `MAX_OUT_*` in
  `hearth-protocol-builder`, and `MaxOut`, `UP_MAX_OUT` and `DOWN_MAX_OUT`
  in the git, mmdebstrap, config and records machines;
- `hearth-service-builder/src/lib.rs` keeps eight capped queues, and
  `told` is the only uncapped one. Before each entry point it asks
  `io_room` or `protocol_room`; after it, `within` asserts the constant.
  23 lines of that file hold the form;
- `Service::new` refuses `limits.queue` under 3, "queues holding every
  MAX_OUT", and `tests/builder/src/fixtures/bounds.rs` sizes a world on
  that figure;
- 26 `.room()` calls remain in non-test step code.

Eight of the 85 are `io::MAX_OUT_*`, the kernel's edge, which skein keeps
and the other trees exempt (D011).

## Why it matters

Each new machine must declare its constants and each pass must loop on
them. Reading `iterate` means following the room checks, not the flow.
skein, smith, jig and temper count zero of this form, so hearth reads
unlike them, and an order a room check keeps by accident is written down
nowhere (`skein/docs/foundation/programming-model.md`, 7.2).

## What pays it back

First list what each room check keeps, a capacity or an order, as 7.2
asks, and write each order's test. Then make the inner queues uncapped,
from the machines up, and take the constants and `within` out. Do it with
D004, which rewrites the same `io_up` and `io_down`.

## What each room check keeps

By `skein/docs/foundation/programming-model.md`, 7.2, each check keeps a
capacity, an order, or nothing. This section lists them, for the round
that removes each one. A check here that goes needs its order's test
first, and each order's test is named beside it.

### The machines and the domains

| Where | Check | Keeps |
|---|---|---|
| `hearth-protocol-git/src/tail.rs`, `Tail::admit` | `self.lines.room() == 0` | A capacity, `tail_lines`, at an edge: git's standard error. An order too: the oldest line leaves first. `the_tail_keeps_the_newest_lines_oldest_first_whatever_the_cuts`, in `hearth/tests/git`. |
| the same | `self.line.room() > 0` | A capacity, `tail_line`, at the same edge. An order too: a line keeps its first bytes. The same test. |
| `hearth-protocol-git/src/tail.rs`, `Tail::lines` | `lines.room() > 0` | A capacity: at `tail_lines` 0 it stops the unended line. `a_tail_of_no_lines_holds_nothing`. |
| `hearth-protocol-mmdebstrap/src/tail.rs`, `admit` | `tail.lines.room() == 0` | A capacity, `tail_lines`, at an edge, and the same order as git's. `generated_failures_keep_the_reference_s_tail_and_cause`, in `hearth/tests/mmdebstrap`. |
| `hearth-protocol-builder/src/lib.rs`, `revision_stdout` | `stdout.room()`, three times | A capacity at an edge: `rev-parse`'s answer, of `MAX_STDOUT` and one byte to see excess. |
| `hearth-protocol-builder/src/indices.rs`, `Indices` | `self.intake.room()` | A capacity: a read of a `Packages` file asks for no more than its intake holds. |
| `hearth-protocol-tree/src/fields.rs`, `Fields::value` | `values.room()` | A capacity at an edge: a stored value's items, refused past `limit`. |
| `hearth-domain-render/src/parts/exports.rs`, `push` | `record.room() == 0` | Nothing: it doubles a list by hand and refuses nothing. `List::uncapped` does the same. |
| `hearth-domain-render/src/schema.rs`, `push`, `fault` and `add_default` | `faults.room() == 0`, twice, and `values.room() == 0` | Nothing, the same. |
| `hearth-domain-render/src/schema/references.rs`, `condition` | `faults.room() == 0` | Nothing, the same. |
| `hearth-protocol-config/src/document.rs` | `pending.room() == 0` | Nothing, the same: a stack whose depth the decoder's limits bound. |
| `hearth-protocol-config/src/json.rs` | `values.room() == 0` | Nothing, the same. |

The capacity-1 queues beside each git machine, in
`hearth-protocol-builder/src/lib.rs` and `read.rs`, hold no room check, but
a machine's second output panicked on them. They kept a capacity, one event
and one demand for each entry, and what the machine's states keep behind
them is in `hearth/docs/design/domain/render.md`, 8.5, with its tests in
`hearth/tests/git`.

### The service

All in `hearth-service-builder/src/lib.rs`.

| Where | Check | Keeps |
|---|---|---|
| `iterate` | `submissions.room() == 0` | A capacity, the kernel's edge. It stays. Io's submissions going before the file layer's is pop order, not room. |
| `io_up`, `io_down` | `io_room`, four times | A capacity, the kernel's edge, through `io_submissions`. `is_ready` and `completions.is_empty()` beside it hold the order. |
| `io_up`, `io_down` | `file_told.room() < 1`, twice | A capacity between the root's own children. It never binds: the file layer takes one request at a time, so the queue holds at most one event. |
| `io_up`, `io_down` | `file_submissions.room() < 2`, three times | A capacity, the kernel's edge, through the file layer's queue: an operation and its cancel. The one before `expire` reserves 2 for one cancel. |
| `protocol_up` | `protocol_room`, before `protocol::fire` | Nothing: `fire` touches neither queue. |
| `protocol_up`, `protocol_down` | `protocol_room`, three times | A capacity between the root's own children: `calls` and `asked`. |
| `domain_stage` | `answers.room()`, three times | A capacity between the root's own children, with a real sink behind it, the client's output. This one does the work: it stops render outrunning a stuck client, at about 2 queued records at `queue` 3 and about 30 at 32. |
| `io_within`, `protocol_within`, `within` | their calls | One entry point's outputs against its constant, which is a capacity, and they keep no queue. They aren't called around `fire`, the file layer, or the `file_told` pass. They go with the constants. |
| `Service::new` | `limits.queue >= 3` | A capacity: the reservations fit. The largest `MAX_OUT` is 2, and nothing found explains 3. |

Room stops no order here: a queue is first in, first out, so a check only
delays. These are the orders beside the checks, which
`hearth/docs/design/domain/render.md`, 8.5 states, and what keeps each:

| Order | Kept by | Its test, in `hearth/tests/builder` |
|---|---|---|
| Resumed work goes before the next event | `work_pending` | `what_a_call_prints_and_stores_does_not_depend_on_its_queues` |
| One record at a time | `takes` | the same |
| An answer follows its resources' close | the machine's state | the same |
| A build's report comes whole | `holds`, `reporting` | `a_build_prints_its_layers_then_its_failure_then_its_reply` |
| The domain takes a store step's answer before the store's next step | no predicate: the queue's order and the indices' `waiting` | `what_a_call_prints_and_stores_does_not_depend_on_its_queues` |

`file_told` is drained before `told`, by the order of the loops. Nothing
found depends on that, so it isn't stated as an order.

### What the service round must finish

- **`answers`:** uncapped, it can hold a copy of every target's spec, so it
  needs a watermark (`skein/docs/foundation/programming-model.md`, 7.1), or a
  stop on `takes`.
- **The loop bounds:** `0..calls.capacity()`, `answers.capacity()`,
  `asked.capacity()` and `file_told.capacity()` become `u32::MAX` once
  those queues are uncapped, so one pass could run a whole render. They
  pace, and need a decision.
- **A named predicate for the store step:** `hearth-protocol-builder`'s `up`
  reads `!above.is_empty()` of all of `calls`, and means "this entry
  emitted". A length before and after says so.
- **`domain_stage`'s step loop** doesn't check `work_pending` between events.
  Nothing depends on that while every step ends `Unavailable`. A build step
  that streams events could open the gap, so read it again then.
