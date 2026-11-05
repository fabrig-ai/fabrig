# D001: The service and its machines pace work by room checks and `MAX_OUT`

- **Kind:** complexity
- **Where:** `hearth-service-builder/src/lib.rs`, `hearth-protocol-builder/src/lib.rs`, `hearth-protocol-config/src/json.rs`, `hearth-client-protocol-records/src/printer.rs`, `hearth-domain-builder/src/lib.rs`, `hearth-domain-render/src/lib.rs`, `hearth-domain-store/src/lib.rs`, `hearth-domain-tree/src/lib.rs`, `hearth-domain-build/src/lib.rs`, `tests/builder/tests/suite/bounds.rs`
- **Seen:** 2026-11-04 at `66968a30ef`, in the request of plan hearth-workspace (its port table, and its rule that adopting the newer model in full is out of scope)
- **Seen:** 2026-11-04 at `66968a30ef`, in `hearth/scripts/old-forms.txt` (`max_out`)
- **Checked:** 2026-11-05 at `e013b19d72`
- **Value:** high (hearth paces as the other trees do; `iterate` shrinks; new code stops copying the old form)
- **Size:** L
- **Status:** open
- **Paid when:** no hearth step crate defines a `MaxOut` or a `MAX_OUT` constant, `iterate` asks no queue for room but the kernel's edge, the queues between a service's own layers are uncapped, and the orders the room checks kept are named predicates with world tests.
- **Measure:** `grep '^max_out ' hearth/scripts/old-forms.txt`: `max_out 57`

## What

The port took the renames and stopped there, as its request said.
Under `skein/docs/foundation/programming-model.md`, 6.3 and 7.1, nothing
caps the entries between a root's own children; only the edges refuse.
hearth still holds the old form:

- step crates of the builder still declare what one entry point may emit:
  `MAX_OUT` in five domains, `MaxOut` and `MAX_OUT_*` in
  `hearth-protocol-builder`, and `MaxOut`, `UP_MAX_OUT` and `DOWN_MAX_OUT`
  in the config and records machines;
- `hearth-service-builder/src/lib.rs` keeps eight capped queues, and
  `told` is the only uncapped one. Before each entry point it asks
  `io_room` or `protocol_room`; after it, `within` asserts the constant.
  23 lines of that file hold the form;
- `Service::new` refuses `limits.queue` under 3, "queues holding every
  MAX_OUT", and `tests/builder/tests/suite/bounds.rs` sizes a world on
  that figure;
- 23 `.room()` calls remain in non-test step code.

The count leaves out the eight `io::MAX_OUT_*` uses, the kernel's edge.

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
Git and mmdebstrap are done; config and records come next, then the service.

## What each room check keeps

By `skein/docs/foundation/programming-model.md`, 7.2, each check keeps a
capacity, an order, or nothing. This section lists them, for the round
that removes each one. A check here that goes needs its order's test
first, and each order's test is named beside it.

### The machines and the domains

| Where | Check | Keeps |
|---|---|---|
| `hearth-protocol-mmdebstrap/src/tail.rs`, `admit` | `tail.lines.room() == 0` | A capacity, `tail_lines`, at an edge. An order too: the oldest line leaves first. `generated_failures_keep_the_reference_s_tail_and_cause`, in `hearth/tests/mmdebstrap`. |
| `hearth-protocol-builder/src/lib.rs`, `revision_stdout` | `stdout.room()`, three times | A capacity at an edge: `rev-parse`'s answer, of `MAX_STDOUT` and one byte to see excess. |
| `hearth-protocol-builder/src/indices.rs`, `Indices` | `self.intake.room()` | A capacity: a read of a `Packages` file asks for no more than its intake holds. |
| `hearth-protocol-tree/src/fields.rs`, `Fields::value` | `values.room()` | A capacity at an edge: a stored value's items, refused past `limit`. |
| `hearth-domain-render/src/parts/exports.rs`, `push` | `record.room() == 0` | Nothing: it doubles a list by hand and refuses nothing. `List::uncapped` does the same. |
| `hearth-domain-render/src/schema.rs`, `push`, `fault` and `add_default` | `faults.room() == 0`, twice, and `values.room() == 0` | Nothing, the same. |
| `hearth-domain-render/src/schema/references.rs`, `condition` | `faults.room() == 0` | Nothing, the same. |
| `hearth-protocol-config/src/document.rs` | `pending.room() == 0` | Nothing, the same: a stack whose depth the decoder's limits bound. |
| `hearth-protocol-config/src/json.rs` | `values.room() == 0` | Nothing, the same. |
| `hearth-protocol-config/src/json.rs`, `Decoder`'s `events` | capacity 1 | Nothing: the tokenizer answers one `Next` with one event, and `drive` takes it before it asks the next. |
| `hearth-protocol-builder/src/read.rs`, `Decode`'s `below` | capacity 1 | An order: one read out at a time, which the tokenizer's state keeps. `a_decoder_has_one_read_out_whatever_the_cuts`, in `hearth/tests/config`. |
| `hearth-protocol-builder/src/read.rs`, `Decode`'s `above` | capacity 1 | An order: a delivery gives one event, the one `resume` takes, and the outcome comes once, `Closed` last, once. `drive`'s returns and `Waiting` keep them. `a_delivery_gives_a_decoder_s_caller_at_most_one_event` and `a_decoder_s_outcome_comes_once_and_closed_last`, in `hearth/tests/config`. |

The queues beside git's machines and mmdebstrap's, these last in
`hearth-protocol-builder/src/resolve.rs` and `indices.rs`, are uncapped.
The machines' states keep the orders in
`hearth/docs/design/domain/render.md`, 8.5, and their tests stand in
`hearth/tests/git` and `hearth/tests/mmdebstrap`. The queues beside the
records machine, `Client`'s in `client.rs`, stay capped with no row yet:
each gets one, and its order's test, in that machine's round.

### The service

All in `hearth-service-builder/src/lib.rs`.

| Where | Check | Keeps |
|---|---|---|
| `iterate` | `submissions.room() == 0` | A capacity, the kernel's edge. It stays. Io's submissions going before the file layer's is pop order, not room. |
| `io_up`, `io_down` | `io_room`, four times | A capacity, the kernel's edge, through `io_submissions`. `is_ready` and `completions.is_empty()` beside it hold the order. |
| `io_up`, `io_down` | `file_told.room() < 1`, twice | A capacity between the root's own children. It never binds: the file layer takes one request at a time, so the queue holds at most one event. |
| `io_up`, `io_down` | `file_submissions.room() < 2`, three times | A capacity, the kernel's edge, through the file layer's queue: an operation and its cancel. The one before `expire` reserves 2 for one cancel. |
| `protocol_up` | `protocol_room`, before `protocol::fire` | Nothing: `fire` touches neither queue. |
| `protocol_up`, `protocol_down` | `protocol_room`, four times | A capacity between the root's own children: `calls` and `asked`. |
| `domain_stage` | `answers.room()`, three times | A capacity between the root's own children, with a real sink behind it, the client's output. This one does the work: it stops render outrunning a stuck client, at about 2 queued records at `queue` 3 and about 30 at 32. |
| `io_within`, `protocol_within`, `within` | their calls | One entry point's outputs against its constant, which is a capacity, and they keep no queue. They aren't called around `fire`, the file layer, or the `file_told` pass. They go with the constants. |
| `Service::new` | `limits.queue >= 3` | A capacity: the reservations fit. The largest `MAX_OUT` is 2, and nothing found explains 3. |

Room stops no order here: a queue is first in, first out, so a check only
delays. These are the orders beside the checks, which
`hearth/docs/design/domain/render.md`, 8.5 states, and what keeps each:

| Order | Kept by | Its test, in `hearth/tests/builder` |
|---|---|---|
| Resumed work goes before the next event | `work_pending` | Held only by the design; the bounded service-order search below found no biting case. |
| One record at a time | `takes` | `what_a_call_prints_and_stores_does_not_depend_on_its_queues` |
| An answer follows its resources' close | the machine's state | the same |
| A build's report comes whole | `holds`, `reporting` | `a_build_prints_its_layers_then_its_failure_then_its_reply` |
| The domain takes a store step's answer before the store's next step | no predicate: the queue's order and the indices' `waiting` | `what_a_call_prints_and_stores_does_not_depend_on_its_queues` |

`file_told` is drained before `told`, by the order of the loops. Nothing
failed when those loops were swapped:
`what_a_call_prints_and_stores_does_not_depend_on_its_queues` and the eight
protocol tests all passed. No order was found, so it isn't stated as one.

### The service-order search

The queue comparison does not pin resumed work before the next io event.
Removing `work_pending` from `protocol_up`'s `told` guard survived 384
fixed cases: thin, eight blobs, templates, four-target resolution, a
resolving diff and git killed after seven stdout bytes; seeds 0 through 7;
queue sizes 3, 4, 8 and 32; git chunks and file reads of 64 bytes. Half
used calm completions; half forced short reads and writes and delayed
every completion by up to 1 ms. Each case's client submissions and
completions, excluding timestamps, were identical with and without the guard. A temporary assertion confirmed
that the missing guard admits an io event while protocol work is pending,
including thin at seed 0 and queue 3 with delayed completions. This is
an order held only by the design, not a passing mutation check.

Before removing that check, the service round must find an observable
consequence of admitting the next io event while work is pending, and pin
it with a biting test; recheck pending cache work alongside stdin room,
child exit and pipe-end events. The guard stays until then.

The store-step mutation does change the order of operations in thin's
resolution at seed 7 and queue 3, although its printed and stored results
are unchanged. The resolving row of
`what_a_call_prints_and_stores_does_not_depend_on_its_queues` also checks
that the first index's cache record finishes storing, including its
directory close, before the next index opens. Treating every store event
as an emitted answer reverses those two milestones and fails the test.

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
